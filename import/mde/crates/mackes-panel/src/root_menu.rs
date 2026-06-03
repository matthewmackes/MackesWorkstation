//! Root-window right-click menu — Phase 8.4 lock (v3.0.0 Q40).
//!
//! Right-clicking the wallpaper drops a Mackes-themed `gtk::Menu` with
//! four canonical actions:
//!
//! 1. **Change wallpaper…** — opens Workbench focused on the Look & Feel
//!    panel (`mackes --focus look_and_feel`). Look & Feel hosts the
//!    appearance sub-panel where the wallpaper picker lives, so this
//!    is the same surface the user would reach via the status cluster.
//! 2. **Open mesh share…** — opens Thunar (via `xdg-open`) at
//!    `~/QNM-Shared/`. Matches the mesh-share peer popover's "Files"
//!    button grammar but rooted at the share top level.
//! 3. **Send file to peer…** — fans out into a per-peer submenu. Each
//!    peer entry opens a `zenity` file picker and copies the chosen
//!    file into `~/QNM-Shared/<peer>/`. Mirrors
//!    `mesh_module::send_file_dialog` so the wallpaper-rooted entry
//!    and the dock peer popover share semantics.
//! 4. **Display settings** — opens Workbench focused on the Devices
//!    panel (`mackes --focus devices`). Devices hosts the Display
//!    sub-panel where resolution / scaling / multi-monitor live.
//!
//! ### Approach (a) vs (b)
//!
//! The lock text mentions `XGrabButton` on the X11 root. We deliberately
//! chose approach (a): handle `button-press-event` on the Desktop-type
//! window mackes-panel already paints (`build_desktop` in `main.rs`).
//! That window covers every pixel of the wallpaper, sits below every
//! other window via `WindowTypeHint::Desktop`, and is owned by our
//! process so we can attach a GTK event handler directly. `XGrabButton`
//! would force us to depend on the `x11`/`xcb` crate, handle button
//! ungrabbing on exit, and route the event into GTK manually — strictly
//! worse for the same observable behavior.
//!
//! ### Subprocess matrix
//!
//! | Item            | Subprocess argv                                  |
//! |-----------------|--------------------------------------------------|
//! | Change wallpaper| `mackes --focus look_and_feel`                   |
//! | Open mesh share | `xdg-open ~/QNM-Shared/`                         |
//! | Send file (per-peer) | `/bin/sh -c "f=$(zenity --file-selection) && cp -- \"$f\" '<dir>'"` |
//! | Display settings| `mackes --focus devices`                         |
//!
//! Patterns:
//! - `mackes --focus <slug>` matches `status_cluster.rs:326` and
//!   `start_menu.rs:84` — the canonical Workbench entrypoint.
//! - `xdg-open <path>` matches `mesh_module::spawn_xdg_open_path` —
//!   Thunar is the user's xdg-MIME handler for `inode/directory`.
//! - zenity wrapping matches `mesh_module::send_file_dialog` — same
//!   shell-escape grammar, same destination layout.

use std::path::{Path, PathBuf};
use std::process::Command;

use gtk::prelude::*;

/// Root directory the "Open mesh share" item opens — Thunar at
/// `~/QNM-Shared/`. Matches `mesh_module::QNM_SHARED_ROOT`.
const QNM_SHARED_ROOT: &str = "QNM-Shared";

/// Carbon glyph used as a fallback for the "Send file to peer" peer
/// entries that don't have a per-peer icon override.
const PEER_GLYPH: &str = "laptop";

/// Build the four-item Mackes-themed root-window menu. Caller is
/// responsible for `show_all()` + `popup_at_pointer(event)`. The
/// menu's accessibles are populated so screen-readers announce each
/// row with a description of its action.
#[must_use]
pub fn build() -> gtk::Menu {
    let menu = gtk::Menu::new();
    menu.set_widget_name("mackes-root-menu");

    append_menu_item(
        &menu,
        "Change wallpaper…",
        "image-x-generic-symbolic",
        "Open Workbench → Look & Feel to change the wallpaper.",
        || focus_workbench("look_and_feel"),
    );

    append_menu_item(
        &menu,
        "Open mesh share…",
        "folder-remote-symbolic",
        "Open Thunar at ~/QNM-Shared/ — the mesh-replicated share root.",
        open_mesh_share_root,
    );

    // Send-file-to-peer fans out into a peer submenu when peers exist,
    // and degrades to a single placeholder row when no peers have been
    // seen yet (mirrors the `Recent Items` empty-state convention from
    // `top_bar::build_recents_submenu`).
    menu.append(&build_send_file_item());

    menu.append(&gtk::SeparatorMenuItem::new());

    append_menu_item(
        &menu,
        "Display settings",
        "preferences-desktop-display-symbolic",
        "Open Workbench → Devices → Display for resolution and scaling.",
        || focus_workbench("devices"),
    );

    menu
}

/// One menu row — Carbon glyph + label + accessible description, with
/// an `on_activate` closure. Carbon icons load via `crate::icons`; when
/// the theme isn't installed we still ship a labeled `MenuItem` so the
/// row remains usable.
fn append_menu_item<F>(
    menu: &gtk::Menu,
    label: &str,
    icon_name: &str,
    accessible_desc: &str,
    on_activate: F,
) where
    F: Fn() + 'static,
{
    let item = build_glyph_item(label, icon_name);
    if let Some(atk) = item.accessible() {
        atk.set_name(label);
        atk.set_description(accessible_desc);
    }
    item.connect_activate(move |_| on_activate());
    menu.append(&item);
}

/// A `gtk::MenuItem` whose child is a Carbon icon + label hbox. The
/// fallback (no theme) gives us a plain text item so the menu is
/// always usable.
fn build_glyph_item(label: &str, icon_name: &str) -> gtk::MenuItem {
    let item = gtk::MenuItem::new();
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    if let Some(pb) = crate::icons::load(icon_name, 16) {
        row.pack_start(&gtk::Image::from_pixbuf(Some(&pb)), false, false, 0);
    }
    let lbl = gtk::Label::new(Some(label));
    lbl.set_xalign(0.0);
    row.pack_start(&lbl, true, true, 0);
    item.add(&row);
    item
}

/// "Send file to peer…" — top-level menu item with a submenu of every
/// known peer. Peer enumeration walks `~/QNM-Shared/<peer>/` for any
/// directory entry (which is how `mesh_module::send_file_dialog`
/// already routes the destination). Hidden / dotfile entries are
/// filtered out. When no peers exist we still surface the row with a
/// disabled "(no peers — open mesh setup)" placeholder so the user
/// sees an obvious diagnostic.
fn build_send_file_item() -> gtk::MenuItem {
    let parent = build_glyph_item("Send file to peer…", "mail-send-symbolic");
    if let Some(atk) = parent.accessible() {
        atk.set_name("Send file to peer");
        atk.set_description(
            "Pick a file with zenity and copy it into the chosen peer's ~/QNM-Shared/<peer>/ folder.",
        );
    }
    let submenu = gtk::Menu::new();
    submenu.set_widget_name("mackes-root-menu-send-file");

    let peers = discover_peers();
    if peers.is_empty() {
        let placeholder = gtk::MenuItem::with_label("(no peers — open mesh setup)");
        placeholder.set_sensitive(false);
        submenu.append(&placeholder);
        // Add a separator + a fallback into the Workbench mesh page so
        // the dead-end is recoverable in one click.
        submenu.append(&gtk::SeparatorMenuItem::new());
        let open_mesh = gtk::MenuItem::with_label("Open mesh settings…");
        open_mesh.connect_activate(|_| focus_workbench("connectivity"));
        submenu.append(&open_mesh);
    } else {
        for peer in peers {
            let item = build_glyph_item(&peer, PEER_GLYPH);
            if let Some(atk) = item.accessible() {
                atk.set_name(&peer);
                atk.set_description(&format!(
                    "Pick a file and copy it into ~/QNM-Shared/{peer}/."
                ));
            }
            let peer_owned = peer.clone();
            item.connect_activate(move |_| send_file_to_peer(&peer_owned));
            submenu.append(&item);
        }
    }

    parent.set_submenu(Some(&submenu));
    parent
}

/// Enumerate peer names by reading `~/QNM-Shared/` and treating each
/// non-hidden subdirectory as a peer. Returns names sorted
/// case-insensitively so the menu order is stable across boots.
fn discover_peers() -> Vec<String> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let root = PathBuf::from(home).join(QNM_SHARED_ROOT);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut peers: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect();
    peers.sort_by_key(|n| n.to_lowercase());
    peers
}

/// `mackes --focus <slug>` — same grammar as `status_cluster.rs:326`
/// and `start_menu.rs:84`. The Python Workbench is the single source
/// of truth for the slug catalogue (`look_and_feel`, `devices`,
/// `connectivity`, …) — see
/// `mackes/workbench/shell/sidebar_window.py:457`.
fn focus_workbench(slug: &str) {
    if let Err(e) = Command::new("mackes").args(["--focus", slug]).spawn() {
        eprintln!("mackes-panel: workbench launch failed ({slug}): {e}");
    }
}

/// Open Thunar (or whatever the user's xdg-MIME handler for
/// `inode/directory` is) at `~/QNM-Shared/`. Matches the path-style
/// grammar used by `mesh_module::spawn_xdg_open_path`.
fn open_mesh_share_root() {
    let Some(home) = std::env::var_os("HOME") else {
        eprintln!("mackes-panel: cannot open mesh share — $HOME unset");
        return;
    };
    let target = PathBuf::from(home).join(QNM_SHARED_ROOT);
    // Best-effort create — without this, a fresh install with no mesh
    // peers yet hits an xdg-open error and the user gets nothing.
    if let Err(e) = std::fs::create_dir_all(&target) {
        eprintln!(
            "mackes-panel: cannot ensure {} exists: {e}",
            target.display()
        );
    }
    spawn_xdg_open(&target);
}

fn spawn_xdg_open(path: &Path) {
    if let Err(e) = Command::new("xdg-open").arg(path).spawn() {
        eprintln!("mackes-panel: xdg-open {} failed: {e}", path.display());
    }
}

/// Run a zenity file picker and copy the result into the peer's mesh
/// share directory. Matches `mesh_module::send_file_dialog` exactly —
/// same shell-escape grammar, same destination layout — so a file
/// sent from the dock peer popover and one sent from the root menu
/// land in the same place.
fn send_file_to_peer(peer: &str) {
    let Some(home) = std::env::var_os("HOME") else {
        eprintln!("mackes-panel: cannot send to peer — $HOME unset");
        return;
    };
    let target_dir = PathBuf::from(home).join(QNM_SHARED_ROOT).join(peer);
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        eprintln!(
            "mackes-panel: cannot create {} for send-file: {e}",
            target_dir.display()
        );
        return;
    }
    let target = target_dir.to_string_lossy().to_string();
    let cmd = format!(
        "f=$(zenity --file-selection 2>/dev/null) && cp -- \"$f\" {}",
        shell_escape(&target)
    );
    if let Err(e) = Command::new("/bin/sh").arg("-c").arg(&cmd).spawn() {
        eprintln!("mackes-panel: send-file picker failed: {e}");
    }
}

/// Minimal POSIX shell single-quote escape. Mirrors
/// `mesh_module::shell_escape` so the two send-file paths can't drift.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Walk the menu's children and collect plain-text labels. Submenu
    /// parents store their label inside a nested hbox / `gtk::Label`,
    /// so we recurse into each `MenuItem`'s child tree to find labels.
    fn collect_labels(menu: &gtk::Menu) -> Vec<String> {
        menu.children()
            .into_iter()
            .filter_map(|c| c.downcast::<gtk::MenuItem>().ok())
            .filter_map(|item| item_label_text(&item))
            .collect()
    }

    fn item_label_text(item: &gtk::MenuItem) -> Option<String> {
        // MenuItem::with_label gives `item.label()` directly.
        if let Some(text) = item.label() {
            let s = text.to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
        // Composite items (`build_glyph_item`) wrap the label in a Box
        // → Label, so walk the child tree.
        find_label_in_widget(item.upcast_ref::<gtk::Widget>())
    }

    fn find_label_in_widget(w: &gtk::Widget) -> Option<String> {
        if let Some(lbl) = w.downcast_ref::<gtk::Label>() {
            return Some(lbl.text().to_string());
        }
        if let Some(container) = w.downcast_ref::<gtk::Container>() {
            for child in container.children() {
                if let Some(t) = find_label_in_widget(&child) {
                    return Some(t);
                }
            }
        }
        None
    }

    use crate::test_env::try_init_gtk_serialized;

    #[test]
    fn menu_has_four_actions_plus_one_separator() {
        let _g = crate::test_env::env_lock();
        if !try_init_gtk_serialized() {
            return;
        }
        let menu = build();
        let kids = menu.children();
        // 4 action items + 1 separator (between "Send file" and
        // "Display settings"). The lock text says 4 menu items; the
        // separator is structural chrome, not a fifth action.
        let separators = kids
            .iter()
            .filter(|w| w.is::<gtk::SeparatorMenuItem>())
            .count();
        let actions = kids
            .iter()
            .filter(|w| !w.is::<gtk::SeparatorMenuItem>())
            .count();
        assert_eq!(actions, 4, "Phase 8.4 locked 4 menu actions");
        assert_eq!(separators, 1, "exactly one structural separator");
    }

    #[test]
    fn menu_labels_match_phase_8_4_lock() {
        let _g = crate::test_env::env_lock();
        if !try_init_gtk_serialized() {
            return;
        }
        let menu = build();
        let labels = collect_labels(&menu);
        // Order in the lock: Change wallpaper / Open mesh share /
        // Send file to peer / Display settings. The trailing ellipsis
        // (…) is the canonical macOS-grammar marker for items that
        // open a dialog or new window.
        assert_eq!(
            labels,
            vec![
                "Change wallpaper…".to_owned(),
                "Open mesh share…".to_owned(),
                "Send file to peer…".to_owned(),
                "Display settings".to_owned(),
            ],
        );
    }

    #[test]
    fn every_action_item_has_an_accessible_name() {
        let _g = crate::test_env::env_lock();
        if !try_init_gtk_serialized() {
            return;
        }
        let menu = build();
        for item in menu
            .children()
            .into_iter()
            .filter_map(|c| c.downcast::<gtk::MenuItem>().ok())
        {
            // Separators are MenuItem subclasses we already filtered
            // upstream — here we keep them so the test catches a
            // future refactor that drops the accessible on a real
            // item. Skip explicit separators.
            if item
                .upcast_ref::<gtk::Widget>()
                .is::<gtk::SeparatorMenuItem>()
            {
                continue;
            }
            let atk = item.accessible().expect("AtkObject must exist on MenuItem");
            let name = atk.name().map(|s| s.to_string()).unwrap_or_default();
            assert!(
                !name.is_empty(),
                "menu item missing accessible name: {:?}",
                item_label_text(&item)
            );
        }
    }

    #[test]
    fn shell_escape_quotes_single_quotes() {
        assert_eq!(shell_escape("plain"), "'plain'");
        assert_eq!(shell_escape("with space"), "'with space'");
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn discover_peers_lists_non_hidden_subdirs_sorted() {
        let _g = crate::test_env::env_lock();
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let qnm = home.join(QNM_SHARED_ROOT);
        fs::create_dir_all(qnm.join("zeta")).unwrap();
        fs::create_dir_all(qnm.join("Alpha")).unwrap();
        fs::create_dir_all(qnm.join("Beta")).unwrap();
        fs::create_dir_all(qnm.join(".hidden")).unwrap();
        // A regular file should be ignored.
        fs::write(qnm.join("not-a-peer.txt"), b"x").unwrap();

        std::env::set_var("HOME", home);
        let peers = discover_peers();
        std::env::remove_var("HOME");

        assert_eq!(
            peers,
            vec!["Alpha".to_owned(), "Beta".to_owned(), "zeta".to_owned()]
        );
    }

    #[test]
    fn discover_peers_returns_empty_when_share_missing() {
        let _g = crate::test_env::env_lock();
        let dir = tempdir().expect("tempdir");
        std::env::set_var("HOME", dir.path());
        let peers = discover_peers();
        std::env::remove_var("HOME");
        assert!(peers.is_empty());
    }

    #[test]
    fn discover_peers_handles_missing_home() {
        let _g = crate::test_env::env_lock();
        let saved = std::env::var_os("HOME");
        std::env::remove_var("HOME");
        let peers = discover_peers();
        if let Some(h) = saved {
            std::env::set_var("HOME", h);
        }
        assert!(peers.is_empty());
    }

    #[test]
    fn send_file_submenu_has_per_peer_entries_when_peers_present() {
        let _g = crate::test_env::env_lock();
        if !try_init_gtk_serialized() {
            return;
        }
        let dir = tempdir().expect("tempdir");
        let qnm = dir.path().join(QNM_SHARED_ROOT);
        fs::create_dir_all(qnm.join("anvil")).unwrap();
        fs::create_dir_all(qnm.join("forge")).unwrap();
        std::env::set_var("HOME", dir.path());

        let item = build_send_file_item();
        let submenu = item
            .submenu()
            .and_then(|w| w.downcast::<gtk::Menu>().ok())
            .expect("send-file item must have a Menu submenu");
        let labels: Vec<String> = submenu
            .children()
            .into_iter()
            .filter_map(|c| c.downcast::<gtk::MenuItem>().ok())
            .filter_map(|i| item_label_text(&i))
            .collect();
        std::env::remove_var("HOME");

        assert_eq!(labels, vec!["anvil".to_owned(), "forge".to_owned()]);
    }

    #[test]
    fn send_file_submenu_shows_placeholder_when_no_peers() {
        let _g = crate::test_env::env_lock();
        if !try_init_gtk_serialized() {
            return;
        }
        let dir = tempdir().expect("tempdir");
        // QNM-Shared does not exist under HOME.
        std::env::set_var("HOME", dir.path());

        let item = build_send_file_item();
        let submenu = item
            .submenu()
            .and_then(|w| w.downcast::<gtk::Menu>().ok())
            .expect("send-file item must have a Menu submenu");
        let kids: Vec<gtk::MenuItem> = submenu
            .children()
            .into_iter()
            .filter_map(|c| c.downcast::<gtk::MenuItem>().ok())
            .collect();
        std::env::remove_var("HOME");

        // Placeholder + separator + "Open mesh settings…" recovery.
        assert_eq!(kids.len(), 3, "expected placeholder + separator + recovery");
        // First row is the disabled placeholder.
        assert!(!kids[0].is_sensitive());
        // Last row is the recovery item; must be enabled.
        assert!(kids[2].is_sensitive());
    }
}
