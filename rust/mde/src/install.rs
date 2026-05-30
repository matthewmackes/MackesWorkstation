//! First-run asset installer.
//!
//! The RPM ships code only; this fetches the visual assets so their upstream
//! licenses travel with the bytes (never redistributed by us):
//!   * Chicago95 (icons/cursors/sounds/GTK theme)  — github grassmunk/Chicago95
//!   * Win2k icon theme                            — KDE-Store item 1120706
//!
//! Planned: ports `assets/install-chicago95.sh` and
//! `home/.config/sway/scripts/install-win2k-icons.py` into Rust (rustls HTTP),
//! deploying under ~/.local/share. See task: mde control-panel + installers.

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    crate::not_implemented("install (asset fetch)")
}
