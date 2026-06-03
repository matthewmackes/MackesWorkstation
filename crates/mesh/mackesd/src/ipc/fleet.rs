//! `dev.mackes.MDE.Fleet` — fleet control (push setting revisions,
//! list revisions, rollback) served by mackesd.
//!
//! Phase A ships the schema; Phase G (`v2.0.0`) wires it through to
//! the reconcile loop + the `settings` table.
//!
//! v2.0.0 Phase 0.4 rebrand — interface name moved from
//! `org.mackes.Fleet`. Backward-compat alias .service file ships
//! under the old name for one release; see `data/dbus-1/services/`.

#![cfg(feature = "async-services")]

use zbus::interface;

/// Object exposed at `/dev/mackes/MDE/Fleet`. Phase A: shell.
#[derive(Debug, Default, Clone)]
pub struct FleetService;

/// Stable D-Bus name used by Phase 0.4-onward callers.
pub const SERVICE_NAME: &str = "dev.mackes.MDE.Fleet";

/// Object-path under [`SERVICE_NAME`].
pub const OBJECT_PATH: &str = "/dev/mackes/MDE/Fleet";

#[interface(name = "dev.mackes.MDE.Fleet")]
impl FleetService {
    /// Push a new desired-config revision targeting a set of peers.
    /// `peers_selector` follows the same grammar as
    /// `mackesd fleet push-setting … --peers <sel>` (e.g.
    /// `"all"`, `"region:lab"`, `"node:laptop-01,desktop-02"`).
    /// Returns the new revision id (`r-YYYY-MM-DD-NNNN`).
    async fn push_revision(
        &self,
        _settings_json: &str,
        _peers_selector: &str,
    ) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::Failed(
            "Fleet.PushRevision — not implemented until v2.0.0 Phase G".into(),
        ))
    }

    /// List revision IDs in descending chronological order.
    async fn list_revisions(&self, _limit: u32) -> zbus::fdo::Result<Vec<String>> {
        Err(zbus::fdo::Error::Failed(
            "Fleet.ListRevisions — not implemented until v2.0.0 Phase G".into(),
        ))
    }

    /// Diff two revisions. Returns a JSON-encoded RevisionDiff.
    async fn diff_revisions(&self, _from: &str, _to: &str) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::Failed(
            "Fleet.DiffRevisions — not implemented until v2.0.0 Phase G".into(),
        ))
    }

    /// Rollback to a given revision (fleet-wide or per-peer based on
    /// selector grammar).
    async fn rollback(&self, _revision_id: &str, _peers_selector: &str) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::Failed(
            "Fleet.Rollback — not implemented until v2.0.0 Phase G".into(),
        ))
    }

    /// Signal: a fleet revision has been applied on this peer.
    #[zbus(signal)]
    pub async fn revision_applied(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        revision_id: &str,
    ) -> zbus::Result<()>;
}
