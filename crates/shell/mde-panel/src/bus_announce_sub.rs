//! E4.21 — re-homes the BUS-2.5 urgent-theater spawner that the retired
//! `mde-portal` Dock used to own (`app.rs::spawn_urgent_theater`, fed by
//! `workspace::bus_announce_subscription`). Polls the `fleet/announce`
//! Bus topic and, on a new `priority=urgent` segment, spawns the
//! `mde-popover urgent` full-screen theater — title / body / action
//! buttons handed over via `MDE_URGENT_*` env vars (the surface reads
//! them back and dispatches each action URL through `mde open-uri`).
//!
//! ## Why a blocking OS thread (not the portal's tokio poll)
//!
//! The retired portal polled with `tokio::time::sleep` inside an
//! `async_stream`. That pattern **deadlocks** under iced_layershell,
//! which polls subscription streams *outside* the tokio runtime guard
//! (see `toplevels_sub.rs`). So this port matches the panel's proven
//! idiom: `iced::stream::channel` feeding one dedicated OS thread that
//! does pure-`std` file polling (`std::thread::sleep` + `std::fs`). No
//! `tokio::time` / `async_stream` / `chrono` — the panel dep tree stays
//! one resolution.
//!
//! ## Lean port
//!
//! The portal also rendered `high`/`default` segments as Dock
//! breadcrumbs + ack-cards; those were Dock UI and retired *with* the
//! Dock. Only the standalone `urgent` theater is a separable surface, so
//! this module carries just what the spawn needs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::futures::stream::Stream;
use iced::stream;
use iced::Subscription;

/// BUS-2.2.a poll cadence for the announce topic dir — matches the
/// retired portal's 500 ms file-poll.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Cap the action buttons handed to the theater (the `mde-popover
/// urgent` surface re-caps at the same `MAX_URGENT_ACTIONS = 5` per the
/// §9 lock; we bound the env var to match).
const MAX_BUS_ACTIONS: usize = 5;

/// Drop half the seen-ULID set once it grows past this, so a long-lived
/// panel session doesn't accumulate the set unboundedly.
const SEEN_CAP: usize = 1000;

/// A `priority=urgent` `fleet/announce` segment, reduced to exactly what
/// the theater spawn needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrgentAlert {
    pub title: Option<String>,
    pub body: Option<String>,
    /// `(label, url)` action buttons → `MDE_URGENT_ACTIONS` JSON.
    pub actions: Vec<(String, String)>,
}

/// `~/.local/share/mde/bus/fleet/announce` — the BUS-1.4 on-disk dir for
/// the `fleet/announce` topic. Mirrors `mde_bus::default_data_dir()`
/// joined with the topic path; resolved here with `dirs` (already a
/// panel dep) so the panel needn't pull the whole `mde-bus` crate for
/// one path join.
#[must_use]
pub fn announce_topic_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("mde").join("bus").join("fleet").join("announce"))
}

/// Build the panel-side urgent-theater subscription. Wire into
/// `App::subscription` via `Subscription::batch`.
///
/// The mapper is a generic `impl Fn` (zero-sized through monomorphization),
/// NOT a `fn(...)` pointer — iced_futures 0.13.2's `Subscription::map`
/// asserts the mapper is zero-sized, and an 8-byte fn pointer panics at
/// startup (the E4.23 panel-boot regression; see `applet_host`).
pub fn subscription<M: 'static>(
    map: impl Fn(UrgentAlert) -> M + Clone + Send + 'static,
) -> Subscription<M> {
    Subscription::run(event_stream).map(map)
}

fn event_stream() -> impl Stream<Item = UrgentAlert> {
    stream::channel(64, |sender| async move {
        tracing::info!("bus_announce_sub: urgent-theater subscription started");
        thread::Builder::new()
            .name("mde-panel-bus-announce".into())
            .spawn(move || drive_announce_poll_blocking(&sender))
            .expect("spawn mde-panel-bus-announce thread");
        // The blocking thread owns the cloned sender; this async closure
        // just keeps the channel stream alive for iced's runtime.
        std::future::pending::<()>().await;
    })
}

/// Poll `fleet/announce` forever. On the FIRST poll, seed the seen-set
/// with every existing ULID **without** emitting — otherwise a panel
/// restart would replay every historical urgent as a fresh theater. Only
/// segments that appear *after* startup raise the theater.
fn drive_announce_poll_blocking(sender: &mpsc::Sender<UrgentAlert>) {
    let topic_dir = match announce_topic_dir() {
        Some(p) => p,
        None => {
            tracing::warn!("bus_announce_sub: no data dir; urgent theater disabled");
            return;
        }
    };

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut first_poll = true;

    loop {
        thread::sleep(POLL_INTERVAL);

        let entries = match std::fs::read_dir(&topic_dir) {
            Ok(e) => e,
            // Topic dir not created yet (no announce traffic) — retry.
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(ulid) = name.strip_suffix(".json") else {
                continue;
            };
            if seen.contains(ulid) {
                continue;
            }
            seen.insert(ulid.to_string());

            // First poll only records what's already on disk; it never
            // spawns. After that, a new urgent ULID raises the theater.
            if first_poll {
                continue;
            }
            if let Some(alert) = parse_urgent_announce(&path) {
                if try_send_alert(sender, alert).is_err() {
                    // Subscription dropped (panel shutting down).
                    return;
                }
            }
        }

        first_poll = false;

        if seen.len() > SEEN_CAP {
            let drop_count = seen.len() / 2;
            let to_drop: Vec<String> = seen.iter().take(drop_count).cloned().collect();
            for ulid in to_drop {
                seen.remove(&ulid);
            }
        }
    }
}

/// Parse a BUS-1.4 StoredMessage JSON envelope from `fleet/announce` and
/// return `Some(UrgentAlert)` **only** when `priority == "urgent"`. Any
/// other priority (or malformed input) → `None`, so non-urgent traffic
/// on the topic is silently ignored without crashing the poll.
fn parse_urgent_announce(path: &Path) -> Option<UrgentAlert> {
    let raw = std::fs::read_to_string(path).ok()?;
    let outer: serde_json::Value = serde_json::from_str(&raw).ok()?;

    let priority = outer
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    if priority != "urgent" {
        return None;
    }

    let title = outer
        .get("title")
        .and_then(|v| v.as_str())
        .map(String::from);
    let body = outer.get("body").and_then(|v| v.as_str()).map(String::from);

    // BUS-2.7.c — lift the optional action buttons. Missing / non-array
    // → none; entries missing label or url are skipped; capped at
    // MAX_BUS_ACTIONS.
    let actions = outer
        .get("actions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    let label = a.get("label").and_then(|v| v.as_str())?.to_string();
                    let url = a.get("url").and_then(|v| v.as_str())?.to_string();
                    Some((label, url))
                })
                .take(MAX_BUS_ACTIONS)
                .collect()
        })
        .unwrap_or_default();

    Some(UrgentAlert {
        title,
        body,
        actions,
    })
}

/// Non-blocking send into the iced subscription channel. Mirrors
/// `toplevels_sub::try_send_event`: a disconnected channel means the
/// panel is shutting down (`Err`); a full buffer drops the alert and
/// continues (`Ok`).
fn try_send_alert(sender: &mpsc::Sender<UrgentAlert>, alert: UrgentAlert) -> Result<(), ()> {
    let mut s = sender.clone();
    match s.try_send(alert) {
        Ok(()) => Ok(()),
        Err(e) if e.is_disconnected() => Err(()),
        Err(_) => Ok(()),
    }
}

/// BUS-2.5 — spawn the full-screen urgent theater (`mde-popover urgent`)
/// for an `urgent` alert, handing title + body + actions via env vars
/// (mirrors the WM-3 / icon-mapper hand-off the retired portal used).
/// Best-effort: a failed spawn is logged, never fatal. The child is
/// reaped on a detached thread so no zombie accumulates and the GUI
/// thread never blocks waiting on a surface that lives until dismissed.
pub fn spawn_urgent_theater(alert: &UrgentAlert) {
    let mut cmd = std::process::Command::new("mde-popover");
    cmd.arg("urgent");
    if let Some(title) = &alert.title {
        cmd.env("MDE_URGENT_TITLE", title);
    }
    if let Some(body) = &alert.body {
        cmd.env("MDE_URGENT_BODY", body);
    }
    if !alert.actions.is_empty() {
        let arr = serde_json::Value::Array(
            alert
                .actions
                .iter()
                .map(|(label, url)| serde_json::json!({ "label": label, "url": url }))
                .collect(),
        );
        cmd.env("MDE_URGENT_ACTIONS", arr.to_string());
    }
    match cmd.spawn() {
        Ok(mut child) => {
            // Detached reap — the theater stays up until dismissed; we
            // must not block the GUI thread on it.
            let _ = thread::Builder::new()
                .name("mde-urgent-theater-reap".into())
                .spawn(move || {
                    let _ = child.wait();
                });
        }
        Err(e) => {
            tracing::debug!("bus_announce_sub: failed to spawn mde-popover urgent theater: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_msg(dir: &Path, ulid: &str, json: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{ulid}.json")), json).unwrap();
    }

    #[test]
    fn parse_lifts_urgent_with_title_body_actions() {
        let tmp = std::env::temp_dir().join("mde-panel-busann-test-urgent");
        let _ = std::fs::remove_dir_all(&tmp);
        write_msg(
            &tmp,
            "01HZURGENT",
            r#"{"priority":"urgent","title":"Disk full","body":"node-3 root at 99%","actions":[{"label":"Open","url":"mde://control"}]}"#,
        );
        let got = parse_urgent_announce(&tmp.join("01HZURGENT.json")).unwrap();
        assert_eq!(got.title.as_deref(), Some("Disk full"));
        assert_eq!(got.body.as_deref(), Some("node-3 root at 99%"));
        assert_eq!(
            got.actions,
            vec![("Open".to_string(), "mde://control".to_string())]
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parse_rejects_non_urgent_priority() {
        let tmp = std::env::temp_dir().join("mde-panel-busann-test-high");
        let _ = std::fs::remove_dir_all(&tmp);
        write_msg(
            &tmp,
            "01HZHIGH",
            r#"{"priority":"high","title":"FYI","body":"not a takeover"}"#,
        );
        assert!(parse_urgent_announce(&tmp.join("01HZHIGH.json")).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parse_rejects_malformed_json() {
        let tmp = std::env::temp_dir().join("mde-panel-busann-test-bad");
        let _ = std::fs::remove_dir_all(&tmp);
        write_msg(&tmp, "01HZBAD", "{not json");
        assert!(parse_urgent_announce(&tmp.join("01HZBAD.json")).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parse_caps_actions_at_five() {
        let tmp = std::env::temp_dir().join("mde-panel-busann-test-cap");
        let _ = std::fs::remove_dir_all(&tmp);
        let actions: Vec<String> = (0..8)
            .map(|i| format!(r#"{{"label":"a{i}","url":"mde://x{i}"}}"#))
            .collect();
        write_msg(
            &tmp,
            "01HZCAP",
            &format!(
                r#"{{"priority":"urgent","actions":[{}]}}"#,
                actions.join(",")
            ),
        );
        let got = parse_urgent_announce(&tmp.join("01HZCAP.json")).unwrap();
        assert_eq!(got.actions.len(), MAX_BUS_ACTIONS);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn announce_dir_ends_with_topic_path() {
        // Path resolution mirrors mde_bus::default_data_dir() + topic.
        if let Some(p) = announce_topic_dir() {
            assert!(p.ends_with("mde/bus/fleet/announce"));
        }
    }
}
