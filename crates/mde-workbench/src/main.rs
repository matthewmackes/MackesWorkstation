//! `mde-workbench` binary entry.
//!
//! Launches the Iced workbench window. CB-1.13 will introduce
//! the `--focus <slug>` arg + the live `dev.mackes.MDE.Shell.
//! Workbench.Focus` D-Bus hand-off; for now the binary just
//! opens a fresh window with the default Dashboard view.

use mde_workbench::App;

fn main() -> iced::Result {
    App::run()
}
