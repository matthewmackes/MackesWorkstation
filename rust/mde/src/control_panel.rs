//! Control Panel — Win2000-named mapping of installed Fedora tools.
//!
//! Planned: ports `control-panel.py` (Add/Remove Programs → dnfdragora,
//! Network → nm-connection-editor, Users and Passwords → seahorse, ...),
//! auto-hiding entries whose backing tool is absent. See task: mde control-panel.

use std::process::ExitCode;

pub fn run(_args: &[String]) -> ExitCode {
    crate::not_implemented("control-panel")
}
