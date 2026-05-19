//! Top-bar window-management buttons (Phase 8.7 — locked
//! 2026-05-19 via 5-Q survey).
//!
//! Three Carbon-symbolic glyphs at the far-right corner of the
//! top bar: minimize / maximize / close. Operate the i3 focused
//! window via `i3-msg`. Greyed out at 45% opacity when no
//! container is focused (empty workspace, all closed). Per
//! Phase 8.8 (xfwm4 removal, 2026-05-19), i3 is the only WM —
//! we no longer hide the buttons based on `wmctrl -m`.
//!
//! Maximize semantics (Q3 lock): `floating enable + resize to
//! fill workspace`. The panel chrome stays visible — NOT
//! `fullscreen toggle` (which would hide the panel and trap
//! the user). Second click toggles `floating disable` to
//! restore.
//!
//! Refresh cadence: 2 s poll of `i3-msg -t get_tree` — matches
//! the existing dock + status cluster cadence. The polling
//! approach keeps the implementation simple and avoids a long-
//! lived subscription thread; the focused-window churn is bound
//! by user input speed anyway.

use std::process::Command;

use gtk::glib;
use gtk::prelude::*;

use crate::icons;

const BUTTON_ICON_PX: i32 = 18;

/// Three buttons in render order (left-to-right): minimize,
/// maximize, close. Carbon glyphs match the existing status
/// cluster's icon family.
const BUTTON_DEFS: &[ButtonDef] = &[
    ButtonDef {
        slug: "minimize",
        icon_name: "subtract-large",
        title: "Minimize active window",
        action: Action::Minimize,
    },
    ButtonDef {
        slug: "maximize",
        icon_name: "maximize",
        title: "Maximize active window",
        action: Action::Maximize,
    },
    ButtonDef {
        slug: "close",
        icon_name: "close-large",
        title: "Close active window",
        action: Action::Close,
    },
];

#[derive(Clone, Copy)]
struct ButtonDef {
    slug: &'static str,
    icon_name: &'static str,
    title: &'static str,
    action: Action,
}

#[derive(Clone, Copy)]
enum Action {
    Minimize,
    Maximize,
    Close,
}

#[derive(Clone)]
struct ButtonWidgets {
    button: gtk::Button,
    title: &'static str,
}

/// Build the three-button cluster. Returns a `gtk::Box` ready
/// to drop into the top bar's right slot AFTER the status
/// cluster. The widget owns its own 2 s polling timer that
/// flips enabled/disabled state based on i3's focused container.
#[must_use]
pub fn build() -> gtk::Box {
    let cluster = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    cluster.set_widget_name("mackes-window-buttons");

    let mut widgets: Vec<ButtonWidgets> = Vec::with_capacity(BUTTON_DEFS.len());

    for def in BUTTON_DEFS {
        let button = gtk::Button::new();
        button.set_widget_name(&format!("mackes-window-button-{}", def.slug));
        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(false);

        if let Some(pb) = icons::load(def.icon_name, BUTTON_ICON_PX) {
            button.set_image(Some(&gtk::Image::from_pixbuf(Some(&pb))));
            button.set_always_show_image(true);
        } else {
            // Dev fallback: small ASCII glyph so the slot is
            // discoverable when the Mackes-Carbon icon theme
            // isn't on the search path.
            button.set_label(match def.action {
                Action::Minimize => "_",
                Action::Maximize => "□",
                Action::Close => "✕",
            });
        }

        button.set_tooltip_text(Some(def.title));
        if let Some(atk) = button.accessible() {
            atk.set_name(def.title);
        }

        let action = def.action;
        button.connect_clicked(move |btn| {
            // Disabled buttons swallow the click silently — this
            // matches Q4's "no-op on empty focus" lock.
            if !btn.is_sensitive() {
                return;
            }
            dispatch(action);
        });

        cluster.pack_start(&button, false, false, 0);
        widgets.push(ButtonWidgets {
            button,
            title: def.title,
        });
    }

    // First poll immediately so we don't show a greyed-out
    // cluster for the full 2 s after launch on systems where a
    // window is already focused at panel start.
    set_enabled(&widgets, i3_has_focused_window());

    // 2 s polling timer keeps the cluster in sync with i3's
    // focused container. Cheap — i3-msg get_tree is a few KB
    // and runs against the local i3-ipc socket.
    glib::timeout_add_seconds_local(2, move || {
        set_enabled(&widgets, i3_has_focused_window());
        glib::ControlFlow::Continue
    });

    cluster
}

fn set_enabled(widgets: &[ButtonWidgets], enabled: bool) {
    for w in widgets {
        w.button.set_sensitive(enabled);
        let ctx = w.button.style_context();
        if enabled {
            ctx.remove_class("mackes-window-button-disabled");
            w.button.set_tooltip_text(Some(w.title));
            if let Some(atk) = w.button.accessible() {
                atk.set_name(w.title);
            }
        } else {
            ctx.add_class("mackes-window-button-disabled");
            let phrase = format!("{} (no window focused)", w.title);
            w.button.set_tooltip_text(Some(&phrase));
            if let Some(atk) = w.button.accessible() {
                atk.set_name(&phrase);
            }
        }
    }
}

/// Quick check: does i3 report a focused, non-split container?
///
/// We shell out to `i3-msg -t get_tree` and scan the JSON for
/// `"focused":true` paired with a non-empty `"window"` field
/// (the X11 window id — leaf containers have one, split parents
/// don't). Avoids a JSON parser dependency for one boolean.
fn i3_has_focused_window() -> bool {
    let output = Command::new("i3-msg").args(["-t", "get_tree"]).output();
    let Ok(out) = output else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    // Find every "focused":true region and look ahead a short
    // window for a "window":<int> (non-null, non-zero).
    let mut search_from = 0;
    while let Some(rel) = s[search_from..].find("\"focused\":true") {
        let abs = search_from + rel;
        // Look at the surrounding ~400 characters for a window
        // field. i3-msg emits objects in a stable order; the
        // window field is reliably within a small radius.
        let window_start = abs.saturating_sub(400);
        let window_end = (abs + 400).min(s.len());
        let window_region = &s[window_start..window_end];
        if has_non_null_window_field(window_region) {
            return true;
        }
        search_from = abs + "\"focused\":true".len();
    }
    false
}

fn has_non_null_window_field(region: &str) -> bool {
    // Look for `"window":<digits>` where digits != 0.
    let needle = "\"window\":";
    let mut pos = 0;
    while let Some(rel) = region[pos..].find(needle) {
        let abs = pos + rel + needle.len();
        let tail = &region[abs..];
        if let Some(c) = tail.chars().next() {
            if c == 'n' {
                // "window":null — keep looking.
                pos = abs;
                continue;
            }
            if c.is_ascii_digit() {
                // Check it's not just "0".
                let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
                if digits.parse::<u64>().unwrap_or(0) > 0 {
                    return true;
                }
            }
        }
        pos = abs;
    }
    false
}

/// Dispatch the click to i3-msg. Phase 8.8 lock: i3 is the only
/// WM, so we never branch on `wmctrl -m`.
fn dispatch(action: Action) {
    let cmd = match action {
        Action::Minimize => "[con_id=__focused__] move scratchpad",
        Action::Maximize => {
            // Q3 lock: toggle floating + resize to fill workspace
            // area. We pass a multi-step i3 command via the shell
            // so all four mutations land atomically.
            "[con_id=__focused__] floating toggle; [con_id=__focused__] resize set 100 ppt 100 ppt; [con_id=__focused__] move position 0 0"
        }
        Action::Close => "[con_id=__focused__] kill",
    };
    if let Err(e) = Command::new("i3-msg").arg(cmd).spawn() {
        eprintln!("mackes-panel: i3-msg dispatch failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_non_null_window_field_accepts_numeric_id() {
        let region = r#"{"focused":true,"window":12345,"name":"firefox"}"#;
        assert!(has_non_null_window_field(region));
    }

    #[test]
    fn has_non_null_window_field_rejects_null() {
        let region = r#"{"focused":true,"window":null,"name":"split"}"#;
        assert!(!has_non_null_window_field(region));
    }

    #[test]
    fn has_non_null_window_field_rejects_zero() {
        let region = r#"{"focused":true,"window":0,"name":"x"}"#;
        assert!(!has_non_null_window_field(region));
    }

    #[test]
    fn has_non_null_window_field_handles_no_field() {
        let region = r#"{"focused":true,"name":"workspace"}"#;
        assert!(!has_non_null_window_field(region));
    }
}
