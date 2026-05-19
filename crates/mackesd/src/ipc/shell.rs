//! `org.mackes.Shell` — top-level shell control (health, version,
//! worker pool status). Phase A: schema only.

#![cfg(feature = "async-services")]

use zbus::interface;

/// Object exposed at `/org/mackes/Shell`.
#[derive(Debug, Default, Clone)]
pub struct ShellService;

#[interface(name = "org.mackes.Shell")]
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
}
