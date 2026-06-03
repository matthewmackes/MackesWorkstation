//! KDE Connect surface link — the `mde connect` daemon + its in-process client (E9.1).
//!
//! The daemon hosts the native KDE Connect stack (the `mde-kdc-host` crate: UDP
//! discovery + the mutual-TLS LAN transport with its inbound listener) and exposes
//! the **paired-device roster** — id, name, online, battery — over the session bus at
//! `org.mde.Connect` so the short-lived shell surfaces (`mde phone`, the panel, the
//! OOBE Your-Phone stage) can query it without each embedding the host.
//!
//! Architecture mirrors `tray.rs` / `notifyd.rs`: the host runs on its own Tokio
//! runtime on a background thread, folding `HostEvent`s into a shared map; the main
//! thread serves the D-Bus interface (blocking zbus) reading that map. The roster is
//! seeded from the on-disk `PairingStore` so it is populated even before any device
//! connects, and the host bring-up is **best-effort**: if the UDP port can't bind
//! (already in use, no permission) the daemon still serves the static roster.
//!
//! Live device round-trips (a real phone connecting, sending battery) are the owner's
//! post-release hardware bench; here the event→roster folding is unit-tested and the
//! daemon is launch-verified (`timeout 3 mde connect` is a clean no-panic park/exit).

use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mde_kdc_host::{EventStream, HostEvent, LanTransport, PairingStore, Transport, UdpDiscovery};
use mde_kdc_proto::discovery::{Announce, DeviceType};
use mde_kdc_proto::plugins::battery::BatteryBody;

/// The well-known bus name + object path the daemon claims and clients dial.
const BUS_NAME: &str = "org.mde.Connect";
const OBJ_PATH: &str = "/org/mde/Connect";
/// KDE Connect's stock UDP/TCP port (identity broadcast + TLS link).
const KDC_PORT: u16 = 1716;

/// One paired peer as the shell sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// The peer's stable KDE Connect device id.
    pub id: String,
    /// Display name (from the pairing record, refreshed by discovery announces).
    pub name: String,
    /// True while a live (authenticated) connection to the peer is up.
    pub online: bool,
    /// Last-reported battery percentage (0..=100), or `None` if unknown.
    pub battery: Option<u8>,
}

impl DeviceInfo {
    fn unknown(id: &str) -> Self {
        DeviceInfo {
            id: id.to_string(),
            name: id.to_string(),
            online: false,
            battery: None,
        }
    }
}

/// The shared roster the host thread writes and the D-Bus interface reads.
type Roster = Arc<Mutex<HashMap<String, DeviceInfo>>>;

/// Fold one host event into the roster: connections flip `online`, battery packets
/// update the charge, discovery announces refresh the display name. Pure (no I/O) so
/// the state machine is unit-tested without a bus or a phone.
fn apply_event(map: &mut HashMap<String, DeviceInfo>, ev: HostEvent) {
    match ev {
        HostEvent::Connected(p) => {
            map.entry(p.0.clone())
                .or_insert_with(|| DeviceInfo::unknown(&p.0))
                .online = true;
        }
        HostEvent::Disconnected(p) => {
            if let Some(d) = map.get_mut(p.as_str()) {
                d.online = false;
            }
        }
        HostEvent::PeerDiscovered(a) => {
            let e = map
                .entry(a.device_id.clone())
                .or_insert_with(|| DeviceInfo::unknown(&a.device_id));
            if !a.device_name.is_empty() {
                e.name = a.device_name;
            }
        }
        // A discovered peer ageing out doesn't drop it from the paired roster; it's
        // simply no longer broadcasting (online already tracks the live link).
        HostEvent::PeerLost(_) => {}
        HostEvent::Packet { peer, packet } => {
            if packet.kind == "kdeconnect.battery" {
                if let Ok(b) = serde_json::from_value::<BatteryBody>(packet.body) {
                    if let Some(d) = map.get_mut(peer.as_str()) {
                        d.battery = b.charge_pct();
                    }
                }
            }
        }
        HostEvent::TransportError(_) => {}
    }
}

/// This host's identity announce. `device_id` is the machine id (stable across boots),
/// `device_name` the hostname; type Desktop, protocol 7 (KDE Connect current).
fn local_announce() -> Announce {
    let device_id = std::fs::read_to_string("/etc/machine-id")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "mde-host".to_string());
    let device_name = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "MDE-Retro".to_string());
    Announce {
        device_id,
        device_name,
        device_type: DeviceType::Desktop,
        protocol_version: 7,
        incoming_capabilities: Vec::new(),
        outgoing_capabilities: Vec::new(),
    }
}

/// Seed the roster from the on-disk pairing store (all paired peers, offline) so the
/// daemon answers `Devices()` even before the host comes up or any device connects.
fn seed_roster() -> HashMap<String, DeviceInfo> {
    let mut map = HashMap::new();
    if let Ok(dir) = PairingStore::default_dir() {
        if let Ok(store) = PairingStore::open(dir) {
            for rec in store.records() {
                map.insert(
                    rec.device_id.clone(),
                    DeviceInfo {
                        id: rec.device_id.clone(),
                        name: if rec.device_name.is_empty() {
                            rec.device_id.clone()
                        } else {
                            rec.device_name.clone()
                        },
                        online: false,
                        battery: None,
                    },
                );
            }
        }
    }
    map
}

/// Run the KDE Connect host on a Tokio runtime (own thread), folding its events into
/// `roster`. Best-effort: a discovery-bind or transport-start failure is logged and the
/// thread exits, leaving the daemon serving the seeded (static) roster.
fn spawn_host(roster: Roster) {
    thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("mde connect: tokio runtime: {e} (serving static roster)");
                return;
            }
        };
        rt.block_on(async move {
            let announce = local_announce();
            let dir = match PairingStore::default_dir() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("mde connect: pairing dir: {e}");
                    return;
                }
            };
            let pairing = match PairingStore::open(dir) {
                Ok(p) => Arc::new(p),
                Err(e) => {
                    eprintln!("mde connect: pairing store: {e}");
                    return;
                }
            };
            let bind = std::net::SocketAddr::from(([0, 0, 0, 0], KDC_PORT));
            let discovery = match UdpDiscovery::bind(bind, announce.clone()).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("mde connect: UDP {KDC_PORT} bind: {e} (serving static roster)");
                    return;
                }
            };
            let transport = LanTransport::new(announce, discovery, pairing).with_listen_addr(bind);
            let (sink, mut stream) = EventStream::channel();
            if let Err(e) = transport.start(sink).await {
                eprintln!("mde connect: transport start: {e} (serving static roster)");
                return;
            }
            while let Some(ev) = stream.recv().await {
                if let Ok(mut m) = roster.lock() {
                    apply_event(&mut m, ev);
                }
            }
        });
    });
}

/// The D-Bus object: serves the roster.
struct ConnectDaemon {
    roster: Roster,
}

#[zbus::interface(name = "org.mde.Connect1")]
impl ConnectDaemon {
    /// The paired-device roster as `(id, name, online, battery)` tuples; battery is
    /// `-1` when unknown (D-Bus has no optional int).
    fn devices(&self) -> Vec<(String, String, bool, i32)> {
        self.roster
            .lock()
            .map(|m| {
                let mut v: Vec<_> = m
                    .values()
                    .map(|d| {
                        (
                            d.id.clone(),
                            d.name.clone(),
                            d.online,
                            d.battery.map(i32::from).unwrap_or(-1),
                        )
                    })
                    .collect();
                v.sort_by(|a, b| a.0.cmp(&b.0));
                v
            })
            .unwrap_or_default()
    }
}

/// `mde connect` — run the daemon (seed the roster, spawn the host, serve the bus).
/// `mde connect --list` instead queries a *running* daemon via the client and prints
/// the roster (the scriptable read path that exercises [`devices`]).
pub fn run(args: &[String]) -> ExitCode {
    if args.iter().any(|a| a == "--list") {
        let devs = devices();
        if devs.is_empty() {
            println!("(no paired devices, or the connect daemon isn't running)");
        }
        for d in devs {
            let batt = d
                .battery
                .map(|b| format!("{b}%"))
                .unwrap_or_else(|| "?".into());
            println!(
                "{}  {}  [{}]  battery {batt}",
                d.id,
                d.name,
                if d.online { "online" } else { "offline" },
            );
        }
        return ExitCode::SUCCESS;
    }
    let roster: Roster = Arc::new(Mutex::new(seed_roster()));
    spawn_host(Arc::clone(&roster));
    match serve(roster) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde connect: {e}");
            ExitCode::FAILURE
        }
    }
}

fn serve(roster: Roster) -> zbus::Result<()> {
    let daemon = ConnectDaemon { roster };
    let conn = zbus::blocking::connection::Builder::session()?
        .serve_at(OBJ_PATH, daemon)?
        .build()?;
    // Single-owner service: if another `mde connect` already holds the name, exit
    // rather than run a second host on the same port.
    conn.request_name(BUS_NAME)?;
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// ── Client (used by `mde phone`, the panel, the OOBE Your-Phone stage) ────────

#[zbus::proxy(
    interface = "org.mde.Connect1",
    default_service = "org.mde.Connect",
    default_path = "/org/mde/Connect"
)]
trait Connect {
    fn devices(&self) -> zbus::Result<Vec<(String, String, bool, i32)>>;
}

/// Query the paired-device roster from the running `mde connect` daemon. Returns an
/// empty list (never panics) when the daemon or the session bus isn't available, so
/// callers render an honest "no devices" state rather than failing.
#[must_use]
pub fn devices() -> Vec<DeviceInfo> {
    let Ok(conn) = zbus::blocking::Connection::session() else {
        return Vec::new();
    };
    let Ok(proxy) = ConnectProxyBlocking::new(&conn) else {
        return Vec::new();
    };
    proxy
        .devices()
        .map(|v| {
            v.into_iter()
                .map(|(id, name, online, batt)| DeviceInfo {
                    id,
                    name,
                    online,
                    battery: u8::try_from(batt).ok().filter(|b| *b <= 100),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mde_kdc_host::PeerId;
    use mde_kdc_proto::plugins::battery::battery_packet;

    fn announce(id: &str, name: &str) -> Announce {
        Announce {
            device_id: id.into(),
            device_name: name.into(),
            device_type: DeviceType::Phone,
            protocol_version: 7,
            incoming_capabilities: vec![],
            outgoing_capabilities: vec![],
        }
    }

    #[test]
    fn connected_then_disconnected_flips_online() {
        let mut m = HashMap::new();
        apply_event(&mut m, HostEvent::Connected(PeerId::from("p1")));
        assert!(m["p1"].online, "Connected brings the peer online");
        apply_event(&mut m, HostEvent::Disconnected(PeerId::from("p1")));
        assert!(
            !m["p1"].online,
            "Disconnected takes it offline (kept in roster)"
        );
        assert!(m.contains_key("p1"));
    }

    #[test]
    fn discovery_refreshes_the_display_name() {
        let mut m = HashMap::new();
        m.insert("p1".to_string(), DeviceInfo::unknown("p1"));
        apply_event(&mut m, HostEvent::PeerDiscovered(announce("p1", "Pixel 8")));
        assert_eq!(m["p1"].name, "Pixel 8");
    }

    #[test]
    fn battery_packet_updates_charge_and_clamps_unknown() {
        let mut m = HashMap::new();
        m.insert("p1".to_string(), DeviceInfo::unknown("p1"));
        apply_event(
            &mut m,
            HostEvent::Packet {
                peer: PeerId::from("p1"),
                packet: battery_packet(
                    1,
                    BatteryBody {
                        current_charge: 73,
                        is_charging: false,
                        threshold_event: String::new(),
                    },
                ),
            },
        );
        assert_eq!(m["p1"].battery, Some(73));
        // Upstream's -1 "unknown" sentinel sanitizes to None.
        apply_event(
            &mut m,
            HostEvent::Packet {
                peer: PeerId::from("p1"),
                packet: battery_packet(
                    2,
                    BatteryBody {
                        current_charge: -1,
                        is_charging: false,
                        threshold_event: String::new(),
                    },
                ),
            },
        );
        assert_eq!(m["p1"].battery, None);
    }

    #[test]
    fn battery_for_unknown_peer_is_ignored() {
        // A battery packet from a peer not in the roster doesn't create a ghost entry.
        let mut m = HashMap::new();
        apply_event(
            &mut m,
            HostEvent::Packet {
                peer: PeerId::from("ghost"),
                packet: battery_packet(
                    1,
                    BatteryBody {
                        current_charge: 50,
                        is_charging: true,
                        threshold_event: String::new(),
                    },
                ),
            },
        );
        assert!(m.is_empty());
    }
}
