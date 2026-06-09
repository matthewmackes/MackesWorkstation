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

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PresentAbsent {
    /// The resource must exist / be installed.
    #[default]
    Present,
    /// The resource must not exist / be removed.
    Absent,
}

/// A systemd service's desired run-state.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// The service must be running.
    #[default]
    Started,
    /// The service must be stopped.
    Stopped,
}

/// A package the node must have / not have.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageReq {
    /// Package name (dnf/rpm).
    pub name: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
}

/// A systemd service's desired enablement + run-state.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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

/// The Ansible `state:` string for a present/absent desire.
const fn pa(state: PresentAbsent) -> &'static str {
    match state {
        PresentAbsent::Present => "present",
        PresentAbsent::Absent => "absent",
    }
}

/// A user account the node must have (`present`) or not (`absent`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserReq {
    /// Login name.
    pub name: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
    /// Supplementary groups (appended, not exclusive).
    #[serde(default)]
    pub groups: Vec<String>,
    /// Login shell, when the baseline pins one.
    #[serde(default)]
    pub shell: Option<String>,
    /// Create as a system account (UID below the login range).
    #[serde(default)]
    pub system: bool,
}

/// A group the node must have (`present`) or not (`absent`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct GroupReq {
    /// Group name.
    pub name: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
    /// Create as a system group.
    #[serde(default)]
    pub system: bool,
}

/// A scheduled task (a crontab entry, keyed by `name`).
///
/// Each unset schedule field falls through to Ansible's own `*` default, so a
/// baseline declares only the fields it constrains.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CronReq {
    /// Unique job identifier (Ansible's `name`, written as a crontab comment).
    pub name: String,
    /// Command to run (required when `present`).
    #[serde(default)]
    pub job: String,
    /// Desired presence (default `present`).
    #[serde(default)]
    pub state: PresentAbsent,
    /// Minute field (`*` when unset).
    #[serde(default)]
    pub minute: Option<String>,
    /// Hour field (`*` when unset).
    #[serde(default)]
    pub hour: Option<String>,
    /// Day-of-week field (`*` when unset).
    #[serde(default)]
    pub weekday: Option<String>,
    /// Crontab owner (root's crontab when unset).
    #[serde(default)]
    pub user: Option<String>,
}

/// A declarative desired-state baseline (Q121/Q123) — the YAML a fleet revision
/// carries.
///
/// [`to_playbook`] renders it to an Ansible playbook that converges the node.
/// Covers the common OS domains — packages, services, files, users, groups, and
/// scheduled tasks (cron); every section defaults empty (a baseline declares only
/// what it manages), and new domains extend this without breaking older revisions.
/// (`sysctl` + firewall await the `ansible.posix` collection — a follow-up slice.)
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct BaselineSpec {
    /// Packages to install/remove.
    pub packages: Vec<PackageReq>,
    /// systemd services to enable/start/stop.
    pub services: Vec<ServiceReq>,
    /// Files to place/remove.
    pub files: Vec<FileReq>,
    /// User accounts to create/remove.
    pub users: Vec<UserReq>,
    /// Groups to create/remove.
    pub groups: Vec<GroupReq>,
    /// Scheduled tasks (crontab entries) to install/remove.
    pub cron: Vec<CronReq>,
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

/// A versioned fleet revision (Q115) — a monotonic `version` plus the desired-state
/// baseline it carries, stamped with the authoring node + time.
///
/// Revisions gossip peer-to-peer with no fixed center (Q113/Q116), so a node may
/// hold several at once and must pick deterministically. [`supersedes`] defines
/// "newest wins": higher `version` first, ties broken by later `at`, then by
/// `author` for a total order every node agrees on.
///
/// [`supersedes`]: Revision::supersedes
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    /// Monotonic revision number; the fleet's notion of "newer".
    pub version: u64,
    /// Node id that authored this revision.
    #[serde(default)]
    pub author: String,
    /// Authoring time, Unix seconds (the version tiebreak).
    #[serde(default)]
    pub at: u64,
    /// The desired-state baseline this revision pins.
    #[serde(default)]
    pub spec: BaselineSpec,
}

impl Revision {
    /// Parse a revision from its YAML representation.
    ///
    /// # Errors
    /// On malformed YAML or an unknown field.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Serialise the revision to YAML (for gossiping it to a peer).
    ///
    /// # Errors
    /// On YAML serialisation failure (practically never for this shape).
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Does this revision win over `other` under the fleet's total order?
    ///
    /// Higher `version` wins; equal versions break to the later `at`; an exact
    /// tie breaks to the lexically greater `author` so every node elects the same
    /// revision without coordination. A revision never supersedes an identical one.
    #[must_use]
    pub fn supersedes(&self, other: &Self) -> bool {
        (self.version, self.at, self.author.as_str())
            > (other.version, other.at, other.author.as_str())
    }
}

/// Elect the winning revision from a set the node currently holds (the highest
/// under [`Revision::supersedes`]), or `None` when the set is empty.
///
/// This is the "newest wins, no fixed center" selection a node runs before
/// converging: gather the revisions gossiped from peers, pick one, apply it.
#[must_use]
pub fn elect_revision(revisions: &[Revision]) -> Option<&Revision> {
    revisions
        .iter()
        .reduce(|win, r| if r.supersedes(win) { r } else { win })
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
        let state = pa(p.state);
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
    // Groups render before users so a user's supplementary group already exists
    // when the user task references it (otherwise the apply fails).
    for g in &spec.groups {
        let state = pa(g.state);
        let mut args = serde_json::Map::new();
        args.insert("name".into(), json!(g.name));
        args.insert("state".into(), json!(state));
        if g.system {
            args.insert("system".into(), json!(true));
        }
        tasks.push(json!({
            "name": format!("group {} -> {state}", g.name),
            "ansible.builtin.group": args,
        }));
    }
    for u in &spec.users {
        let state = pa(u.state);
        let mut args = serde_json::Map::new();
        args.insert("name".into(), json!(u.name));
        args.insert("state".into(), json!(state));
        if !u.groups.is_empty() {
            args.insert("groups".into(), json!(u.groups.join(",")));
            args.insert("append".into(), json!(true));
        }
        if let Some(shell) = &u.shell {
            args.insert("shell".into(), json!(shell));
        }
        if u.system {
            args.insert("system".into(), json!(true));
        }
        tasks.push(json!({
            "name": format!("user {} -> {state}", u.name),
            "ansible.builtin.user": args,
        }));
    }
    for c in &spec.cron {
        let state = pa(c.state);
        let mut args = serde_json::Map::new();
        args.insert("name".into(), json!(c.name));
        args.insert("state".into(), json!(state));
        // job + schedule are only meaningful when installing the entry.
        if c.state == PresentAbsent::Present {
            args.insert("job".into(), json!(c.job));
            for (key, val) in [
                ("minute", &c.minute),
                ("hour", &c.hour),
                ("weekday", &c.weekday),
            ] {
                if let Some(v) = val {
                    args.insert(key.into(), json!(v));
                }
            }
        }
        if let Some(user) = &c.user {
            args.insert("user".into(), json!(user));
        }
        tasks.push(json!({
            "name": format!("cron {} -> {state}", c.name),
            "ansible.builtin.cron": args,
        }));
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

/// One persisted line of the drift-watch audit trail (Q108: auto-heal **with
/// audit**). Serialises to a single JSON object — the audit log is JSONL, one
/// record per converge, append-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AuditRecord {
    /// Wall-clock time of the converge, Unix seconds.
    pub at: u64,
    /// The drift outcome (`insync` / `healed` / `failed`).
    pub status: DriftStatus,
    /// Tasks already in the desired state.
    pub ok: u32,
    /// Tasks the converge changed (drift that was healed).
    pub changed: u32,
    /// Tasks that failed.
    pub failures: u32,
    /// Hosts that were unreachable.
    pub unreachable: u32,
}

impl AuditRecord {
    /// Build a record from a converge outcome stamped at `at` (Unix seconds).
    #[must_use]
    pub const fn new(at: u64, status: DriftStatus, report: &ApplyReport) -> Self {
        Self {
            at,
            status,
            ok: report.ok,
            changed: report.changed,
            failures: report.failures,
            unreachable: report.unreachable,
        }
    }

    /// The record as one JSONL line (trailing newline included).
    ///
    /// # Errors
    /// On JSON serialisation failure (practically never for this fixed shape).
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        Ok(format!("{}\n", serde_json::to_string(self)?))
    }
}

/// Current wall-clock time in Unix seconds (0 if the clock predates the epoch).
#[must_use]
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Append an [`AuditRecord`] as a JSONL line to the audit log at `log_path`,
/// creating the file (and parent dirs) if absent.
///
/// # Errors
/// On a serialisation failure or any filesystem error creating/opening/writing
/// the log.
pub fn append_audit(log_path: &Path, record: &AuditRecord) -> io::Result<()> {
    use std::io::Write;
    if let Some(parent) = log_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let line = record.to_jsonl().map_err(io::Error::other)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    f.write_all(line.as_bytes())
}

/// One drift-watch tick: converge the node to `spec`, stamp the outcome, and
/// append it to the audit log. Returns the persisted record so a caller (or the
/// `watch` loop) can react to the drift status.
///
/// This is the unit the scheduled drift-watch daemon repeats; running it once is
/// the daemon's single-shot mode.
///
/// # Errors
/// Propagates [`converge`] errors (render / ansible-runner) and
/// [`append_audit`] filesystem errors.
pub fn drift_watch_tick(
    spec: &BaselineSpec,
    root: &Path,
    audit_log: &Path,
) -> io::Result<AuditRecord> {
    let (status, report) = converge(spec, root)?;
    let record = AuditRecord::new(now_unix(), status, &report);
    append_audit(audit_log, &record)?;
    Ok(record)
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
    fn baseline_renders_users_groups_and_cron() {
        let yaml = "
groups:
  - name: developers
    system: true
users:
  - name: deploy
    groups: [developers, wheel]
    shell: /bin/bash
    system: true
  - name: olduser
    state: absent
cron:
  - name: nightly-heal
    job: magic-fleet converge /etc/magic/baseline.yml
    minute: \"0\"
    hour: \"3\"
  - name: stale-job
    state: absent
";
        let spec = BaselineSpec::from_yaml(yaml).expect("baseline parses");
        assert_eq!(spec.groups.len(), 1);
        assert_eq!(spec.users.len(), 2);
        assert_eq!(spec.cron.len(), 2);
        assert_eq!(spec.users[0].groups, vec!["developers", "wheel"]);
        assert_eq!(spec.users[1].state, PresentAbsent::Absent);

        let pb = to_playbook(&spec).expect("renders");
        let v: serde_yaml::Value = serde_yaml::from_str(&pb).unwrap();
        let tasks = v[0]["tasks"].as_sequence().unwrap();
        // 1 group + 2 users + 2 cron = 5 (no packages/services/files here).
        assert_eq!(tasks.len(), 5);

        let group = &tasks[0]["ansible.builtin.group"];
        assert_eq!(group["name"].as_str(), Some("developers"));
        assert_eq!(group["system"].as_bool(), Some(true));

        let deploy = &tasks[1]["ansible.builtin.user"];
        assert_eq!(deploy["name"].as_str(), Some("deploy"));
        assert_eq!(deploy["state"].as_str(), Some("present"));
        assert_eq!(deploy["groups"].as_str(), Some("developers,wheel"));
        assert_eq!(deploy["append"].as_bool(), Some(true));
        assert_eq!(deploy["shell"].as_str(), Some("/bin/bash"));
        assert_eq!(deploy["system"].as_bool(), Some(true));

        let removed = &tasks[2]["ansible.builtin.user"];
        assert_eq!(removed["state"].as_str(), Some("absent"));
        // an absent user carries no group/shell churn.
        assert!(removed.get("groups").is_none());
        assert!(removed.get("shell").is_none());

        let nightly = &tasks[3]["ansible.builtin.cron"];
        assert_eq!(nightly["name"].as_str(), Some("nightly-heal"));
        assert_eq!(
            nightly["job"].as_str(),
            Some("magic-fleet converge /etc/magic/baseline.yml")
        );
        assert_eq!(nightly["minute"].as_str(), Some("0"));
        assert_eq!(nightly["hour"].as_str(), Some("3"));
        // an unset schedule field falls through to Ansible's own `*` default.
        assert!(nightly.get("weekday").is_none());

        let stale = &tasks[4]["ansible.builtin.cron"];
        assert_eq!(stale["state"].as_str(), Some("absent"));
        // an absent cron entry needs no job/schedule.
        assert!(stale.get("job").is_none());
        assert!(stale.get("minute").is_none());
    }

    #[test]
    fn revision_election_is_newest_wins_with_deterministic_tiebreaks() {
        let rev = |version, at, author: &str| Revision {
            version,
            at,
            author: author.to_string(),
            spec: BaselineSpec::default(),
        };
        // Higher version wins outright.
        assert!(rev(5, 0, "a").supersedes(&rev(4, 999, "z")));
        // Equal version -> later `at` wins.
        assert!(rev(5, 200, "a").supersedes(&rev(5, 100, "z")));
        // Equal version + at -> lexically greater author wins (total order).
        assert!(rev(5, 100, "z").supersedes(&rev(5, 100, "a")));
        // A revision never supersedes its identical twin.
        assert!(!rev(5, 100, "a").supersedes(&rev(5, 100, "a")));

        // elect picks the winner regardless of input order.
        let set = [
            rev(2, 0, "a"),
            rev(7, 0, "b"),
            rev(7, 0, "a"),
            rev(3, 0, "c"),
        ];
        let winner = elect_revision(&set).unwrap();
        assert_eq!((winner.version, winner.author.as_str()), (7, "b"));
        assert!(elect_revision(&[]).is_none());
    }

    #[test]
    fn revision_round_trips_through_yaml_with_its_spec() {
        let yaml = "
version: 12
author: node-7
at: 1700000000
spec:
  packages:
    - name: htop
  services:
    - name: sshd
";
        let rev = Revision::from_yaml(yaml).expect("revision parses");
        assert_eq!(rev.version, 12);
        assert_eq!(rev.author, "node-7");
        assert_eq!(rev.spec.packages.len(), 1);

        // serialise it back out and re-parse: same revision (gossip round-trip).
        let out = rev.to_yaml().unwrap();
        let again = Revision::from_yaml(&out).unwrap();
        assert_eq!(rev, again);
    }

    #[test]
    fn revision_rejects_unknown_fields() {
        assert!(Revision::from_yaml("version: 1\nbogus: true\n").is_err());
    }

    #[test]
    fn audit_record_serialises_to_one_jsonl_line() {
        let report = ApplyReport {
            ok: 4,
            changed: 2,
            failures: 0,
            unreachable: 0,
        };
        let rec = AuditRecord::new(1_700_000_000, DriftStatus::Healed, &report);
        let line = rec.to_jsonl().unwrap();
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1, "exactly one line");
        // status renders lowercase (the JSONL is grep-friendly).
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["at"].as_u64(), Some(1_700_000_000));
        assert_eq!(v["status"].as_str(), Some("healed"));
        assert_eq!(v["changed"].as_u64(), Some(2));
        assert_eq!(v["ok"].as_u64(), Some(4));
    }

    #[test]
    fn append_audit_creates_dirs_and_appends_jsonl() {
        let dir = std::env::temp_dir().join(format!("magic-fleet-audit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // nested path that does not exist yet — append_audit must create it.
        let log = dir.join("nested").join("drift-audit.jsonl");
        let mk = |at, status, changed| {
            AuditRecord::new(
                at,
                status,
                &ApplyReport {
                    ok: 1,
                    changed,
                    failures: 0,
                    unreachable: 0,
                },
            )
        };
        append_audit(&log, &mk(100, DriftStatus::Healed, 1)).unwrap();
        append_audit(&log, &mk(200, DriftStatus::InSync, 0)).unwrap();

        let body = std::fs::read_to_string(&log).unwrap();
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2, "append, not overwrite");
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(first["at"].as_u64(), Some(100));
        assert_eq!(first["status"].as_str(), Some("healed"));
        assert_eq!(second["at"].as_u64(), Some(200));
        assert_eq!(second["status"].as_str(), Some("insync"));
        let _ = std::fs::remove_dir_all(&dir);
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
