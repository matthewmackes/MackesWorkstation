//! BUS-2.5 urgent-theater spawner — ported into the canonical `mde` shell
//! panel (E0.17 step 1). Polls the `fleet/announce` Bus topic and, on a new
//! `priority=urgent` segment, spawns the `mde-popover urgent` full-screen
//! theater — title / body / action buttons handed over via `MDE_URGENT_*`
//! env vars (the surface reads them back and dispatches each action URL
//! through `mde open-uri`).
//!
//! Provenance: re-homed from the retired `mde-portal` Dock → the `mde-panel`
//! crate (E4.21) → here (E0.17), as the `mde-panel` crate retires. The signal
//! source is the Bus, so this is compositor-agnostic — nothing here touches
//! sway/labwc. `panel.rs` gains the urgent-alert takeover it previously lacked.
//!
//! ## Why a blocking OS thread (not a tokio poll)
//!
//! iced/iced_layershell polls subscription streams *outside* the tokio runtime
//! guard, so a `tokio::time::sleep` inside an `async_stream` deadlocks. This
//! matches the panel's proven idiom: `iced::stream::channel` feeding one
//! dedicated OS thread doing pure-`std` file polling.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::futures::stream::Stream;
use iced::stream;
use iced::Subscription;

/// BUS-2.2.a poll cadence for the announce topic dir.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Cap the action buttons handed to the theater (the `mde-popover urgent`
/// surface re-caps at the same `MAX_URGENT_ACTIONS = 5` per the §9 lock).
const MAX_BUS_ACTIONS: usize = 5;

/// Drop half the seen-ULID set once it grows past this, so a long-lived panel
/// session doesn't accumulate the set unboundedly.
const SEEN_CAP: usize = 1000;

/// A `priority=urgent` `fleet/announce` segment, reduced to exactly what the
/// theater spawn needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrgentAlert {
    pub title: Option<String>,
    pub body: Option<String>,
    /// `(label, url)` action buttons → `MDE_URGENT_ACTIONS` JSON.
    pub actions: Vec<(String, String)>,
}

/// `$XDG_DATA_HOME` (else `~/.local/share`). Resolved with std env so the shell
/// needn't pull `dirs` for one path join (matches `install_win2k`'s pattern).
fn data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
}

/// `~/.local/share/mde/bus/fleet/announce` — the BUS-1.4 on-disk dir for the
/// `fleet/announce` topic (mirrors `mde_bus::default_data_dir()` + the topic).
#[must_use]
pub fn announce_topic_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("mde").join("bus").join("fleet").join("announce"))
}

/// Build the panel-side urgent-theater subscription. Wire into
/// `subscription()` via `Subscription::batch`.
///
/// The mapper must be a zero-sized `impl Fn` (NOT an 8-byte `fn` pointer):
/// iced_futures 0.13's `Subscription::map` asserts the mapper is zero-sized and
/// a fn pointer panics at startup (the E4.23 panel-boot regression).
pub fn subscription<M: 'static>(
    map: impl Fn(UrgentAlert) -> M + Clone + Send + 'static,
) -> Subscription<M> {
    Subscription::run(event_stream).map(map)
}

fn event_stream() -> impl Stream<Item = UrgentAlert> {
    stream::channel(64, |sender| async move {
        eprintln!("urgent_theater: subscription started");
        thread::Builder::new()
            .name("mde-bus-announce".into())
            .spawn(move || drive_announce_poll_blocking(&sender))
            .expect("spawn mde-bus-announce thread");
        // The blocking thread owns the cloned sender; this async closure just
        // keeps the channel stream alive for iced's runtime.
        std::future::pending::<()>().await;
    })
}

/// Poll `fleet/announce` forever. On the FIRST poll, seed the seen-set with
/// every existing ULID **without** emitting — otherwise a panel restart would
/// replay every historical urgent as a fresh theater. Only segments that
/// appear *after* startup raise the theater.
fn drive_announce_poll_blocking(sender: &mpsc::Sender<UrgentAlert>) {
    let topic_dir = match announce_topic_dir() {
        Some(p) => p,
        None => {
            eprintln!("urgent_theater: no data dir; urgent theater disabled");
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

            // First poll only records what's already on disk; never spawns.
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

/// Parse a BUS-1.4 StoredMessage JSON envelope and return `Some(UrgentAlert)`
/// **only** when `priority == "urgent"`. Any other priority (or malformed
/// input) → `None`, so non-urgent traffic is silently ignored.
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

    // BUS-2.7.c — lift the optional action buttons. Missing / non-array → none;
    // entries missing label or url are skipped; capped at MAX_BUS_ACTIONS.
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

/// Non-blocking send into the iced subscription channel. A disconnected channel
/// means the panel is shutting down (`Err`); a full buffer drops the alert and
/// continues (`Ok`).
fn try_send_alert(sender: &mpsc::Sender<UrgentAlert>, alert: UrgentAlert) -> Result<(), ()> {
    let mut s = sender.clone();
    match s.try_send(alert) {
        Ok(()) => Ok(()),
        Err(e) if e.is_disconnected() => Err(()),
        Err(_) => Ok(()),
    }
}

/// BUS-2.5 — spawn the full-screen urgent theater (`mde-popover urgent`) for an
/// `urgent` alert, handing title + body + actions via env vars. Best-effort: a
/// failed spawn is logged, never fatal. The child is reaped on a detached
/// thread so no zombie accumulates and the GUI thread never blocks.
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
            // Detached reap — the theater stays up until dismissed; we must not
            // block the GUI thread on it.
            let _ = thread::Builder::new()
                .name("mde-urgent-theater-reap".into())
                .spawn(move || {
                    let _ = child.wait();
                });
        }
        Err(e) => {
            eprintln!("urgent_theater: failed to spawn mde-popover urgent theater: {e}");
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
        let tmp =
            std::env::temp_dir().join(format!("mde-urgent-test-urgent-{}", std::process::id()));
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
        let tmp = std::env::temp_dir().join(format!("mde-urgent-test-high-{}", std::process::id()));
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
        let tmp = std::env::temp_dir().join(format!("mde-urgent-test-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        write_msg(&tmp, "01HZBAD", "{not json");
        assert!(parse_urgent_announce(&tmp.join("01HZBAD.json")).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parse_caps_actions_at_five() {
        let tmp = std::env::temp_dir().join(format!("mde-urgent-test-cap-{}", std::process::id()));
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
        if let Some(p) = announce_topic_dir() {
            assert!(p.ends_with("mde/bus/fleet/announce"));
        }
    }
}
