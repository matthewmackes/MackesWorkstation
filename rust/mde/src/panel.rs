//! Taskbar — a wlr-layer-shell bar anchored to the bottom edge.
//!
//! Planned (via `iced_layershell`): raised Win2000 bar with a ⊞ Start button,
//! a window-button taskbar fed by sway IPC, the notification-area tray, a
//! volume control, and a sunken clock. See task: mde-panel.

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    crate::not_implemented("panel (taskbar)")
}
