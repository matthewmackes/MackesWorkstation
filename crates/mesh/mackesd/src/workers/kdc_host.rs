//! KDC2-3.10 — the KDE Connect host registered as a `mackesd` worker.
//!
//! Owns the `Arc<PairingStore>` + the operator-facing
//! `dev.mackes.MDE.Connect` surface (Ring / Pair / SendSms /
//! SendFile / SendClipboard) + the pending-sends queue.
//!
//! **E2.2 (2026-06-05) — convergence step 1.** The worker formerly also
//! held an `mde_kdc::transport::KdcHost` orchestrator + an
//! `mde_kdc_proto::discovery::DiscoveryRegistry`, exposed via `host()` /
//! `discovery()` "for the future mesh_router / kdc_discovery workers".
//! Those workers were never built and **nothing consumed those
//! accessors** — held-but-unused scaffolding (§3 dead code). Dropped
//! them, which also retires this worker's use of the legacy
//! `transport::KdcHost` + the legacy proto discovery registry. The
//! canonical transport is `mde-kdc-host`'s `LanTransport` (E2.1); the
//! remaining legacy `dbus`/`outbound`/`pairing` usage migrates next
//! (the `dbus::DbusServer` → a Bus responder, the store → the canonical
//! `PairingStore`).

#![cfg(feature = "async-services")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mde_kdc::dbus::{DbusError, DbusServer};
use mde_kdc::outbound::PendingSends;
use mde_kdc::pairing::{PairingError, PairingStore};
use tracing::{debug, error, info, warn};

use super::{ShutdownToken, Worker};

/// Health-tick cadence. 30s is the same window
/// `lan_discovery` uses for its idle scan.
const TICK: Duration = Duration::from_secs(30);

/// Async worker that owns the KDC host objects.
pub struct KdcHostWorker {
    config_dir: PathBuf,
    /// Shared outbound queue. The Connect D-Bus interface
    /// pushes here; the future `kdc_outbound` worker drains.
    outbound: PendingSends,
    /// Live D-Bus host handle. `Some` after a successful
    /// `init_host` + `DbusServer::start`; dropping the worker
    /// surrenders the bus name. `None` when the operator's
    /// session bus isn't reachable (e.g., headless system unit
    /// without DBUS_SESSION_BUS_ADDRESS) — in that case the
    /// worker keeps running its tick loop without the D-Bus
    /// surface, matching the `lan_discovery` graceful-degrade
    /// pattern.
    dbus_server: Option<DbusServer>,
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
            dbus_server: None,
        }
    }

    /// Open the on-disk pairing store (creating the identity on first
    /// run). Idempotent + cheap once `identity.pem` exists, so `run`
    /// can call it freely after a restart.
    fn open_pairing(&self) -> Result<Arc<PairingStore>, PairingError> {
        Ok(Arc::new(PairingStore::open_or_init(&self.config_dir)?))
    }

    /// Start the D-Bus host scaffold (KDC2-3.3) registered at
    /// `dev.mackes.MDE.Connect` / `/dev/mackes/MDE/Connect`.
    /// Holds the live `DbusServer` on the worker so the bus
    /// name stays acquired for the daemon's lifetime.
    ///
    /// On failure, the worker logs but keeps running — the D-Bus
    /// surface degrades to "unavailable" while the rest of the
    /// host functionality (pairing store + transport open) keeps
    /// working. Matches the `lan_discovery` graceful-degrade
    /// pattern: the daemon should never abort because a
    /// non-essential bus interface couldn't register.
    async fn init_dbus(&mut self, pairing: Arc<PairingStore>) {
        match DbusServer::start(pairing, self.outbound.clone()).await {
            Ok(server) => {
                info!(
                    bus = mde_kdc::dbus::BUS_NAME,
                    object_path = mde_kdc::dbus::OBJECT_PATH,
                    "kdc-host: dev.mackes.MDE.Connect registered",
                );
                self.dbus_server = Some(server);
            }
            Err(DbusError::NameAlreadyAcquired) => {
                warn!(
                    "kdc-host: dev.mackes.MDE.Connect already owned by another \
                     process; D-Bus surface skipped",
                );
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "kdc-host: D-Bus host registration failed; surface skipped",
                );
            }
        }
    }

    /// True when the D-Bus host is registered on the session
    /// bus. Used by `healthz` to surface whether the operator-
    /// facing Connect interface is reachable.
    #[must_use]
    pub fn dbus_alive(&self) -> bool {
        self.dbus_server.is_some()
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
        // KDC2-3.3 — register the operator-facing D-Bus host.
        // Graceful-degrade: a session bus that's unreachable
        // doesn't fail worker startup.
        if self.dbus_server.is_none() {
            self.init_dbus(pairing_arc).await;
        }
        info!(
            config_dir = %self.config_dir.display(),
            dbus_alive = self.dbus_alive(),
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
                    // Dropping self.dbus_server here surrenders
                    // the bus name cleanly.
                    self.dbus_server = None;
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
