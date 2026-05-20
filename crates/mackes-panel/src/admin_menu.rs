//! Right-click admin menu — Fedora system-administration shortcuts.
//!
//! Q15 / Q16 lock (2026-05-19): right-clicking the Start (`M`) button
//! drops a 9-item `gtk::Menu` grouped by section. Each item launches a
//! `terminator` window running its command with the shell kept open
//! after the command finishes (suggestion #4) so the user can read
//! output and re-run interactively.
//!
//! Section catalog (locked, Q15 "Comprehensive"):
//!
//! - **Shells**: Root Terminal, `sudoedit` launcher
//! - **Packages**: `dnf` update, `dnf` history
//! - **Services**: `systemctl status`, `journalctl` tail
//! - **Security**: `SELinux` status, `firewall-cmd`
//! - **Storage**: clean (`dnf clean` + journal vacuum)
//!
//! Each menu item's tooltip shows the literal command + the sudo-cache
//! state ("no password" when `sudo -nv` exits 0, "prompts" otherwise)
//! per suggestion #6 — surfaces privilege escalation visibility
//! without requiring any auth at menu open time.

use std::process::Command;

use gtk::prelude::*;

/// One administrative action: a label, a literal shell command, and an
/// optional sudo flag that triggers the "prompts" / "no password"
/// tooltip suffix.
struct AdminAction {
    label: &'static str,
    /// Literal bash command. Run inside `terminator -x bash -c '<cmd>;
    /// bash'` so the shell stays open for inspection / re-run.
    cmd: &'static str,
    /// True when the command needs sudo. Drives the tooltip's sudo
    /// cache hint.
    needs_sudo: bool,
}

const SECTIONS: &[(&str, &[AdminAction])] = &[
    (
        "Shells",
        &[
            AdminAction {
                label: "Root Terminal",
                cmd: "sudo -i",
                needs_sudo: true,
            },
            AdminAction {
                label: "Edit system file (sudoedit)",
                cmd: "sudoedit /etc/hosts",
                needs_sudo: true,
            },
        ],
    ),
    (
        "Packages",
        &[
            AdminAction {
                label: "DNF update",
                cmd: "sudo dnf upgrade --refresh",
                needs_sudo: true,
            },
            AdminAction {
                label: "DNF history",
                cmd: "sudo dnf history list",
                needs_sudo: true,
            },
        ],
    ),
    (
        "Services",
        &[
            AdminAction {
                label: "systemctl status",
                cmd: "sudo systemctl status",
                needs_sudo: true,
            },
            AdminAction {
                label: "journalctl tail",
                cmd: "sudo journalctl -fxe",
                needs_sudo: true,
            },
        ],
    ),
    (
        "Security",
        &[
            AdminAction {
                label: "SELinux status",
                cmd: "sestatus",
                needs_sudo: false,
            },
            AdminAction {
                label: "Firewall (firewall-cmd)",
                cmd: "sudo firewall-cmd --list-all",
                needs_sudo: true,
            },
        ],
    ),
    (
        "Storage",
        &[AdminAction {
            label: "Clean (dnf cache + journal vacuum 7d)",
            cmd: "sudo dnf clean all && sudo journalctl --vacuum-time=7d",
            needs_sudo: true,
        }],
    ),
];

/// Cheap probe: is sudo currently cached without prompting?
/// `sudo -n -v` validates the cached credential silently — exit 0
/// when fresh, non-zero when not. Reading this once per menu open
/// is bounded by sudo's own short-circuit path; a misbehaving sudoers
/// config falls through to the `Err` branch and we report "not cached".
fn sudo_cached() -> bool {
    let result = Command::new("sudo")
        .args(["-n", "-v"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    matches!(result, Ok(status) if status.success())
}

/// Build the menu. Caller calls `show_all()` + `popup_at_widget`; this
/// constructor only assembles the widget tree.
#[must_use]
pub fn build() -> gtk::Menu {
    let menu = gtk::Menu::new();
    menu.set_widget_name("mackes-admin-menu");

    let cached = sudo_cached();

    for (i, (section, actions)) in SECTIONS.iter().enumerate() {
        if i > 0 {
            menu.append(&gtk::SeparatorMenuItem::new());
        }

        // Section header — non-interactive label rendered as a
        // disabled menu item so it visually anchors the group
        // without being clickable.
        let header = gtk::MenuItem::with_label(section);
        header.set_sensitive(false);
        header.set_widget_name("mackes-admin-menu-section");
        menu.append(&header);

        for action in *actions {
            let item = gtk::MenuItem::with_label(action.label);
            let tooltip = format_tooltip(action, cached);
            item.set_tooltip_text(Some(&tooltip));
            let cmd = action.cmd;
            item.connect_activate(move |_| {
                launch_in_terminator(cmd);
            });
            menu.append(&item);
        }
    }

    menu
}

/// `terminator -x bash -c '<cmd>; bash'` keeps the shell open after
/// the command finishes so the user can scroll output or re-run
/// without re-spawning the window. Suggestion #4 (the `--hold` flag
/// is deprecated in terminator; this `; bash` trick is the
/// recommended replacement per the terminator changelog).
fn launch_in_terminator(cmd: &str) {
    let wrapped = format!("{cmd}; bash");
    if let Err(e) = Command::new("terminator")
        .args(["-x", "bash", "-c", &wrapped])
        .spawn()
    {
        eprintln!("mackes-panel: terminator launch failed ({cmd}): {e}");
    }
}

fn format_tooltip(action: &AdminAction, sudo_cached: bool) -> String {
    let cmd_line = format!("Runs: {}", action.cmd);
    if action.needs_sudo {
        let hint = if sudo_cached {
            "sudo cached — no password needed"
        } else {
            "sudo not cached — will prompt"
        };
        format!("{cmd_line}\n{hint}")
    } else {
        cmd_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_have_at_least_one_action_each() {
        for (name, actions) in SECTIONS {
            assert!(!actions.is_empty(), "section {name} has no actions");
        }
    }

    #[test]
    fn total_action_count_matches_q15_lock() {
        // Q15 locked the "Comprehensive — 9 items grouped by section"
        // option. Guard the count so future edits don't silently
        // expand/shrink the lock.
        let total: usize = SECTIONS.iter().map(|(_, a)| a.len()).sum();
        assert_eq!(total, 9, "Q15 locked exactly 9 actions across all sections");
    }

    #[test]
    fn format_tooltip_marks_sudo_cache_state() {
        let action = AdminAction {
            label: "x",
            cmd: "y",
            needs_sudo: true,
        };
        assert!(format_tooltip(&action, true).contains("no password"));
        assert!(format_tooltip(&action, false).contains("will prompt"));

        let no_sudo = AdminAction {
            label: "x",
            cmd: "y",
            needs_sudo: false,
        };
        assert!(!format_tooltip(&no_sudo, true).contains("password"));
    }
}
