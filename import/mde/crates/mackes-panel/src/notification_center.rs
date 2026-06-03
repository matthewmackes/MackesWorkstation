//! Notification center — Rust port of the handoff design.
//!
//! Locked 2026-05-19 via the `Rust Desktop.zip` handoff bundle.
//! Three integration points (per the handoff README):
//!
//! 1. **Tray bell** — `crate::notification_bell::build()` ships a
//!    permanent bell + unread badge at the far right of the taskbar
//!    (immediately before the clock). Pulses (1.6 s) while unread
//!    > 0 and the modal is closed; stops the moment the modal opens.
//!
//! 2. **Modal** — this module's `open()` function. 70% screen with
//!    dimmed backdrop, Esc / click-outside dismiss. Latest 3
//!    notifications highlighted at top; the rest collapse into a
//!    `Node → App → Notification` tree.
//!
//! 3. **Data source** — `~/.cache/mackes/notifications.json` is the
//!    mesh-replicated source (Python `mesh_notifications.py` already
//!    syncs it whole-file via QNM-Shared). Reading from disk gives
//!    us a 1:1 view of every peer's notifications without re-doing
//!    mesh transport.

use std::path::PathBuf;
use std::process::Command;

use gtk::glib;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};

/// One notification row. Shape matches the handoff bundle's data
/// contract (`id`, `node`, `app`, `min`, `title`, `body`, `read`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Stable integer id (monotonic per emission).
    pub id: u64,
    /// `NC_NODES[].id` matching one of the mesh peers
    /// (e.g. `yew`, `pine`, `birch`).
    pub node: String,
    /// `NC_APPS` key matching a known app (e.g. `cargo`, `sshd`,
    /// `mesh`, `clipboard`, `updates`, `battery`, `systemd`).
    pub app: String,
    /// Minutes-ago integer for display (the handoff helper). When a
    /// real timestamp is available, the renderer formats it on the
    /// fly — `min` stays around as a precomputed legacy field.
    #[serde(default)]
    pub min: u32,
    /// Single-line summary.
    pub title: String,
    /// Longer detail.
    #[serde(default)]
    pub body: String,
    /// Whether the user has marked this notification read. Defaults
    /// to `false` so new entries land as unread.
    #[serde(default)]
    pub read: bool,
}

/// Resolve `~/.cache/mackes/notifications.json` honoring
/// `$XDG_CACHE_HOME` per the freedesktop spec.
#[must_use]
pub fn cache_path() -> PathBuf {
    if let Ok(s) = std::env::var("XDG_CACHE_HOME") {
        if !s.is_empty() {
            return PathBuf::from(s).join("mackes/notifications.json");
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        return PathBuf::from(h).join(".cache/mackes/notifications.json");
    }
    PathBuf::from("/tmp/mackes-notifications.json")
}

/// Read every notification from the cache file at `path`. Empty vec
/// on missing or unreadable input.
#[must_use]
pub fn load_from(path: &std::path::Path) -> Vec<Notification> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Read every notification from the on-disk cache (the canonical
/// `~/.cache/mackes/notifications.json` per `cache_path()`).
#[must_use]
pub fn load() -> Vec<Notification> {
    load_from(&cache_path())
}

/// Persist a fresh notification list to `path` via `.tmp` + rename.
///
/// # Errors
/// Returns `std::io::Error` when the parent directory isn't writable.
pub fn save_to(path: &std::path::Path, notifications: &[Notification]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_vec_pretty(notifications)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Persist a fresh notification list to the canonical cache path.
///
/// # Errors
/// Returns `std::io::Error` when the cache directory isn't writable.
pub fn save(notifications: &[Notification]) -> std::io::Result<()> {
    save_to(&cache_path(), notifications)
}

/// Count unread notifications across the entire list. Drives the
/// bell badge + pulse state.
#[must_use]
pub fn unread_count(notifications: &[Notification]) -> usize {
    notifications.iter().filter(|n| !n.read).count()
}

/// Open the notification center modal. The dialog reads + saves
/// from the cache file directly so every modal session reflects the
/// most recent mesh-sync state.
pub fn open() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_widget_name("mackes-notification-center");
    window.set_title("Notification Center");
    window.set_default_size(960, 640);
    window.set_position(gtk::WindowPosition::Center);
    window.set_keep_above(true);
    window.set_skip_taskbar_hint(true);
    window.set_resizable(true);
    window.set_decorated(false);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_widget_name("mackes-nc-outer");

    // ---- Header ----------------------------------------------------
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header.set_widget_name("mackes-nc-header");
    header.set_margin_top(20);
    header.set_margin_bottom(8);
    header.set_margin_start(28);
    header.set_margin_end(28);

    let title = gtk::Label::new(Some("Notifications"));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("mackes-nc-title");

    let count_label = gtk::Label::new(None);
    count_label.set_halign(gtk::Align::Start);
    count_label.style_context().add_class("mackes-nc-count");

    header.pack_start(&title, false, false, 0);
    header.pack_start(&count_label, false, false, 0);

    let clear_btn = gtk::Button::with_label("Clear all");
    clear_btn.set_widget_name("mackes-nc-clear");
    clear_btn.set_relief(gtk::ReliefStyle::None);
    if let Some(atk) = clear_btn.accessible() {
        atk.set_name("Clear all notifications (mark every notification read)");
    }
    let close_btn = gtk::Button::with_label("×");
    close_btn.set_widget_name("mackes-nc-close");
    close_btn.set_relief(gtk::ReliefStyle::None);
    if let Some(atk) = close_btn.accessible() {
        atk.set_name("Close the notification center");
    }
    header.pack_end(&close_btn, false, false, 0);
    header.pack_end(&clear_btn, false, false, 0);
    outer.pack_start(&header, false, false, 0);

    // ---- Body (scrolling list) -------------------------------------
    let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_widget_name("mackes-nc-scroll");
    let list = gtk::Box::new(gtk::Orientation::Vertical, 12);
    list.set_widget_name("mackes-nc-list");
    list.set_margin_top(12);
    list.set_margin_bottom(20);
    list.set_margin_start(28);
    list.set_margin_end(28);
    scroller.add(&list);
    outer.pack_start(&scroller, true, true, 0);

    // Initial render.
    rerender_list(&list, &count_label, &load());

    // ---- Live refresh while modal is open ---------------------------
    // Reads from disk every 2 s so a mesh-pushed notification surfaces
    // without forcing the user to reopen the modal.
    let list_for_timer = list.clone();
    let count_label_for_timer = count_label.clone();
    let window_for_timer = window.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if !window_for_timer.is_visible() {
            return glib::ControlFlow::Break;
        }
        rerender_list(&list_for_timer, &count_label_for_timer, &load());
        glib::ControlFlow::Continue
    });

    // ---- Wire actions ----------------------------------------------
    let window_for_close = window.clone();
    close_btn.connect_clicked(move |_| {
        // Auto-mark everything read on close (per handoff lock).
        let mut items = load();
        for n in &mut items {
            n.read = true;
        }
        let _ = save(&items);
        window_for_close.close();
    });

    clear_btn.connect_clicked({
        let list = list.clone();
        let count_label = count_label.clone();
        move |_| {
            let _ = save(&[]);
            rerender_list(&list, &count_label, &[]);
        }
    });

    // Esc dismisses.
    let window_for_key = window.clone();
    window.connect_key_press_event(move |_, ev| {
        if ev.keyval() == gtk::gdk::keys::constants::Escape {
            window_for_key.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    window.add(&outer);
    window.show_all();
}

fn rerender_list(list: &gtk::Box, count_label: &gtk::Label, notifications: &[Notification]) {
    for c in list.children() {
        list.remove(&c);
    }

    let total = notifications.len();
    let unread = unread_count(notifications);
    count_label.set_text(&format!("{unread} unread · {total} total"));

    if notifications.is_empty() {
        let empty = gtk::Label::new(Some("(no notifications — mesh history syncs here)"));
        empty.style_context().add_class("mackes-nc-empty");
        empty.set_margin_top(40);
        empty.set_margin_bottom(40);
        list.add(&empty);
        list.show_all();
        return;
    }

    // Latest 3 (most-recent by `min` ascending = smallest min).
    let mut sorted = notifications.to_vec();
    sorted.sort_by_key(|n| n.min);
    let latest_count = sorted.len().min(3);
    let latest_header = gtk::Label::new(Some("LATEST"));
    latest_header.set_halign(gtk::Align::Start);
    latest_header.style_context().add_class("mackes-nc-section");
    list.add(&latest_header);
    for n in sorted.iter().take(latest_count) {
        list.add(&build_card(n, /* in_tree = */ false));
    }

    // Below: Node → App → Notification tree.
    let tree_header = gtk::Label::new(Some("ALL NOTIFICATIONS"));
    tree_header.set_halign(gtk::Align::Start);
    tree_header.style_context().add_class("mackes-nc-section");
    list.add(&tree_header);

    let mut by_node: std::collections::BTreeMap<String, Vec<Notification>> =
        std::collections::BTreeMap::new();
    for n in notifications {
        by_node.entry(n.node.clone()).or_default().push(n.clone());
    }
    for (node, items) in &by_node {
        let n_unread = items.iter().filter(|n| !n.read).count();
        let node_header = gtk::Label::new(Some(&format!(
            "▸ {node}  —  {n_unread} unread / {} total",
            items.len()
        )));
        node_header.set_halign(gtk::Align::Start);
        node_header
            .style_context()
            .add_class("mackes-nc-node-header");
        list.add(&node_header);
        for n in items {
            list.add(&build_card(n, /* in_tree = */ true));
        }
    }

    list.show_all();
}

fn build_card(n: &Notification, in_tree: bool) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.style_context().add_class("mackes-nc-card");
    if !n.read {
        row.style_context().add_class("unread");
    }
    if in_tree {
        row.set_margin_start(20);
    }

    let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
    column.set_margin_top(8);
    column.set_margin_bottom(8);
    column.set_margin_start(12);
    column.set_margin_end(12);
    let header = gtk::Label::new(Some(&format!("{}  ·  {}m ago", n.app, n.min)));
    header.set_halign(gtk::Align::Start);
    header.style_context().add_class("mackes-nc-card-header");
    column.pack_start(&header, false, false, 0);
    let title = gtk::Label::new(Some(&n.title));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("mackes-nc-card-title");
    column.pack_start(&title, false, false, 0);
    if !n.body.is_empty() {
        let body = gtk::Label::new(Some(&n.body));
        body.set_halign(gtk::Align::Start);
        body.set_xalign(0.0);
        body.set_line_wrap(true);
        body.style_context().add_class("mackes-nc-card-body");
        column.pack_start(&body, false, false, 0);
    }
    row.pack_start(&column, true, true, 0);

    // Per-card actions: mark read · copy · dismiss.
    let action_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    action_row.set_valign(gtk::Align::Center);
    let id_for_actions = n.id;

    let mark_btn = gtk::Button::with_label("✓");
    mark_btn.set_relief(gtk::ReliefStyle::None);
    mark_btn.set_tooltip_text(Some("Mark read"));
    if let Some(atk) = mark_btn.accessible() {
        atk.set_name(&format!("Mark notification '{}' as read", n.title));
    }
    mark_btn.connect_clicked(move |_| {
        let mut items = load();
        for it in &mut items {
            if it.id == id_for_actions {
                it.read = true;
            }
        }
        let _ = save(&items);
    });
    action_row.pack_start(&mark_btn, false, false, 0);

    let copy_btn = gtk::Button::with_label("⧉");
    copy_btn.set_relief(gtk::ReliefStyle::None);
    copy_btn.set_tooltip_text(Some("Copy title + body"));
    if let Some(atk) = copy_btn.accessible() {
        atk.set_name(&format!("Copy notification '{}' to clipboard", n.title));
    }
    let title_copy = n.title.clone();
    let body_copy = n.body.clone();
    copy_btn.connect_clicked(move |_| {
        let text = if body_copy.is_empty() {
            title_copy.clone()
        } else {
            format!("{title_copy}\n{body_copy}")
        };
        let mut child = match Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                let _ = Command::new("wl-copy").arg(&text).spawn();
                return;
            }
        };
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    });
    action_row.pack_start(&copy_btn, false, false, 0);

    let dismiss_btn = gtk::Button::with_label("🗑");
    dismiss_btn.set_relief(gtk::ReliefStyle::None);
    dismiss_btn.set_tooltip_text(Some("Dismiss"));
    if let Some(atk) = dismiss_btn.accessible() {
        atk.set_name(&format!("Dismiss notification '{}'", n.title));
    }
    dismiss_btn.connect_clicked(move |_| {
        let items: Vec<Notification> = load()
            .into_iter()
            .filter(|n| n.id != id_for_actions)
            .collect();
        let _ = save(&items);
    });
    action_row.pack_start(&dismiss_btn, false, false, 0);

    row.pack_end(&action_row, false, false, 0);
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unread_count_counts_unread() {
        let n = vec![
            Notification {
                id: 1,
                node: "yew".into(),
                app: "cargo".into(),
                min: 1,
                title: "a".into(),
                body: String::new(),
                read: false,
            },
            Notification {
                id: 2,
                node: "yew".into(),
                app: "cargo".into(),
                min: 2,
                title: "b".into(),
                body: String::new(),
                read: true,
            },
            Notification {
                id: 3,
                node: "pine".into(),
                app: "sshd".into(),
                min: 3,
                title: "c".into(),
                body: String::new(),
                read: false,
            },
        ];
        assert_eq!(unread_count(&n), 2);
    }

    #[test]
    fn save_then_load_round_trips() {
        // No env var dependency — pass an explicit path. Parallel-safe.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notifications.json");
        let n = vec![Notification {
            id: 7,
            node: "birch".into(),
            app: "updates".into(),
            min: 5,
            title: "12 updates".into(),
            body: "DNF".into(),
            read: false,
        }];
        save_to(&path, &n).unwrap();
        let back = load_from(&path);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].id, 7);
        assert_eq!(back[0].title, "12 updates");
    }

    #[test]
    fn load_returns_empty_on_missing_file() {
        let path = std::path::PathBuf::from("/nonexistent/zzz/yyy/notifications.json");
        let r = load_from(&path);
        assert!(r.is_empty());
    }

    #[test]
    fn unread_count_zero_when_all_read() {
        let n = vec![Notification {
            id: 1,
            node: "yew".into(),
            app: "cargo".into(),
            min: 1,
            title: "a".into(),
            body: String::new(),
            read: true,
        }];
        assert_eq!(unread_count(&n), 0);
    }

    #[test]
    fn unread_count_empty_input_is_zero() {
        assert_eq!(unread_count(&[]), 0);
    }

    #[test]
    fn cache_path_prefers_xdg_cache_home() {
        let _g = crate::test_env::env_lock();
        let prior = std::env::var_os("XDG_CACHE_HOME");
        std::env::set_var("XDG_CACHE_HOME", "/tmp/cov-xdg");
        let p = cache_path();
        match prior {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        assert_eq!(
            p,
            std::path::PathBuf::from("/tmp/cov-xdg/mackes/notifications.json")
        );
    }

    #[test]
    fn cache_path_falls_to_home_when_xdg_unset() {
        let _g = crate::test_env::env_lock();
        let prior_xdg = std::env::var_os("XDG_CACHE_HOME");
        let prior_home = std::env::var_os("HOME");
        std::env::remove_var("XDG_CACHE_HOME");
        std::env::set_var("HOME", "/tmp/cov-home");
        let p = cache_path();
        match prior_xdg {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        match prior_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        assert_eq!(
            p,
            std::path::PathBuf::from("/tmp/cov-home/.cache/mackes/notifications.json")
        );
    }

    #[test]
    fn save_to_creates_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c/notifications.json");
        let n = vec![Notification {
            id: 1,
            node: "yew".into(),
            app: "cargo".into(),
            min: 1,
            title: "t".into(),
            body: String::new(),
            read: false,
        }];
        save_to(&nested, &n).unwrap();
        assert!(nested.exists());
        let back = load_from(&nested);
        assert_eq!(back.len(), 1);
    }

    #[test]
    fn save_to_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notifications.json");
        let original = vec![Notification {
            id: 1,
            node: "yew".into(),
            app: "cargo".into(),
            min: 1,
            title: "first".into(),
            body: String::new(),
            read: false,
        }];
        save_to(&path, &original).unwrap();
        // Replace with an empty list.
        save_to(&path, &[]).unwrap();
        let back = load_from(&path);
        assert!(back.is_empty());
    }

    #[test]
    fn load_from_malformed_json_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notifications.json");
        std::fs::write(&path, b"not-json").unwrap();
        let back = load_from(&path);
        assert!(back.is_empty());
    }

    #[test]
    fn notification_deserializes_with_optional_defaults() {
        // `body`, `min`, and `read` all carry `#[serde(default)]` —
        // omitting them in JSON must not error.
        let json = r#"[{"id":1,"node":"yew","app":"cargo","title":"hello"}]"#;
        let parsed: Vec<Notification> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].min, 0);
        assert!(!parsed[0].read);
        assert!(parsed[0].body.is_empty());
    }
}
