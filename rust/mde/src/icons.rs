//! Icon resolution + rendering for the shell.
//!
//! Windows 2000's shell is icon-dense (drive/folder icons in Explorer, applet
//! icons in Control Panel, per-item Start-menu icons). We resolve names against
//! the installed freedesktop icon themes — the same Win2k → Chicago95 → hicolor
//! chain the rest of MDE-Retro uses (see the win95-desktop layer) — and render
//! the PNG/SVG with iced (the `image`/`svg` features are already enabled).
//!
//! Two theme layouts exist in the wild and both are indexed:
//!   Win2k:     <theme>/<S>x<S>/<category>/<name>.png
//!   Chicago95: <theme>/<category>/<S>/<name>.png
//! A name can therefore live at several sizes; [`lookup`] picks the closest to
//! the requested size (preferring ≥ requested). The index is built once, lazily.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use iced::widget::{image, svg, Space};
use iced::{Element, Length};

/// Themes searched, in priority order, chosen by the persisted icon set
/// (Display ▸ Appearance). "haiku" ⇒ Haiku then the generic hicolor fallback;
/// anything else ⇒ the Windows 2000 classic chain.
fn themes() -> &'static [&'static str] {
    match crate::state::load().icon_set.as_str() {
        "haiku" => &["Haiku", "hicolor"],
        _ => &["Win2k", "Chicago95", "hicolor"],
    }
}

/// Icon-theme base directories (XDG data dirs + the per-user ~/.icons).
fn base_dirs() -> Vec<PathBuf> {
    let mut bases = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        bases.push(data.join("icons"));
        bases.push(home.join(".icons"));
    }
    bases.push(PathBuf::from("/usr/share/icons"));
    bases.push(PathBuf::from("/usr/local/share/icons"));
    bases
}

/// Extract a pixel size from a path component like "32x32" or a bare "32".
fn size_in(component: &str) -> Option<u16> {
    if let Some((a, b)) = component.split_once('x') {
        if a == b {
            return a.parse().ok();
        }
    }
    component.parse().ok()
}

/// The best size token found anywhere along a path (0 = scalable / unknown).
fn path_size(p: &Path) -> u16 {
    if p.components().any(|c| c.as_os_str() == "scalable") {
        return 0;
    }
    p.components()
        .filter_map(|c| c.as_os_str().to_str().and_then(size_in))
        .max()
        .unwrap_or(0)
}

/// Recursively collect icon files under `dir` (depth-capped) into `out`,
/// keyed by file stem → list of (theme rank, path).
fn walk(dir: &Path, rank: u8, depth: u8, out: &mut HashMap<String, Vec<(u8, PathBuf)>>) {
    if depth == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&path, rank, depth - 1, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "png" || ext == "svg" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    out.entry(stem.to_string()).or_default().push((rank, path.clone()));
                }
            }
        }
    }
}

/// The lazily-built name → (theme rank, path) index. Rank is the theme's
/// position in [`themes`], so a higher-priority theme always wins regardless of
/// size (needed for the scalable-only Haiku set to beat hicolor rasters).
fn index() -> &'static HashMap<String, Vec<(u8, PathBuf)>> {
    static INDEX: OnceLock<HashMap<String, Vec<(u8, PathBuf)>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map: HashMap<String, Vec<(u8, PathBuf)>> = HashMap::new();
        for base in base_dirs() {
            for (rank, theme) in themes().iter().enumerate() {
                let dir = base.join(theme);
                if dir.is_dir() {
                    walk(&dir, rank as u8, 5, &mut map);
                }
            }
        }
        map
    })
}

/// Resolve an icon `name` at the desired pixel `size`, or `None` if absent.
/// Theme priority dominates; within a theme, prefers an exact size, then the
/// nearest size ≥ requested, then the largest smaller one, then scalable.
pub fn lookup(name: &str, size: u16) -> Option<PathBuf> {
    let candidates = index().get(name)?;
    candidates
        .iter()
        .min_by_key(|(rank, p)| {
            let s = path_size(p);
            let size_score = match s {
                _ if s == size => 0u32,
                0 => 100_000,
                _ if s > size => 1_000 + (s - size) as u32,
                _ => 10_000 + (size - s) as u32,
            };
            (*rank, size_score)
        })
        .map(|(_, p)| p.clone())
}

/// An iced element rendering the first of `names` that resolves at `size`,
/// boxed to a `size`×`size` square. Falls back to empty space (never tofu /
/// never a broken-image marker) when nothing matches.
pub fn icon_any<'a, Message: 'a>(names: &[&str], size: u16) -> Element<'a, Message> {
    let len = Length::Fixed(size as f32);
    for name in names {
        if let Some(path) = lookup(name, size) {
            let is_svg = path.extension().and_then(|e| e.to_str()) == Some("svg");
            return if is_svg {
                svg(svg::Handle::from_path(path)).width(len).height(len).into()
            } else {
                image(image::Handle::from_path(path)).width(len).height(len).into()
            };
        }
    }
    Space::new(len, len).into()
}

/// Convenience: a single-name [`icon_any`].
pub fn icon<'a, Message: 'a>(name: &str, size: u16) -> Element<'a, Message> {
    icon_any(&[name], size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_parsing() {
        assert_eq!(size_in("32x32"), Some(32));
        assert_eq!(size_in("16"), Some(16));
        assert_eq!(size_in("actions"), None);
        assert_eq!(size_in("48x16"), None); // non-square dir is not a size
    }

    #[test]
    fn path_size_prefers_largest_token() {
        assert_eq!(path_size(Path::new("/x/Win2k/32x32/apps/foo.png")), 32);
        assert_eq!(path_size(Path::new("/x/Chicago95/devices/16/bar.png")), 16);
        assert_eq!(path_size(Path::new("/x/hicolor/scalable/apps/baz.svg")), 0);
    }

    #[test]
    fn missing_icon_is_none() {
        assert!(lookup("definitely-not-an-icon-name-xyzzy", 32).is_none());
    }
}
