//! Control Panel — Win2000-named mapping of Fedora system tools.
//!
//! The GUI (iced, matching the reference screenshot's blue info-pane + white
//! icon grid) is built in a later pass; this provides the working backend now:
//! list tools with install status, launch one, and install any that are
//! missing via a single `pkexec dnf` prompt.
//!
//!   mde control-panel              list tools + [installed]/[MISSING]
//!   mde control-panel --launch N   launch tool number N
//!   mde control-panel --install-missing   pkexec dnf install the missing ones

use std::process::ExitCode;

use crate::fedora;

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("--launch") => launch(args.get(1)),
        Some("--install-missing") => install_missing(),
        _ => {
            list();
            ExitCode::SUCCESS
        }
    }
}

fn list() {
    println!("Control Panel — Fedora system tools\n");
    for (i, tool) in fedora::TOOLS.iter().enumerate() {
        let status = if fedora::is_installed(tool.command) {
            "installed"
        } else {
            "MISSING  "
        };
        println!(
            "  {:>2}. [{}]  {:<32}  ({})",
            i + 1,
            status,
            tool.name,
            fedora::binary(tool.command)
        );
    }
    let missing = fedora::missing_packages();
    if missing.is_empty() {
        println!("\nAll backing tools are installed.");
    } else {
        println!(
            "\n{} missing. Install with:  mde control-panel --install-missing",
            missing.len()
        );
        println!("Packages: {}", missing.join(" "));
    }
}

fn launch(arg: Option<&String>) -> ExitCode {
    let n = arg.and_then(|s| s.parse::<usize>().ok());
    match n.and_then(|n| fedora::TOOLS.get(n.saturating_sub(1))) {
        Some(tool) => match fedora::launch(tool) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("mde control-panel: launch failed: {e}");
                ExitCode::FAILURE
            }
        },
        None => {
            eprintln!("mde control-panel: --launch needs a valid tool number (see the list)");
            ExitCode::from(2)
        }
    }
}

fn install_missing() -> ExitCode {
    let packages = fedora::missing_packages();
    if packages.is_empty() {
        println!("Nothing to install — all tools present.");
        return ExitCode::SUCCESS;
    }
    println!("Installing missing tools: {}", packages.join(" "));
    match fedora::install(&packages) {
        Ok(status) if status.success() => {
            println!("Done.");
            ExitCode::SUCCESS
        }
        Ok(status) => {
            eprintln!("dnf exited with {status}");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("could not run pkexec dnf: {e}");
            ExitCode::FAILURE
        }
    }
}
