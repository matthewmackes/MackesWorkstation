//! Start menu — a layer-shell popup anchored above the Start button.
//!
//! Planned: ports `win95-menu.py` (modes main/programs/system/run) with the
//! Win2000 banner stripe, .desktop scanning, and toggle-to-close behavior.
//! See task: mde-menu.

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    crate::not_implemented("menu (Start menu)")
}
