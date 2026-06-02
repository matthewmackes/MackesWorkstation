//! Windows 10 tiled Start — headless tile management.
//!
//! This module currently provides the headless CLI for the Win10 Start tile area
//! (`state::start_tiles`), so tiles can be pinned/resized/listed without the GUI
//! and the behaviour is bench-testable (a dbus-free read/write round-trip). The
//! interactive three-region overlay is layered on in a later story.
//!
//!   mde start-win10 --list-tiles
//!   mde start-win10 --pin <name> <command>
//!   mde start-win10 --unpin <name>
//!   mde start-win10 --resize <name> <small|medium|wide|large>

use std::process::ExitCode;

use crate::state::{self, MenuState, StartTile, TileSize};

const USAGE: &str = "\
mde start-win10 — Windows 10 Start tiles
  --list-tiles                list the Start tiles (seeded from pinned items on a fresh config)
  --pin <name> <command>      pin a Medium tile (replacing one of the same name)
  --unpin <name>              remove the tile named <name>
  --resize <name> <size>      set tile size: small | medium | wide | large
";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("--list-tiles") => {
            for t in state::seed_start_tiles(&state::load()) {
                let (cols, rows) = t.size.span();
                println!(
                    "{}\t{}\t{}\t{}x{}\t{}",
                    t.name,
                    t.command,
                    t.size.token(),
                    cols,
                    rows,
                    t.group
                );
            }
            ExitCode::SUCCESS
        }
        Some("--pin") => match (args.get(1), args.get(2)) {
            (Some(name), Some(command)) => {
                let mut st = materialized();
                st.start_tiles.retain(|t| t.name != *name);
                st.start_tiles.push(StartTile {
                    name: name.clone(),
                    command: command.clone(),
                    icon: String::new(),
                    size: TileSize::Medium,
                    group: String::new(),
                });
                persist(&st)
            }
            _ => usage_err("--pin <name> <command>"),
        },
        Some("--unpin") => match args.get(1) {
            Some(name) => {
                let mut st = materialized();
                st.start_tiles.retain(|t| t.name != *name);
                persist(&st)
            }
            None => usage_err("--unpin <name>"),
        },
        Some("--resize") => match (args.get(1), args.get(2)) {
            (Some(name), Some(size)) => {
                let sz = TileSize::from_token(size);
                let mut st = materialized();
                let mut hit = false;
                for t in st.start_tiles.iter_mut().filter(|t| t.name == *name) {
                    t.size = sz;
                    hit = true;
                }
                if !hit {
                    eprintln!("mde start-win10: no tile named {name:?}");
                    return ExitCode::FAILURE;
                }
                persist(&st)
            }
            _ => usage_err("--resize <name> <small|medium|wide|large>"),
        },
        _ => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
    }
}

/// Load state with the seed materialized into `start_tiles`, so a mutation never
/// silently drops the first-run seed (the pins) on the floor.
fn materialized() -> MenuState {
    let mut st = state::load();
    st.start_tiles = state::seed_start_tiles(&st);
    st
}

fn persist(st: &MenuState) -> ExitCode {
    match state::save(st) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde start-win10: save failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage_err(form: &str) -> ExitCode {
    eprintln!("mde start-win10 {form}");
    ExitCode::FAILURE
}
