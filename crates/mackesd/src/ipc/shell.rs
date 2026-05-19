//! `dev.mackes.MDE.Shell` — top-level shell control (health, version,
//! worker pool status). Phase A: schema only.
//!
//! v2.0.0 Phase 0.4 rebrand — interface name moved from
//! `org.mackes.Shell` to `dev.mackes.MDE.Shell`. Backward-compat
//! alias .service file ships under the old name for one release; see
//! `data/dbus-1/services/`.

#![cfg(feature = "async-services")]

use zbus::interface;

/// Object exposed at `/dev/mackes/MDE/Shell`.
#[derive(Debug, Default, Clone)]
pub struct ShellService;

/// Stable D-Bus name used by Phase 0.4-onward callers. The legacy
/// `org.mackes.Shell` alias ships through one v2.x line for
/// backward-compat.
pub const SERVICE_NAME: &str = "dev.mackes.MDE.Shell";

/// Object-path under [`SERVICE_NAME`]. Matches the
/// reverse-slash convention zbus picks by default.
pub const OBJECT_PATH: &str = "/dev/mackes/MDE/Shell";

#[interface(name = "dev.mackes.MDE.Shell")]
impl ShellService {
    /// Compiled crate version (`CARGO_PKG_VERSION`).
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// JSON-encoded [`crate::health::HealthReport`]. Phase B fills
    /// this in by reading the live supervisor state.
    async fn healthz(&self) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::Failed(
            "Shell.Healthz — wired in Phase B alongside the worker pool".into(),
        ))
    }

    /// List currently-spawned worker names.
    async fn workers(&self) -> zbus::fdo::Result<Vec<String>> {
        Err(zbus::fdo::Error::Failed(
            "Shell.Workers — wired in Phase B".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn version_matches_crate() {
        let svc = ShellService;
        assert_eq!(svc.version().await, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn service_name_carries_mde_namespace() {
        assert_eq!(SERVICE_NAME, "dev.mackes.MDE.Shell");
        assert!(SERVICE_NAME.starts_with("dev.mackes.MDE."));
    }

    #[test]
    fn object_path_mirrors_service_name_segments() {
        assert_eq!(OBJECT_PATH, "/dev/mackes/MDE/Shell");
    }
}
