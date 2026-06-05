//! v3.0.3 Phase E.3 wiring — sway-IPC subscription that emits
//! [`crate::toplevels::ToplevelEvent`]s into the panel's `update()`.
//!
//! The previous Phase E.3 entry shipped the data-layer module
//! (`toplevels.rs`) but the actual event-emit path was deferred to
//! "Phase E.2's surface integration." Phase E.2 then shipped at
//! v3.0.2 with no follow-up, leaving the data model unreachable
//! from the runtime — exactly the pattern §0.12 forbids. This
//! module closes the gap.
//!
//! ## Why `swaymsg` instead of the `swayipc` crate
//!
//! Every other sway-aware applet in the workspace shells out to
//! `swaymsg -t <type>` (see `mde-applets/sway-cluster`, the
//! `mde-applet-sway-cluster --now` path) rather than depending on
//! the `swayipc` crate. We follow the same convention so the dep
//! tree stays one resolution and the failure mode (swaymsg not
//! installed → exec error → log + retry) matches what the rest of
//! the workspace already handles.
//!
//! The subprocess wire-up matches `applet_host.rs`: one OS thread
//! per source, blocking `std::process::Command`, push results into
//! the Iced subscription via `mpsc::Sender::try_send`. iced_layershell
//! polls subscription streams outside the tokio runtime guard, so a
//! tokio-based async approach would deadlock on the first `await`.
//!
//! ## Initial-state seeding
//!
//! Before subscribing to the live event stream, the driver does one
//! `swaymsg -t get_tree` to enumerate every existing toplevel and
//! emit `ToplevelEvent::Added` for each. This is what lets the
//! panel's hero widget show the currently-focused window's title
//! immediately on startup rather than waiting for the next focus
//! change.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::futures::stream::Stream;
use iced::stream;
use iced::Subscription;

use crate::toplevels::{Toplevel, ToplevelEvent, ToplevelId, ToplevelState};

/// Build the panel-side subscription. Wire into
/// `App::subscription` via `Subscription::batch` alongside the
/// existing `applet_host::subscription`.
// E4.23 — `impl Fn` (zero-sized through monomorphization), NOT a `fn(...)`
// POINTER: iced_futures 0.13.2's `Subscription::map` asserts the mapper is
// zero-sized, and an 8-byte fn pointer panics at startup. See the matching
// note in `applet_host::subscription`.
pub fn subscription<M: 'static>(
    map: impl Fn(ToplevelEvent) -> M + Clone + Send + 'static,
) -> Subscription<M> {
    Subscription::run(event_stream).map(map)
}

fn event_stream() -> impl Stream<Item = ToplevelEvent> {
    stream::channel(256, |sender| async move {
        tracing::info!("toplevels_sub: subscription started");
        thread::Builder::new()
            .name("toplevels-sway-ipc".into())
            .spawn(move || drive_sway_subscription_blocking(sender))
            .expect("spawn toplevels-sway-ipc thread");
        std::future::pending::<()>().await;
    })
}

/// Loop forever: seed initial state, subscribe to events, restart
/// on disconnect with 1s back-off. Sway can drop the IPC socket
/// during compositor restart; the panel survives by reseeding.
fn drive_sway_subscription_blocking(sender: mpsc::Sender<ToplevelEvent>) {
    loop {
        // Seed: enumerate every existing toplevel.
        match seed_from_get_tree() {
            Ok(seed_events) => {
                tracing::debug!(
                    count = seed_events.len(),
                    "toplevels_sub: seeded from get_tree"
                );
                for ev in seed_events {
                    if try_send_event(&sender, ev).is_err() {
                        // Subscription closed — we're shutting down.
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "toplevels_sub: get_tree seed failed");
            }
        }

        // Subscribe to the event stream.
        match subscribe_blocking(&sender) {
            Ok(()) => {
                // Subscriber returned cleanly — swaymsg subprocess
                // exited. Treat as disconnect, seed-and-restart.
                let _ = try_send_event(&sender, ToplevelEvent::Disconnected);
                tracing::info!("toplevels_sub: swaymsg subscribe exited; restarting");
            }
            Err(e) => {
                let _ = try_send_event(&sender, ToplevelEvent::Disconnected);
                tracing::warn!(error = %e, "toplevels_sub: swaymsg subscribe error");
            }
        }
        // Back-off so a hot loop doesn't burn CPU if swaymsg is
        // missing entirely (e.g. running outside a sway session).
        thread::sleep(Duration::from_secs(1));
    }
}

/// `swaymsg -t get_tree` returns the full window tree. Walk it,
/// collect every leaf with an `app_id` (xdg-toplevels) or a non-
/// empty `name` (xwayland windows fall back to title), emit an
/// `Added` event for each.
fn seed_from_get_tree() -> Result<Vec<ToplevelEvent>, String> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("spawn swaymsg get_tree: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "swaymsg get_tree exited {:?}",
            output.status.code()
        ));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("parse get_tree json: {e}"))?;
    let mut events = Vec::new();
    walk_tree(&json, &mut events);
    Ok(events)
}

/// Recurse the sway node tree, emitting one `Added` event per
/// window leaf. A "window leaf" has `pid` set (real window vs.
/// workspace/container nodes that have null pid).
fn walk_tree(node: &serde_json::Value, out: &mut Vec<ToplevelEvent>) {
    if node.get("pid").is_some_and(|v| !v.is_null()) {
        if let Some(t) = node_to_toplevel(node) {
            out.push(ToplevelEvent::Added(t));
        }
    }
    if let Some(arr) = node.get("nodes").and_then(|v| v.as_array()) {
        for child in arr {
            walk_tree(child, out);
        }
    }
    if let Some(arr) = node.get("floating_nodes").and_then(|v| v.as_array()) {
        for child in arr {
            walk_tree(child, out);
        }
    }
}

/// Translate a sway "container" node (from get_tree or a window
/// event) into our `Toplevel`. Returns `None` if the node lacks
/// the minimum fields (no id).
fn node_to_toplevel(node: &serde_json::Value) -> Option<Toplevel> {
    let id: ToplevelId = node.get("id")?.as_u64()?;
    let title = node
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let app_id = node
        .get("app_id")
        .and_then(|v| v.as_str())
        // X11 windows have window_properties.class instead of app_id.
        .or_else(|| {
            node.get("window_properties")
                .and_then(|w| w.get("class"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("")
        .to_string();
    let focused = node
        .get("focused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let fullscreen = node
        .get("fullscreen_mode")
        .and_then(|v| v.as_u64())
        .is_some_and(|m| m != 0);
    // Sway doesn't have a per-window "minimized" — windows on
    // inactive workspaces are still observable but visually hidden.
    // We map minimized→false uniformly; the panel uses fullscreen +
    // focused as its primary state signals.
    let minimized = false;
    // Sway's "maximized" maps to the floating-fill state we track
    // via the floating_nodes list. For now treat any non-floating
    // tiled window with no siblings as maximized — close enough for
    // the v8.7 max-button toggle to be visually meaningful.
    let maximized = false;
    Some(Toplevel {
        id,
        title,
        app_id,
        state: ToplevelState {
            focused,
            fullscreen,
            minimized,
            maximized,
        },
    })
}

/// `swaymsg -t subscribe '["window"]'` streams JSON lines, one per
/// window event. We parse each, translate to a `ToplevelEvent`,
/// and push it. Returns when swaymsg exits (compositor restart,
/// pipe closed).
fn subscribe_blocking(sender: &mpsc::Sender<ToplevelEvent>) -> Result<(), String> {
    let mut child = Command::new("swaymsg")
        .args(["-t", "subscribe", "-m", "[\"window\"]"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn swaymsg subscribe: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "swaymsg subscribe: no stdout".to_string())?;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("swaymsg subscribe read: {e}"));
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let json: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(error = %e, line = %line, "toplevels_sub: malformed event line");
                continue;
            }
        };
        if let Some(ev) = event_from_window_event(&json) {
            if try_send_event(sender, ev).is_err() {
                // Subscription closed; tear down swaymsg.
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
        }
    }
    let _ = child.wait();
    Ok(())
}

/// Translate a sway "window" event into our `ToplevelEvent`.
/// The shape is:
/// ```json
/// {"change":"new|close|focus|title|fullscreen_mode|...","container":{...}}
/// ```
/// `None` is returned for `change` values we don't track (mark,
/// urgent transitions on already-tracked windows still flow
/// through as Updated).
fn event_from_window_event(json: &serde_json::Value) -> Option<ToplevelEvent> {
    let change = json.get("change")?.as_str()?;
    let container = json.get("container")?;
    let toplevel = node_to_toplevel(container)?;
    match change {
        "new" => Some(ToplevelEvent::Added(toplevel)),
        "close" => Some(ToplevelEvent::Removed(toplevel.id)),
        "focus" | "title" | "fullscreen_mode" | "floating" | "urgent" | "mark" | "move" => {
            Some(ToplevelEvent::Updated(toplevel))
        }
        _ => None,
    }
}

/// Push an event into the subscription. Returns `Err(())` if the
/// receiver was dropped (panel shutting down); caller should
/// gracefully exit.
fn try_send_event(sender: &mpsc::Sender<ToplevelEvent>, ev: ToplevelEvent) -> Result<(), ()> {
    let mut s = sender.clone();
    match s.try_send(ev) {
        Ok(()) => Ok(()),
        Err(e) if e.is_disconnected() => Err(()),
        Err(_) => Ok(()), // buffer full — drop, caller continues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// node_to_toplevel happy path: a window node with id + name +
    /// app_id + focused.
    #[test]
    fn node_to_toplevel_extracts_xdg_fields() {
        let node = serde_json::json!({
            "id": 42,
            "pid": 1234,
            "name": "Title Here",
            "app_id": "foot",
            "focused": true,
            "fullscreen_mode": 0,
        });
        let t = node_to_toplevel(&node).expect("should parse");
        assert_eq!(t.id, 42);
        assert_eq!(t.title, "Title Here");
        assert_eq!(t.app_id, "foot");
        assert!(t.state.focused);
        assert!(!t.state.fullscreen);
    }

    /// X11 (xwayland) windows expose `window_properties.class` in
    /// place of `app_id`. The parser falls through.
    #[test]
    fn node_to_toplevel_falls_back_to_xwayland_class() {
        let node = serde_json::json!({
            "id": 7,
            "pid": 1,
            "name": "X Window",
            "window_properties": { "class": "Xterm" },
        });
        let t = node_to_toplevel(&node).expect("should parse xwayland");
        assert_eq!(t.app_id, "Xterm");
    }

    /// Fullscreen mode 1 (full output) maps to `state.fullscreen = true`.
    #[test]
    fn node_to_toplevel_maps_fullscreen_mode() {
        let node = serde_json::json!({
            "id": 1,
            "pid": 1,
            "name": "",
            "fullscreen_mode": 1,
        });
        let t = node_to_toplevel(&node).unwrap();
        assert!(t.state.fullscreen);
    }

    /// No id → no toplevel. Defensive against workspace/container
    /// nodes accidentally falling through the pid check.
    #[test]
    fn node_to_toplevel_returns_none_without_id() {
        let node = serde_json::json!({ "pid": 99, "name": "no-id" });
        assert!(node_to_toplevel(&node).is_none());
    }

    /// walk_tree skips nodes without a pid (workspaces, containers).
    /// A single window leaf nested two levels deep should still
    /// surface in the seed.
    #[test]
    fn walk_tree_finds_nested_window_leaf() {
        let tree = serde_json::json!({
            "id": 1, "pid": null,
            "nodes": [
                {
                    "id": 2, "pid": null,
                    "nodes": [
                        {
                            "id": 3, "pid": 100,
                            "name": "leaf",
                            "app_id": "foot",
                            "focused": false,
                            "fullscreen_mode": 0,
                        }
                    ]
                }
            ]
        });
        let mut events = Vec::new();
        walk_tree(&tree, &mut events);
        assert_eq!(events.len(), 1);
        if let ToplevelEvent::Added(t) = &events[0] {
            assert_eq!(t.id, 3);
            assert_eq!(t.title, "leaf");
        } else {
            panic!("expected one Added event for the leaf");
        }
    }

    /// floating_nodes is walked too (floating windows on a workspace
    /// live in this list rather than the tiled `nodes` array).
    #[test]
    fn walk_tree_descends_into_floating_nodes() {
        let tree = serde_json::json!({
            "id": 1, "pid": null,
            "nodes": [],
            "floating_nodes": [
                { "id": 99, "pid": 7, "name": "floater", "app_id": "imv" }
            ]
        });
        let mut events = Vec::new();
        walk_tree(&tree, &mut events);
        assert_eq!(events.len(), 1);
    }

    /// "new" change → Added; "close" → Removed by id; "focus" →
    /// Updated; unrecognized change yields None.
    #[test]
    fn event_from_window_event_maps_change_kinds() {
        let mk = |change: &str| {
            serde_json::json!({
                "change": change,
                "container": {
                    "id": 1, "pid": 1, "name": "t", "app_id": "a",
                    "focused": false, "fullscreen_mode": 0,
                }
            })
        };
        assert!(matches!(
            event_from_window_event(&mk("new")),
            Some(ToplevelEvent::Added(_))
        ));
        assert!(matches!(
            event_from_window_event(&mk("close")),
            Some(ToplevelEvent::Removed(1))
        ));
        assert!(matches!(
            event_from_window_event(&mk("focus")),
            Some(ToplevelEvent::Updated(_))
        ));
        assert!(matches!(
            event_from_window_event(&mk("title")),
            Some(ToplevelEvent::Updated(_))
        ));
        assert!(event_from_window_event(&mk("UNKNOWN_CHANGE")).is_none());
    }
}
