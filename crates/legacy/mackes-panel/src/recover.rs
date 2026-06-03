//! Phase 10.6.8 — read-only preview of the birthright rollback ledger.
//!
//! The Rust panel binary has no path to root: there's no AdminSession in
//! the Rust crate, and the panel runs in the user's GTK session. So
//! `mackes-panel --recover` is a *preview* — it reads the same JSON
//! records the Python `mackes recover` subcommand writes (see
//! `mackes/birthright_rollback.py`), prints which step would restore
//! what, surfaces the dnf install command the operator should copy-paste
//! (or run via `mackes recover all`), and exits 0 without launching any
//! GTK surfaces.
//!
//! This keeps the panel usable as a triage tool when it segfaults
//! repeatedly: an operator can `mackes-panel --recover` from any TTY
//! and see exactly which actions need to be reversed without touching
//! `mackes` (Python) or sudo state.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// One on-disk rollback record. Mirrors `RollbackStep` in
/// `mackes/birthright_rollback.py` — schema is owned by the Python side.
#[derive(Debug, Deserialize)]
pub struct RollbackStep {
    pub step_name: String,
    pub timestamp: String,
    #[serde(default)]
    pub restore_actions: Vec<RestoreAction>,
}

/// One restore action. Only the `shell` variant carries an argv we can
/// surface; `write_file` / `delete_file` / `xfconf_*` need filesystem
/// or session access we don't want to do from the panel binary.
#[derive(Debug, Deserialize)]
pub struct RestoreAction {
    pub kind: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default)]
    pub needs_root: bool,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub property: Option<String>,
}

/// Resolve `~/.config/mackes-panel/rollback/` honoring `XDG_CONFIG_HOME`.
fn rollback_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("mackes-panel").join("rollback");
        return Some(p);
    }
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("mackes-panel")
            .join("rollback"),
    )
}

/// Read every `*.json` in the rollback dir, parse what's valid, and
/// return the surviving records sorted newest-first by timestamp.
fn read_records() -> Vec<RollbackStep> {
    let Some(dir) = rollback_dir() else {
        return Vec::new();
    };
    if !dir.is_dir() {
        return Vec::new();
    }
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<RollbackStep> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match parse_one(&path) {
            Ok(step) => out.push(step),
            Err(e) => {
                eprintln!(
                    "mackes-panel --recover: skipping corrupt record {}: {e}",
                    path.display()
                );
            }
        }
    }
    out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    out
}

fn parse_one(path: &Path) -> Result<RollbackStep, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<RollbackStep>(&text).map_err(|e| e.to_string())
}

/// Public entry: print a preview report to stdout. Returns the process
/// exit code (always 0 — even an empty ledger is a successful preview).
#[must_use]
pub fn run_preview() -> i32 {
    let dir = rollback_dir();
    let records = read_records();

    println!("mackes-panel --recover (Phase 10.6.8 rollback preview)");
    println!("------------------------------------------------------");
    if let Some(d) = dir.as_ref() {
        println!("ledger dir : {}", d.display());
    } else {
        println!("ledger dir : (HOME unset — cannot resolve)");
    }
    if records.is_empty() {
        println!("records    : 0");
        println!();
        println!("no rollback records found — nothing to do");
        println!(
            "(if you expected records here, run the birthright wizard first; \
             panel-swap / panel-archive / uninstall-legacy-xfce each write \
             one record before they mutate the system)"
        );
        return 0;
    }
    println!("records    : {}", records.len());
    println!();
    for step in &records {
        println!(
            "▼ {} (recorded {})  — {} action(s)",
            step.step_name,
            step.timestamp,
            step.restore_actions.len()
        );
        for (i, act) in step.restore_actions.iter().rev().enumerate() {
            println!(
                "    [{:>2}] {:<13} {}",
                i + 1,
                act.kind,
                if act.description.is_empty() {
                    "(no description)"
                } else {
                    &act.description
                }
            );
            match act.kind.as_str() {
                "shell" => {
                    let cmd = act.argv.join(" ");
                    if act.needs_root {
                        println!("         sudo {cmd}");
                    } else {
                        println!("         {cmd}");
                    }
                }
                "write_file" | "delete_file" => {
                    if let Some(p) = act.path.as_deref() {
                        println!("         path: {p}");
                    }
                }
                "xfconf_set" | "xfconf_unset" => {
                    let ch = act.channel.as_deref().unwrap_or("?");
                    let pr = act.property.as_deref().unwrap_or("?");
                    println!("         channel={ch}  property={pr}");
                }
                _ => {}
            }
        }
        println!();
    }
    println!("The panel binary does not have root to apply these directly.");
    println!("Run the full restore from a TTY:");
    println!("    mackes recover all          # reverse every record");
    println!("    mackes recover one <step>   # reverse one step by name");
    println!("    mackes recover list         # show ledger as a table");
    0
}
