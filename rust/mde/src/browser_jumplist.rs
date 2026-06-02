//! The Windows 10 taskbar **Firefox jump list** (E18.3): a flat, themed
//! layer-shell popup of the browser's quick tasks. It reuses `popup.rs`'s
//! layer-shell launcher (no title bar, era-anchored), so only the browser-specific
//! item list lives here. The Recent section — Firefox history from
//! `places.sqlite` — lands in E18.4, inserted between Tasks and the footer.
//!
//!   mde browser-jumplist   open the Firefox jump list (panel right-click, E18.6)

use std::process::ExitCode;

use crate::popup::{launch_with, sep, Item};

pub fn run(_args: &[String]) -> ExitCode {
    // No compositor → nothing to anchor to; exit cleanly (the popup is normally
    // spawned by the panel), matching popup.rs.
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return ExitCode::SUCCESS;
    }
    match launch_with(items()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde browser-jumplist: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The Firefox jump-list entries: Tasks (New / New Private window) then a footer
/// that launches the browser. E18.4 inserts the Recent section between them.
fn items() -> Vec<Item> {
    vec![
        Item::new("New Window", "firefox --new-window"),
        Item::new("New Private Window", "firefox --private-window"),
        sep(),
        Item::new("Firefox", "firefox"),
    ]
}
