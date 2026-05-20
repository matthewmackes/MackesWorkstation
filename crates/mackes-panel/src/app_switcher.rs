//! Super+Tab app switcher — modal overlay strip driven by i3-ipc.
//!
//! Phase 6.1 (v3.0.0 §6 lock, 2026-05-19). i3 itself ships no
//! thumbnail-strip Alt-Tab; this module provides one as a GTK widget
//! that:
//!
//! 1. Talks to i3 via `i3-msg -t get_tree` (same shell-out shape as
//!    [`crate::i3_cluster`], so we don't pull a fresh
//!    `i3ipc`/`swayipc` dependency for one feature).
//! 2. Walks the JSON tree and flattens to leaves whose
//!    `window_type == "normal"` — system bars, splash screens, the
//!    panel's own surfaces, and i3 scratchpad containers stay out.
//! 3. Renders a centered, undecorated popup with one button per
//!    candidate window — icon (via [`crate::icons::load_with_fallback`])
//!    plus title. The currently focused window is selected by default
//!    so a single Tab cycles to the *previous* window — matching
//!    every macOS / Windows / GNOME Alt-Tab convention.
//! 4. Listens for Tab / Shift+Tab to cycle, Escape to dismiss, and
//!    the release of `Super_L`/`Super_R` to commit. Commit dispatches
//!    `i3-msg [con_id=<N>] focus`.
//!
//! **X11 only.** i3 is X11-only (per memory `project_v8_8_i3_only.md`
//! — xfwm4 was fully replaced in 1.0.8). No Wayland / sway code path
//! lives here; the binding in `data/i3/config.d/mackes-defaults.conf`
//! is the sole entry point.
//!
//! ## Testability split
//!
//! The cycling and commit logic is broken out as pure functions over
//! [`Candidate`] + an index — [`cycle_forward`], [`cycle_back`], and
//! [`commit_selection`] — so the entire switcher decision tree is
//! unit-tested without spawning GTK or i3.

use std::process::Command;

use gtk::prelude::*;
use serde::Deserialize;

/// One candidate window the switcher can land on. A flat projection of
/// an i3 tree leaf — never tracks i3's container hierarchy beyond what
/// the user needs to see (icon + title) and what we need to focus
/// (`con_id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// i3 container id (`con_id`). Passed to
    /// `i3-msg [con_id=<N>] focus`.
    pub con_id: i64,
    /// Free-form window title (i3's `name` field). Shown next to the
    /// icon in the overlay.
    pub title: String,
    /// X11 `WM_CLASS` second component, lowercased, used to resolve
    /// the launcher icon. May be empty when i3 hasn't populated
    /// `window_properties.class` yet (rare; the row still renders
    /// with the freedesktop fallback glyph).
    pub class: String,
    /// True for the i3 leaf carrying the input focus when the tree
    /// snapshot was taken. Drives the default selection index.
    pub focused: bool,
}

// ---------------------------------------------------------------------
// JSON shapes
// ---------------------------------------------------------------------

/// Minimal projection of an i3 tree node. i3 ships many more fields;
/// we deserialize only those the switcher consumes.
#[derive(Debug, Deserialize)]
struct I3Node {
    /// Container id. Always present.
    id: i64,
    /// Container `type` — `"con"`, `"workspace"`, `"floating_con"`,
    /// `"output"`, `"dockarea"`, `"root"`. We only care about
    /// distinguishing leaves from internal nodes (leaves carry a
    /// `window` id).
    #[serde(default)]
    #[allow(dead_code)]
    r#type: String,
    /// Title (free-form). May be empty.
    #[serde(default)]
    name: Option<String>,
    /// X11 window id when the node represents a real application
    /// window. `None` on internal tree nodes (workspaces, splits).
    #[serde(default)]
    window: Option<i64>,
    /// EWMH window type as i3 has classified it. Populated on leaves
    /// only. We keep candidates whose value is exactly `"normal"`.
    #[serde(default)]
    window_type: Option<String>,
    /// Per-window properties bag — carries `WM_CLASS`, instance, role.
    #[serde(default)]
    window_properties: Option<WindowProperties>,
    /// `true` when this node holds the input focus.
    #[serde(default)]
    focused: bool,
    /// Tiled children.
    #[serde(default)]
    nodes: Vec<I3Node>,
    /// Floating children (i3 stores these on a separate list).
    #[serde(default)]
    floating_nodes: Vec<I3Node>,
}

/// `window_properties` bag for an i3 leaf. Provides `WM_CLASS`'s
/// instance + class plus role; we only need the class for icon lookup.
#[derive(Debug, Deserialize)]
struct WindowProperties {
    /// `WM_CLASS` second component (the "class", e.g. `firefox`).
    #[serde(default)]
    class: Option<String>,
    /// `WM_CLASS` first component (the "instance"). Unused today but
    /// retained for future tie-breaking.
    #[serde(default)]
    #[allow(dead_code)]
    instance: Option<String>,
    /// Some EWMH types arrive here instead of the top-level
    /// `window_type` field on older i3 builds. We fall back to it
    /// when the top-level is missing.
    #[serde(default)]
    window_type: Option<String>,
}

// ---------------------------------------------------------------------
// Pure helpers (tested without GTK)
// ---------------------------------------------------------------------

/// Parse an i3 `get_tree` JSON blob and flatten the tree to its real
/// application leaves. Filters to `window_type == "normal"` so docks,
/// splash screens, tooltips, and the panel's own surfaces drop out.
///
/// The currently focused leaf is sorted first; the remaining leaves
/// keep their natural tree order. The intent matches macOS Cmd-Tab:
/// the first slot is the active app, the second slot is the previous
/// app — one Tab tap = "swap with the previous window".
#[must_use]
pub fn parse_tree(json: &str) -> Vec<Candidate> {
    let Ok(root) = serde_json::from_str::<I3Node>(json) else {
        return Vec::new();
    };
    let mut leaves: Vec<Candidate> = Vec::new();
    collect_normal_leaves(&root, &mut leaves);
    promote_focused_first(&mut leaves);
    leaves
}

/// Recurse through `node` and its `nodes` + `floating_nodes` arrays,
/// pushing one [`Candidate`] per real-application leaf encountered.
fn collect_normal_leaves(node: &I3Node, out: &mut Vec<Candidate>) {
    // Leaves are nodes with a real X11 `window` id. Internal nodes
    // (workspaces, splits) report `window == None`.
    if node.window.is_some() {
        let wtype = node
            .window_type
            .as_deref()
            .or_else(|| {
                node.window_properties
                    .as_ref()
                    .and_then(|p| p.window_type.as_deref())
            })
            .unwrap_or("");
        // Treat missing window_type as "normal" — i3 sometimes leaves
        // the field blank for X11 windows that don't set _NET_WM_WINDOW_TYPE
        // (legacy Xt-based apps still in the wild).
        if wtype.is_empty() || wtype == "normal" {
            let class = node
                .window_properties
                .as_ref()
                .and_then(|p| p.class.as_deref())
                .unwrap_or("")
                .to_ascii_lowercase();
            out.push(Candidate {
                con_id: node.id,
                title: node.name.clone().unwrap_or_default(),
                class,
                focused: node.focused,
            });
        }
    }
    for child in &node.nodes {
        collect_normal_leaves(child, out);
    }
    for child in &node.floating_nodes {
        collect_normal_leaves(child, out);
    }
}

/// Reorder so the leaf carrying input focus lands at index 0. Stable
/// for every other entry — the natural i3 tree order is preserved.
fn promote_focused_first(leaves: &mut Vec<Candidate>) {
    if let Some(idx) = leaves.iter().position(|c| c.focused) {
        if idx > 0 {
            let focused = leaves.remove(idx);
            leaves.insert(0, focused);
        }
    }
}

/// Advance the selection by one slot, wrapping past the end back to
/// zero. Returns the new index. An empty list collapses to 0 so
/// callers never index out of bounds.
#[must_use]
pub fn cycle_forward(current: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (current + 1) % len
}

/// Retreat the selection by one slot, wrapping past zero back to the
/// end. Mirror of [`cycle_forward`] for Shift+Tab.
#[must_use]
pub fn cycle_back(current: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (current + len - 1) % len
}

/// Resolve the selection index to the candidate's `con_id`. Returns
/// `None` when the list is empty or the index is out of range — both
/// cases drop the commit silently rather than focusing a stale window.
#[must_use]
pub fn commit_selection(candidates: &[Candidate], index: usize) -> Option<i64> {
    candidates.get(index).map(|c| c.con_id)
}

/// Shell out to `i3-msg -t get_tree` and parse the result. Returns an
/// empty Vec when `i3-msg` is missing, errors, or returns
/// non-deserializable JSON — every call site treats the switcher as
/// best-effort.
#[must_use]
pub fn fetch_candidates() -> Vec<Candidate> {
    let Ok(output) = Command::new("i3-msg").args(["-t", "get_tree"]).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_tree(&text)
}

/// Dispatch the focus command for the selected `con_id`. Best-effort —
/// failures log to stderr and the switcher still closes.
fn focus_con(con_id: i64) {
    let target = format!("[con_id={con_id}] focus");
    if let Err(e) = Command::new("i3-msg").arg(&target).spawn() {
        eprintln!("mackes-panel: i3-msg {target} failed: {e}");
    }
}

// ---------------------------------------------------------------------
// GTK overlay
// ---------------------------------------------------------------------

/// Width of the modal overlay window in CSS pixels. Chosen to fit ~6
/// candidate buttons before horizontal scrolling kicks in — the
/// switcher is meant to be glanceable, not exhaustive.
const OVERLAY_WIDTH_PX: i32 = 720;

/// Per-row icon size in CSS pixels. Matches the dock's 32 px tile
/// height so the overlay reads as a horizontal slice of the dock.
const ROW_ICON_PX: i32 = 32;

/// Build and present the switcher modal. Runs the GTK main loop
/// **inline** via `gtk::main()` so the binary entry point can return
/// the moment the user commits a selection — no `gtk::Application`
/// scaffolding required.
///
/// The function returns once the user has committed (Super release)
/// or dismissed (Escape / focus loss). Designed to be called by the
/// `mackes-panel --app-switcher` CLI handler.
pub fn run_switcher_modal() {
    if gtk::init().is_err() {
        eprintln!("mackes-panel: failed to initialize GTK for app switcher");
        return;
    }

    let candidates = fetch_candidates();
    if candidates.is_empty() {
        // Nothing to switch to — closing immediately avoids flashing
        // an empty popover on Mod+Tab.
        return;
    }

    // Selection starts on slot 1 when there's a previous window
    // available; otherwise slot 0. macOS / GNOME Alt-Tab convention.
    let initial = if candidates.len() > 1 { 1 } else { 0 };

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_widget_name("mackes-app-switcher");
    window.set_title("Mackes App Switcher");
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);
    window.set_keep_above(true);
    window.set_position(gtk::WindowPosition::Center);
    window.set_type_hint(gtk::gdk::WindowTypeHint::PopupMenu);
    window.set_default_size(OVERLAY_WIDTH_PX, -1);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_widget_name("mackes-app-switcher-row");
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.set_margin_start(8);
    row.set_margin_end(8);

    // Build one button per candidate; the selection-styling helper
    // toggles the `selected` class on whichever row matches the live
    // index.
    let buttons: Vec<gtk::Button> = candidates
        .iter()
        .map(|c| build_candidate_button(c))
        .collect();
    for b in &buttons {
        row.pack_start(b, false, false, 0);
    }
    window.add(&row);

    // ---- Shared selection state -----------------------------------
    let state = std::rc::Rc::new(std::cell::Cell::new(initial));
    let candidates_rc = std::rc::Rc::new(candidates);
    let buttons_rc = std::rc::Rc::new(buttons);
    apply_selection_class(&buttons_rc, state.get());

    // Clicking a row commits immediately to that index.
    for (i, btn) in buttons_rc.iter().enumerate() {
        let state_for_click = state.clone();
        let buttons_for_click = buttons_rc.clone();
        let candidates_for_click = candidates_rc.clone();
        let window_for_click = window.clone();
        btn.connect_clicked(move |_| {
            state_for_click.set(i);
            apply_selection_class(&buttons_for_click, i);
            if let Some(con_id) = commit_selection(&candidates_for_click, i) {
                focus_con(con_id);
            }
            window_for_click.close();
        });
    }

    // ---- Keyboard handling ----------------------------------------
    {
        let state_for_press = state.clone();
        let buttons_for_press = buttons_rc.clone();
        let candidates_for_press = candidates_rc.clone();
        let window_for_press = window.clone();
        window.connect_key_press_event(move |_, ev| {
            let keyval = ev.keyval();
            let shift_held = ev.state().contains(gtk::gdk::ModifierType::SHIFT_MASK);
            if keyval == gtk::gdk::keys::constants::Escape {
                window_for_press.close();
                return glib::Propagation::Stop;
            }
            if keyval == gtk::gdk::keys::constants::Tab
                || keyval == gtk::gdk::keys::constants::ISO_Left_Tab
            {
                let len = candidates_for_press.len();
                let next = if shift_held || keyval == gtk::gdk::keys::constants::ISO_Left_Tab {
                    cycle_back(state_for_press.get(), len)
                } else {
                    cycle_forward(state_for_press.get(), len)
                };
                state_for_press.set(next);
                apply_selection_class(&buttons_for_press, next);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }

    {
        let state_for_release = state.clone();
        let candidates_for_release = candidates_rc.clone();
        let window_for_release = window.clone();
        window.connect_key_release_event(move |_, ev| {
            let keyval = ev.keyval();
            if keyval == gtk::gdk::keys::constants::Super_L
                || keyval == gtk::gdk::keys::constants::Super_R
            {
                if let Some(con_id) =
                    commit_selection(&candidates_for_release, state_for_release.get())
                {
                    focus_con(con_id);
                }
                window_for_release.close();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }

    // Focus loss = dismiss without committing. Otherwise the overlay
    // can linger if the user clicks the desktop or another monitor.
    let window_for_focus = window.clone();
    window.connect_focus_out_event(move |_, _| {
        window_for_focus.close();
        glib::Propagation::Proceed
    });

    // When the toplevel goes away, drop out of the main loop so the
    // CLI handler returns control to systemd / the user shell.
    window.connect_destroy(|_| {
        gtk::main_quit();
    });

    window.show_all();
    // Grab focus so key events route here even when the parent app
    // had focus a moment ago. Without this, the very first Tab tap
    // can leak to the previously-focused window.
    window.present();
    gtk::main();
}

/// Build one button for a candidate. Icon + truncated title; tooltip
/// carries the full title for windows whose name is longer than the
/// row width.
fn build_candidate_button(c: &Candidate) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name("mackes-app-switcher-item");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);
    button.set_tooltip_text(Some(&c.title));

    let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
    column.set_halign(gtk::Align::Center);

    let icon_widget: gtk::Widget = crate::icons::load_with_fallback(
        if c.class.is_empty() {
            None
        } else {
            Some(c.class.as_str())
        },
        &[],
        ROW_ICON_PX,
    )
    .map_or_else(
        || gtk::Label::new(Some("•")).upcast::<gtk::Widget>(),
        |pb| gtk::Image::from_pixbuf(Some(&pb)).upcast::<gtk::Widget>(),
    );
    column.pack_start(&icon_widget, false, false, 0);

    let label_text: String = if c.title.is_empty() {
        "(untitled)".to_owned()
    } else {
        c.title.chars().take(20).collect()
    };
    let label = gtk::Label::new(Some(&label_text));
    label.set_halign(gtk::Align::Center);
    label.style_context().add_class("mackes-app-switcher-label");
    column.pack_start(&label, false, false, 0);

    button.add(&column);

    if let Some(atk) = button.accessible() {
        atk.set_name(&format!("Switch to window: {}", c.title));
    }
    button
}

/// Toggle the `selected` style class on the button at `index` and
/// remove it from every other row. Called from every cycle event so
/// the visual selection ring tracks the live index.
fn apply_selection_class(buttons: &[gtk::Button], index: usize) {
    for (i, b) in buttons.iter().enumerate() {
        let ctx = b.style_context();
        if i == index {
            ctx.add_class("selected");
        } else {
            ctx.remove_class("selected");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal i3 tree carrying three leaves: a real "normal" window,
    /// a dock-type window (must be filtered out), and a second normal
    /// window that holds focus. Wrapped in a root → workspace ladder
    /// the way i3 actually nests its tree.
    fn sample_tree_json() -> String {
        r#"
        {
            "id": 1, "type": "root", "name": "root",
            "nodes": [
                {
                    "id": 2, "type": "output", "name": "DP-1",
                    "nodes": [
                        {
                            "id": 3, "type": "workspace", "name": "1",
                            "nodes": [
                                {
                                    "id": 100, "type": "con", "name": "Firefox — Inbox",
                                    "window": 4194305,
                                    "window_type": "normal",
                                    "window_properties": {"class": "Firefox", "instance": "Navigator"},
                                    "focused": false
                                },
                                {
                                    "id": 101, "type": "con", "name": "mackes-panel",
                                    "window": 4194306,
                                    "window_type": "dock",
                                    "window_properties": {"class": "Mackes-shell"},
                                    "focused": false
                                },
                                {
                                    "id": 102, "type": "con", "name": "vim notes.md",
                                    "window": 4194307,
                                    "window_type": "normal",
                                    "window_properties": {"class": "URxvt"},
                                    "focused": true
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#
        .to_owned()
    }

    fn fixture_candidates() -> Vec<Candidate> {
        vec![
            Candidate {
                con_id: 100,
                title: "Firefox — Inbox".into(),
                class: "firefox".into(),
                focused: false,
            },
            Candidate {
                con_id: 102,
                title: "vim notes.md".into(),
                class: "urxvt".into(),
                focused: true,
            },
            Candidate {
                con_id: 105,
                title: "Thunar — Home".into(),
                class: "thunar".into(),
                focused: false,
            },
        ]
    }

    #[test]
    fn window_list_filters_to_normal_windows() {
        let v = parse_tree(&sample_tree_json());
        // The dock window must be filtered out — only the two
        // window_type=normal leaves survive.
        assert_eq!(v.len(), 2);
        let con_ids: Vec<i64> = v.iter().map(|c| c.con_id).collect();
        assert!(con_ids.contains(&100));
        assert!(con_ids.contains(&102));
        assert!(!con_ids.contains(&101), "dock window leaked into switcher");
    }

    #[test]
    fn focused_window_is_promoted_to_index_zero() {
        let v = parse_tree(&sample_tree_json());
        assert!(!v.is_empty());
        assert_eq!(v[0].con_id, 102, "focused leaf should land at idx 0");
        assert!(v[0].focused);
    }

    #[test]
    fn parse_tree_extracts_wm_class_lowercased() {
        let v = parse_tree(&sample_tree_json());
        let firefox = v.iter().find(|c| c.con_id == 100).expect("firefox leaf");
        assert_eq!(firefox.class, "firefox");
        let vim = v.iter().find(|c| c.con_id == 102).expect("vim leaf");
        assert_eq!(vim.class, "urxvt");
    }

    #[test]
    fn parse_tree_returns_empty_on_malformed_json() {
        let v = parse_tree("not json at all");
        assert!(v.is_empty());
    }

    #[test]
    fn parse_tree_keeps_leaf_with_missing_window_type() {
        // Legacy Xt apps don't set _NET_WM_WINDOW_TYPE; i3 leaves the
        // field blank. We must still treat them as normal app windows.
        let json = r#"
        {
            "id": 1, "type": "root", "name": "root",
            "nodes": [{
                "id": 2, "type": "workspace", "name": "1",
                "nodes": [{
                    "id": 50, "type": "con", "name": "xclock",
                    "window": 12345,
                    "window_properties": {"class": "XClock"},
                    "focused": false
                }]
            }]
        }
        "#;
        let v = parse_tree(json);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].con_id, 50);
    }

    #[test]
    fn cycle_advances_wraps_around() {
        let cands = fixture_candidates();
        let len = cands.len();
        assert_eq!(cycle_forward(0, len), 1);
        assert_eq!(cycle_forward(1, len), 2);
        // Wrap past the end back to start.
        assert_eq!(cycle_forward(2, len), 0);
    }

    #[test]
    fn cycle_shift_retreats() {
        let cands = fixture_candidates();
        let len = cands.len();
        assert_eq!(cycle_back(2, len), 1);
        assert_eq!(cycle_back(1, len), 0);
        // Wrap past zero to end.
        assert_eq!(cycle_back(0, len), 2);
    }

    #[test]
    fn cycle_handles_empty_and_singleton() {
        // Empty list never panics — returns 0.
        assert_eq!(cycle_forward(0, 0), 0);
        assert_eq!(cycle_back(0, 0), 0);
        // Singleton stays put.
        assert_eq!(cycle_forward(0, 1), 0);
        assert_eq!(cycle_back(0, 1), 0);
    }

    #[test]
    fn commit_returns_selected_con_id() {
        let cands = fixture_candidates();
        assert_eq!(commit_selection(&cands, 0), Some(100));
        assert_eq!(commit_selection(&cands, 1), Some(102));
        assert_eq!(commit_selection(&cands, 2), Some(105));
    }

    #[test]
    fn commit_out_of_range_returns_none() {
        let cands = fixture_candidates();
        assert_eq!(commit_selection(&cands, 3), None);
        assert_eq!(commit_selection(&cands, 99), None);
        assert_eq!(commit_selection(&[], 0), None);
    }

    #[test]
    fn floating_nodes_are_walked() {
        // i3 stores floating windows on a separate `floating_nodes`
        // array. The walker must descend both lists.
        let json = r#"
        {
            "id": 1, "type": "root",
            "nodes": [{
                "id": 2, "type": "workspace", "name": "1",
                "nodes": [],
                "floating_nodes": [{
                    "id": 60, "type": "floating_con",
                    "nodes": [{
                        "id": 61, "type": "con", "name": "Calculator",
                        "window": 222,
                        "window_type": "normal",
                        "window_properties": {"class": "Gnome-calculator"},
                        "focused": true
                    }]
                }]
            }]
        }
        "#;
        let v = parse_tree(json);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].con_id, 61);
        assert_eq!(v[0].class, "gnome-calculator");
    }
}
