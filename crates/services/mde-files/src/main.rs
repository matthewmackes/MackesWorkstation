//! `mde-files` binary entry.
//!
//! Launches the Iced application that renders the Artifact Manager UI.

use mde_files::MdeFiles;

fn main() -> iced::Result {
    MdeFiles::run()
}
