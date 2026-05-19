//! `mackesd_core` — the authoritative read API for the Mesh control
//! plane. Linked directly into `mackes-panel` (no IPC, no networked
//! API per Phase 12.A.3 lock 2026-05-19).
//!
//! Module organization mirrors the 8-layer architecture in
//! `docs/PROJECT_WORKLIST.md` § Phase 12. Modules land one at a time
//! as their substeps ship; only those whose substep is `[✓] Done`
//! are exposed here.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod audit;
pub mod enrollment;
pub mod events;
pub mod health;
pub mod identity;
pub mod leader;
pub mod legacy_inventory;
pub mod logging;
pub mod metrics;
pub mod passcode;
pub mod policy;
pub mod reconcile;
pub mod revisions;
pub mod secrets;
pub mod settings;
pub mod store;
pub mod telemetry;
pub mod topology;
pub mod validation;
pub mod worker;

// v2.0.0 Phase A modules — async surface for the unified backend.
// Gated behind `async-services` so the legacy sync read-API still
// builds with only the original Phase 12 deps. Library consumers
// that need DBus / async workers enable the feature.
#[cfg(feature = "async-services")]
pub mod ipc;
#[cfg(feature = "async-services")]
pub mod workers;

/// Crate-wide error type. Every public function returns
/// `Result<T, mackesd_core::Error>` so callers don't have to import
/// half a dozen error types from internal modules.
pub type Error = anyhow::Error;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Default `SQLite` path inside `$MACKESD_HOME` (or `/var/lib/mackesd`).
#[must_use]
pub fn default_db_path() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("MACKESD_HOME") {
        return std::path::PathBuf::from(home).join("mackesd.db");
    }
    std::path::PathBuf::from("/var/lib/mackesd/mackesd.db")
}

/// Default QNM-Shared sync root. Heartbeats + link telemetry land at
/// `<root>/<peer>/mackesd/{heartbeat,links}.json`; the leader lock is
/// `<root>/.mackesd-leader.lock`.
#[must_use]
pub fn default_qnm_shared_root() -> std::path::PathBuf {
    if let Ok(root) = std::env::var("QNM_SHARED_ROOT") {
        return std::path::PathBuf::from(root);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join("QNM-Shared");
    }
    std::path::PathBuf::from("/var/lib/mackesd/qnm-shared")
}
