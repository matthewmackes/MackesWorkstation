//! Exposé grid window picker — Mission-Control–style overlay.
//!
//! Phase 6.2 (v3.0.0 §6). i3 has no built-in "show every window in a
//! tile grid" gesture, so we provide one: when the user presses
//! `Super+F3` (configured in `data/i3/config.d/mackes-defaults.conf`),
//! the binding spawns `mackes-panel --expose`, which builds a
//! fullscreen, dimmed `gtk::Window` containing one Carbon card per
//! visible top-level window. Clicking a card sends an `i3-msg`
//! focus request and dismisses the overlay; Escape or a background
//! click dismisses without changing focus.
//!
//! ## Why no `i3ipc` crate?
//!
//! The rest of the panel (`i3_cluster.rs`, `windows.rs`) deliberately
//! shells out to `i3-msg`, `wmctrl`, and `xprop` rather than pulling
//! in the unmaintained `i3ipc` / `swayipc` crates. We follow the same
//! pattern here so the expose path doesn't add a new dependency tree
//! that has to be tracked through the RPM. `wmctrl -lp` gives us the
//! X11 window id (e.g. `0x03800001`), and `i3-msg [id=<x11>] focus`
//! is the canonical way to focus an X11 window via i3 — equivalent
//! to the `[con_id=…]` form the i3 IPC layer would expose, with the
//! same end behavior.
//!
//! ## X11 only
//!
//! Mackes XFCE Workstation is X11 by design (xfsettingsd, i3, xfwm4
//! fallback). The overlay uses GTK3 features (`gtk::WindowType::Popup`,
//! fullscreen via geometry move + `fullscreen()`) that behave under
//! XWayland but are not tested there. No Wayland support is planned.
//!
//! ## Grid math
//!
//! Cards are laid out in a square-ish grid with the column count
//! computed as `ceil(sqrt(n))` capped at six. Once the column count
//! exceeds six the grid grows in rows rather than getting too wide
//! to scan visually. See [`grid_columns`] and [`card_layout`].

use std::process::Command;

use gtk::glib;
use gtk::prelude::*;

use crate::{icons, windows};

/// Maximum number of columns the grid is allowed to grow to. Beyond
/// this we stack additional rows. Locked at 6 per the Phase 6.2 spec
/// — wider grids become hard to scan, narrower grids waste vertical
/// real estate when the user has more than ~9 windows open.
pub const MAX_COLUMNS: usize = 6;

/// Card edge in CSS pixels. Square cards keep the grid math simple;
/// the icon + title stack vertically inside.
const CARD_PX: i32 = 200;

/// Icon edge inside each card.
const CARD_ICON_PX: i32 = 96;

/// Card padding (every side).
const CARD_PADDING_PX: i32 = 12;

/// Cell gap in the grid.
const GRID_GAP_PX: u32 = 20;

/// A normal top-level window eligible for the Exposé grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposeWindow {
    /// X11 window id (e.g. `0x03800001`). Passed to `i3-msg [id=…] focus`.
    pub window_id: String,
    /// Window title as shown in the original tasklist. May be empty in
    /// pathological cases; the card falls back to the `WM_CLASS` token.
    pub title: String,
    /// Lower-cased `WM_CLASS` class component, used for icon lookup
    /// against `.desktop` `StartupWMClass` entries.
    pub class: String,
}

/// Compute the grid column count for `n` cards. Returns
/// `min(MAX_COLUMNS, ceil(sqrt(n)))`, with `n == 0` yielding `0`
/// (no grid to draw).
///
/// ```ignore
/// assert_eq!(grid_columns(0), 0);
/// assert_eq!(grid_columns(1), 1);
/// assert_eq!(grid_columns(4), 2);
/// assert_eq!(grid_columns(7), 3);
/// assert_eq!(grid_columns(36), 6);
/// assert_eq!(grid_columns(50), 6); // capped
/// ```
#[must_use]
pub const fn grid_columns(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    // Integer ceil(sqrt(n)) without floats: smallest k where k*k >= n,
    // searched linearly up to MAX_COLUMNS + 1.
    let mut k: usize = 1;
    while k * k < n {
        if k >= MAX_COLUMNS {
            return MAX_COLUMNS;
        }
        k += 1;
    }
    if k > MAX_COLUMNS {
        MAX_COLUMNS
    } else {
        k
    }
}

/// Layout produced for a given window count: number of columns, number
/// of rows, and the count of empty cells in the last row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridLayout {
    /// Column count (0 when `n == 0`).
    pub cols: usize,
    /// Row count (0 when `n == 0`).
    pub rows: usize,
    /// Empty trailing cells in the last row (`cols * rows - n`).
    pub empty: usize,
}

/// Compute the full grid layout for `n` cards.
///
/// For `n == 0` returns `GridLayout { cols: 0, rows: 0, empty: 0 }`.
/// For `n >= 1`: `cols = grid_columns(n)`, `rows = ceil(n / cols)`,
/// and `empty = cols * rows - n`.
///
/// ```ignore
/// // 7 windows → 3 cols × 3 rows with 2 empty trailing cells.
/// assert_eq!(card_layout(7), GridLayout { cols: 3, rows: 3, empty: 2 });
/// ```
#[must_use]
pub const fn card_layout(n: usize) -> GridLayout {
    if n == 0 {
        return GridLayout {
            cols: 0,
            rows: 0,
            empty: 0,
        };
    }
    let cols = grid_columns(n);
    // ceil(n / cols) without floats.
    let rows = n.div_ceil(cols);
    let empty = cols * rows - n;
    GridLayout { cols, rows, empty }
}

/// Return the window matching the given card index, or `None` when the
/// index is out of range (e.g. one of the empty trailing cells in the
/// last row of the grid).
#[must_use]
pub fn pick_window_for_card<'a>(
    windows: &'a [ExposeWindow],
    card_index: usize,
) -> Option<&'a ExposeWindow> {
    windows.get(card_index)
}

/// True if the window should appear in the Exposé grid. Filters out:
/// - Empty titles (almost always background / utility windows).
/// - Mackes-owned surfaces (the panel itself, the desktop wallpaper,
///   any other panel-spawned toplevel).
/// - Desktop-shell helpers (`xfdesktop`, our own `mackes-desktop`).
/// - i3's internal scratchpad-trash class `i3bar` (no point focusing it).
///
/// This is the same filter `is_panel_owned_window` in `main.rs` applies
/// to the tasklist segment of the dock, plus the `i3bar` exclusion.
/// Implemented here on a `(title, class, pid)` triple so it can be
/// unit-tested without spawning `wmctrl`.
#[must_use]
pub fn is_normal_window(title: &str, class: &str, pid: u32, our_pid: u32) -> bool {
    if title.is_empty() || title == "mackes-panel" {
        return false;
    }
    if title.starts_with("mackes-panel-") {
        return false;
    }
    if title.eq_ignore_ascii_case("desktop") || title.eq_ignore_ascii_case("xfdesktop") {
        return false;
    }
    if class.eq_ignore_ascii_case("i3bar") {
        return false;
    }
    if pid == our_pid {
        return false;
    }
    true
}

/// Project a slice of `(title, class, pid)` triples down to the normal
/// windows eligible for the grid. Pure for testability — the live path
/// builds the triples from `wmctrl -lp` + `xprop WM_CLASS`.
#[must_use]
pub fn enumerate_normal_windows<I, S1, S2>(
    rows: I,
    our_pid: u32,
) -> Vec<ExposeWindow>
where
    I: IntoIterator<Item = (String, S1, S2, u32)>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    rows.into_iter()
        .filter_map(|(window_id, title, class, pid)| {
            let title = title.as_ref();
            let class = class.as_ref();
            if !is_normal_window(title, class, pid, our_pid) {
                return None;
            }
            Some(ExposeWindow {
                window_id,
                title: title.to_owned(),
                class: class.to_ascii_lowercase(),
            })
        })
        .collect()
}

/// Live enumeration: shells `wmctrl -lp` (via [`windows::list_open_windows`])
/// then `xprop -id <w> WM_CLASS` per row, drops Mackes-owned surfaces and
/// `i3bar`, and returns the eligible cards.
#[must_use]
pub fn live_window_list() -> Vec<ExposeWindow> {
    let our_pid = std::process::id();
    let snapshot = windows::list_open_windows();
    let rows = snapshot.into_iter().map(|w| {
        let class = windows::window_wm_class(&w.window_id).unwrap_or_default();
        (w.window_id, w.title, class, w.pid)
    });
    enumerate_normal_windows(rows, our_pid)
}

/// Focus a window via i3 IPC. Uses `i3-msg [id=<X11_id>] focus`, which
/// is the canonical way to ask i3 to switch focus to a specific X11
/// window. Equivalent in effect to a `con_id=…` focus message — i3
/// resolves the X11 id to the matching container internally.
pub fn focus_via_i3(window_id: &str) {
    let criteria = format!("[id={window_id}] focus");
    match Command::new("i3-msg").arg(&criteria).spawn() {
        Ok(mut child) => {
            // Don't leave a zombie around — `wait` is cheap because
            // `i3-msg` exits immediately after writing the IPC frame.
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("mackes-panel: i3-msg {criteria} failed: {e}");
            // Fallback for environments without i3 (xfwm4 sessions on
            // the same install): wmctrl can also focus by X11 id.
            let _ = Command::new("wmctrl")
                .args(["-i", "-a", window_id])
                .spawn();
        }
    }
}

/// Open the Exposé overlay. The window is fullscreen on the primary
/// monitor (via `gtk::Window::fullscreen()`), centered, decoration-free,
/// and dismisses on Escape / background click / card click.
///
/// Picks up windows from [`live_window_list`]; renders one card per
/// window. If no eligible windows exist, the overlay shows a centered
/// "(no windows open)" hint so the gesture stays discoverable instead
/// of silently no-op'ing.
pub fn open() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_widget_name("mackes-expose");
    window.set_title("mackes-panel-expose");
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);
    window.set_keep_above(true);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 800);
    window.fullscreen();

    // GTK has to be told to listen for button-press; default Toplevel
    // window event masks don't include it. Without this the background
    // click-to-dismiss handler silently never fires.
    window.add_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);
    window.add_events(gtk::gdk::EventMask::KEY_PRESS_MASK);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_widget_name("mackes-expose-outer");
    outer.set_halign(gtk::Align::Fill);
    outer.set_valign(gtk::Align::Fill);

    let windows_for_render = live_window_list();
    let layout = card_layout(windows_for_render.len());

    if windows_for_render.is_empty() {
        let empty = gtk::Label::new(Some("(no windows open)"));
        empty.set_widget_name("mackes-expose-empty");
        empty.set_halign(gtk::Align::Center);
        empty.set_valign(gtk::Align::Center);
        outer.pack_start(&empty, true, true, 0);
    } else {
        let grid = gtk::Grid::new();
        grid.set_widget_name("mackes-expose-grid");
        grid.set_halign(gtk::Align::Center);
        grid.set_valign(gtk::Align::Center);
        grid.set_row_spacing(GRID_GAP_PX);
        grid.set_column_spacing(GRID_GAP_PX);
        grid.set_margin_top(40);
        grid.set_margin_bottom(40);
        grid.set_margin_start(40);
        grid.set_margin_end(40);

        for (idx, w) in windows_for_render.iter().enumerate() {
            // grid_columns(n) is 0 only when n == 0 — already handled.
            let col = i32::try_from(idx % layout.cols).unwrap_or(0);
            let row = i32::try_from(idx / layout.cols).unwrap_or(0);
            let card = build_card(w, &window);
            grid.attach(&card, col, row, 1, 1);
        }

        // Wrap the grid in a scrolled window so very large window
        // counts (rare, but possible — a chat-heavy session) don't
        // overflow the fullscreen viewport.
        let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scroller.set_widget_name("mackes-expose-scroll");
        scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroller.add(&grid);
        outer.pack_start(&scroller, true, true, 0);
    }

    // Background click dismisses. The grid sits inside an EventBox so
    // we can distinguish "clicked on the dimmed background" from
    // "clicked on a card" — cards stop propagation in their own
    // button-press handler.
    let event_bg = gtk::EventBox::new();
    event_bg.set_widget_name("mackes-expose-bg");
    event_bg.set_above_child(false);
    event_bg.add(&outer);
    let window_for_bg = window.clone();
    event_bg.connect_button_press_event(move |_, _| {
        window_for_bg.close();
        glib::Propagation::Stop
    });

    window.add(&event_bg);

    // Escape dismisses.
    let window_for_key = window.clone();
    window.connect_key_press_event(move |_, ev| {
        if ev.keyval() == gtk::gdk::keys::constants::Escape {
            window_for_key.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    window.show_all();
    // The fullscreen() call before show_all() is honored by most WMs
    // including i3, but xfwm4 has been observed to ignore the pre-show
    // request. Re-issue after realize as a belt-and-suspenders.
    window.fullscreen();
}

/// Build one card for the grid. `parent` is captured so the click
/// handler can dismiss the overlay after focusing the target.
fn build_card(w: &ExposeWindow, parent: &gtk::Window) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.set_widget_name("mackes-expose-card");
    frame.set_size_request(CARD_PX, CARD_PX);
    frame.set_shadow_type(gtk::ShadowType::None);
    if let Some(atk) = frame.accessible() {
        atk.set_name(&format!("Focus window: {}", w.title));
        atk.set_description(&format!(
            "Brings the window titled '{}' (class {}) to the front",
            w.title, w.class
        ));
    }
    frame.set_tooltip_text(Some(&w.title));

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_widget_name("mackes-expose-card-column");
    column.set_margin_top(CARD_PADDING_PX);
    column.set_margin_bottom(CARD_PADDING_PX);
    column.set_margin_start(CARD_PADDING_PX);
    column.set_margin_end(CARD_PADDING_PX);
    column.set_halign(gtk::Align::Fill);
    column.set_valign(gtk::Align::Fill);

    // Icon. Carbon-only — falls back to applications-other-symbolic if
    // we don't have a curated mapping for this class.
    let icon_widget: gtk::Widget =
        icons::load_with_fallback(Some(&w.class), &[], CARD_ICON_PX).map_or_else(
            || {
                gtk::Image::from_icon_name(
                    Some("applications-other-symbolic"),
                    gtk::IconSize::Dialog,
                )
                .upcast::<gtk::Widget>()
            },
            |pb| gtk::Image::from_pixbuf(Some(&pb)).upcast::<gtk::Widget>(),
        );
    icon_widget.set_halign(gtk::Align::Center);
    column.pack_start(&icon_widget, true, true, 0);

    // Title. Truncate long titles so they fit one or two lines without
    // pushing the card height.
    let title_text = truncate_title(&w.title, 60);
    let title_label = gtk::Label::new(Some(&title_text));
    title_label.set_widget_name("mackes-expose-card-title");
    title_label.set_line_wrap(true);
    title_label.set_lines(2);
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_label.set_max_width_chars(28);
    title_label.set_halign(gtk::Align::Center);
    title_label.set_justify(gtk::Justification::Center);
    column.pack_start(&title_label, false, false, 0);

    // The frame itself is not natively clickable; wrap in an EventBox.
    let event_box = gtk::EventBox::new();
    event_box.add(&column);
    event_box.add_events(gtk::gdk::EventMask::BUTTON_PRESS_MASK);

    let window_id = w.window_id.clone();
    let parent_for_click = parent.clone();
    event_box.connect_button_press_event(move |_, _| {
        focus_via_i3(&window_id);
        parent_for_click.close();
        glib::Propagation::Stop
    });

    frame.add(&event_box);
    frame
}

/// Truncate a title at the closest word boundary at or before
/// `max_chars`. Falls back to a hard char-limit truncation with an
/// ellipsis when no whitespace is found.
#[must_use]
pub fn truncate_title(title: &str, max_chars: usize) -> String {
    if title.chars().count() <= max_chars {
        return title.to_owned();
    }
    // Find the last whitespace at or before `max_chars` characters.
    let truncated: String = title.chars().take(max_chars).collect();
    if let Some(idx) = truncated.rfind(char::is_whitespace) {
        let head: String = truncated[..idx].trim_end().to_owned();
        if !head.is_empty() {
            return format!("{head}…");
        }
    }
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_columns_is_ceil_sqrt_capped_at_6() {
        // 0 → 0 (no grid).
        assert_eq!(grid_columns(0), 0);
        // ceil(sqrt(n)) over the small range.
        assert_eq!(grid_columns(1), 1);
        assert_eq!(grid_columns(2), 2);
        assert_eq!(grid_columns(3), 2);
        assert_eq!(grid_columns(4), 2);
        assert_eq!(grid_columns(5), 3);
        assert_eq!(grid_columns(7), 3);
        assert_eq!(grid_columns(9), 3);
        assert_eq!(grid_columns(10), 4);
        assert_eq!(grid_columns(16), 4);
        assert_eq!(grid_columns(17), 5);
        assert_eq!(grid_columns(25), 5);
        assert_eq!(grid_columns(26), 6);
        assert_eq!(grid_columns(36), 6);
        // Cap kicks in.
        assert_eq!(grid_columns(37), 6);
        assert_eq!(grid_columns(50), 6);
        assert_eq!(grid_columns(1_000), 6);
    }

    #[test]
    fn card_layout_matches_window_count() {
        // 7 windows → 3 cols × 3 rows with 2 empty trailing cells.
        assert_eq!(
            card_layout(7),
            GridLayout {
                cols: 3,
                rows: 3,
                empty: 2
            }
        );
        // 1 window → 1 × 1, no empties.
        assert_eq!(
            card_layout(1),
            GridLayout {
                cols: 1,
                rows: 1,
                empty: 0
            }
        );
        // 4 windows → 2 × 2, no empties (perfect square).
        assert_eq!(
            card_layout(4),
            GridLayout {
                cols: 2,
                rows: 2,
                empty: 0
            }
        );
        // 5 windows → 3 cols × 2 rows with 1 empty.
        assert_eq!(
            card_layout(5),
            GridLayout {
                cols: 3,
                rows: 2,
                empty: 1
            }
        );
        // 36 windows → 6 × 6, no empties (at the cap, perfect).
        assert_eq!(
            card_layout(36),
            GridLayout {
                cols: 6,
                rows: 6,
                empty: 0
            }
        );
        // 37 windows → 6 × 7 with 5 empty (cap forces extra row).
        assert_eq!(
            card_layout(37),
            GridLayout {
                cols: 6,
                rows: 7,
                empty: 5
            }
        );
        // 0 windows → zero everything.
        assert_eq!(
            card_layout(0),
            GridLayout {
                cols: 0,
                rows: 0,
                empty: 0
            }
        );
    }

    #[test]
    fn pick_returns_con_id_for_card_index() {
        let windows = vec![
            ExposeWindow {
                window_id: "0x01000001".into(),
                title: "Firefox".into(),
                class: "firefox".into(),
            },
            ExposeWindow {
                window_id: "0x01000002".into(),
                title: "vim notes.md".into(),
                class: "alacritty".into(),
            },
            ExposeWindow {
                window_id: "0x01000003".into(),
                title: "Thunar".into(),
                class: "thunar".into(),
            },
        ];

        // Indexed lookup returns the X11 window id we'll feed to i3-msg.
        let picked = pick_window_for_card(&windows, 0).expect("present");
        assert_eq!(picked.window_id, "0x01000001");
        assert_eq!(picked.title, "Firefox");

        let picked = pick_window_for_card(&windows, 2).expect("present");
        assert_eq!(picked.window_id, "0x01000003");

        // Out-of-range index (one of the empty trailing cells in the
        // last grid row) yields None.
        assert!(pick_window_for_card(&windows, 3).is_none());
        assert!(pick_window_for_card(&windows, 99).is_none());

        // Empty list always returns None.
        let empty: Vec<ExposeWindow> = Vec::new();
        assert!(pick_window_for_card(&empty, 0).is_none());
    }

    #[test]
    fn enumerate_filters_to_normal_windows() {
        // Fake `wmctrl -lp` + xprop tree. Our pid is 1000; we drop our
        // own panel surfaces, empty titles, the i3bar class, and the
        // wallpaper layer.
        let rows: Vec<(String, &str, &str, u32)> = vec![
            ("0x01".into(), "Firefox — Inbox", "firefox", 2001),
            ("0x02".into(), "vim ~/notes.md", "Alacritty", 2002),
            ("0x03".into(), "mackes-panel-top", "Mackes-shell", 1000),
            ("0x04".into(), "mackes-panel-desktop", "Mackes-shell", 1000),
            ("0x05".into(), "", "Random", 3003),
            ("0x06".into(), "Desktop", "xfdesktop", 4004),
            ("0x07".into(), "i3bar", "i3bar", 5005),
            ("0x08".into(), "Thunar — home", "Thunar", 6006),
            ("0x09".into(), "owned-by-us", "Anything", 1000),
        ];

        let kept = enumerate_normal_windows(rows.into_iter().map(|(w, t, c, p)| {
            (w, t.to_owned(), c.to_owned(), p)
        }), 1000);

        let ids: Vec<&str> = kept.iter().map(|w| w.window_id.as_str()).collect();
        assert_eq!(ids, vec!["0x01", "0x02", "0x08"]);

        // Classes are normalized to lower-case for downstream icon
        // resolution — `Alacritty` and `Thunar` should round-trip.
        assert_eq!(kept[1].class, "alacritty");
        assert_eq!(kept[2].class, "thunar");
    }

    #[test]
    fn is_normal_window_rejects_panel_surfaces() {
        assert!(!is_normal_window("mackes-panel", "Mackes-shell", 99, 1000));
        assert!(!is_normal_window(
            "mackes-panel-dock",
            "Mackes-shell",
            99,
            1000
        ));
        assert!(!is_normal_window("Desktop", "xfdesktop", 99, 1000));
        assert!(!is_normal_window("", "anything", 99, 1000));
        assert!(!is_normal_window("anything", "i3bar", 99, 1000));
        assert!(!is_normal_window(
            "self-owned",
            "anything",
            1000, // same pid as ours
            1000,
        ));
        // Normal app window.
        assert!(is_normal_window("Firefox — Inbox", "firefox", 9999, 1000));
    }

    #[test]
    fn enumerate_empty_when_no_rows() {
        let rows: Vec<(String, String, String, u32)> = Vec::new();
        assert!(enumerate_normal_windows(rows, 1000).is_empty());
    }

    #[test]
    fn truncate_title_short_inputs_passthrough() {
        assert_eq!(truncate_title("Firefox", 60), "Firefox");
        assert_eq!(truncate_title("", 60), "");
    }

    #[test]
    fn truncate_title_breaks_on_word_boundary() {
        let long =
            "Some Really Long Title That Definitely Exceeds The Allowed Character Budget";
        let out = truncate_title(long, 30);
        assert!(out.ends_with('…'));
        // The trimmed prefix must not exceed max_chars, AND must
        // terminate at the last space — i.e. no partial word fragments.
        let bare = out.trim_end_matches('…');
        assert!(bare.chars().count() <= 30);
        assert!(!bare.ends_with(char::is_whitespace));
        assert!(!bare.is_empty());
    }

    #[test]
    fn truncate_title_hard_truncates_when_no_whitespace() {
        // No spaces — truncate hard to max_chars + ellipsis.
        let long = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let out = truncate_title(long, 10);
        assert_eq!(out.chars().count(), 11); // 10 chars + 1 ellipsis
        assert!(out.ends_with('…'));
    }

    #[test]
    fn max_columns_constant_is_six() {
        // Spec lock — guard against an accidental bump.
        assert_eq!(MAX_COLUMNS, 6);
    }

    #[test]
    fn expose_window_round_trips() {
        let w = ExposeWindow {
            window_id: "0xabc".into(),
            title: "Title".into(),
            class: "class".into(),
        };
        let cloned = w.clone();
        assert_eq!(w, cloned);
    }
}
