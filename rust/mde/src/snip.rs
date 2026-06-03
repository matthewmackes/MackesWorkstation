//! Snip & Sketch-style screenshot tool (E16.4), the Win+Shift+S surface.
//!
//! `mde snip` (or `mde snip rect`) selects a region with `slurp`, captures it with
//! `grim -g`, saves a PNG under `~/Pictures/Screenshots/`, and copies it to the
//! clipboard as `image/png` (so the clipboard daemon, E16.2, also records it).
//! `mde snip full` skips the region picker and grabs the whole screen — the same
//! save+copy path, and the headless-testable one. Windows 10-era only.

use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// `~/Pictures/Screenshots/`.
fn shots_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Pictures").join("Screenshots"))
}

/// The screenshot filename for an epoch-seconds stamp (pure, unit-tested).
fn shot_name(epoch: u64) -> String {
    format!("Screenshot_{epoch}.png")
}

fn epoch_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Pick a region with `slurp`; `None` if it was cancelled (empty / non-zero exit).
fn region() -> Option<String> {
    let o = Command::new("slurp").output().ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

pub fn run(args: &[String]) -> ExitCode {
    if !mde_ui::palette::is_windows10() {
        eprintln!("mde snip: the snipping tool is a Windows 10-era surface.");
        return ExitCode::SUCCESS;
    }
    // `full`/`screen` grabs the whole output; the default (or `rect`) picks a region.
    let full = args.iter().any(|a| a == "full" || a == "screen");
    let geom = if full {
        None
    } else {
        match region() {
            Some(g) => Some(g),
            None => return ExitCode::SUCCESS, // user cancelled the selection
        }
    };

    let Some(dir) = shots_dir() else {
        eprintln!("mde snip: no HOME");
        return ExitCode::FAILURE;
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("mde snip: {e}");
        return ExitCode::FAILURE;
    }
    let path = dir.join(shot_name(epoch_now()));

    let mut grim = Command::new("grim");
    if let Some(g) = &geom {
        grim.args(["-g", g]);
    }
    grim.arg(&path);
    let ok = grim.status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        eprintln!("mde snip: grim failed");
        return ExitCode::FAILURE;
    }

    // Also copy to the clipboard as image/png (so it pastes + the daemon records it).
    if let Ok(f) = std::fs::File::open(&path) {
        let _ = Command::new("wl-copy")
            .args(["--type", "image/png"])
            .stdin(f)
            .status();
    }
    let _ = Command::new("notify-send")
        .args(["Screenshot saved", &path.display().to_string()])
        .spawn();
    println!("{}", path.display());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_filename() {
        assert_eq!(shot_name(1_700_000_000), "Screenshot_1700000000.png");
        assert!(shot_name(0).ends_with(".png"));
    }
}
