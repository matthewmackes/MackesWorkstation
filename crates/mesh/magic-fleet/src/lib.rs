//! E11.7 — Magic Mesh "Automation Mesh" node engine.
//!
//! The fleet-sync model (design `mesh-decoupling.md` §2a, Q121–Q124): each node
//! converges its own OS desired-state locally by running an Ansible playbook via
//! `ansible-runner` (Podman/local), rather than a central controller SSH-ing in.
//! This crate is that **local-apply primitive**: lay out an ansible-runner
//! private-data-dir for a desired-state playbook, run it against `localhost`, and
//! parse the convergence result. The peer-to-peer revision routing over Nebula,
//! the YAML-DSL → playbook render, drift auto-heal, and the Workbench authoring
//! UI all build on top of `apply()`.
//!
//! Toolchain: `ansible-runner` (orchestrator) + `ansible-core` (`ansible-playbook`).

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// Convergence summary parsed from an ansible-runner `playbook_on_stats` event.
///
/// The PLAY RECAP totals, summed across hosts (a node applies to `localhost`, so
/// it's a single host today, but the sum generalises to multi-host inventories).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyReport {
    /// Tasks that completed already in the desired state (ansible `ok`).
    pub ok: u32,
    /// Tasks that changed a resource this run (ansible `changed`).
    pub changed: u32,
    /// Tasks that failed (ansible `failures`).
    pub failures: u32,
    /// Ansible's "dark" hosts — unreachable.
    pub unreachable: u32,
}

impl ApplyReport {
    /// The node converged to its desired state: reachable and no task failed.
    #[must_use]
    pub const fn converged(&self) -> bool {
        self.failures == 0 && self.unreachable == 0
    }

    /// At least one resource was brought to its desired state this run (false on a
    /// no-op re-apply — the idempotence signal the drift loop keys off).
    #[must_use]
    pub const fn made_changes(&self) -> bool {
        self.changed > 0
    }
}

/// Sum the per-host count dicts in an ansible-runner `playbook_on_stats` event
/// JSON into an [`ApplyReport`]. Returns `None` when `event_json` is not a
/// stats event. `dark` is Ansible's unreachable bucket.
#[must_use]
pub fn parse_stats(event_json: &str) -> Option<ApplyReport> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    if v.get("event")?.as_str()? != "playbook_on_stats" {
        return None;
    }
    let ed = v.get("event_data")?;
    let sum = |key: &str| -> u32 {
        ed.get(key)
            .and_then(serde_json::Value::as_object)
            .map_or(0, |m| {
                m.values()
                    .filter_map(serde_json::Value::as_u64)
                    .map(|n| u32::try_from(n).unwrap_or(u32::MAX))
                    .sum()
            })
    };
    Some(ApplyReport {
        ok: sum("ok"),
        changed: sum("changed"),
        failures: sum("failures"),
        unreachable: sum("dark"),
    })
}

/// Lay out an ansible-runner private-data-dir under `root`: `project/site.yml`
/// (the desired-state playbook) + `inventory/hosts` (this node as `localhost`,
/// local connection — no SSH).
///
/// # Errors
/// Propagates filesystem errors creating the `project`/`inventory` dirs or
/// writing the playbook/inventory files.
pub fn write_private_data_dir(root: &Path, playbook_yaml: &str) -> io::Result<()> {
    std::fs::create_dir_all(root.join("project"))?;
    std::fs::create_dir_all(root.join("inventory"))?;
    std::fs::write(root.join("project").join("site.yml"), playbook_yaml)?;
    std::fs::write(
        root.join("inventory").join("hosts"),
        "[local]\nlocalhost ansible_connection=local\n",
    )?;
    Ok(())
}

/// The `ansible-runner` argv that applies the laid-out `site.yml` against the
/// local inventory, quietly.
#[must_use]
pub fn runner_argv(root: &Path) -> Vec<String> {
    vec![
        "run".to_string(),
        root.display().to_string(),
        "-p".to_string(),
        "site.yml".to_string(),
        "--quiet".to_string(),
    ]
}

/// Apply `playbook_yaml` (a desired-state Ansible playbook) to the local node.
///
/// Lays out a private-data-dir under `root`, runs `ansible-runner`, and parses
/// the newest `playbook_on_stats` event into the convergence report.
///
/// # Errors
/// When ansible-runner can't be spawned, or it exits non-zero AND produced no
/// parseable stats (e.g. `ansible-playbook` missing → the run never ran).
pub fn apply(playbook_yaml: &str, root: &Path) -> io::Result<ApplyReport> {
    write_private_data_dir(root, playbook_yaml)?;
    let status = Command::new("ansible-runner")
        .args(runner_argv(root))
        .status()?;
    latest_stats(root).map_or_else(
        || {
            Err(io::Error::other(format!(
                "ansible-runner produced no playbook_on_stats (exit {})",
                status.code().unwrap_or(-1)
            )))
        },
        Ok,
    )
}

/// Drift state of a node relative to its assigned baseline (Q108).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftStatus {
    /// Already in the desired state — the apply changed nothing.
    InSync,
    /// The node had drifted; re-applying the baseline re-converged it (`changed > 0`).
    Healed,
    /// The apply could not converge the node (a task failed, or it was unreachable).
    Failed,
}

impl DriftStatus {
    /// Classify a completed [`ApplyReport`] into a drift outcome.
    #[must_use]
    pub const fn classify(report: &ApplyReport) -> Self {
        if !report.converged() {
            Self::Failed
        } else if report.made_changes() {
            Self::Healed
        } else {
            Self::InSync
        }
    }
}

/// Converge the local node to its `playbook_yaml` baseline; report the drift outcome.
///
/// Q108 auto-heal: re-applying the desired state heals any drift in place, and
/// `changed > 0` is the signal that drift was present. The returned
/// `(status, report)` pair is the audit record the caller persists.
///
/// # Errors
/// Propagates [`apply`] errors (ansible-runner unavailable / produced no stats).
pub fn heal_to_baseline(
    playbook_yaml: &str,
    root: &Path,
) -> io::Result<(DriftStatus, ApplyReport)> {
    let report = apply(playbook_yaml, root)?;
    Ok((DriftStatus::classify(&report), report))
}

/// `present` (the resource must exist) / `absent` (must not).
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PresentAbsent {
    /// The resource must exist / be installed.
    #[default]
    Present,
    /// The resource must not exist / be removed.
    Absent,
}

/// A systemd service's desired run-state.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// The service must be running.
    #[default]
    Started,
    /// The service must be stopped.
    Stopped,
}

/// A package the node must have / not have.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PackageReq {
    /// Package name (dnf/rpm).
    pub name: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
}

/// A systemd service's desired enablement + run-state.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ServiceReq {
    /// systemd unit name.
    pub name: String,
    /// Enable at boot (default `true`).
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Desired run-state (default `started`).
    #[serde(default)]
    pub state: ServiceState,
}

/// A managed file: `content` placed at `path` when `present`, removed when `absent`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FileReq {
    /// Absolute destination path.
    pub path: String,
    /// File body, written when `present`.
    #[serde(default)]
    pub content: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
}

const fn yes() -> bool {
    true
}

/// A declarative desired-state baseline (Q121/Q123) — the YAML a fleet revision
/// carries.
///
/// [`to_playbook`] renders it to an Ansible playbook that converges the node.
/// Covers the most common OS domains; every section defaults empty (a baseline
/// declares only what it manages), and new domains extend this without breaking
/// older revisions.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct BaselineSpec {
    /// Packages to install/remove.
    pub packages: Vec<PackageReq>,
    /// systemd services to enable/start/stop.
    pub services: Vec<ServiceReq>,
    /// Files to place/remove.
    pub files: Vec<FileReq>,
}

impl BaselineSpec {
    /// Parse a baseline from its YAML representation.
    ///
    /// # Errors
    /// On malformed YAML or an unknown top-level field.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

/// Render a [`BaselineSpec`] into an Ansible playbook (one local play, `become`).
///
/// Uses the `ansible.builtin` modules, built as structured values and serialised
/// via serde so resource names/content are correctly YAML-escaped.
///
/// # Errors
/// On YAML serialisation failure (practically never for this fixed shape).
pub fn to_playbook(spec: &BaselineSpec) -> Result<String, serde_yaml::Error> {
    use serde_json::json;
    let mut tasks: Vec<serde_json::Value> = Vec::new();
    for p in &spec.packages {
        let state = match p.state {
            PresentAbsent::Present => "present",
            PresentAbsent::Absent => "absent",
        };
        tasks.push(json!({
            "name": format!("package {} -> {state}", p.name),
            "ansible.builtin.package": { "name": p.name, "state": state },
        }));
    }
    for s in &spec.services {
        let state = match s.state {
            ServiceState::Started => "started",
            ServiceState::Stopped => "stopped",
        };
        tasks.push(json!({
            "name": format!("service {} -> {state} (enabled={})", s.name, s.enabled),
            "ansible.builtin.service": { "name": s.name, "state": state, "enabled": s.enabled },
        }));
    }
    for f in &spec.files {
        match f.state {
            PresentAbsent::Present => tasks.push(json!({
                "name": format!("file {} -> present", f.path),
                "ansible.builtin.copy": { "dest": f.path, "content": f.content },
            })),
            PresentAbsent::Absent => tasks.push(json!({
                "name": format!("file {} -> absent", f.path),
                "ansible.builtin.file": { "path": f.path, "state": "absent" },
            })),
        }
    }
    let playbook = json!([{
        "hosts": "local",
        "become": true,
        "gather_facts": false,
        "tasks": tasks,
    }]);
    serde_yaml::to_string(&playbook)
}

/// Converge the node to a desired-state [`BaselineSpec`] (render → heal).
///
/// The full node-side fleet-sync path: a revision carries a `BaselineSpec`, the
/// node renders it to a playbook and heals to it, reporting the drift outcome.
///
/// # Errors
/// On render ([`to_playbook`]) or [`apply`] failure.
pub fn converge(spec: &BaselineSpec, root: &Path) -> io::Result<(DriftStatus, ApplyReport)> {
    let playbook = to_playbook(spec).map_err(io::Error::other)?;
    heal_to_baseline(&playbook, root)
}

/// Read the newest `artifacts/<ident>/job_events/*` `playbook_on_stats` event
/// under a private-data-dir `root`.
fn latest_stats(root: &Path) -> Option<ApplyReport> {
    let mut idents: Vec<PathBuf> = std::fs::read_dir(root.join("artifacts"))
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    idents.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let events = idents.last()?.join("job_events");
    std::fs::read_dir(&events)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .find_map(|s| parse_stats(&s))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real `playbook_on_stats` event emitted by ansible-runner 2.4.2 /
    /// ansible-core 2.20.6 (captured from a localhost apply: ok=1, changed=1).
    const REAL_STATS: &str = r#"{"uuid":"7585","counter":7,"event":"playbook_on_stats","event_data":{"playbook":"site.yml","changed":{"localhost":1},"dark":{},"failures":{},"ignored":{},"ok":{"localhost":1},"processed":{"localhost":1},"rescued":{},"skipped":{}}}"#;

    #[test]
    fn parse_stats_reads_a_real_event() {
        let r = parse_stats(REAL_STATS).expect("real stats event parses");
        assert_eq!(
            r,
            ApplyReport {
                ok: 1,
                changed: 1,
                failures: 0,
                unreachable: 0
            }
        );
        assert!(r.converged());
        assert!(r.made_changes());
    }

    #[test]
    fn parse_stats_sums_multiple_hosts_and_reads_dark_as_unreachable() {
        let json = r#"{"event":"playbook_on_stats","event_data":{
            "ok":{"a":3,"b":2},"changed":{"a":1},"failures":{"b":1},"dark":{"c":1}}}"#;
        let r = parse_stats(json).unwrap();
        assert_eq!(r.ok, 5);
        assert_eq!(r.changed, 1);
        assert_eq!(r.failures, 1);
        assert_eq!(r.unreachable, 1);
        assert!(
            !r.converged(),
            "a failure + an unreachable host is not converged"
        );
    }

    #[test]
    fn parse_stats_rejects_non_stats_events() {
        assert!(parse_stats(r#"{"event":"runner_on_ok","event_data":{}}"#).is_none());
        assert!(parse_stats("not json").is_none());
    }

    #[test]
    fn converged_idempotent_reapply_made_no_changes() {
        let r = ApplyReport {
            ok: 1,
            changed: 0,
            failures: 0,
            unreachable: 0,
        };
        assert!(r.converged());
        assert!(
            !r.made_changes(),
            "a no-op re-apply reports converged but unchanged"
        );
    }

    #[test]
    fn drift_classify_maps_apply_outcomes() {
        let mk = |ok, changed, failures, unreachable| ApplyReport {
            ok,
            changed,
            failures,
            unreachable,
        };
        assert_eq!(DriftStatus::classify(&mk(2, 0, 0, 0)), DriftStatus::InSync);
        assert_eq!(DriftStatus::classify(&mk(1, 1, 0, 0)), DriftStatus::Healed);
        assert_eq!(DriftStatus::classify(&mk(0, 0, 1, 0)), DriftStatus::Failed);
        assert_eq!(
            DriftStatus::classify(&mk(0, 0, 0, 1)),
            DriftStatus::Failed,
            "an unreachable node is a failed heal, not in-sync"
        );
    }

    #[test]
    fn baseline_spec_parses_and_renders_to_a_playbook() {
        let yaml = "
packages:
  - name: htop
  - name: telnet
    state: absent
services:
  - name: sshd
    enabled: true
    state: started
files:
  - path: /etc/motd
    content: \"welcome\\n\"
";
        let spec = BaselineSpec::from_yaml(yaml).expect("baseline parses");
        assert_eq!(spec.packages.len(), 2);
        assert_eq!(spec.packages[0].state, PresentAbsent::Present); // default
        assert_eq!(spec.packages[1].state, PresentAbsent::Absent);
        assert!(spec.services[0].enabled);

        let pb = to_playbook(&spec).expect("renders");
        let v: serde_yaml::Value = serde_yaml::from_str(&pb).unwrap();
        let play = &v[0];
        assert_eq!(play["hosts"].as_str(), Some("local"));
        assert_eq!(play["become"].as_bool(), Some(true));
        let tasks = play["tasks"].as_sequence().unwrap();
        assert_eq!(tasks.len(), 4, "2 packages + 1 service + 1 file");
        assert_eq!(
            tasks[0]["ansible.builtin.package"]["name"].as_str(),
            Some("htop")
        );
        assert_eq!(
            tasks[0]["ansible.builtin.package"]["state"].as_str(),
            Some("present")
        );
        assert_eq!(
            tasks[1]["ansible.builtin.package"]["state"].as_str(),
            Some("absent")
        );
        assert_eq!(
            tasks[2]["ansible.builtin.service"]["state"].as_str(),
            Some("started")
        );
        assert_eq!(
            tasks[2]["ansible.builtin.service"]["enabled"].as_bool(),
            Some(true)
        );
        assert_eq!(
            tasks[3]["ansible.builtin.copy"]["dest"].as_str(),
            Some("/etc/motd")
        );
    }

    #[test]
    fn baseline_rejects_unknown_top_level_fields() {
        // deny_unknown_fields stops a typo'd domain from silently no-op'ing.
        assert!(BaselineSpec::from_yaml("widgets:\n  - name: x\n").is_err());
    }

    #[test]
    fn file_absent_renders_a_remove_task() {
        let spec =
            BaselineSpec::from_yaml("files:\n  - path: /tmp/x\n    state: absent\n").unwrap();
        let pb = to_playbook(&spec).unwrap();
        assert!(pb.contains("ansible.builtin.file"));
        assert!(pb.contains("absent"));
        assert!(!pb.contains("ansible.builtin.copy"));
    }

    #[test]
    fn runner_argv_targets_site_yml_quietly() {
        let argv = runner_argv(Path::new("/run/pdd"));
        assert_eq!(argv, vec!["run", "/run/pdd", "-p", "site.yml", "--quiet"]);
    }

    #[test]
    fn write_private_data_dir_lays_out_project_and_local_inventory() {
        let root = std::env::temp_dir().join(format!("magic-fleet-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        write_private_data_dir(&root, "- hosts: local\n  tasks: []\n").unwrap();
        let pb = std::fs::read_to_string(root.join("project/site.yml")).unwrap();
        assert!(pb.contains("hosts: local"));
        let inv = std::fs::read_to_string(root.join("inventory/hosts")).unwrap();
        assert!(inv.contains("localhost ansible_connection=local"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
