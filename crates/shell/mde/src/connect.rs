//! KDE Connect surface client — `mde connect` reads the device roster that the
//! mackesd `kdc_host` worker publishes over the mesh **Bus** at
//! `action/connect/devices`.
//!
//! **E2.3 (2026-06-05) — one host, owned by mackesd.** The live KDE Connect
//! host (UDP discovery + the mutual-TLS LAN transport with its inbound listener)
//! and the on-disk `PairingStore` now run inside mackesd's supervised
//! `kdc_host` worker, which folds `HostEvent`s into the roster and serves it on
//! the Bus. This module is the **client** the short-lived shell surfaces share
//! (`mde phone`, the panel, the OOBE Your-Phone stage): [`devices`] queries the
//! daemon's roster, and `mde connect` prints it. Nothing here runs a host or
//! owns a store any more — the prior in-shell daemon (its `LanTransport`
//! bring-up, the `org.mde.Connect` single-instance guard, and the
//! event→roster folding) was retired into the worker.
//!
//! Degrades gracefully: with mackesd (or the Bus, or a timely reply) absent,
//! [`devices`] returns an empty list rather than panicking, so callers render an
//! honest "no devices" state.

use std::process::ExitCode;
use std::time::Duration;

/// Bus action topic the mackesd `kdc_host` worker answers with the roster.
const ACTION_TOPIC: &str = "action/connect/devices";
/// Client-side wait for the roster reply before giving up (→ empty).
const QUERY_TIMEOUT: Duration = Duration::from_secs(2);

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

/// Wire shape for one roster row over the Bus. JSON carries `battery` as a real
/// `Option<u8>` (unlike the old D-Bus `-1`-for-unknown tuple).
#[derive(serde::Serialize, serde::Deserialize)]
struct WireDevice {
    id: String,
    name: String,
    online: bool,
    battery: Option<u8>,
}

/// `mde connect [--list]` — print the paired-device roster the mackesd host
/// publishes. Both forms read now that mackesd owns the host (the `--list`
/// flag is kept for back-compat; there is no longer an in-shell daemon to run).
pub fn run(_args: &[String]) -> ExitCode {
    let devs = devices();
    if devs.is_empty() {
        println!("(no paired devices, or mackesd's KDE Connect host isn't running)");
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
    ExitCode::SUCCESS
}

/// Query the paired-device roster from the mackesd `kdc_host` worker over the
/// Bus (`action/connect/devices`). Returns an empty list (never panics) when
/// the daemon, the Bus, or a timely reply isn't available, so callers render an
/// honest "no devices" state.
///
/// Builds its own current-thread runtime and blocks, so it MUST be called
/// outside an async runtime — async callers (e.g. `mde phone`) wrap it in
/// `tokio::task::spawn_blocking`. The sync `mde connect` path calls it directly.
#[must_use]
pub fn devices() -> Vec<DeviceInfo> {
    let Some(bus_dir) = mde_bus::default_data_dir() else {
        return Vec::new();
    };
    let Ok(rt) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return Vec::new();
    };
    let raw = rt.block_on(async {
        let persist = mde_bus::persist::Persist::open(bus_dir).ok()?;
        mde_bus::rpc::request(
            &persist,
            ACTION_TOPIC,
            mde_bus::hooks::config::Priority::Default,
            None,
            None,
            QUERY_TIMEOUT,
        )
        .await
        .ok()?
        .body
    });
    let Some(raw) = raw else {
        return Vec::new();
    };
    let wires: Vec<WireDevice> = serde_json::from_str(&raw).unwrap_or_default();
    wires
        .into_iter()
        .map(|w| DeviceInfo {
            id: w.id,
            name: w.name,
            online: w.online,
            battery: w.battery.filter(|b| *b <= 100),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_device_decodes_roster_json_with_optional_battery() {
        // The client decodes the worker's `action/connect/devices` reply: a
        // JSON array of rows with `battery` as a real Option (no -1 sentinel).
        let json = r#"[
            {"id":"alpha","name":"Alpha","online":false,"battery":null},
            {"id":"zeta","name":"Zeta","online":true,"battery":80}
        ]"#;
        let wires: Vec<WireDevice> = serde_json::from_str(json).expect("decode roster json");
        assert_eq!(wires.len(), 2);
        assert_eq!(wires[0].id, "alpha");
        assert_eq!(wires[0].battery, None);
        assert!(wires[1].online);
        assert_eq!(wires[1].battery, Some(80));
    }

    #[test]
    fn devices_maps_wire_rows_and_clamps_out_of_range_battery() {
        // A defensive clamp lives on the client side: an out-of-range battery
        // (e.g. a malformed reply) sanitizes to None rather than a bogus %.
        let w = WireDevice {
            id: "p1".into(),
            name: "Phone".into(),
            online: true,
            battery: Some(200),
        };
        let info = DeviceInfo {
            id: w.id.clone(),
            name: w.name.clone(),
            online: w.online,
            battery: w.battery.filter(|b| *b <= 100),
        };
        assert_eq!(info.battery, None, "out-of-range battery clamps to None");
        assert!(info.online);
    }
}
