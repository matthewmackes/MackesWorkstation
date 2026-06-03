// Desktop-files API consumed by Phase 3.2 (Apple-menu Applications →)
// and Phase 5 (pinned launchers).
#![allow(dead_code)]

//! `.desktop` file enumeration and parsing.
//!
//! Walks the standard freedesktop application directories and parses each
//! `.desktop` file into a `DesktopEntry`. Doesn't try to be a full
//! freedesktop spec parser — only the fields the Apple-menu and dock
//! actually need:
//!
//! * `Name`           — display label
//! * `Exec`           — command to launch (with `%U`/`%F` field codes preserved)
//! * `Icon`           — freedesktop icon name (resolved via `icons::load`)
//! * `Categories`     — semicolon-separated list, used by Phase 3.2
//! * `NoDisplay`      — boolean; entries with `NoDisplay=true` are skipped
//! * `Hidden`         — boolean; entries with `Hidden=true` are skipped
//! * `Terminal`       — boolean; passed through for Phase 5's launcher
//!
//! User-side `~/.local/share/applications/` shadows system-side
//! `/usr/share/applications/` by basename, matching how
//! `desktop-file-utils` resolves clashes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Standard search roots, ordered system → user. Later entries shadow
/// earlier ones by `.desktop` basename.
const SEARCH_ROOTS: &[&str] = &["/usr/share/applications", "/usr/local/share/applications"];

/// A parsed `.desktop` entry. Only the fields the panel reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    /// File basename (e.g. `firefox.desktop`). Used as the stable id.
    pub id: String,
    /// `Name=` field.
    pub name: String,
    /// `Exec=` field with `%U`/`%F` codes preserved.
    pub exec: String,
    /// `Icon=` field. May be a freedesktop icon name or an absolute path.
    pub icon: Option<String>,
    /// `Categories=` parsed into a vector.
    pub categories: Vec<String>,
    /// `Terminal=true` → launcher must spawn through a terminal.
    pub terminal: bool,
    /// `StartupWMClass=` if present. When set, the dock matches running
    /// windows to this launcher by checking their X11 `WM_CLASS` second
    /// component against this string (case-insensitive). When absent,
    /// the matcher falls back to the `.desktop` basename — works for the
    /// common case where the `WM_CLASS` already mirrors the launcher id
    /// (e.g. firefox.desktop ↔ "firefox").
    pub startup_wm_class: Option<String>,
}

/// Walk every `SEARCH_ROOTS` plus `$HOME/.local/share/applications` and
/// return every visible `DesktopEntry`. User-side wins on basename clash.
#[must_use]
pub fn scan() -> Vec<DesktopEntry> {
    let mut by_id: HashMap<String, DesktopEntry> = HashMap::new();

    let mut roots: Vec<PathBuf> = SEARCH_ROOTS.iter().map(PathBuf::from).collect();
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".local/share/applications"));
    }

    for root in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            if let Some(parsed) = parse_file(&path) {
                by_id.insert(parsed.id.clone(), parsed);
            }
        }
    }

    let mut all: Vec<DesktopEntry> = by_id.into_values().collect();
    all.sort_by_key(|a| a.name.to_lowercase());
    all
}

/// Parse one `.desktop` file. Returns `None` for files we should skip
/// (`NoDisplay`, `Hidden`, malformed, no `[Desktop Entry]` section, or
/// no `Name`/`Exec`).
pub fn parse_file(path: &Path) -> Option<DesktopEntry> {
    let id = path.file_name()?.to_string_lossy().to_string();
    let text = fs::read_to_string(path).ok()?;
    parse_text(&id, &text)
}

/// Parse the text of one `.desktop` file. Public so tests can exercise
/// the parser without touching the filesystem.
#[must_use]
pub fn parse_text(id: &str, text: &str) -> Option<DesktopEntry> {
    let mut in_section = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut categories: Vec<String> = Vec::new();
    let mut terminal = false;
    let mut no_display = false;
    let mut hidden = false;
    let mut startup_wm_class: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == "[Desktop Entry]";
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // Skip locale-suffixed keys (`Name[fr]=…`) — we use the bare
        // English form. A future phase can pick the right LANG.
        if key.contains('[') {
            continue;
        }
        let value = value.trim();
        match key.trim() {
            "Name" => name = Some(value.to_owned()),
            "Exec" => exec = Some(value.to_owned()),
            "Icon" => icon = Some(value.to_owned()),
            "Categories" => {
                categories = value
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
            "Terminal" => terminal = parse_bool(value),
            "NoDisplay" => no_display = parse_bool(value),
            "Hidden" => hidden = parse_bool(value),
            "StartupWMClass" => startup_wm_class = Some(value.to_owned()),
            _ => {}
        }
    }

    if no_display || hidden {
        return None;
    }
    let name = name?;
    let exec = exec?;

    Some(DesktopEntry {
        id: id.to_owned(),
        name,
        exec,
        icon,
        categories,
        terminal,
        startup_wm_class,
    })
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "true" | "1" | "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_entry() {
        let text = "[Desktop Entry]\nName=Firefox\nExec=firefox %U\n";
        let e = parse_text("firefox.desktop", text).expect("parses");
        assert_eq!(e.id, "firefox.desktop");
        assert_eq!(e.name, "Firefox");
        assert_eq!(e.exec, "firefox %U");
        assert!(e.icon.is_none());
        assert!(e.categories.is_empty());
        assert!(!e.terminal);
    }

    #[test]
    fn skips_no_display() {
        let text = "[Desktop Entry]\nName=Hidden\nExec=true\nNoDisplay=true\n";
        assert!(parse_text("hidden.desktop", text).is_none());
    }

    #[test]
    fn skips_hidden() {
        let text = "[Desktop Entry]\nName=Hidden\nExec=true\nHidden=true\n";
        assert!(parse_text("hidden.desktop", text).is_none());
    }

    #[test]
    fn parses_categories_and_icon_and_terminal() {
        let text = "\
            [Desktop Entry]\n\
            Name=htop\n\
            Exec=htop\n\
            Icon=utilities-system-monitor\n\
            Categories=System;Monitor;\n\
            Terminal=true\n";
        let e = parse_text("htop.desktop", text).expect("parses");
        assert_eq!(e.icon.as_deref(), Some("utilities-system-monitor"));
        assert_eq!(e.categories, vec!["System", "Monitor"]);
        assert!(e.terminal);
    }

    #[test]
    fn ignores_other_sections() {
        let text = "\
            [Desktop Entry]\n\
            Name=Real\n\
            Exec=real\n\
            \n\
            [Desktop Action New]\n\
            Name=Fake\n\
            Exec=fake\n";
        let e = parse_text("x.desktop", text).expect("parses");
        assert_eq!(e.name, "Real");
        assert_eq!(e.exec, "real");
    }

    #[test]
    fn locale_keys_ignored() {
        let text = "\
            [Desktop Entry]\n\
            Name=English\n\
            Name[fr]=Anglais\n\
            Exec=x\n";
        let e = parse_text("x.desktop", text).expect("parses");
        assert_eq!(e.name, "English");
    }

    #[test]
    fn missing_required_field_returns_none() {
        let text = "[Desktop Entry]\nName=NoExec\n";
        assert!(parse_text("x.desktop", text).is_none());
    }

    #[test]
    fn parse_bool_handles_common_forms() {
        assert!(parse_bool("true"));
        assert!(parse_bool("True"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool(""));
    }

    #[test]
    fn missing_section_returns_none() {
        // No `[Desktop Entry]` at all — even though Name/Exec are
        // present they live in no section so they're ignored.
        let text = "Name=Stray\nExec=stray\n";
        assert!(parse_text("stray.desktop", text).is_none());
    }

    #[test]
    fn comments_and_blank_lines_ignored() {
        let text = "\
            # the build comment\n\
            \n\
            [Desktop Entry]\n\
            # a leading comment\n\
            Name=Real\n\
            \n\
            Exec=real %F\n";
        let e = parse_text("real.desktop", text).expect("parses");
        assert_eq!(e.name, "Real");
        assert_eq!(e.exec, "real %F");
    }

    #[test]
    fn startup_wm_class_is_parsed() {
        let text = "\
            [Desktop Entry]\n\
            Name=Code\n\
            Exec=code\n\
            StartupWMClass=Code\n";
        let e = parse_text("code.desktop", text).expect("parses");
        assert_eq!(e.startup_wm_class.as_deref(), Some("Code"));
    }

    #[test]
    fn categories_skip_empty_segments_from_trailing_semicolon() {
        let text = "\
            [Desktop Entry]\n\
            Name=X\n\
            Exec=x\n\
            Categories=;Network;;System;;\n";
        let e = parse_text("x.desktop", text).expect("parses");
        assert_eq!(e.categories, vec!["Network", "System"]);
    }

    #[test]
    fn parse_file_returns_none_for_missing_path() {
        let p = std::path::Path::new("/does/not/exist/xyzzzz.desktop");
        assert!(parse_file(p).is_none());
    }

    #[test]
    fn parse_file_reads_real_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("real.desktop");
        std::fs::write(&path, "[Desktop Entry]\nName=Real\nExec=real\n").unwrap();
        let e = parse_file(&path).expect("parses");
        assert_eq!(e.id, "real.desktop");
        assert_eq!(e.name, "Real");
    }

    #[test]
    fn no_display_false_still_parses() {
        // Explicit false should NOT skip the entry.
        let text = "[Desktop Entry]\nName=Visible\nExec=x\nNoDisplay=false\n";
        assert!(parse_text("v.desktop", text).is_some());
    }

    #[test]
    fn unknown_keys_are_silently_dropped() {
        let text = "\
            [Desktop Entry]\n\
            Name=X\n\
            Exec=x\n\
            UnknownKey=ignored\n\
            Version=1.0\n";
        let e = parse_text("x.desktop", text).expect("parses");
        assert_eq!(e.name, "X");
    }

    #[test]
    fn lines_without_equals_are_ignored() {
        let text = "\
            [Desktop Entry]\n\
            Name=Y\n\
            Exec=y\n\
            no-equals-here\n";
        let e = parse_text("y.desktop", text).expect("parses");
        assert_eq!(e.name, "Y");
    }

    #[test]
    fn scan_returns_only_valid_desktop_entries() {
        let _g = crate::test_env::env_lock();
        // scan() reads $HOME/.local/share/applications. Point HOME at
        // a tempdir, drop one .desktop, and verify it surfaces.
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join(".local/share/applications");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::write(
            apps.join("only.desktop"),
            "[Desktop Entry]\nName=OnlyOne\nExec=only\n",
        )
        .unwrap();
        std::fs::write(apps.join("non-desktop.txt"), "ignored").unwrap();
        // Hidden entry must be skipped.
        std::fs::write(
            apps.join("hidden.desktop"),
            "[Desktop Entry]\nName=Hidden\nExec=h\nHidden=true\n",
        )
        .unwrap();

        let prior_home = std::env::var_os("HOME");
        std::env::set_var("HOME", dir.path());
        let entries = scan();
        match prior_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }

        // System dirs may add more entries; we only assert our own.
        assert!(entries.iter().any(|e| e.id == "only.desktop"));
        assert!(!entries.iter().any(|e| e.id == "hidden.desktop"));
    }
}
