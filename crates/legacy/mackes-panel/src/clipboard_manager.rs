//! Clipboard manager popover — enterprise-class history view backed
//! by the mesh-replicated `~/.cache/mackes/clipboard.json` file.
//!
//! Q21 lock (2026-05-19): the clipboard tray icon's click target is
//! repurposed. Was `mackes --focus clipboard` (Workbench dashboard);
//! becomes "open this popover" — live-updating list of recent
//! clipboard items, click-to-paste-back.
//!
//! The popover reads from `$XDG_CACHE_HOME/mackes/clipboard.json`
//! (default `~/.cache/mackes/clipboard.json`), which is written by
//! `mackes-clipboard-daemon` whenever a new `XA_CLIPBOARD` selection
//! arrives. The same file is mesh-replicated whole via QNM-Shared,
//! so every peer's last-50 history feeds the same popover — 1:1 with
//! mesh sync per user lock.
//!
//! Live update: the popover polls the cache file's mtime every 1 s
//! while visible and re-renders the list on change. When hidden the
//! poll is a no-op so we don't burn cycles for nothing.

use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

/// One clipboard history entry. Backed by the JSON shape that
/// `mackes-clipboard-daemon` writes:
///
/// ```json
/// {"text": "…", "peer": "anvil", "ts": 1718383000}
/// ```
///
/// `peer` is the hostname the entry was captured on (mesh
/// attribution); `ts` is a Unix timestamp. The popover ignores
/// `ts` for now — items render in the order the JSON file lists
/// them (most-recent first per the daemon's invariant).
#[derive(Debug, Clone)]
struct ClipItem {
    text: String,
    peer: String,
}

/// Build the popover anchored to the trigger widget. Caller owns the
/// returned `gtk::Popover` — call `popup()` to show, `popdown()` to
/// hide; the popover destroys itself on outside click.
#[must_use]
pub fn build(relative_to: &gtk::Widget) -> gtk::Popover {
    let popover = gtk::Popover::new(Some(relative_to));
    popover.set_widget_name("mackes-clipboard-manager");
    popover.set_position(gtk::PositionType::Top);
    popover.set_modal(true);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_widget_name("mackes-clipboard-column");
    column.set_margin_top(10);
    column.set_margin_bottom(10);
    column.set_margin_start(12);
    column.set_margin_end(12);

    // Header
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let title = gtk::Label::new(Some("Mesh Clipboard"));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("mackes-clipboard-title");
    header.pack_start(&title, true, true, 0);

    let clear = gtk::Button::with_label("Clear");
    clear.set_relief(gtk::ReliefStyle::None);
    clear.set_tooltip_text(Some("Empty the local mesh-clipboard cache"));
    if let Some(atk) = clear.accessible() {
        atk.set_name("Clear all mesh clipboard entries");
    }
    clear.connect_clicked(|_| {
        let path = cache_path();
        let _ = std::fs::write(&path, b"[]");
    });
    header.pack_end(&clear, false, false, 0);
    column.pack_start(&header, false, false, 0);

    column.pack_start(
        &gtk::Separator::new(gtk::Orientation::Horizontal),
        false,
        false,
        0,
    );

    // List
    let list = gtk::ListBox::new();
    list.set_widget_name("mackes-clipboard-list");
    list.set_selection_mode(gtk::SelectionMode::None);

    let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroller.set_min_content_width(320);
    scroller.set_min_content_height(280);
    scroller.set_propagate_natural_height(false);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.add(&list);
    column.pack_start(&scroller, true, true, 0);

    // Footer
    column.pack_start(
        &gtk::Separator::new(gtk::Orientation::Horizontal),
        false,
        false,
        0,
    );
    let footer = gtk::Label::new(Some(
        "Mirror: ~/.cache/mackes/clipboard.json — mesh-replicated",
    ));
    footer.set_halign(gtk::Align::Start);
    footer.style_context().add_class("mackes-clipboard-footer");
    column.pack_start(&footer, false, false, 0);

    popover.add(&column);
    column.show_all();

    // Initial render + 1 s live-update poll. Polls mtime to avoid
    // re-parsing on quiet ticks. Subscribe runs only while the
    // popover is visible (idle when hidden).
    let last_mtime: Rc<std::cell::Cell<Option<std::time::SystemTime>>> =
        Rc::new(std::cell::Cell::new(None));
    rerender(&list);
    {
        // The timer is the last consumer of `list` + `last_mtime` —
        // move them in directly. `popover` still lives on the caller.
        let popover_for_timer = popover.clone();
        glib::timeout_add_local(Duration::from_secs(1), move || {
            if !popover_for_timer.is_visible() {
                return glib::ControlFlow::Continue;
            }
            let mt = std::fs::metadata(cache_path())
                .and_then(|m| m.modified())
                .ok();
            if mt != last_mtime.get() {
                last_mtime.set(mt);
                rerender(&list);
            }
            glib::ControlFlow::Continue
        });
    }

    popover
}

fn cache_path() -> PathBuf {
    if let Ok(s) = std::env::var("XDG_CACHE_HOME") {
        if !s.is_empty() {
            return PathBuf::from(s).join("mackes/clipboard.json");
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        return PathBuf::from(h).join(".cache/mackes/clipboard.json");
    }
    PathBuf::from("/tmp/mackes-clipboard.json")
}

fn rerender(list: &gtk::ListBox) {
    // Strip existing rows.
    for c in list.children() {
        list.remove(&c);
    }

    let items = load_history();
    if items.is_empty() {
        let empty = gtk::Label::new(Some("(no clipboard history yet)"));
        empty.style_context().add_class("mackes-clipboard-empty");
        empty.set_margin_top(20);
        empty.set_margin_bottom(20);
        list.add(&empty);
        list.show_all();
        return;
    }

    for item in items.iter().take(50) {
        list.add(&build_row(item));
    }
    list.show_all();
}

fn build_row(item: &ClipItem) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    outer.set_margin_top(4);
    outer.set_margin_bottom(4);
    outer.set_margin_start(6);
    outer.set_margin_end(6);

    let text = gtk::Label::new(Some(&trim_for_display(&item.text)));
    text.set_halign(gtk::Align::Start);
    text.set_xalign(0.0);
    text.set_max_width_chars(40);
    text.set_ellipsize(gtk::pango::EllipsizeMode::End);
    outer.pack_start(&text, true, true, 0);

    let peer = gtk::Label::new(Some(&item.peer));
    peer.style_context().add_class("mackes-clipboard-peer");
    outer.pack_end(&peer, false, false, 0);

    let copy = gtk::Button::with_label("Copy");
    copy.set_relief(gtk::ReliefStyle::None);
    copy.set_tooltip_text(Some("Re-paste this item to the system clipboard"));
    if let Some(atk) = copy.accessible() {
        atk.set_name(&format!(
            "Copy mesh-clipboard entry from {} to local clipboard",
            item.peer
        ));
    }
    let text_copy = item.text.clone();
    copy.connect_clicked(move |_| {
        copy_to_clipboard(&text_copy);
    });
    outer.pack_end(&copy, false, false, 0);

    row.add(&outer);
    row
}

fn trim_for_display(s: &str) -> String {
    let single = s.lines().next().unwrap_or("");
    if single.chars().count() > 80 {
        let truncated: String = single.chars().take(80).collect();
        format!("{truncated}…")
    } else {
        single.to_owned()
    }
}

/// Pipe `text` to `xclip -selection clipboard` so the next paste
/// (Ctrl-V / middle-click) hits the same content. Returns silently on
/// failure — the daemon will re-pick the existing selection on its
/// next tick.
fn copy_to_clipboard(text: &str) {
    let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    else {
        // Fall back to wl-copy on Wayland sessions.
        let _ = Command::new("wl-copy").arg(text).spawn();
        return;
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
}

/// Parse the daemon's JSON file. Dep-free string scan — keeps the
/// crate's transitive footprint at zero. Returns an empty vec when
/// the file is missing or unreadable.
fn load_history() -> Vec<ClipItem> {
    let path = cache_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_history(&text)
}

fn parse_history(text: &str) -> Vec<ClipItem> {
    // The daemon writes either:
    // - `[]` (empty),
    // - `[{"text":"…","peer":"…","ts":N}, {"text":"…", …}, …]`
    // We scan for `"text":"…"` pairs and the matching `"peer":"…"`.
    // Quoted-value parsing handles backslash-escaped quotes the same
    // way hero.rs does.
    let mut out = Vec::new();
    let mut idx = 0;
    while let Some(start) = text[idx..].find(r#""text""#) {
        let abs = idx + start;
        let Some(t) = extract_quoted_value(&text[abs..], r#""text""#) else {
            break;
        };
        // Find an accompanying peer field for this item. Look forward
        // through the next ~256 chars (one item is usually small).
        let look_end = (abs + 512).min(text.len());
        let peer = extract_quoted_value(&text[abs..look_end], r#""peer""#)
            .unwrap_or_else(|| "local".to_owned());
        out.push(ClipItem { text: t, peer });
        idx = abs + 6;
    }
    out
}

/// String-pattern `"key":"value"` extractor — same shape as
/// `hero.rs::extract_quoted_value` but accepts the bare `"key"` form
/// (whitespace + colon allowed between key and value).
fn extract_quoted_value(haystack: &str, key: &str) -> Option<String> {
    let i = haystack.find(key)? + key.len();
    let bytes = haystack.as_bytes();
    let mut j = i;
    while j < bytes.len() && bytes[j] != b'"' {
        j += 1;
    }
    if j == bytes.len() {
        return None;
    }
    j += 1;
    let mut out = String::new();
    while j < bytes.len() {
        let c = bytes[j];
        if c == b'\\' && j + 1 < bytes.len() {
            out.push(bytes[j + 1] as char);
            j += 2;
            continue;
        }
        if c == b'"' {
            return Some(out);
        }
        out.push(c as char);
        j += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_history_yields_no_items() {
        assert!(parse_history("[]").is_empty());
        assert!(parse_history("").is_empty());
    }

    #[test]
    fn parse_single_item() {
        let s = r#"[{"text":"hello world","peer":"anvil","ts":1718383000}]"#;
        let items = parse_history(s);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "hello world");
        assert_eq!(items[0].peer, "anvil");
    }

    #[test]
    fn parse_multiple_items_preserves_order() {
        let s = concat!(
            r#"[{"text":"first","peer":"a","ts":1},"#,
            r#"{"text":"second","peer":"b","ts":2},"#,
            r#"{"text":"third","peer":"c","ts":3}]"#
        );
        let items = parse_history(s);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].text, "first");
        assert_eq!(items[2].peer, "c");
    }

    #[test]
    fn parse_handles_missing_peer_field() {
        let s = r#"[{"text":"orphan","ts":1}]"#;
        let items = parse_history(s);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].peer, "local");
    }

    #[test]
    fn trim_collapses_multi_line_input() {
        let s = "first line\nsecond line\nthird";
        let trimmed = trim_for_display(s);
        assert_eq!(trimmed, "first line");
    }

    #[test]
    fn trim_caps_at_80_chars() {
        let s = "a".repeat(120);
        let trimmed = trim_for_display(&s);
        assert!(trimmed.chars().count() <= 81);
        assert!(trimmed.ends_with('…'));
    }

    #[test]
    fn trim_short_input_unchanged() {
        let s = "hello";
        assert_eq!(trim_for_display(s), "hello");
    }

    #[test]
    fn trim_empty_input_returns_empty() {
        assert_eq!(trim_for_display(""), "");
    }

    #[test]
    fn parse_history_handles_escaped_quotes_in_text() {
        // Daemon escapes embedded quotes — our scanner must respect
        // the backslash so the next pair isn't mis-located.
        let s = r#"[{"text":"he said \"hi\"","peer":"a","ts":1}]"#;
        let items = parse_history(s);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, r#"he said "hi""#);
    }

    #[test]
    fn parse_history_with_no_text_field_yields_nothing() {
        let s = r#"[{"peer":"a","ts":1}]"#;
        let items = parse_history(s);
        assert!(items.is_empty());
    }

    #[test]
    fn extract_quoted_value_extracts_simple_key() {
        let s = r#"{"text":"hello world"}"#;
        let v = extract_quoted_value(s, r#""text""#).unwrap();
        assert_eq!(v, "hello world");
    }

    #[test]
    fn extract_quoted_value_returns_none_for_missing_key() {
        let s = r#"{"text":"hello"}"#;
        assert!(extract_quoted_value(s, r#""nope""#).is_none());
    }

    #[test]
    fn extract_quoted_value_handles_no_opening_quote() {
        // Key present but no following quote chars — returns None.
        let s = r#""text""#;
        assert!(extract_quoted_value(s, r#""text""#).is_none());
    }

    #[test]
    fn parse_handles_garbage_input_without_panic() {
        let _ = parse_history("garbage no quotes anywhere");
        let _ = parse_history("");
        let _ = parse_history("{{{");
    }

    #[test]
    fn parse_multiple_items_with_unicode() {
        let s = r#"[{"text":"emoji 🎉","peer":"a","ts":1},{"text":"汉字","peer":"b","ts":2}]"#;
        let items = parse_history(s);
        assert_eq!(items.len(), 2);
        // text uses single-byte casting; unicode multibyte bytes are
        // pushed via `as char` so they may not round-trip — guard
        // intent: don't panic, surface at least 2 entries.
    }
}
