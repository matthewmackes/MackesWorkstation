//! First-run asset installer (`mde install --assets`).
//!
//! Per locked decision #7, the RPM ships CODE ONLY — the binary plus the asset
//! *installer scripts* (which are code). The visual assets themselves are
//! fetched from upstream at runtime so their licenses travel with the bytes and
//! nothing third-party is redistributed:
//!   * Chicago95 (icons/cursors/sounds/GTK theme) — github grassmunk/Chicago95
//!   * Win2k icon theme                            — KDE-Store item 1120706
//!
//! This is a *per-user* operation: the orchestrator deploys into the caller's
//! `~/.local/share`, and the Win2k step reads the cached tarball + generates
//! its aliases under `~/.config/labwc` — so the config tree must be deployed
//! first (the system installer does that, then triggers this per user).
//!
//! Usage:
//!   mde install [--assets] [--only chicago95|win2k] [--dry-run]

use std::path::PathBuf;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
mde install — fetch the MDE-Retro visual assets (per user)

USAGE:
    mde install [--assets] [--only chicago95|win2k] [--dry-run]

Fetches Chicago95 + the Win2k icon theme from upstream into ~/.local/share
(nothing is redistributed by the RPM). Run after the config tree is deployed.";

pub fn run(args: &[String]) -> ExitCode {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let dry = args.iter().any(|a| a == "--dry-run");
    let only = args
        .iter()
        .position(|a| a == "--only")
        .and_then(|i| args.get(i + 1))
        .cloned();
    if let Some(o) = &only {
        if o != "chicago95" && o != "win2k" {
            eprintln!("mde install: --only takes 'chicago95' or 'win2k', got '{o}'");
            return ExitCode::from(2);
        }
    }

    let do_chicago = only.as_deref() != Some("win2k");
    let do_win2k = only.as_deref() != Some("chicago95");

    if dry {
        println!("mde install --assets (dry run)");
        match &only {
            Some(o) => println!("  scope        : --only {o}"),
            None => println!("  scope        : Chicago95 + Win2k icon theme"),
        }
        if do_chicago {
            match locate_orchestrator() {
                Some(s) => println!("  chicago95    : {} --only chicago95", s.display()),
                None => println!("  chicago95    : orchestrator not in tree yet (RETIRE-PY.6b)"),
            }
        }
        if do_win2k {
            println!("  win2k        : native Rust (no python) → ~/.local/share/icons/Win2k");
        }
        println!("  deploys into : ~/.local/share/{{icons,themes,sounds}} (this user)");
        println!("  source       : fetched from upstream at runtime (not redistributed)");
        return ExitCode::SUCCESS;
    }

    // Order mirrors the v1.x orchestrator: Chicago95 first (broad coverage +
    // cursors + sounds + GTK theme), then the Win2k icon theme (now a native
    // Rust step — RETIRE-PY.6a — so no `python3` is ever spawned). A missing
    // Chicago95 orchestrator is fatal only when it was the explicit target;
    // for the default `--assets` run it degrades to a warning so the native
    // Win2k step still lands (the orchestrator move is RETIRE-PY.6b).
    if do_chicago {
        if let Err(code) = run_chicago95(only.as_deref() == Some("chicago95")) {
            return code;
        }
    }
    if do_win2k {
        // Native installer is the final step → return its code directly.
        return crate::install_win2k::run();
    }
    ExitCode::SUCCESS
}

/// Run the Chicago95 step via the bash orchestrator (`--only chicago95`).
/// `required` is true when Chicago95 was the explicit `--only` target; when
/// false (the default both-assets run) a missing orchestrator is a warning,
/// not an error, so the native Win2k step still runs.
fn run_chicago95(required: bool) -> Result<(), ExitCode> {
    let Some(script) = locate_orchestrator() else {
        if required {
            eprintln!(
                "mde install: Chicago95 orchestrator not found.\n\
                 Looked in /usr/share/mde/scripts and the dev tree. On an installed\n\
                 system this ships with the `mde` RPM; in a checkout it is not yet\n\
                 vendored (tracked as RETIRE-PY.6b)."
            );
            return Err(ExitCode::FAILURE);
        }
        eprintln!(
            "mde install: skipping Chicago95 — orchestrator not vendored yet \
             (RETIRE-PY.6b); continuing with the native Win2k step."
        );
        return Ok(());
    };
    let status = Command::new("bash")
        .arg(&script)
        .arg("--only")
        .arg("chicago95")
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => {
            eprintln!("mde install: Chicago95 installer exited with {s}");
            Err(ExitCode::from(s.code().unwrap_or(1).clamp(1, 255) as u8))
        }
        Err(e) => {
            eprintln!("mde install: failed to run {}: {e}", script.display());
            Err(ExitCode::FAILURE)
        }
    }
}

/// Find `install-assets.sh`: the RPM ships it under `/usr/share/mde/scripts`;
/// in a dev checkout it lives at `<repo>/assets/`, next to the `rust/` tree.
fn locate_orchestrator() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/usr/share/mde/scripts/install-assets.sh"),
        PathBuf::from("/usr/share/mde/assets/install-assets.sh"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        // exe = <repo>/rust/target/<profile>/mde -> ancestors().nth(3) = <repo>/rust
        if let Some(rust_dir) = exe.ancestors().nth(3) {
            candidates.push(rust_dir.join("../assets/install-assets.sh"));
        }
    }
    candidates.into_iter().find(|p| p.exists())
}
