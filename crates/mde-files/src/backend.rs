//! Backend trait — v2.0.0 Phase 2.1.
//!
//! Abstracts the data + operation surface MDE Files renders. Two
//! concrete implementations ship:
//!
//!   * [`DemoBackend`] (Phase 2.2) — wraps the existing const
//!     `demo_data::*` tables so the UI renders without a live
//!     mded connection. Used for headless tests + the "panel
//!     boots in 200 ms on a fresh login" smoke gate.
//!   * `DBusBackend` (Phase 2.3, gated behind the `dbus` feature)
//!     — talks to `dev.mackes.MDE.Files` over zbus. Lands when the
//!     mded matching surface ships (Phase 2.4).
//!
//! The trait is sync + non-blocking — Iced calls each method from
//! its `view()` / `update()` callbacks, both of which run on the
//! GUI thread. Real I/O (network, disk) lives behind futures that
//! the `DBusBackend` returns; today's `DemoBackend` is in-memory so
//! every call is constant-time.

use std::path::PathBuf;

use crate::model::{FileRow, Peer, SelfNode};

/// Stable identifier for a long-running transfer operation. Iced
/// renders the transfer drawer keyed by this.
pub type OpId = u64;

/// One destination of a Send-To. Per the Phase 3.x spec, a
/// destination is either a single peer, a peer group, a role, or a
/// "site" (a region). Today the demo backend only supports
/// per-peer destinations; the richer selectors land with the DBus
/// backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// One named peer.
    Peer(String),
    /// Peer group by name.
    Group(String),
    /// All peers carrying the given role.
    Role(String),
    /// All peers in the given region/site.
    Site(String),
}

/// Send-To mode per Phase 3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    Copy,
    Move,
    Sync,
    Deploy,
    Stage,
}

/// Conflict policy per Phase 3.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    Ask,
    Skip,
    Overwrite,
    Rename,
}

/// One audit-log row from the operation history (Phase 2.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub op_id: OpId,
    pub kind: &'static str,
    pub source: PathBuf,
    pub destination: Destination,
    pub mode: SendMode,
    pub bytes: u64,
    pub at_ms: i64,
    pub ok: bool,
}

/// Surface every backend implements. Pure abstraction over data +
/// operations — Iced never reaches past this trait.
pub trait Backend {
    /// Self-identity. Iced surfaces this in the breadcrumb + the
    /// header pill ("you are peer:anvil").
    fn self_node(&self) -> SelfNode;
    /// Mesh roster. Sidebar + Send-To picker iterate this.
    fn peers(&self) -> Vec<Peer>;
    /// Files visible under a path. Empty path = the mesh overview.
    fn list(&self, path: &str) -> Vec<FileRow>;
    /// Audit history (newest first).
    fn audit_log(&self) -> Vec<AuditEntry>;
    /// Fire a Send-To. Demo backend records the audit row +
    /// returns a synthetic op id immediately; DBus backend
    /// returns once mded has accepted the request.
    fn send_to(
        &mut self,
        sources: &[PathBuf],
        destination: Destination,
        mode: SendMode,
        conflict: ConflictPolicy,
    ) -> Result<OpId, BackendError>;
    /// Roll back a completed operation (Phase 2.7). Returns the
    /// new audit row's op id.
    fn rollback(&mut self, op_id: OpId) -> Result<OpId, BackendError>;
}

/// Backend-surface errors. Surfaced to the UI as toasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// Source file doesn't exist.
    SourceMissing(PathBuf),
    /// Destination unknown / unreachable.
    DestinationUnreachable(Destination),
    /// Operation rejected by validation (Phase 2.5 path-safety, etc).
    Rejected(String),
    /// Op id not in history (rollback).
    NotFound(OpId),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceMissing(p) => write!(f, "source missing: {}", p.display()),
            Self::DestinationUnreachable(d) => write!(f, "destination unreachable: {d:?}"),
            Self::Rejected(reason) => write!(f, "rejected: {reason}"),
            Self::NotFound(id) => write!(f, "op {id} not found"),
        }
    }
}

impl std::error::Error for BackendError {}

/// v2.0.0 Phase 2.2 — in-memory `Backend` impl wrapping the demo
/// constants. Used for headless tests + the panel-boot smoke gate.
pub struct DemoBackend {
    next_op_id: OpId,
    audit: Vec<AuditEntry>,
}

impl Default for DemoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoBackend {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_op_id: 1,
            audit: Vec::new(),
        }
    }

    fn alloc_id(&mut self) -> OpId {
        let id = self.next_op_id;
        self.next_op_id += 1;
        id
    }
}

impl Backend for DemoBackend {
    fn self_node(&self) -> SelfNode {
        crate::demo_data::SELF_NODE
    }

    fn peers(&self) -> Vec<Peer> {
        crate::demo_data::PEERS.to_vec()
    }

    fn list(&self, path: &str) -> Vec<FileRow> {
        match path {
            "" | "/" => crate::demo_data::INBOX.to_vec(),
            "downloads" => crate::demo_data::DOWNLOADS.to_vec(),
            "peer:pine" => crate::demo_data::PINE_FILES.to_vec(),
            "peer:birch" => crate::demo_data::BIRCH_FILES.to_vec(),
            "peer:oak" => crate::demo_data::OAK_FILES.to_vec(),
            _ => Vec::new(),
        }
    }

    fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit.iter().rev().cloned().collect()
    }

    fn send_to(
        &mut self,
        sources: &[PathBuf],
        destination: Destination,
        mode: SendMode,
        _conflict: ConflictPolicy,
    ) -> Result<OpId, BackendError> {
        if sources.is_empty() {
            return Err(BackendError::Rejected("empty source list".into()));
        }
        let id = self.alloc_id();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let total_bytes: u64 = sources
            .iter()
            .filter_map(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();
        self.audit.push(AuditEntry {
            op_id: id,
            kind: "send_to",
            source: sources[0].clone(),
            destination,
            mode,
            bytes: total_bytes,
            at_ms: now_ms,
            ok: true,
        });
        Ok(id)
    }

    fn rollback(&mut self, op_id: OpId) -> Result<OpId, BackendError> {
        let original = self.audit.iter().find(|a| a.op_id == op_id).cloned();
        let Some(original) = original else {
            return Err(BackendError::NotFound(op_id));
        };
        let id = self.alloc_id();
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.audit.push(AuditEntry {
            op_id: id,
            kind: "rollback",
            source: original.source.clone(),
            destination: original.destination.clone(),
            mode: original.mode,
            bytes: original.bytes,
            at_ms: now_ms,
            ok: true,
        });
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_backend_returns_demo_self_node() {
        let b = DemoBackend::new();
        let self_node = b.self_node();
        let demo = crate::demo_data::SELF_NODE;
        assert_eq!(self_node.host, demo.host);
    }

    #[test]
    fn demo_backend_peers_match_demo_data() {
        let b = DemoBackend::new();
        assert_eq!(b.peers().len(), crate::demo_data::PEERS.len());
    }

    #[test]
    fn demo_backend_list_returns_inbox_for_empty_path() {
        let b = DemoBackend::new();
        let rows = b.list("");
        assert_eq!(rows.len(), crate::demo_data::INBOX.len());
    }

    #[test]
    fn demo_backend_list_returns_per_peer_files() {
        let b = DemoBackend::new();
        assert!(!b.list("peer:pine").is_empty());
        assert!(!b.list("peer:birch").is_empty());
        assert!(!b.list("peer:oak").is_empty());
    }

    #[test]
    fn demo_backend_list_returns_empty_for_unknown_path() {
        let b = DemoBackend::new();
        assert!(b.list("not-a-real-path").is_empty());
    }

    #[test]
    fn demo_backend_audit_log_starts_empty() {
        let b = DemoBackend::new();
        assert!(b.audit_log().is_empty());
    }

    #[test]
    fn send_to_rejects_empty_source_list() {
        let mut b = DemoBackend::new();
        let r = b.send_to(
            &[],
            Destination::Peer("pine".into()),
            SendMode::Copy,
            ConflictPolicy::Ask,
        );
        assert!(matches!(r, Err(BackendError::Rejected(_))));
    }

    #[test]
    fn send_to_records_audit_row_and_returns_increasing_op_ids() {
        let mut b = DemoBackend::new();
        let id1 = b
            .send_to(
                &[PathBuf::from("/tmp/a")],
                Destination::Peer("pine".into()),
                SendMode::Copy,
                ConflictPolicy::Ask,
            )
            .expect("send_to");
        let id2 = b
            .send_to(
                &[PathBuf::from("/tmp/b")],
                Destination::Peer("birch".into()),
                SendMode::Move,
                ConflictPolicy::Overwrite,
            )
            .expect("send_to");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        let log = b.audit_log();
        // newest-first
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].op_id, id2);
        assert_eq!(log[1].op_id, id1);
    }

    #[test]
    fn rollback_records_rollback_audit_row() {
        let mut b = DemoBackend::new();
        let original = b
            .send_to(
                &[PathBuf::from("/tmp/x")],
                Destination::Peer("oak".into()),
                SendMode::Copy,
                ConflictPolicy::Ask,
            )
            .expect("send_to");
        let rb = b.rollback(original).expect("rollback");
        assert_ne!(original, rb);
        let log = b.audit_log();
        // rollback is newest.
        assert_eq!(log[0].op_id, rb);
        assert_eq!(log[0].kind, "rollback");
    }

    #[test]
    fn rollback_unknown_id_returns_not_found() {
        let mut b = DemoBackend::new();
        let r = b.rollback(999);
        assert!(matches!(r, Err(BackendError::NotFound(999))));
    }

    #[test]
    fn backend_error_display_includes_context() {
        let e = BackendError::SourceMissing(PathBuf::from("/missing"));
        assert!(format!("{e}").contains("source missing"));
        assert!(format!("{e}").contains("/missing"));

        let e = BackendError::DestinationUnreachable(Destination::Peer("x".into()));
        assert!(format!("{e}").contains("destination"));

        let e = BackendError::NotFound(7);
        assert!(format!("{e}").contains("7"));
    }
}
