//! Lightweight in-process toast surface for the Rust panel.
//!
//! Phase 4.4 lock (2026-05-19): the Start menu's Quick Toggles need
//! to confirm every click with a short, non-modal message. The drawer
//! itself isn't ported yet (Phase 4.3 is in flight) so we don't have
//! its toast rail to lean on; instead we delegate to `notify-send`
//! when it's on the path and fall back to `stderr` so headless test
//! environments still see the message.
//!
//! The public surface is intentionally tiny — `show(msg)` and
//! `show_error(msg)`. Anything richer (icons, action buttons, sticky
//! toasts) lives downstream in the Phase 4.3 drawer port.
//!
//! Both helpers MUST be non-blocking: clicking a Quick Toggle should
//! never spin the panel's GTK main loop waiting on a subprocess.
//! `notify-send` is spawned (not `output()`), and we silently degrade
//! to stderr if the binary isn't on the path.

use std::process::{Command, Stdio};

/// Application id used for every toast we emit. Tracks the panel's
/// freedesktop notification origin so the user can filter ours from
/// other apps' notifications in xfce4-notifyd's history.
pub const APP_NAME: &str = "mackes-panel";

/// Show an informational toast. Non-blocking; always returns.
pub fn show(msg: &str) {
    emit("normal", msg);
}

/// Show an error-class toast. Renders identically to `show()` on
/// xfce4-notifyd but the `--urgency=critical` flag prevents auto-
/// dismiss on most notification daemons — important when a toggle
/// click actually failed and we need the user to see it.
pub fn show_error(msg: &str) {
    emit("critical", msg);
}

fn emit(urgency: &str, msg: &str) {
    // `notify-send` syntax: `notify-send [-u low|normal|critical]
    // -a <app> "<summary>"`. Body is optional — we keep it summary-
    // only because the Quick-Toggle confirmations are one-liners.
    let spawn_result = Command::new("notify-send")
        .args(["-u", urgency, "-a", APP_NAME, msg])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if spawn_result.is_err() {
        // notify-send missing or the daemon refused — degrade to
        // stderr so journalctl still captures the event.
        eprintln!("mackes-panel: toast ({urgency}): {msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_is_mackes_panel() {
        // Stable identifier the notification daemon (and downstream
        // notification-center tests) groups our toasts under.
        assert_eq!(APP_NAME, "mackes-panel");
    }

    #[test]
    fn show_does_not_panic_without_notify_send() {
        // The test harness has no GUI session — emit() will hit the
        // stderr fallback path. The contract is "never panic, never
        // block."
        show("phase 4.4 self-test");
    }

    #[test]
    fn show_error_does_not_panic_without_notify_send() {
        show_error("phase 4.4 self-test (error path)");
    }
}
