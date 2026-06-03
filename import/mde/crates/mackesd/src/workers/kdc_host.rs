//! KDC2-3.10 — `KdcHost` registered as a `mackesd` worker.
//!
//! Owns the `Arc<mde_kdc::pairing::PairingStore>`, the
//! `Arc<mde_kdc::transport::KdcHost>`, and (when running under
//! the `kdc-dbus` feature path) the live
//! `mde_kdc::dbus::DbusServer` registered at
//! `/dev/mackes/MDE/Connect`. The mesh-router worker (KDC2-1.8)
//! pulls the KdcHost out of the registry to dispatch through
//! it; the D-Bus host exposes the operator-facing actions
//! (Ring / Pair / SendSms / SendFile / SendClipboard).
//!
//! KDC2-3.3 wire-up (2026-05-23): `DbusServer::start` runs once
//! during `init_host` and the returned handle is held on the
//! worker for the daemon's lifetime. Dropping the worker
//! surrenders the bus name. The pending-sends queue is shared
//! with the future `kdc_outbound` worker (KDC2-3.2.a follow-up)
//! that drains the queue and writes packets onto the live
//! `KdcTlsConnection`.

#![cfg(feature = "async-services")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mde_kdc::dbus::{DbusError, DbusServer};
use mde_kdc::outbound::PendingSends;
use mde_kdc::pairing::{PairingError, PairingStore};
use mde_kdc::transport::KdcHost;
use mde_kdc_proto::discovery::DiscoveryRegistry;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, warn};

use super::{ShutdownToken, Worker};

/// Health-tick cadence. 30s is the same window
/// `lan_discovery` uses for its idle scan.
const TICK: Duration = Duration::from_secs(30);

/// Async worker that owns the KDC host objects.
pub struct KdcHostWorker {
    config_dir: PathBuf,
    host: Option<Arc<KdcHost>>,
    /// Shared discovery registry. The future `kdc_discovery`
    /// worker (KDC2-2.9.a wire-up) writes to this via the
    /// host's `discovery()` handle; `KdcHost::open` reads from
    /// it. Held on the worker so a daemon restart re-uses the
    /// same registry across host re-init.
    discovery: Arc<AsyncMutex<DiscoveryRegistry>>,
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
            host: None,
            discovery: Arc::new(AsyncMutex::new(DiscoveryRegistry::new())),
            outbound: PendingSends::new(),
            dbus_server: None,
        }
    }

    /// Open the on-disk pairing store + construct the KdcHost.
    /// Pure helper so the run loop can re-init after a restart.
    fn init_host(&mut self) -> Result<Arc<PairingStore>, PairingError> {
        let store = PairingStore::open_or_init(&self.config_dir)?;
        let store_arc = Arc::new(store);
        self.host = Some(Arc::new(KdcHost::new(
            Arc::clone(&store_arc),
            Arc::clone(&self.discovery),
        )));
        Ok(store_arc)
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

    /// Borrow the live KdcHost. `None` before `init_host()` runs
    /// or after a fatal error. Exposed for the future
    /// mesh_router registration (KDC2-1.8 → 3.10 bridge) +
    /// tests.
    #[must_use]
    pub fn host(&self) -> Option<&Arc<KdcHost>> {
        self.host.as_ref()
    }

    /// Borrow the shared discovery registry. The future
    /// `kdc_discovery` worker (which owns the UDP/1716 socket +
    /// the mDNS browser) consumes this handle to inject real
    /// announces — the same `Arc` the host's `open()` reads
    /// from.
    #[must_use]
    pub fn discovery(&self) -> Arc<AsyncMutex<DiscoveryRegistry>> {
        Arc::clone(&self.discovery)
    }

    /// Borrow the shared outbound queue. The future
    /// `kdc_outbound` worker (KDC2-3.2.a follow-up) drains this
    /// queue and dispatches packets through the KdcHost's
    /// live `Connection`s.
    #[must_use]
    pub fn outbound(&self) -> PendingSends {
        self.outbound.clone()
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
        // Lazy init the host. On failure, surface to the
        // supervisor so the restart policy can act.
        let pairing_arc = if self.host.is_none() {
            self.init_host().map_err(|e| {
                error!(error = %e, "kdc-host: pairing store init failed");
                anyhow::anyhow!("kdc-host init failed: {e}")
            })?
        } else {
            // Host was pre-initialized (test path). Re-read the
            // store so we can hand it to DbusServer::start. The
            // open_or_init call is idempotent + cheap when the
            // identity.pem already exists.
            Arc::new(PairingStore::open_or_init(&self.config_dir).map_err(|e| {
                anyhow::anyhow!("re-open pairing store: {e}")
            })?)
        };
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
    fn host_is_none_before_init() {
        let w = KdcHostWorker::new(PathBuf::from("/tmp/never-touched"));
        assert!(w.host().is_none());
    }

    #[test]
    fn init_host_populates_the_arc() {
        let tmp = tempdir().unwrap();
        let mut w = KdcHostWorker::new(tmp.path().to_path_buf());
        w.init_host().unwrap();
        assert!(w.host().is_some());
        // The store created an identity.pem.
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
