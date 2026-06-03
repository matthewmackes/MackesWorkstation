//! `mde-applet-clock` binary entry — Phase E1.2.1.
//!
//! Run modes:
//!   * `mde-applet-clock --manifest` — prints the JSON
//!     manifest. Used by `make` + RPM `%install` to
//!     generate `/usr/share/mde/applets/clock.json`.
//!   * `mde-applet-clock --now` — prints the current
//!     formatted clock string. Useful for shell scripts +
//!     a sanity check.
//!   * (default) — reads JSON-line HostMessages from stdin,
//!     prints the rendered clock string to stdout on every
//!     tick. The panel host pipes a wl-text frame back.

use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use mde_applet_api::HostMessage;
use mde_applet_clock::{format_clock, handle_host, manifest};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.iter().any(|a| a == "--manifest") {
        match serde_json::to_string_pretty(&manifest()) {
            Ok(j) => {
                println!("{j}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("mde-applet-clock: serialize manifest: {e}");
                ExitCode::FAILURE
            }
        }
    } else if argv.iter().any(|a| a == "--now") {
        println!("{}", current_clock_string());
        ExitCode::SUCCESS
    } else {
        run_loop()
    }
}

/// Current local time formatted via `format_clock`.
/// Wrapped so the test surface can substitute a fixed
/// timestamp.
fn current_clock_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    format_clock(secs)
}

/// Read HostMessage JSON lines from stdin; emit rendered
/// clock strings on stdout. On `Shutdown` flush + exit 0.
/// Malformed input aborts the loop with exit code 2 so the
/// host can supervise.
fn run_loop() -> ExitCode {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let reader = BufReader::new(stdin.lock());
    // Emit the initial clock string on startup so the host
    // has something to render before the first host event.
    let _ = writeln!(stdout, "{}", current_clock_string());
    let _ = stdout.flush();
    for line in reader.lines() {
        let Ok(line) = line else {
            return ExitCode::from(2);
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<HostMessage>(&line) else {
            return ExitCode::from(2);
        };
        if matches!(msg, HostMessage::Shutdown) {
            return ExitCode::SUCCESS;
        }
        if handle_host(&msg) {
            let _ = writeln!(stdout, "{}", current_clock_string());
            let _ = stdout.flush();
        }
    }
    ExitCode::SUCCESS
}
