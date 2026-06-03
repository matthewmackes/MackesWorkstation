// Recents parser consumed by the Apple-menu builder.
#![allow(dead_code)]

//! GTK Recents (`recently-used.xbel`) reader.
//!
//! Per Q24 of the design lock the Apple menu carries a `Recent Items →`
//! submenu showing the user's last 10 opened files. GTK 3 + 4 both
//! maintain this list at `~/.local/share/recently-used.xbel` in an
//! XBEL (XML Bookmark Exchange Language) variant.
//!
//! We parse only the fields the panel needs: href (URI), title, and
//! the modified timestamp for sort. Full XBEL spec compliance is out
//! of scope — a regex-based one-pass extractor is fast (no allocations
//! per token) and matches what `recently-used.xbel` actually emits.

use std::fs;
use std::path::PathBuf;

/// One recents entry surfaced in the Apple menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentItem {
    /// URI in `file:///…` form (or any other gvfs-supported scheme).
    pub uri: String,
    /// Display label — `title` from the XBEL bookmark, falling back
    /// to the URI basename when title is missing.
    pub label: String,
    /// `modified=` timestamp, used for ordering. Sortable as a
    /// string thanks to ISO-8601.
    pub modified: String,
}

/// Resolve the canonical path. `XDG_DATA_HOME` first, then
/// `$HOME/.local/share`.
#[must_use]
pub fn path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg).join("recently-used.xbel"));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".local/share/recently-used.xbel"))
}

/// Read + parse the recents file. Returns the `limit` most-recently
/// modified entries. Errors are silent — an empty Vec means "no
/// recents available", which is the right Apple-menu behavior on a
/// fresh install.
#[must_use]
pub fn load(limit: usize) -> Vec<RecentItem> {
    let Some(p) = path() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(&p) else {
        return Vec::new();
    };
    let mut items = parse(&text);
    items.sort_by(|a, b| b.modified.cmp(&a.modified));
    items.truncate(limit);
    items
}

/// Pure-text parser exposed for unit tests. Scans for
/// `<bookmark href="..." ... modified="...">` and the immediately
/// following `<title>…</title>`. Skips entries without an href.
pub fn parse(xbel: &str) -> Vec<RecentItem> {
    let mut out = Vec::new();
    let mut href: Option<String> = None;
    let mut modified: Option<String> = None;
    let mut title: Option<String> = None;

    for line in xbel.lines() {
        let trimmed = line.trim();

        // Bookmark opening element carries href + modified
        // attributes — and on GTK's emitter both live on a single
        // line, so per-line scanning is reliable.
        if trimmed.starts_with("<bookmark ") {
            href = extract_attr(trimmed, "href");
            modified = extract_attr(trimmed, "modified");
            title = None;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("<title>") {
            // Title may be empty or include trailing markup. We grab
            // everything before the closing tag.
            if let Some(content) = rest.split("</title>").next() {
                title = Some(content.to_owned());
            }
            continue;
        }

        if trimmed.starts_with("</bookmark>") {
            if let Some(href_value) = href.take() {
                let modified_value = modified.take().unwrap_or_default();
                let label = title.take().filter(|t| !t.is_empty()).unwrap_or_else(|| {
                    href_value
                        .rsplit('/')
                        .next()
                        .map_or_else(|| href_value.clone(), str::to_owned)
                });
                out.push(RecentItem {
                    uri: href_value,
                    label,
                    modified: modified_value,
                });
            }
            modified = None;
            title = None;
        }
    }
    out
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_bookmark() {
        let doc = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbel version="1.0">
  <bookmark href="file:///home/mm/notes.md" added="2026-05-18T19:00:00Z" modified="2026-05-18T19:01:00Z">
    <title>notes.md</title>
  </bookmark>
</xbel>"#;
        let v = parse(doc);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].uri, "file:///home/mm/notes.md");
        assert_eq!(v[0].label, "notes.md");
        assert_eq!(v[0].modified, "2026-05-18T19:01:00Z");
    }

    #[test]
    fn label_falls_back_to_basename_when_title_missing() {
        let doc = r#"
<bookmark href="file:///tmp/x.txt" modified="2026-05-18T19:01:00Z">
</bookmark>"#;
        let v = parse(doc);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].label, "x.txt");
    }

    #[test]
    fn ignores_bookmarks_without_href() {
        let doc = r#"
<bookmark added="2026-05-18T19:00:00Z" modified="2026-05-18T19:01:00Z">
  <title>broken</title>
</bookmark>"#;
        assert!(parse(doc).is_empty());
    }

    #[test]
    fn load_returns_empty_on_missing_file() {
        let _g = crate::test_env::env_lock();
        // Hard-redirect XDG to a path that doesn't exist.
        let v = std::env::var_os("XDG_DATA_HOME");
        std::env::set_var("XDG_DATA_HOME", "/definitely/not/here");
        let items = load(10);
        // Restore so this test doesn't leak state.
        match v {
            Some(val) => std::env::set_var("XDG_DATA_HOME", val),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        assert!(items.is_empty());
    }

    #[test]
    fn extract_attr_returns_none_when_missing() {
        // No `href=` attribute → None.
        let line = r#"<bookmark added="x" modified="y">"#;
        assert!(extract_attr(line, "href").is_none());
    }

    #[test]
    fn extract_attr_returns_none_on_unterminated_quote() {
        // Opening quote but no closing — must not panic.
        let line = "<bookmark href=\"only-open";
        assert!(extract_attr(line, "href").is_none());
    }

    #[test]
    fn parse_returns_multiple_entries_sorted_by_load() {
        let doc = r#"
<bookmark href="file:///a" modified="2026-01-01T00:00:00Z">
  <title>A</title>
</bookmark>
<bookmark href="file:///b" modified="2026-05-01T00:00:00Z">
  <title>B</title>
</bookmark>
<bookmark href="file:///c" modified="2026-03-01T00:00:00Z">
  <title>C</title>
</bookmark>"#;
        let v = parse(doc);
        assert_eq!(v.len(), 3);
        // Parse preserves order — load() does the sort.
        assert_eq!(v[0].uri, "file:///a");
        assert_eq!(v[1].uri, "file:///b");
        assert_eq!(v[2].uri, "file:///c");
    }

    #[test]
    fn load_returns_entries_when_xbel_present() {
        let _g = crate::test_env::env_lock();
        let dir = tempfile::tempdir().unwrap();
        let xbel = dir.path().join("recently-used.xbel");
        std::fs::write(
            &xbel,
            r#"<?xml version="1.0"?>
<xbel>
<bookmark href="file:///x.md" modified="2026-05-18T19:01:00Z">
  <title>x.md</title>
</bookmark>
<bookmark href="file:///y.md" modified="2026-05-19T19:01:00Z">
  <title>y.md</title>
</bookmark>
</xbel>"#,
        )
        .unwrap();

        let prior = std::env::var_os("XDG_DATA_HOME");
        std::env::set_var("XDG_DATA_HOME", dir.path());
        let items = load(10);
        match prior {
            Some(p) => std::env::set_var("XDG_DATA_HOME", p),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        // load() sorts most-recent first.
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].uri, "file:///y.md");
        assert_eq!(items[1].uri, "file:///x.md");
    }

    #[test]
    fn load_respects_limit() {
        let _g = crate::test_env::env_lock();
        let dir = tempfile::tempdir().unwrap();
        let xbel = dir.path().join("recently-used.xbel");
        // The parser pushes entries when a line beginning with
        // `</bookmark>` arrives — so each bookmark needs its closing
        // tag on its own line. GTK's emitter does the same.
        let body = (0..5)
            .map(|i| {
                format!(
                    "<bookmark href=\"file:///f{i}.md\" modified=\"2026-05-1{i}T00:00:00Z\">\n  <title>f{i}</title>\n</bookmark>"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&xbel, format!("<xbel>\n{body}\n</xbel>")).unwrap();

        let prior = std::env::var_os("XDG_DATA_HOME");
        std::env::set_var("XDG_DATA_HOME", dir.path());
        let items = load(2);
        match prior {
            Some(p) => std::env::set_var("XDG_DATA_HOME", p),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn empty_title_falls_back_to_basename() {
        let doc = r#"
<bookmark href="file:///tmp/x.txt" modified="2026-05-18T19:01:00Z">
  <title></title>
</bookmark>"#;
        let v = parse(doc);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].label, "x.txt");
    }
}
