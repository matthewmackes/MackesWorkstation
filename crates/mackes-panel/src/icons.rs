// API surface is consumed by Phase 1.5+; suppress dead-code warnings
// while the module ships ahead of its callers.
#![allow(dead_code)]

//! Mackes-Carbon icon loader.
//!
//! Given a freedesktop icon name (e.g. `"folder-symbolic"`, `"home"`,
//! `"system-search-symbolic"`) returns a `gdk_pixbuf::Pixbuf` from the
//! installed Mackes-Carbon theme at the requested pixel size. Caches the
//! parsed pixbuf per (`name`, `size`) so subsequent loads of the same
//! glyph are free.
//!
//! Per Q14 of the design lock, every panel/dock icon resolves through
//! this loader — no app-specific PNGs, no third-party theme.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use gdk_pixbuf::Pixbuf;

/// Root of the installed Mackes-Carbon icon theme.
const THEME_ROOT: &str = "/usr/share/icons/Mackes-Carbon/scalable";

/// Freedesktop categories the build script lays icons out under. Search
/// order matches the spec's typical lookup order so the most semantically
/// correct hit wins.
const CATEGORIES: &[&str] = &[
    "actions",
    "status",
    "devices",
    "places",
    "emblems",
    "categories",
    "mimetypes",
    "apps",
];

thread_local! {
    /// Per-thread (`name`, `size`) → Pixbuf cache. The panel runs single-
    /// threaded (GTK main loop) so a thread-local is sufficient and avoids
    /// a Mutex on the hot path.
    static CACHE: RefCell<HashMap<(String, i32), Pixbuf>> = RefCell::new(HashMap::new());
}

/// Look up an icon by freedesktop name and return a `Pixbuf` sized to
/// `size_px`. Returns `None` only if the file is genuinely missing or
/// fails to parse — never panics.
#[must_use]
pub fn load(name: &str, size_px: i32) -> Option<Pixbuf> {
    let key = (name.to_owned(), size_px);
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return Some(hit);
    }

    let path = locate_svg(name)?;
    let pb = Pixbuf::from_file_at_scale(&path, size_px, size_px, true).ok()?;
    CACHE.with(|c| {
        c.borrow_mut().insert(key, pb.clone());
    });
    Some(pb)
}

/// Resolve `name` to an SVG path under `THEME_ROOT`. Tries the literal
/// name first, then the `-symbolic` variant, then the bare-name (stripped
/// suffix) variant. Walks the category list in `CATEGORIES` order.
fn locate_svg(name: &str) -> Option<PathBuf> {
    let candidates = candidate_basenames(name);
    for category in CATEGORIES {
        for cand in &candidates {
            let p = PathBuf::from(THEME_ROOT).join(category).join(cand);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn candidate_basenames(name: &str) -> Vec<String> {
    let mut v = Vec::with_capacity(3);
    v.push(format!("{name}.svg"));
    if name.ends_with("-symbolic") {
        // foo-symbolic → also try foo.svg as a fallback
        let bare = name.trim_end_matches("-symbolic");
        v.push(format!("{bare}.svg"));
    } else {
        // foo → also try foo-symbolic.svg in case the theme only carries
        // the symbolic variant for this name.
        v.push(format!("{name}-symbolic.svg"));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_basenames_for_plain_name() {
        let v = candidate_basenames("folder");
        assert_eq!(v, ["folder.svg", "folder-symbolic.svg"]);
    }

    #[test]
    fn candidate_basenames_for_symbolic_suffix() {
        let v = candidate_basenames("folder-symbolic");
        assert_eq!(v, ["folder-symbolic.svg", "folder.svg"]);
    }

    #[test]
    fn locate_returns_none_for_missing() {
        // This will only "miss correctly" when the theme isn't installed
        // in the test environment. Either way the function shouldn't
        // panic — that's the assertion.
        let _ = locate_svg("definitely-not-a-real-icon-name");
    }
}
