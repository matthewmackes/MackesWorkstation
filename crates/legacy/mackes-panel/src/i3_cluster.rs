//! Centered i3 control cluster — SPLIT / LAYOUT / WINDOW chips.
//!
//! Q6 lock (2026-05-19): three labeled chips between the pinned apps
//! and the system tray. Workspaces switcher intentionally dropped per
//! user lock — i3's Mod+1..4 keybindings carry workspace navigation.
//!
//! Each chip is a `gtk::Box` housing a small-caps Red Hat Mono label
//! (left, light, 9 px) plus 2–3 line-glyph buttons (Carbon-style 16 px
//! SVGs). Every button dispatches a single `i3-msg` command; tooltips
//! carry the literal command + the default `$mod+…` keybinding so the
//! cluster doubles as a cheatsheet.
//!
//! Chips (left → right):
//!
//! - **SPLIT**: H (`i3-msg split h`), V (`i3-msg split v`).
//! - **LAYOUT**: default-split (`i3-msg layout default`),
//!   tabbed (`i3-msg layout tabbed`), stacking (`i3-msg layout
//!   stacking`). The active layout shows an inset amber underline.
//! - **WINDOW**: floating toggle (`i3-msg floating toggle`),
//!   fullscreen toggle (`i3-msg fullscreen toggle`), focus parent
//!   (`i3-msg focus parent`).
//!
//! The cluster does NOT subscribe to i3 IPC events in this initial
//! ship — the LAYOUT chip's active-underline state will be wired up
//! when the focused-app hero ships, since both need the same
//! subscription pipe. Today the chip stays in its last-clicked state
//! until a re-click; that's a known limitation and explicitly OK per
//! Q6 ("WINDOW + SPLIT + LAYOUT only — no live state").

use std::process::Command;

use gtk::prelude::*;

/// Build the entire i3 cluster as a horizontal `gtk::Box` ready to
/// drop into the taskbar's center slot.
#[must_use]
pub fn build() -> gtk::Box {
    let cluster = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    cluster.set_widget_name("mackes-i3-cluster");

    cluster.pack_start(&build_chip("SPLIT", SPLIT_BUTTONS), false, false, 0);
    cluster.pack_start(&build_chip("LAYOUT", LAYOUT_BUTTONS), false, false, 0);
    cluster.pack_start(&build_chip("WINDOW", WINDOW_BUTTONS), false, false, 0);

    cluster
}

struct ChipButton {
    /// Carbon-symbolic icon name. Falls back to the label glyph when
    /// the icon theme isn't available (dev tree without
    /// Mackes-Carbon installed).
    icon_name: &'static str,
    /// Single-letter fallback that renders when icon lookup fails.
    fallback_glyph: &'static str,
    /// Human label for the tooltip first line.
    label: &'static str,
    /// The literal `i3-msg` command to run on click.
    command: &'static str,
    /// Default `$mod+…` keybinding shown on the tooltip's second line
    /// so the cluster doubles as a cheatsheet.
    keybind: &'static str,
}

const SPLIT_BUTTONS: &[ChipButton] = &[
    ChipButton {
        icon_name: "object-flip-horizontal-symbolic",
        fallback_glyph: "H",
        label: "Split horizontal",
        command: "split h",
        keybind: "(no default keybind)",
    },
    ChipButton {
        icon_name: "object-flip-vertical-symbolic",
        fallback_glyph: "V",
        label: "Split vertical",
        command: "split v",
        keybind: "(no default keybind)",
    },
];

const LAYOUT_BUTTONS: &[ChipButton] = &[
    ChipButton {
        icon_name: "view-grid-symbolic",
        fallback_glyph: "□",
        label: "Default (tiled)",
        command: "layout default",
        keybind: "$mod+e",
    },
    ChipButton {
        icon_name: "view-list-symbolic",
        fallback_glyph: "≡",
        label: "Tabbed",
        command: "layout tabbed",
        keybind: "$mod+w",
    },
    ChipButton {
        icon_name: "view-paged-symbolic",
        fallback_glyph: "⧉",
        label: "Stacking",
        command: "layout stacking",
        keybind: "$mod+s",
    },
];

const WINDOW_BUTTONS: &[ChipButton] = &[
    ChipButton {
        icon_name: "view-restore-symbolic",
        fallback_glyph: "◇",
        label: "Toggle floating",
        command: "floating toggle",
        keybind: "$mod+Shift+space",
    },
    ChipButton {
        icon_name: "view-fullscreen-symbolic",
        fallback_glyph: "⛶",
        label: "Toggle fullscreen",
        command: "fullscreen toggle",
        keybind: "$mod+f",
    },
    ChipButton {
        icon_name: "go-up-symbolic",
        fallback_glyph: "↑",
        label: "Focus parent",
        command: "focus parent",
        keybind: "$mod+a",
    },
];

fn build_chip(label: &'static str, buttons: &'static [ChipButton]) -> gtk::Box {
    let chip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    chip.set_widget_name(&format!("mackes-i3-chip-{}", label.to_ascii_lowercase()));

    let label_widget = gtk::Label::new(Some(label));
    label_widget.set_widget_name("mackes-i3-chip-label");
    chip.pack_start(&label_widget, false, false, 0);

    for spec in buttons {
        chip.pack_start(&build_button(spec), false, false, 0);
    }
    chip
}

fn build_button(spec: &'static ChipButton) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name(&format!("mackes-i3-btn-{}", spec.command.replace(' ', "-")));
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);

    // Try Carbon-symbolic first; fall back to the single-char glyph.
    if let Some(pb) = crate::icons::load(spec.icon_name, 16) {
        button.set_image(Some(&gtk::Image::from_pixbuf(Some(&pb))));
        button.set_always_show_image(true);
    } else {
        button.set_label(spec.fallback_glyph);
    }

    button.set_tooltip_text(Some(&format!(
        "{}\ni3-msg {}\nkeybind: {}",
        spec.label, spec.command, spec.keybind
    )));
    if let Some(atk) = button.accessible() {
        atk.set_name(spec.label);
        atk.set_description(&format!(
            "Runs i3-msg {} — default keybind {}",
            spec.command, spec.keybind
        ));
    }

    let cmd = spec.command;
    button.connect_clicked(move |_| {
        send_i3_msg(cmd);
    });
    button
}

fn send_i3_msg(command: &str) {
    if let Err(e) = Command::new("i3-msg").arg(command).spawn() {
        eprintln!("mackes-panel: i3-msg {command} failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_chips_with_expected_button_counts() {
        assert_eq!(SPLIT_BUTTONS.len(), 2);
        assert_eq!(LAYOUT_BUTTONS.len(), 3);
        assert_eq!(WINDOW_BUTTONS.len(), 3);
    }

    #[test]
    fn no_workspaces_chip_per_q6_lock() {
        // Q6 locked SPLIT + LAYOUT + WINDOW only — guard against an
        // accidental reintroduction of the workspace switcher.
        for chip in [SPLIT_BUTTONS, LAYOUT_BUTTONS, WINDOW_BUTTONS] {
            for btn in chip {
                assert!(
                    !btn.command.starts_with("workspace "),
                    "Q6 lock prohibits workspace buttons in the cluster"
                );
            }
        }
    }

    #[test]
    fn every_button_has_a_fallback_glyph() {
        for chip in [SPLIT_BUTTONS, LAYOUT_BUTTONS, WINDOW_BUTTONS] {
            for btn in chip {
                assert!(!btn.fallback_glyph.is_empty(), "{}", btn.label);
                assert!(!btn.label.is_empty());
                assert!(!btn.command.is_empty());
            }
        }
    }

    #[test]
    fn every_button_carries_an_icon_name_ending_in_symbolic() {
        // The cluster's design lock requires every glyph to be a
        // Carbon symbolic. Guards against a future commit using a
        // non-symbolic alias by accident.
        for chip in [SPLIT_BUTTONS, LAYOUT_BUTTONS, WINDOW_BUTTONS] {
            for btn in chip {
                assert!(
                    btn.icon_name.ends_with("-symbolic"),
                    "icon for {} is not symbolic: {}",
                    btn.label,
                    btn.icon_name
                );
            }
        }
    }

    #[test]
    fn split_buttons_carry_canonical_commands() {
        let cmds: Vec<&str> = SPLIT_BUTTONS.iter().map(|b| b.command).collect();
        assert_eq!(cmds, vec!["split h", "split v"]);
    }

    #[test]
    fn layout_buttons_carry_canonical_commands() {
        let cmds: Vec<&str> = LAYOUT_BUTTONS.iter().map(|b| b.command).collect();
        assert_eq!(
            cmds,
            vec!["layout default", "layout tabbed", "layout stacking"]
        );
    }

    #[test]
    fn window_buttons_carry_canonical_commands() {
        let cmds: Vec<&str> = WINDOW_BUTTONS.iter().map(|b| b.command).collect();
        assert_eq!(
            cmds,
            vec!["floating toggle", "fullscreen toggle", "focus parent"]
        );
    }

    #[test]
    fn every_command_is_unique_across_chips() {
        // Different chips MUST issue distinct i3-msg commands —
        // otherwise we'd render two buttons that do the same thing.
        let mut all: Vec<&str> = Vec::new();
        for chip in [SPLIT_BUTTONS, LAYOUT_BUTTONS, WINDOW_BUTTONS] {
            for btn in chip {
                all.push(btn.command);
            }
        }
        let pre = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(pre, all.len(), "duplicate command found across cluster");
    }

    #[test]
    fn total_button_count_is_eight() {
        // 2 (SPLIT) + 3 (LAYOUT) + 3 (WINDOW) per Q6 lock.
        let total = SPLIT_BUTTONS.len() + LAYOUT_BUTTONS.len() + WINDOW_BUTTONS.len();
        assert_eq!(total, 8);
    }
}
