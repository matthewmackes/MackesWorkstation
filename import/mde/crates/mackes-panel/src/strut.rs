//! `_NET_WM_STRUT_PARTIAL` publication for the top bar and bottom dock.
//!
//! `WindowTypeHint::Dock` alone does not stop maximized windows from drawing
//! over the panel — EWMH-compliant window managers (xfwm4, i3, bspwm, …)
//! only reserve screen space when the panel publishes `_NET_WM_STRUT` /
//! `_NET_WM_STRUT_PARTIAL`. This module does that so the same call site
//! works under any compliant WM.
//!
//! gtk-rs 0.18 doesn't expose the XID on the safe `gdk::Window` surface,
//! and `unsafe_code` is `forbid`-en at workspace level, so we look up the
//! XID with `xdotool search --name <title>` (already a hard dep from
//! Phase 5.3's window-switching path) and publish the property via
//! `xprop -id`. Both tools ship on every workstation we target.

use std::process::Command;

use gtk::prelude::*;

use crate::FallbackGeometry;

/// Reserve `height` pixels at the top of the primary monitor.
pub fn set_top_strut(window: &gtk::ApplicationWindow, geom: &FallbackGeometry, height: i32) {
    apply_strut(
        window,
        Strut {
            top: height,
            top_start_x: geom.x,
            top_end_x: geom.x + geom.width - 1,
            ..Strut::default()
        },
    );
}

/// Reserve `height` pixels at the bottom of the primary monitor.
pub fn set_bottom_strut(window: &gtk::ApplicationWindow, geom: &FallbackGeometry, height: i32) {
    apply_strut(
        window,
        Strut {
            bottom: height,
            bottom_start_x: geom.x,
            bottom_end_x: geom.x + geom.width - 1,
            ..Strut::default()
        },
    );
}

/// Twelve-cardinal `_NET_WM_STRUT_PARTIAL` payload per the EWMH spec.
#[derive(Debug, Clone, Copy, Default)]
struct Strut {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    left_start_y: i32,
    left_end_y: i32,
    right_start_y: i32,
    right_end_y: i32,
    top_start_x: i32,
    top_end_x: i32,
    bottom_start_x: i32,
    bottom_end_x: i32,
}

fn apply_strut(window: &gtk::ApplicationWindow, strut: Strut) {
    let Some(title) = window.title().map(|g| g.to_string()) else {
        eprintln!("mackes-panel: window has no title — strut skipped");
        return;
    };
    let Some(xid) = xid_for_title(&title) else {
        eprintln!("mackes-panel: xdotool could not find window {title:?} — strut skipped");
        return;
    };

    let cardinals = [
        strut.left,
        strut.right,
        strut.top,
        strut.bottom,
        strut.left_start_y,
        strut.left_end_y,
        strut.right_start_y,
        strut.right_end_y,
        strut.top_start_x,
        strut.top_end_x,
        strut.bottom_start_x,
        strut.bottom_end_x,
    ];
    set_cardinal_property(xid, "_NET_WM_STRUT_PARTIAL", &cardinals);
    set_cardinal_property(xid, "_NET_WM_STRUT", &cardinals[..4]);
}

/// Resolve a window XID from its title via `xdotool`. Returns `None`
/// when xdotool is missing, the window hasn't realized yet, or the
/// session is on a non-X11 backend (Wayland / nested).
fn xid_for_title(title: &str) -> Option<u64> {
    let out = Command::new("xdotool")
        .args(["search", "--name", &format!("^{title}$")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let line = std::str::from_utf8(&out.stdout)
        .ok()?
        .lines()
        .next()?
        .trim();
    line.parse::<u64>().ok()
}

fn set_cardinal_property(xid: u64, name: &str, values: &[i32]) {
    let joined = values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let xid_arg = format!("0x{xid:x}");
    let result = Command::new("xprop")
        .args(["-id", &xid_arg, "-f", name, "32c", "-set", name, &joined])
        .status();
    match result {
        Ok(s) if s.success() => {}
        Ok(s) => eprintln!("mackes-panel: xprop {name} exited {s}"),
        Err(e) => eprintln!("mackes-panel: xprop {name} spawn failed: {e}"),
    }
}
