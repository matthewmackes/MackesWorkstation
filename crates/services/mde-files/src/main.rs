//! `mde-files` binary entry.
//!
//! Launches the Iced application that renders the Artifact Manager UI, or — when
//! invoked with `--pick` — the Open/Save file chooser (E10.3), the single file
//! engine the shell's `filedialog` subcommand now drives.
//!
//! Native file-ops parity (E11.6) is also reachable headlessly for scripting +
//! the shell's delete path: `--trash <path>…`, `--list-trash`, `--restore
//! <trash-name>…`, `--empty-trash` drive the freedesktop home trash directly,
//! and `--properties <path>…` prints native file metadata.

use std::process::ExitCode;

use mde_files::trash::TrashDir;
use mde_files::MdeFiles;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Native trash ops short-circuit the GUI (the --pick pattern).
    match args.first().map(String::as_str) {
        Some("--trash") => return trash_paths(&args[1..]),
        Some("--list-trash") => return list_trash(),
        Some("--restore") => return restore_names(&args[1..]),
        Some("--empty-trash") => return empty_trash(),
        Some("--properties") => return properties(&args[1..]),
        _ => {}
    }
    if args.iter().any(|a| a == "--pick") {
        // The chooser prints the chosen path to stdout + exits 0 (non-zero on
        // Cancel) — the contract `mde filedialog` execs against.
        return match mde_files::picker::run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        };
    }
    match MdeFiles::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde-files: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Open the home trash, reporting a failure as a non-zero exit.
fn home_trash() -> Result<TrashDir, ExitCode> {
    TrashDir::home().map_err(|e| {
        eprintln!("mde-files: cannot open trash: {e}");
        ExitCode::FAILURE
    })
}

/// `--trash <path>…` — move each path to the trash.
fn trash_paths(paths: &[String]) -> ExitCode {
    if paths.is_empty() {
        eprintln!("usage: mde-files --trash <path> [path ...]");
        return ExitCode::FAILURE;
    }
    let trash = match home_trash() {
        Ok(t) => t,
        Err(code) => return code,
    };
    let mut failed = false;
    for p in paths {
        match trash.trash(std::path::Path::new(p)) {
            Ok(item) => println!("trashed {p} -> {}", item.trash_name),
            Err(e) => {
                eprintln!("mde-files: cannot trash {p}: {e}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `--list-trash` — print the recoverable items, one per line.
fn list_trash() -> ExitCode {
    let trash = match home_trash() {
        Ok(t) => t,
        Err(code) => return code,
    };
    match trash.list() {
        Ok(items) => {
            for item in items {
                println!(
                    "{}\t{}\t{}",
                    item.trash_name,
                    item.deletion_date,
                    item.original_path.display()
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mde-files: cannot read trash: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `--restore <trash-name>…` — restore each named item to its original path.
fn restore_names(names: &[String]) -> ExitCode {
    if names.is_empty() {
        eprintln!("usage: mde-files --restore <trash-name> [trash-name ...]");
        return ExitCode::FAILURE;
    }
    let trash = match home_trash() {
        Ok(t) => t,
        Err(code) => return code,
    };
    let items = match trash.list() {
        Ok(items) => items,
        Err(e) => {
            eprintln!("mde-files: cannot read trash: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut failed = false;
    for name in names {
        match items.iter().find(|i| &i.trash_name == name) {
            Some(item) => match trash.restore(item) {
                Ok(()) => println!("restored {name} -> {}", item.original_path.display()),
                Err(e) => {
                    eprintln!("mde-files: cannot restore {name}: {e}");
                    failed = true;
                }
            },
            None => {
                eprintln!("mde-files: no trashed item named {name}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `--properties <path>…` — print each path's native file properties.
fn properties(paths: &[String]) -> ExitCode {
    if paths.is_empty() {
        eprintln!("usage: mde-files --properties <path> [path ...]");
        return ExitCode::FAILURE;
    }
    let mut failed = false;
    for (i, p) in paths.iter().enumerate() {
        if i > 0 {
            println!();
        }
        match mde_files::properties::FileProperties::of(std::path::Path::new(p)) {
            Ok(props) => print!("{}", mde_files::properties::report(&props)),
            Err(e) => {
                eprintln!("mde-files: cannot stat {p}: {e}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `--empty-trash` — permanently delete everything in the trash.
fn empty_trash() -> ExitCode {
    let trash = match home_trash() {
        Ok(t) => t,
        Err(code) => return code,
    };
    match trash.empty() {
        Ok(n) => {
            println!("emptied trash ({n} item(s))");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mde-files: cannot empty trash: {e}");
            ExitCode::FAILURE
        }
    }
}
