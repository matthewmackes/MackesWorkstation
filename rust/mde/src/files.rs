//! File manager — an Explorer-style window (stock iced xdg toplevel).
//!
//! Planned: folder tree + details/icon views, a toolbar (back/forward/up),
//! address bar, and status bar. Copy/move/trash/mount operations follow
//! Thunar's source as the behavioral reference. See task: mde-files.

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    crate::not_implemented("files (file manager)")
}
