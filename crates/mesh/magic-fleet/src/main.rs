//! `magic-fleet` — the Magic Mesh Automation Mesh node CLI (E11.7).
//!
//!   magic-fleet apply <playbook.yml>   converge this node to the desired state
//!
//! Applies a desired-state Ansible playbook to the local node via ansible-runner
//! and prints the convergence report; exit 0 only when the node converged.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (Some(verb @ ("apply" | "heal")), Some(path)) =
        (args.get(1).map(String::as_str), args.get(2))
    else {
        eprintln!("usage: magic-fleet <apply|heal> <playbook.yml>");
        return ExitCode::FAILURE;
    };
    let yaml = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("magic-fleet: cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let root = std::env::temp_dir().join(format!("magic-fleet-{}", std::process::id()));
    match verb {
        "heal" => match magic_fleet::heal_to_baseline(&yaml, &root) {
            Ok((status, r)) => {
                println!(
                    "magic-fleet: drift={status:?} ok={} changed={} failures={} unreachable={}",
                    r.ok, r.changed, r.failures, r.unreachable
                );
                if status == magic_fleet::DriftStatus::Failed {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("magic-fleet: heal failed: {e}");
                ExitCode::FAILURE
            }
        },
        // "apply" (the only other verb the let-else admits).
        _ => match magic_fleet::apply(&yaml, &root) {
            Ok(r) => {
                println!(
                    "magic-fleet: ok={} changed={} failures={} unreachable={} -> {}",
                    r.ok,
                    r.changed,
                    r.failures,
                    r.unreachable,
                    if r.converged() {
                        "CONVERGED"
                    } else {
                        "NOT CONVERGED"
                    }
                );
                if r.converged() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                eprintln!("magic-fleet: apply failed: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
