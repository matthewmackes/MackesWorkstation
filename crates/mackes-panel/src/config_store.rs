// Config-store API used by main + Phase 2.3 (inotify reload). Some entry
// points haven't been wired in yet.
#![allow(dead_code)]

//! On-disk persistence for `~/.config/mackes-panel/panel.toml`.
//!
//! Per Q18 the panel config lives in TOML under `XDG_CONFIG_HOME`. This
//! module is the single read/write boundary for that file:
//!
//! * `path()` resolves the canonical location (`XDG_CONFIG_HOME` with a
//!   `$HOME/.config/mackes-panel/panel.toml` fallback).
//! * `load_or_default()` parses the file if present, else writes the
//!   `default_config()` and returns it. First-launch behavior per
//!   Phase 2.2 of `docs/PROJECT_WORKLIST.md`.
//! * Phase 2.3 will extend with `watch()` (inotify diff-and-apply).

use std::path::{Path, PathBuf};

use mackes_config::{default_config, parse, to_toml_string, PanelConfig};

const REL_PATH: &str = "mackes-panel/panel.toml";

/// Canonical config file path. Reads `XDG_CONFIG_HOME` first; falls back
/// to `$HOME/.config`. Returns `None` only when neither variable nor
/// `$HOME` is set (extremely unusual — bare /bin/sh sessions).
#[must_use]
pub fn path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join(REL_PATH));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join(REL_PATH))
}

/// Load the panel config from `path()` if it exists; otherwise write the
/// default and return it. Bad TOML is logged to stderr and falls back to
/// defaults so the panel always starts in a usable state.
#[must_use]
pub fn load_or_default() -> PanelConfig {
    let Some(p) = path() else {
        return default_config();
    };
    if p.is_file() {
        match std::fs::read_to_string(&p) {
            Ok(text) => match parse(&text) {
                Ok(cfg) => return cfg,
                Err(e) => {
                    eprintln!("mackes-panel: ignoring malformed {}: {e}", p.display());
                }
            },
            Err(e) => {
                eprintln!("mackes-panel: cannot read {}: {e}", p.display());
            }
        }
    } else {
        write_default(&p);
    }
    default_config()
}

fn write_default(path: &Path) {
    let cfg = default_config();
    let text = match to_toml_string(&cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mackes-panel: cannot serialize default config: {e}");
            return;
        }
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("mackes-panel: cannot create {}: {e}", parent.display());
            return;
        }
    }
    if let Err(e) = std::fs::write(path, text) {
        eprintln!(
            "mackes-panel: cannot write default to {}: {e}",
            path.display()
        );
    } else {
        eprintln!("mackes-panel: wrote default config to {}", path.display());
    }
}
