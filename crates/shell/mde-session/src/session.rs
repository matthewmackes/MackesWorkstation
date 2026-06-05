//! Session lifecycle control surface (DBUS-1 — migrated to Bus).
//!
//! Per the Q96 Bus-canonical lock (EPIC-RETIRE-DBUS), the session
//! lifecycle verbs (logout / restart / shutdown / lock) are served on
//! the Bus at `action/session/<verb>` instead of the retired
//! `dev.mackes.MDE.Session` D-Bus interface. The `mde-logout-dialog` +
//! panel publish a request on the action topic; this responder applies
//! the effect and replies on `reply/<ulid>`.
//!
//! The verb → action mapping ([`action_for_verb`]) is a pure function
//! (unit-tested); [`apply`] performs the side effects (kill / systemctl
//! / lock), which are exercised at the §0.15 HW bench.
//!
//! E0.16 — the `save-layout` verb was retired: it ran `swaymsg -t
//! get_tree`, which labwc does not provide, and wlr-foreign-toplevel
//! exposes no window geometry to reconstruct a layout from. No surface
//! produced `action/session/save-layout`, so the verb was dropped rather
//! than left as an unimplementable stub (§3).

use std::collections::HashMap;

use mde_bus::hooks::config::Priority;
use mde_bus::persist::Persist;
use mde_bus::rpc::reply_topic;
use serde_json::json;

/// Poll cadence for the action topics.
pub const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(400);

/// The lifecycle verbs served on `action/session/<verb>`.
pub const ACTION_VERBS: [&str; 4] = ["logout", "restart", "shutdown", "lock"];

/// A session lifecycle action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Logout,
    Restart,
    Shutdown,
    Lock,
}

/// Map an `action/session/<verb>` verb to its action. Pure + testable.
#[must_use]
pub fn action_for_verb(verb: &str) -> Option<SessionAction> {
    match verb {
        "logout" => Some(SessionAction::Logout),
        "restart" => Some(SessionAction::Restart),
        "shutdown" => Some(SessionAction::Shutdown),
        "lock" => Some(SessionAction::Lock),
        _ => None,
    }
}

/// Apply a lifecycle action (the side-effecting half).
///
/// # Errors
/// Returns a message when the effect's shell-out / IO fails.
pub async fn apply(action: SessionAction) -> Result<(), String> {
    match action {
        SessionAction::Logout => {
            tracing::info!("session: logout");
            // SIGTERM our own PID; systemd's graphical-session.target
            // tear-down handles the rest. (No `unsafe`: shell out to kill.)
            let pid = std::process::id();
            let _ = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();
            Ok(())
        }
        SessionAction::Restart => {
            tracing::info!("session: restart");
            run("systemctl", &["reboot"]).await
        }
        SessionAction::Shutdown => {
            tracing::info!("session: shutdown");
            run("systemctl", &["poweroff"]).await
        }
        SessionAction::Lock => {
            tracing::info!("session: lock");
            crate::lock::run_lock_command()
                .await
                .map_err(|e| format!("lock command failed: {e}"))
        }
    }
}

async fn run(bin: &str, args: &[&str]) -> Result<(), String> {
    let status = tokio::process::Command::new(bin)
        .args(args)
        .status()
        .await
        .map_err(|e| format!("spawn {bin}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{bin} exited {}", status.code().unwrap_or(-1)))
    }
}

/// Run the Bus responder loop on the current thread, building a local
/// tokio runtime for the async effects (`Persist`/rusqlite isn't `Send`,
/// so this runs off the main async executor — see `mde-session` main).
/// Loops until `should_stop()` returns true.
pub fn serve_bus<F: Fn() -> bool>(persist: &Persist, should_stop: F) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!("session responder: runtime build failed: {e}");
            return;
        }
    };
    let mut cursors: HashMap<String, String> = HashMap::new();
    while !should_stop() {
        poll_once(persist, &rt, &mut cursors);
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// One poll sweep across the action verbs (split out so a test can drive
/// it without the sleep loop).
pub fn poll_once(
    persist: &Persist,
    rt: &tokio::runtime::Runtime,
    cursors: &mut HashMap<String, String>,
) {
    for verb in ACTION_VERBS {
        let topic = format!("action/session/{verb}");
        let since = cursors.get(&topic).map(String::as_str);
        let msgs = match persist.list_since(&topic, since) {
            Ok(m) => m,
            Err(_) => continue,
        };
        for msg in msgs {
            cursors.insert(topic.clone(), msg.ulid.clone());
            let reply = match action_for_verb(verb) {
                Some(action) => match rt.block_on(apply(action)) {
                    Ok(()) => json!({ "ok": true }).to_string(),
                    Err(e) => json!({ "ok": false, "error": e }).to_string(),
                },
                None => json!({ "ok": false, "error": "unknown verb" }).to_string(),
            };
            let _ = persist.write(
                &reply_topic(&msg.ulid),
                Priority::Default,
                None,
                Some(&reply),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verb_mapping_covers_all_and_rejects_unknown() {
        assert_eq!(action_for_verb("logout"), Some(SessionAction::Logout));
        assert_eq!(action_for_verb("restart"), Some(SessionAction::Restart));
        assert_eq!(action_for_verb("shutdown"), Some(SessionAction::Shutdown));
        assert_eq!(action_for_verb("lock"), Some(SessionAction::Lock));
        assert_eq!(action_for_verb("frobnicate"), None);
    }

    #[test]
    fn save_layout_verb_is_retired() {
        // E0.16 — the sway-tree save-layout verb was dropped under labwc;
        // it must no longer map to an action nor appear in the verb set.
        assert_eq!(action_for_verb("save-layout"), None);
        assert!(!ACTION_VERBS.contains(&"save-layout"));
    }
}
