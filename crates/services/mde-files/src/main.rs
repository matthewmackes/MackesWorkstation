//! `mde-files` binary entry.
//!
//! Launches the Iced application that renders the Artifact Manager UI, or — when
//! invoked with `--pick` — the Open/Save file chooser (E10.3), the single file
//! engine the shell's `filedialog` subcommand now drives.

use mde_files::MdeFiles;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--pick") {
        // The chooser prints the chosen path to stdout + exits 0 (non-zero on
        // Cancel) — the contract `mde filedialog` execs against.
        return mde_files::picker::run(&args);
    }
    MdeFiles::run()
}
