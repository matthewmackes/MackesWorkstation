//! `magic-fleet` — the Magic Mesh Automation Mesh node CLI (E11.7).
//!
//!   magic-fleet apply    <playbook.yml>   apply a desired-state playbook locally
//!   magic-fleet heal     <playbook.yml>   re-apply + report drift (InSync/Healed/Failed)
//!   magic-fleet converge <baseline.yml>   render a desired-state baseline → apply + report drift
//!
//! Exit 0 only when the node converged / the heal did not fail.

use std::io;
use std::process::ExitCode;

use magic_fleet::{ApplyReport, DriftStatus};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (Some(verb @ ("apply" | "heal" | "converge")), Some(path)) =
        (args.get(1).map(String::as_str), args.get(2))
    else {
        eprintln!("usage: magic-fleet <apply|heal> <playbook.yml> | converge <baseline.yml>");
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
        "apply" => match magic_fleet::apply(&yaml, &root) {
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
                exit_for(r.converged())
            }
            Err(e) => fail("apply", &e),
        },
        "heal" => drift_exit(verb, magic_fleet::heal_to_baseline(&yaml, &root)),
        // "converge" — the only remaining verb the let-else admits.
        _ => match magic_fleet::BaselineSpec::from_yaml(&yaml) {
            Ok(spec) => drift_exit(verb, magic_fleet::converge(&spec, &root)),
            Err(e) => {
                eprintln!("magic-fleet: invalid baseline: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Print a drift outcome and map it to an exit code (failure only on a failed heal).
fn drift_exit(verb: &str, res: io::Result<(DriftStatus, ApplyReport)>) -> ExitCode {
    match res {
        Ok((status, r)) => {
            println!(
                "magic-fleet: drift={status:?} ok={} changed={} failures={} unreachable={}",
                r.ok, r.changed, r.failures, r.unreachable
            );
            exit_for(status != DriftStatus::Failed)
        }
        Err(e) => fail(verb, &e),
    }
}

fn fail(verb: &str, e: &io::Error) -> ExitCode {
    eprintln!("magic-fleet: {verb} failed: {e}");
    ExitCode::FAILURE
}

const fn exit_for(ok: bool) -> ExitCode {
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
