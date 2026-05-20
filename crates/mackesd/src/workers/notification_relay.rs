//! v2.0.0 Phase B.9 — cross-peer notification relay.
//!
//! Watches `~/QNM-Shared/<peer>/.qnm-notifications/` for new
//! notification JSON files written by other peers, parses them, and
//! inserts each as a row in the local `notifications` table with
//! `origin_peer_id` set. The local `NotificationsService` (Phase
//! B.10) then surfaces them through the standard `org.freedesktop.
//! Notifications` signals so the panel's bell tray + drawer pick
//! them up alongside locally-emitted notifications.
//!
//! Polling vs inotify: this worker polls every 5 s rather than
//! using inotify because `~/QNM-Shared/` is a sshfs mount on
//! several peers and inotify events on FUSE mounts aren't always
//! delivered. The poll cadence is cheap (we stat each peer dir's
//! mtime first, only scan when it bumps).

#![cfg(feature = "async-services")]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Mutex;

use super::{ShutdownToken, Worker};

/// Cadence at which the worker rescans every peer's
/// `.qnm-notifications/` directory.
pub const TICK_INTERVAL_S: u64 = 5;

/// One notification JSON file written by a remote peer. Shape
/// matches what `mesh_notifications.py` writes today; the local
/// `notifications` table stores a superset.
#[derive(Debug, Clone, Deserialize)]
struct MirroredNotification {
    /// Stable id assigned by the originating peer's notifier.
    #[serde(default)]
    pub source_id: i64,
    /// App that emitted the notification.
    pub app: String,
    /// Notification title.
    pub title: String,
    /// Free-form body. Optional — defaults to empty.
    #[serde(default)]
    pub body: String,
    /// urgency level (0=low, 1=normal, 2=critical).
    #[serde(default = "default_urgency")]
    pub urgency: u8,
}

const fn default_urgency() -> u8 {
    1
}

/// Worker driving the relay loop.
pub struct NotificationRelayWorker {
    qnm_root: PathBuf,
    conn: Arc<Mutex<rusqlite::Connection>>,
    /// Source ids we've already imported, keyed by (peer, source_id),
    /// so a polling tick that sees an already-seen file doesn't
    /// double-insert.
    seen: HashSet<(String, i64)>,
}

impl NotificationRelayWorker {
    /// Construct a worker pinned to the given QNM-Shared root and
    /// backing SQLite connection.
    #[must_use]
    pub fn new(qnm_root: PathBuf, conn: rusqlite::Connection) -> Self {
        Self {
            qnm_root,
            conn: Arc::new(Mutex::new(conn)),
            seen: HashSet::new(),
        }
    }

    /// Open the store at the default path and construct a worker.
    ///
    /// # Errors
    /// Returns whatever `store::open` returns.
    pub fn open_default() -> crate::Result<Self> {
        let qnm_root = crate::default_qnm_shared_root();
        let conn = crate::store::open(&crate::default_db_path())?;
        Ok(Self::new(qnm_root, conn))
    }
}

#[async_trait::async_trait]
impl Worker for NotificationRelayWorker {
    fn name(&self) -> &'static str {
        "notification-relay"
    }

    async fn run(&mut self, mut shutdown: ShutdownToken) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.wait() => return Ok(()),
                _ = tokio::time::sleep(Duration::from_secs(TICK_INTERVAL_S)) => {}
            }
            if let Err(e) = self.tick().await {
                tracing::warn!("notification-relay tick failed: {e}");
            }
        }
    }
}

impl NotificationRelayWorker {
    /// One scan cycle. Walks every direct child of the QNM-Shared
    /// root, looking for `<child>/.qnm-notifications/*.json`, and
    /// imports any source_id we haven't seen yet.
    async fn tick(&mut self) -> anyhow::Result<()> {
        let Ok(entries) = std::fs::read_dir(&self.qnm_root) else {
            return Ok(());
        };
        let mut to_import: Vec<(String, MirroredNotification)> = Vec::new();
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let peer = entry.file_name().to_string_lossy().into_owned();
            let notif_dir = entry.path().join(".qnm-notifications");
            if !notif_dir.is_dir() {
                continue;
            }
            let Ok(files) = std::fs::read_dir(&notif_dir) else {
                continue;
            };
            for file in files.flatten() {
                let path = file.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(n) = serde_json::from_str::<MirroredNotification>(&text) else {
                    continue;
                };
                if self.seen.contains(&(peer.clone(), n.source_id)) {
                    continue;
                }
                to_import.push((peer.clone(), n));
            }
        }
        if to_import.is_empty() {
            return Ok(());
        }
        let now = chrono::Utc::now().to_rfc3339();
        let guard = self.conn.lock().await;
        for (peer, n) in &to_import {
            let _ = guard.execute(
                "INSERT INTO notifications \
                 (sender, summary, body, hints_json, urgency, created_at, \
                  origin_peer_id) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                (
                    &n.app,
                    &n.title,
                    &n.body,
                    "{}",
                    i64::from(n.urgency),
                    &now,
                    peer,
                ),
            );
        }
        drop(guard);
        for (peer, n) in to_import {
            self.seen.insert((peer, n.source_id));
        }
        Ok(())
    }
}

/// Pure helper: parse one mirrored notification file from a string.
/// Lifted out for unit tests so the parser is covered without a
/// real disk hit.
///
/// # Errors
/// Returns serde error when the input isn't valid JSON or doesn't
/// match the expected schema.
pub fn parse_mirrored(text: &str) -> Result<MirroredEntry, serde_json::Error> {
    let parsed: MirroredNotification = serde_json::from_str(text)?;
    Ok(MirroredEntry {
        source_id: parsed.source_id,
        app: parsed.app,
        title: parsed.title,
        body: parsed.body,
        urgency: parsed.urgency,
    })
}

/// Public projection of a mirrored notification. Mirrors the
/// internal struct but with `Debug + Clone + PartialEq + Eq` so
/// test assertions are clean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirroredEntry {
    /// Stable id assigned by the originating peer's notifier.
    pub source_id: i64,
    /// App that emitted the notification.
    pub app: String,
    /// Notification title.
    pub title: String,
    /// Free-form body.
    pub body: String,
    /// Urgency 0..=2.
    pub urgency: u8,
}

/// Inspect [`MirroredEntry`] for the seen-set key the relay uses
/// when deduping per-peer files. Lifted as a pure helper so the
/// key shape is testable.
#[must_use]
pub fn seen_key(peer: &str, entry: &MirroredEntry) -> (String, i64) {
    (peer.to_owned(), entry.source_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mirrored_round_trips_full_payload() {
        let text = r#"{
            "source_id": 42,
            "app":       "weather",
            "title":     "Rain in 30 min",
            "body":      "Pack an umbrella",
            "urgency":   2
        }"#;
        let entry = parse_mirrored(text).expect("parse");
        assert_eq!(
            entry,
            MirroredEntry {
                source_id: 42,
                app: "weather".into(),
                title: "Rain in 30 min".into(),
                body: "Pack an umbrella".into(),
                urgency: 2,
            }
        );
    }

    #[test]
    fn parse_mirrored_uses_defaults_for_missing_body_and_urgency() {
        let text = r#"{"source_id": 7, "app": "x", "title": "T"}"#;
        let entry = parse_mirrored(text).expect("parse");
        assert_eq!(entry.body, "");
        assert_eq!(entry.urgency, 1);
    }

    #[test]
    fn parse_mirrored_rejects_missing_required_field() {
        // No `title` — required.
        let text = r#"{"source_id": 1, "app": "x"}"#;
        assert!(parse_mirrored(text).is_err());
    }

    #[test]
    fn seen_key_pairs_peer_with_source_id() {
        let entry = MirroredEntry {
            source_id: 99,
            app: "x".into(),
            title: "t".into(),
            body: "".into(),
            urgency: 1,
        };
        assert_eq!(seen_key("peer:anvil", &entry), ("peer:anvil".into(), 99));
    }

    #[tokio::test]
    async fn relay_worker_name_matches_phase_b_lock() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let conn = crate::store::open_in_memory().expect("open");
        let w = NotificationRelayWorker::new(tmp.path().to_path_buf(), conn);
        assert_eq!(w.name(), "notification-relay");
    }

    #[tokio::test]
    async fn relay_tick_imports_new_notifications_then_dedupes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let conn = crate::store::open_in_memory().expect("open");
        let mut w = NotificationRelayWorker::new(tmp.path().to_path_buf(), conn);
        let peer_dir = tmp.path().join("peer:anvil").join(".qnm-notifications");
        std::fs::create_dir_all(&peer_dir).expect("mkdir");
        std::fs::write(
            peer_dir.join("1.json"),
            r#"{"source_id": 1, "app": "a", "title": "first"}"#,
        )
        .expect("write");

        w.tick().await.expect("first tick");

        let count: i64 = {
            let guard = w.conn.lock().await;
            guard
                .query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))
                .expect("count")
        };
        assert_eq!(count, 1);

        // Second tick with no new files: no double-insert.
        w.tick().await.expect("second tick");
        let count: i64 = {
            let guard = w.conn.lock().await;
            guard
                .query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))
                .expect("count")
        };
        assert_eq!(count, 1, "duplicate source_id must not insert again");

        // Add a fresh file with a new source_id; next tick imports it.
        std::fs::write(
            peer_dir.join("2.json"),
            r#"{"source_id": 2, "app": "a", "title": "second"}"#,
        )
        .expect("write");
        w.tick().await.expect("third tick");
        let count: i64 = {
            let guard = w.conn.lock().await;
            guard
                .query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))
                .expect("count")
        };
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn relay_tick_skips_non_json_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let conn = crate::store::open_in_memory().expect("open");
        let mut w = NotificationRelayWorker::new(tmp.path().to_path_buf(), conn);
        let peer_dir = tmp.path().join("peer:cedar").join(".qnm-notifications");
        std::fs::create_dir_all(&peer_dir).expect("mkdir");
        std::fs::write(peer_dir.join("readme.txt"), "not json").expect("w1");
        std::fs::write(peer_dir.join("bad.json"), "{not json").expect("w2");

        w.tick().await.expect("tick");
        let count: i64 = {
            let guard = w.conn.lock().await;
            guard
                .query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))
                .expect("count")
        };
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn relay_tick_skips_peers_without_notifications_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let conn = crate::store::open_in_memory().expect("open");
        let mut w = NotificationRelayWorker::new(tmp.path().to_path_buf(), conn);
        // peer:lonely exists but has no .qnm-notifications dir.
        std::fs::create_dir_all(tmp.path().join("peer:lonely")).expect("mkdir");
        // Should not crash.
        w.tick().await.expect("tick");
    }

    #[tokio::test]
    async fn relay_tick_handles_missing_qnm_root() {
        let conn = crate::store::open_in_memory().expect("open");
        let mut w = NotificationRelayWorker::new(std::path::PathBuf::from("/does/not/exist"), conn);
        // Should return Ok cleanly when the root is missing.
        w.tick().await.expect("tick");
    }
}
