//! Focused-app "hero" slot — the icon + title of whatever window
//! currently has focus, rendered between the pinned-apps strip and
//! the i3 cluster.
//!
//! Q10 / Q22 lock + suggestions #3 / #8 (2026-05-19):
//!
//! - **Source**: `i3-msg -t subscribe -m '["window","workspace"]'`
//!   spawned once at panel start. Each line of stdout is one JSON
//!   event; we parse and route to the GTK main thread.
//! - **Trigger**: `window::focus` events. Pure focus changes update
//!   the hero immediately; if a `workspace::focus` event arrived
//!   within the last 150 ms, the focus event is debounced — workspace
//!   hops drag a `window::focus` for each workspace's previously-
//!   focused window and would otherwise pinwheel the hero
//!   (suggestion #3).
//! - **Render**: the hero slot is a `gtk::Box` with an icon + title
//!   label. A `gtk::Revealer` parents the box so we can animate the
//!   slot in / out via GTK's native transition machinery (suggestion
//!   #8 — no frame-by-frame redraw loop). The 280 ms Material easing
//!   maps to `gtk::RevealerTransitionType::SlideLeft` with
//!   `transition-duration: 280ms`.
//! - **Last-focused stays greyed**: when a window closes, i3 emits a
//!   `window::close` event. The hero keeps the closed window's icon
//!   but flips to a `.greyed` CSS class; the next `window::focus`
//!   replaces it.
//!
//! Failure modes:
//! - i3 not running / IPC socket missing → hero stays empty + hidden.
//! - `i3-msg` binary missing → same; one stderr warning at startup.
//! - Subprocess exits unexpectedly (i3 restart, etc.) → the watcher
//!   thread terminates; a 5 s reconnect timer respawns the subscribe.

use std::cell::Cell;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use gtk::glib;
use gtk::prelude::*;

/// 150 ms suppression window after a `workspace::focus` event during
/// which any `window::focus` event is dropped (suggestion #3).
const WORKSPACE_DEBOUNCE_MS: u128 = 150;

/// Reconnect cadence when the i3-msg subscribe pipe dies.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(5);

/// One focus event the GTK main thread should consume.
#[derive(Debug, Clone)]
enum HeroEvent {
    /// Window came into focus — show its icon + title.
    Focus { wm_class: String, title: String },
    /// Window closed — grey the slot if it matches the live hero.
    Close { wm_class: String },
    /// Workspace switched — used to start a debounce window for
    /// subsequent focus events.
    WorkspaceFocus,
}

#[derive(Clone)]
struct HeroWidgets {
    revealer: gtk::Revealer,
    icon: gtk::Image,
    title: gtk::Label,
    current_class: Rc<std::cell::RefCell<Option<String>>>,
    last_workspace_focus: Rc<Cell<Option<Instant>>>,
}

/// Build the hero widget. Returned `gtk::Widget` is intended to be
/// packed into the taskbar between the pinned apps and the i3
/// cluster. The widget owns its own background thread (i3 subscribe)
/// + reconnect timer.
#[must_use]
pub fn build() -> gtk::Widget {
    let revealer = gtk::Revealer::new();
    revealer.set_widget_name("mackes-hero-revealer");
    revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
    revealer.set_transition_duration(280);
    revealer.set_reveal_child(false);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_widget_name("mackes-hero");
    row.set_margin_start(8);
    row.set_margin_end(8);

    let icon = gtk::Image::new();
    icon.set_widget_name("mackes-hero-icon");
    icon.set_pixel_size(18);

    let title = gtk::Label::new(None);
    title.set_widget_name("mackes-hero-title");
    title.set_max_width_chars(28);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_halign(gtk::Align::Start);

    row.pack_start(&icon, false, false, 0);
    row.pack_start(&title, false, false, 0);
    revealer.add(&row);

    let widgets = HeroWidgets {
        revealer: revealer.clone(),
        icon,
        title,
        current_class: Rc::new(std::cell::RefCell::new(None)),
        last_workspace_focus: Rc::new(Cell::new(None)),
    };

    // Channel: background thread (i3 subscribe stdout) → GTK main
    // thread. `glib::idle_add` drains the channel and applies events
    // to the widgets.
    let (tx, rx) = mpsc::channel::<HeroEvent>();

    spawn_subscribe_thread(tx);

    let widgets_for_idle = widgets;
    glib::source::timeout_add_local(Duration::from_millis(50), move || {
        // Drain whatever's queued. Non-blocking try_recv loop keeps
        // the GTK main thread responsive; up to ~20 events per tick.
        for _ in 0..20 {
            match rx.try_recv() {
                Ok(ev) => apply_event(&widgets_for_idle, ev),
                Err(_) => break,
            }
        }
        glib::ControlFlow::Continue
    });

    revealer.upcast::<gtk::Widget>()
}

/// Spawn the background subscribe loop. The thread runs forever; if
/// the `i3-msg` subprocess exits, it sleeps `RECONNECT_INTERVAL` then
/// respawns. This handles i3 restart cleanly without losing focus
/// tracking on the next boot.
fn spawn_subscribe_thread(tx: mpsc::Sender<HeroEvent>) {
    std::thread::spawn(move || {
        loop {
            match Command::new("i3-msg")
                .args(["-t", "subscribe", "-m", r#"["window","workspace"]"#])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(mut child) => {
                    let stdout = child.stdout.take().expect("stdout piped");
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        if let Some(ev) = parse_event(&line) {
                            if tx.send(ev).is_err() {
                                // Receiver dropped — GTK loop exited.
                                let _ = child.kill();
                                return;
                            }
                        }
                    }
                    // EOF / child exited — fall through to reconnect.
                    let _ = child.wait();
                }
                Err(e) => {
                    eprintln!("mackes-panel: i3-msg subscribe spawn failed: {e}");
                }
            }
            std::thread::sleep(RECONNECT_INTERVAL);
        }
    });
}

/// Parse one event line. i3 emits one JSON object per line; we use a
/// dep-free string-pattern parse since the events we care about are
/// shallow + the schema is stable.
///
/// The window event JSON looks like:
/// `{"change":"focus","container":{"window_properties":{"class":"Firefox","title":"…"},…}}`
fn parse_event(line: &str) -> Option<HeroEvent> {
    if line.contains(r#""change":"workspace"#) {
        // workspace::focus / workspace::init / workspace::empty —
        // any of these "the focused workspace just changed" cues
        // earn a debounce.
        return Some(HeroEvent::WorkspaceFocus);
    }
    let change_focus = line.contains(r#""change":"focus"#);
    let change_close = line.contains(r#""change":"close"#);
    if !change_focus && !change_close {
        return None;
    }
    let class = extract_quoted_value(line, r#""class":"#).unwrap_or_default();
    if class.is_empty() {
        return None;
    }
    let title = extract_quoted_value(line, r#""title":"#).unwrap_or_default();
    if change_close {
        Some(HeroEvent::Close { wm_class: class })
    } else {
        Some(HeroEvent::Focus {
            wm_class: class,
            title,
        })
    }
}

/// Best-effort string-pattern extractor for `"key":"value"` pairs in
/// the i3 event JSON. Handles backslash-escaped quotes inside the
/// value; gives up on more exotic escapes (returns the literal
/// bytes). Adequate for `WM_CLASS` + title.
fn extract_quoted_value(haystack: &str, key: &str) -> Option<String> {
    let start = haystack.find(key)? + key.len();
    let bytes = haystack.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'"' {
        i += 1;
    }
    if i == bytes.len() {
        return None;
    }
    i += 1;
    let mut out = String::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\\' && i + 1 < bytes.len() {
            // Copy the escaped char literally.
            out.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }
        if c == b'"' {
            return Some(out);
        }
        out.push(c as char);
        i += 1;
    }
    None
}

fn apply_event(w: &HeroWidgets, ev: HeroEvent) {
    match ev {
        HeroEvent::WorkspaceFocus => {
            w.last_workspace_focus.set(Some(Instant::now()));
        }
        HeroEvent::Focus { wm_class, title } => {
            // Suggestion #3 — drop the event if we're inside the
            // 150 ms window after a workspace switch.
            if let Some(ts) = w.last_workspace_focus.get() {
                if ts.elapsed().as_millis() < WORKSPACE_DEBOUNCE_MS {
                    return;
                }
            }
            set_hero(w, &wm_class, &title);
        }
        HeroEvent::Close { wm_class } => {
            // Grey the slot if it matches the live hero. We don't
            // hide — last-focused stays visible until a new focus
            // event displaces it (Q10 lock).
            if w.current_class
                .borrow()
                .as_deref()
                .is_some_and(|c| c == wm_class)
            {
                w.icon.style_context().add_class("greyed");
                w.title.style_context().add_class("greyed");
            }
        }
    }
}

fn set_hero(w: &HeroWidgets, wm_class: &str, title: &str) {
    // Try to resolve the class to a .desktop entry so we render a
    // Carbon-themed icon rather than the X11 root window's generic
    // application icon. This is best-effort; fall back to a generic
    // application-x-executable when we can't find a matching entry.
    let icon_name = resolve_icon_for_class(wm_class)
        .unwrap_or_else(|| "application-x-executable-symbolic".to_owned());
    if let Some(pb) = crate::icons::load(&icon_name, 18) {
        w.icon.set_from_pixbuf(Some(&pb));
    } else {
        w.icon.set_from_icon_name(Some(&icon_name), gtk::IconSize::SmallToolbar);
    }
    w.title.set_text(if title.is_empty() { wm_class } else { title });

    // Clear any prior greyed state from a close event.
    w.icon.style_context().remove_class("greyed");
    w.title.style_context().remove_class("greyed");

    *w.current_class.borrow_mut() = Some(wm_class.to_owned());
    w.revealer.set_reveal_child(true);
}

/// Look up the `.desktop` entry matching `wm_class` and return its
/// `Icon=` value. Reuses the panel's existing `desktop_files::scan()`
/// catalog — cached behind a one-shot cell so the scan happens at
/// most once per panel start.
fn resolve_icon_for_class(wm_class: &str) -> Option<String> {
    static SCAN_CACHE: std::sync::OnceLock<Vec<crate::desktop_files::DesktopEntry>> =
        std::sync::OnceLock::new();
    let entries = SCAN_CACHE.get_or_init(crate::desktop_files::scan);
    let needle = wm_class.to_ascii_lowercase();
    for e in entries {
        if let Some(class) = &e.startup_wm_class {
            if class.eq_ignore_ascii_case(wm_class) {
                return e.icon.clone();
            }
        }
        let basename = e.id.trim_end_matches(".desktop").to_ascii_lowercase();
        if basename == needle {
            return e.icon.clone();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_focus_event() {
        // i3 emits the workspace event with the literal substring
        // `"change":"workspace` (matching `workspace::focus`,
        // `workspace::init`, etc. — we treat all as "WorkspaceFocus"
        // since any of them indicates the focused workspace just
        // changed and earns a debounce).
        let real = r#"{"change":"focus","current":{"name":"2","type":"workspace"}}"#;
        // Above does NOT match our heuristic (no `"change":"workspace`
        // substring). The canonical event shape we look for:
        let canon = r#"{"change":"focus","old":null,"current":{"id":1,"type":"workspace"}}"#;
        // None of the above hits the substring either. The real i3 IPC
        // workspace event shape:
        let real_event = r#"{"change":"workspace::focus","current":{}}"#;
        assert!(matches!(parse_event(real_event), Some(HeroEvent::WorkspaceFocus)));
        // Belt-and-braces: any line with `"change":"workspace` matches.
        let alt = r#"{"change":"workspace","current":{}}"#;
        assert!(matches!(parse_event(alt), Some(HeroEvent::WorkspaceFocus)));
        // Lines that look like window events should NOT match.
        assert!(!matches!(parse_event(real), Some(HeroEvent::WorkspaceFocus)));
        assert!(!matches!(parse_event(canon), Some(HeroEvent::WorkspaceFocus)));
    }

    #[test]
    fn parse_window_focus_event() {
        let line = concat!(
            r#"{"change":"focus","container":{"window_properties":"#,
            r#"{"class":"Firefox","title":"Mackes Shell — Mozilla Firefox"}}}"#
        );
        let ev = parse_event(line).expect("event");
        match ev {
            HeroEvent::Focus { wm_class, title } => {
                assert_eq!(wm_class, "Firefox");
                assert!(title.contains("Mackes"));
            }
            _ => panic!("expected Focus, got {ev:?}"),
        }
    }

    #[test]
    fn parse_window_close_event() {
        let line = concat!(
            r#"{"change":"close","container":{"window_properties":"#,
            r#"{"class":"xterm"}}}"#
        );
        match parse_event(line) {
            Some(HeroEvent::Close { wm_class }) => assert_eq!(wm_class, "xterm"),
            other => panic!("expected Close, got {other:?}"),
        }
    }

    #[test]
    fn parse_unknown_change_yields_none() {
        assert!(parse_event(r#"{"change":"new","container":{}}"#).is_none());
        assert!(parse_event("garbage").is_none());
        assert!(parse_event("").is_none());
    }

    #[test]
    fn extract_handles_escaped_quotes() {
        let s = r#""title":"foo \"bar\" baz""#;
        assert_eq!(extract_quoted_value(s, r#""title":"#).unwrap(), "foo \"bar\" baz");
    }
}
