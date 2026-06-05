//! KDC2-3.10 — the KDE Connect host registered as a `mackesd` worker.
//!
//! Owns the `Arc<PairingStore>` + the operator-facing **Connect** surface
//! over the Bus (`action/connect/<verb>`: version / list / get / pair /
//! unpair / ring / sms / clipboard) + the pending-sends queue.
//!
//! **E2.2 (2026-06-05) — KDC host convergence.**
//! *Step 1* dropped the held-but-unused `mde_kdc::transport::KdcHost`
//! orchestrator + `mde_kdc_proto::discovery::DiscoveryRegistry`
//! scaffolding (nothing consumed the `host()`/`discovery()` accessors —
//! §3 dead code for never-built workers).
//! *Step 2 (this file)* retired the legacy `mde_kdc::dbus::DbusServer`
//! (`dev.mackes.MDE.Connect` D-Bus) in favour of a **Bus responder**
//! ([`serve_connect_bus`] + the pure [`handle_connect_verb`]) over
//! `action/connect/<verb>` request → `reply/<ulid>`, per the
//! EPIC-RETIRE-DBUS lock — which also advances E0.3.7's final D-Bus
//! sweep. The store verbs are faithful ports; `ring`/`sms`/`clipboard`
//! keep enqueuing onto the outbound queue (the live send is the 2-device
//! bench / the `kdc_outbound` drainer follow-up).
//! *Remaining:* swap the legacy `pairing::PairingStore`/`outbound` for the
//! canonical `mde-kdc-host` equivalents (E2.3 — one store), then drop the
//! `crates/legacy/mde-kdc` path-dep so `cargo tree` shows one host.

#![cfg(feature = "async-services")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use mde_bus::hooks::config::Priority;
use mde_bus::persist::Persist;
use mde_bus::rpc::reply_topic;
use mde_kdc::outbound::{OutboundSend, PendingSends};
use mde_kdc::pairing::{PairedDevice, PairingError, PairingStore};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use super::{ShutdownToken, Worker};

/// The Connect verbs served over `action/connect/<verb>` (E2.2 — replacing
/// the retired `dev.mackes.MDE.Connect` D-Bus surface). `version`/`list`/
/// `get` read the store; `pair`/`unpair` mutate it; `ring`/`sms`/
/// `clipboard` enqueue a `Packet` onto the outbound queue.
const CONNECT_VERBS: [&str; 8] = [
    "version",
    "list",
    "get",
    "pair",
    "unpair",
    "ring",
    "sms",
    "clipboard",
];

/// Poll cadence for the Connect action topics (operator-scale — clicks).
const CONNECT_POLL: Duration = Duration::from_millis(400);

/// Health-tick cadence. 30s is the same window
/// `lan_discovery` uses for its idle scan.
const TICK: Duration = Duration::from_secs(30);

/// Async worker that owns the KDC host objects.
pub struct KdcHostWorker {
    config_dir: PathBuf,
    /// Shared outbound queue. The Connect Bus responder pushes
    /// here; the future `kdc_outbound` worker drains.
    outbound: PendingSends,
    /// Stop flag for the `action/connect/*` Bus responder thread.
    responder_stop: Arc<AtomicBool>,
}

impl KdcHostWorker {
    /// Construct with the on-disk config directory. The host
    /// itself is constructed lazily inside `run()` so a failed
    /// keygen / load doesn't abort the daemon startup — the
    /// supervisor sees a worker error + restarts according to
    /// `restart_policy`.
    #[must_use]
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            outbound: PendingSends::new(),
            responder_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Open the on-disk pairing store (creating the identity on first
    /// run). Idempotent + cheap once `identity.pem` exists, so `run`
    /// can call it freely after a restart.
    fn open_pairing(&self) -> Result<Arc<PairingStore>, PairingError> {
        Ok(Arc::new(PairingStore::open_or_init(&self.config_dir)?))
    }
}

/// Build an outbound `Packet` from a kind token + body (id = wall-clock
/// ms, the receiver's dual-send dedupe key). Replicates the legacy
/// `dbus::build_packet` so the responder doesn't depend on `mde_kdc::dbus`.
fn build_packet(kind: &str, body: Value) -> mde_kdc_proto::wire::Packet {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    mde_kdc_proto::wire::Packet {
        id,
        kind: kind.to_string(),
        body,
        ..Default::default()
    }
}

/// A paired device as the Bus reply renders it (the wire subset, no
/// `public_key_b64`).
fn device_json(d: &PairedDevice) -> Value {
    json!({
        "id": d.id,
        "name": d.name,
        "kind": d.kind,
        "fingerprint": d.fingerprint,
        "capabilities": d.capabilities,
        "paired_at": d.paired_at,
        "last_seen_at": d.last_seen_at,
    })
}

/// Handle one `action/connect/<verb>` request and return the reply JSON.
/// Pure over (`store`, `outbound`) — the unit tests drive it directly.
/// E2.2 — faithfully ports the retired `dev.mackes.MDE.Connect1` methods:
/// `version`/`list`/`get` read; `pair`/`unpair` mutate the store;
/// `ring`/`sms`/`clipboard` enqueue an outbound `Packet`.
fn handle_connect_verb(
    store: &PairingStore,
    outbound: &PendingSends,
    verb: &str,
    body: &Value,
) -> String {
    let dev_id = || {
        body.get("device_id")
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let reply = match verb {
        "version" => json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }),
        "list" => json!({
            "ok": true,
            "devices": store.list().iter().map(device_json).collect::<Vec<_>>(),
        }),
        "get" => match dev_id().and_then(|id| store.get(&id)) {
            Some(d) => json!({ "ok": true, "device": device_json(&d) }),
            None => json!({ "ok": false, "error": "NoSuchDevice" }),
        },
        "pair" => {
            let device = PairedDevice {
                id: body
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: body
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                kind: body
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
                fingerprint: body
                    .get("fingerprint")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                public_key_b64: body
                    .get("public_key_b64")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                capabilities: body
                    .get("capabilities")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                paired_at: body.get("paired_at").and_then(Value::as_i64).unwrap_or(0),
                last_seen_at: 0,
            };
            let name = device.name.clone();
            match store.upsert(device) {
                Ok(()) => {
                    // Best-effort mesh-storage drop folder (GF-15.1).
                    let _ = mde_kdc::receive::ensure_phone_drop_folder(&name);
                    json!({ "ok": true })
                }
                Err(e) => json!({ "ok": false, "error": format!("PersistFailed: {e}") }),
            }
        }
        "unpair" => match dev_id() {
            Some(id) => match store.forget(&id) {
                Ok(true) => json!({ "ok": true }),
                Ok(false) => json!({ "ok": false, "error": "NoSuchDevice" }),
                Err(e) => json!({ "ok": false, "error": format!("PersistFailed: {e}") }),
            },
            None => json!({ "ok": false, "error": "NoSuchDevice" }),
        },
        "ring" | "sms" | "clipboard" => {
            let Some(id) = dev_id() else {
                return json!({ "ok": false, "error": "NoSuchDevice" }).to_string();
            };
            if store.get(&id).is_none() {
                return json!({ "ok": false, "error": "NoSuchDevice" }).to_string();
            }
            let packet = match verb {
                "ring" => build_packet("kdeconnect.findmyphone.request", json!({})),
                "sms" => build_packet(
                    "kdeconnect.sms.request",
                    json!({
                        "sendSms": true,
                        "phoneNumber": body.get("recipient").and_then(Value::as_str).unwrap_or_default(),
                        "messageBody": body.get("message").and_then(Value::as_str).unwrap_or_default(),
                    }),
                ),
                _ => build_packet(
                    "kdeconnect.clipboard",
                    json!({ "content": body.get("content").and_then(Value::as_str).unwrap_or_default() }),
                ),
            };
            outbound.push(OutboundSend {
                device_id: id,
                packet,
            });
            json!({ "ok": true })
        }
        other => json!({ "ok": false, "error": format!("unknown verb: {other}") }),
    };
    reply.to_string()
}

/// The `action/connect/*` Bus responder loop. Sync (the verb handlers +
/// `Persist` are all sync), so it runs on its own `std::thread` and stops
/// when `stop` is set. Mirrors `mde-session`'s poll responder.
fn serve_connect_bus(
    persist: &Persist,
    store: &PairingStore,
    outbound: &PendingSends,
    stop: &AtomicBool,
) {
    let mut cursors: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    while !stop.load(Ordering::Relaxed) {
        for verb in CONNECT_VERBS {
            let topic = format!("action/connect/{verb}");
            let since = cursors.get(&topic).map(String::as_str);
            let msgs = match persist.list_since(&topic, since) {
                Ok(m) => m,
                Err(_) => continue,
            };
            for msg in msgs {
                cursors.insert(topic.clone(), msg.ulid.clone());
                let body: Value = msg
                    .body
                    .as_deref()
                    .and_then(|b| serde_json::from_str(b).ok())
                    .unwrap_or(Value::Null);
                let reply = handle_connect_verb(store, outbound, verb, &body);
                let _ = persist.write(
                    &reply_topic(&msg.ulid),
                    Priority::Default,
                    None,
                    Some(&reply),
                );
            }
        }
        std::thread::sleep(CONNECT_POLL);
    }
}

#[async_trait::async_trait]
impl Worker for KdcHostWorker {
    fn name(&self) -> &'static str {
        "kdc-host"
    }

    async fn run(&mut self, mut shutdown: ShutdownToken) -> anyhow::Result<()> {
        // Open the pairing store (idempotent). On failure, surface to
        // the supervisor so the restart policy can act.
        let pairing_arc = self.open_pairing().map_err(|e| {
            error!(error = %e, "kdc-host: pairing store init failed");
            anyhow::anyhow!("kdc-host init failed: {e}")
        })?;
        // E2.2 — serve the operator-facing Connect actions over the Bus
        // (`action/connect/<verb>`), replacing the retired
        // `dev.mackes.MDE.Connect` D-Bus surface. Runs on its own thread
        // (`Persist` is `!Send`) until the stop flag is set on shutdown;
        // a missing Bus dir / open failure degrades the surface to
        // "unavailable" without failing worker startup.
        self.responder_stop.store(false, Ordering::Relaxed);
        let stop = Arc::clone(&self.responder_stop);
        let store = Arc::clone(&pairing_arc);
        let outbound = self.outbound.clone();
        let responder = std::thread::Builder::new()
            .name("kdc-connect-bus".into())
            .spawn(move || {
                let Some(bus_root) = mde_bus::default_data_dir() else {
                    warn!("kdc-host: no Bus data dir; Connect actions unavailable");
                    return;
                };
                match Persist::open(bus_root) {
                    Ok(p) => serve_connect_bus(&p, &store, &outbound, &stop),
                    Err(e) => {
                        warn!(error = %e, "kdc-host: opening Bus store for Connect responder")
                    }
                }
            })
            .ok();
        info!(
            config_dir = %self.config_dir.display(),
            connect_bus = responder.is_some(),
            "kdc-host: started",
        );

        let mut interval = tokio::time::interval(TICK);
        // First tick fires immediately; skip it so we don't
        // double-log "started" + "tick" at startup.
        interval.tick().await;

        loop {
            tokio::select! {
                _ = shutdown.wait() => {
                    info!("kdc-host: shutdown requested; exiting");
                    // Stop the Connect Bus responder thread + join it.
                    self.responder_stop.store(true, Ordering::Relaxed);
                    if let Some(h) = responder {
                        let _ = h.join();
                    }
                    return Ok(());
                }
                _ = interval.tick() => {
                    debug!(
                        outbound_backlog = self.outbound.len(),
                        "kdc-host: tick",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn worker_name_matches_module() {
        let w = KdcHostWorker::new(PathBuf::from("/tmp"));
        assert_eq!(w.name(), "kdc-host");
    }

    #[test]
    fn open_pairing_creates_the_identity() {
        // E2.2 — the worker holds only the pairing store now (the dead
        // KdcHost/discovery scaffolding was dropped). open_pairing opens
        // it, creating identity.pem on first run.
        let tmp = tempdir().unwrap();
        let w = KdcHostWorker::new(tmp.path().to_path_buf());
        let store = w.open_pairing().unwrap();
        assert!(Arc::strong_count(&store) >= 1);
        assert!(tmp.path().join("identity.pem").exists());
    }

    fn test_store(dir: &std::path::Path) -> PairingStore {
        PairingStore::open_or_init(dir).unwrap()
    }

    fn pair_body(id: &str, name: &str) -> Value {
        json!({
            "id": id, "name": name, "kind": "phone",
            "fingerprint": "AB:CD", "public_key_b64": "", "capabilities": [],
            "paired_at": 123,
        })
    }

    #[test]
    fn connect_verb_version_and_empty_list() {
        let tmp = tempdir().unwrap();
        let store = test_store(tmp.path());
        let outbound = PendingSends::new();
        let v: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "version",
            &Value::Null,
        ))
        .unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["version"].is_string());
        let l: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "list",
            &Value::Null,
        ))
        .unwrap();
        assert_eq!(l["devices"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn connect_verb_pair_get_unpair_roundtrip() {
        let tmp = tempdir().unwrap();
        let store = test_store(tmp.path());
        let outbound = PendingSends::new();
        // pair
        let r: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "pair",
            &pair_body("d1", "Pixel"),
        ))
        .unwrap();
        assert_eq!(r["ok"], true);
        // get
        let g: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "get",
            &json!({ "device_id": "d1" }),
        ))
        .unwrap();
        assert_eq!(g["device"]["name"], "Pixel");
        // get unknown
        let gx: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "get",
            &json!({ "device_id": "nope" }),
        ))
        .unwrap();
        assert_eq!(gx["error"], "NoSuchDevice");
        // unpair, then unpair-again
        let u: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "unpair",
            &json!({ "device_id": "d1" }),
        ))
        .unwrap();
        assert_eq!(u["ok"], true);
        let u2: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "unpair",
            &json!({ "device_id": "d1" }),
        ))
        .unwrap();
        assert_eq!(u2["error"], "NoSuchDevice");
    }

    #[test]
    fn connect_verb_ring_requires_paired_and_enqueues() {
        let tmp = tempdir().unwrap();
        let store = test_store(tmp.path());
        let outbound = PendingSends::new();
        // ring an unpaired device -> NoSuchDevice, nothing queued.
        let r: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "ring",
            &json!({ "device_id": "d1" }),
        ))
        .unwrap();
        assert_eq!(r["error"], "NoSuchDevice");
        assert_eq!(outbound.len(), 0);
        // pair then ring -> ok + one queued packet.
        handle_connect_verb(&store, &outbound, "pair", &pair_body("d1", "Pixel"));
        let r2: Value = serde_json::from_str(&handle_connect_verb(
            &store,
            &outbound,
            "ring",
            &json!({ "device_id": "d1" }),
        ))
        .unwrap();
        assert_eq!(r2["ok"], true);
        assert_eq!(outbound.len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn worker_exits_on_shutdown_request() {
        let tmp = tempdir().unwrap();
        let mut w = KdcHostWorker::new(tmp.path().to_path_buf());
        let (tx, rx) = tokio::sync::watch::channel(false);
        let token = super::super::ShutdownToken::from_receiver(rx);

        let handle = tokio::spawn(async move { w.run(token).await });
        tx.send(true).expect("shutdown channel intact");
        let result = handle.await.expect("worker join");
        assert!(result.is_ok(), "worker must exit Ok on shutdown");
        // identity.pem was created during init.
        assert!(tmp.path().join("identity.pem").exists());
    }
}
