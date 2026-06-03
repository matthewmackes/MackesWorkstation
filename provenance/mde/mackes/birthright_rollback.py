"""Birthright rollback ledger — Phase 10.6.8.

Every destructive birthright step (apply_panel_swap, apply_panel_archive,
apply_uninstall_legacy_xfce) writes a JSON record under
`~/.config/mackes-panel/rollback/<step_name>.json` BEFORE it mutates the
system. If the panel binary segfaults or the daemon-stop wedges, the
operator runs `mackes recover` (privileged path via AdminSession) or
`mackes-panel --recover` (read-only preview from the Rust binary) and the
recorded `restore_actions` reverse the step in question.

Record schema (JSON on disk):

  {
    "step_name": "apply_panel_swap",
    "timestamp": "2026-05-19T12:34:56Z",
    "prior_state": { ... step-specific JSON ... },
    "restore_actions": [
        {"kind": "shell", "argv": ["dnf", "install", "-y", "xfce4-panel"],
         "needs_root": true, "description": "re-install xfce4-panel"},
        {"kind": "write_file", "path": "/home/.../foo.desktop",
         "content": "<text>",
         "description": "restore prior autostart override"},
        {"kind": "delete_file", "path": "/home/.../bar.desktop",
         "description": "remove autostart override added by panel-swap"},
        {"kind": "xfconf_set", "channel": "xfce4-keyboard-shortcuts",
         "property": "/commands/custom/<Super>l",
         "value_type": "string", "value": "xfce4-popup-whiskermenu",
         "description": "restore Whisker Super-l binding"},
        {"kind": "xfconf_unset", "channel": "xfce4-keyboard-shortcuts",
         "property": "/commands/custom/<Super>l",
         "description": "clear panel-swap override"}
    ]
  }

Restore action kinds (executed by `restore_one`):

  * `shell`        — argv list. `needs_root=true` routes through
                     AdminSession; otherwise subprocess.run direct.
  * `write_file`   — overwrite path with content (string). mkdir -p parent.
  * `delete_file`  — unlink path if it exists.
  * `xfconf_set`   — `xfconf-query --create --set` to restore a channel
                     property.
  * `xfconf_unset` — `xfconf-query --reset` to drop a panel-swap override
                     when no prior value existed.

Public API:

  RollbackStep — dataclass mirroring the on-disk JSON record.
  record(step_name, prior_state, restore_actions)  — write the record.
  list_recent(limit=10)                            — newest first.
  restore_one(step_name)                           — read JSON + run actions.
  restore_all()                                    — every record, newest-first.
  rollback_dir() / set_rollback_dir_override()     — XDG-aware path helpers.

The directory lives at `~/.config/mackes-panel/rollback/` so it shares
parent with the Rust panel's `panel.toml` — the same path is read by the
Rust `--recover` previewer (see `crates/mackes-panel/src/recover.rs`).
"""
from __future__ import annotations

import datetime as _dt
import json
import os
import shutil
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional

from mackes.logging import log_action


# ---------------------------------------------------------------------------
# Path resolution
# ---------------------------------------------------------------------------

# When set (tests + headless CLI sandboxing) overrides the XDG-derived
# default. Production code uses `rollback_dir()` to honor this hook
# transparently.
_DIR_OVERRIDE: Optional[Path] = None


def set_rollback_dir_override(path: Optional[Path]) -> None:
    """Tests / CLI bootstrap call this to redirect the rollback ledger
    out of the user's real `~/.config/`. Pass `None` to clear."""
    global _DIR_OVERRIDE
    _DIR_OVERRIDE = Path(path) if path is not None else None


def rollback_dir() -> Path:
    """Resolve the on-disk rollback directory.

    Order of precedence:
        1. The override set via `set_rollback_dir_override()`.
        2. `$XDG_CONFIG_HOME/mackes-panel/rollback/`.
        3. `~/.config/mackes-panel/rollback/`.

    The directory is not created here — `record()` creates it on demand
    so a fresh box has no stray empty dir before anything has rolled back.
    """
    if _DIR_OVERRIDE is not None:
        return _DIR_OVERRIDE
    xdg = os.environ.get("XDG_CONFIG_HOME")
    if xdg:
        return Path(xdg) / "mackes-panel" / "rollback"
    return Path(os.path.expanduser("~")) / ".config" / "mackes-panel" / "rollback"


# ---------------------------------------------------------------------------
# Record dataclass
# ---------------------------------------------------------------------------


@dataclass
class RollbackStep:
    """One rollback record. Serializes 1:1 to the on-disk JSON."""
    step_name: str
    timestamp: str
    prior_state: Dict[str, Any] = field(default_factory=dict)
    restore_actions: List[Dict[str, Any]] = field(default_factory=list)

    def to_json(self) -> str:
        return json.dumps(asdict(self), indent=2, sort_keys=True)

    @classmethod
    def from_json(cls, text: str) -> "RollbackStep":
        data = json.loads(text)
        return cls(
            step_name=str(data.get("step_name", "")),
            timestamp=str(data.get("timestamp", "")),
            prior_state=dict(data.get("prior_state") or {}),
            restore_actions=list(data.get("restore_actions") or []),
        )

    @classmethod
    def load(cls, path: Path) -> "RollbackStep":
        return cls.from_json(path.read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# record() — called from birthright steps before they mutate state.
# ---------------------------------------------------------------------------


def _utc_timestamp() -> str:
    """ISO-8601 UTC, second-precision, suffixed with 'Z'.

    Stable sort key on disk: lexicographic order matches chronological
    order for this format, which keeps `list_recent` cheap.
    """
    now = _dt.datetime.now(tz=_dt.timezone.utc).replace(microsecond=0)
    return now.strftime("%Y-%m-%dT%H:%M:%SZ")


def record(
    step_name: str,
    prior_state: Dict[str, Any],
    restore_actions: List[Dict[str, Any]],
) -> Path:
    """Persist a `RollbackStep` to `<rollback_dir>/<step_name>.json`.

    The filename is the step name (not timestamped) — when the same step
    runs twice (e.g. wizard reruns), the second invocation overwrites the
    first. This is the desired behavior: rollback should target the most
    recent application of each step, not an interleaved history.

    Returns the path written. Logs the action via `log_action`.

    Raises OSError on I/O failure — caller (a birthright step) is
    responsible for catching + degrading gracefully if it considers a
    missing rollback record non-fatal.
    """
    if not step_name:
        raise ValueError("step_name must be non-empty")
    step = RollbackStep(
        step_name=step_name,
        timestamp=_utc_timestamp(),
        prior_state=prior_state,
        restore_actions=restore_actions,
    )
    target_dir = rollback_dir()
    target_dir.mkdir(parents=True, exist_ok=True)
    out = target_dir / f"{step_name}.json"
    out.write_text(step.to_json() + "\n", encoding="utf-8")
    log_action(f"rollback: recorded prior state for {step_name} → {out}")
    return out


# ---------------------------------------------------------------------------
# list_recent() / load
# ---------------------------------------------------------------------------


def list_recent(limit: int = 10) -> List[RollbackStep]:
    """Return every well-formed rollback record, newest first.

    Corrupted JSON files are silently skipped (logged once per file) so
    a single bad record never prevents the rest of the ledger from being
    surfaced. `limit` caps the result length.
    """
    out: List[RollbackStep] = []
    rdir = rollback_dir()
    if not rdir.is_dir():
        return out
    try:
        entries = sorted(rdir.iterdir())
    except OSError:
        return out
    parsed: List[RollbackStep] = []
    for entry in entries:
        if entry.suffix != ".json" or not entry.is_file():
            continue
        try:
            parsed.append(RollbackStep.load(entry))
        except (OSError, json.JSONDecodeError, ValueError) as e:
            log_action(f"rollback: skipping corrupt record {entry}: {e}")
            continue
    parsed.sort(key=lambda s: s.timestamp, reverse=True)
    if limit > 0:
        parsed = parsed[:limit]
    out.extend(parsed)
    return out


def load_step(step_name: str) -> Optional[RollbackStep]:
    """Load one record by step_name. Returns `None` when missing or
    corrupted."""
    path = rollback_dir() / f"{step_name}.json"
    if not path.is_file():
        return None
    try:
        return RollbackStep.load(path)
    except (OSError, json.JSONDecodeError, ValueError) as e:
        log_action(f"rollback: could not load {path}: {e}")
        return None


# ---------------------------------------------------------------------------
# restore_one() / restore_all()
# ---------------------------------------------------------------------------


# Hook for the privileged CLI path. When set, all `needs_root=True` shell
# actions route through it; otherwise they fall back to plain `subprocess.run`.
# The CLI bootstrap wires this to `AdminSession.instance().run`.
_ROOT_RUNNER: Optional[Callable[[List[str]], int]] = None


def set_root_runner(runner: Optional[Callable[[List[str]], int]]) -> None:
    """`runner` must take an argv list and return an exit code. Pass
    `None` to clear (back to plain subprocess.run)."""
    global _ROOT_RUNNER
    _ROOT_RUNNER = runner


def _exec_shell(action: Dict[str, Any]) -> tuple[bool, str]:
    argv = list(action.get("argv") or [])
    if not argv:
        return False, "shell action with empty argv"
    needs_root = bool(action.get("needs_root"))
    timeout = int(action.get("timeout") or 600)
    try:
        if needs_root and _ROOT_RUNNER is not None:
            rc = _ROOT_RUNNER(argv)
            return rc == 0, f"rc={rc}"
        proc = subprocess.run(
            argv, capture_output=True, text=True, timeout=timeout,
        )
        return proc.returncode == 0, (
            (proc.stdout + proc.stderr).strip() or f"rc={proc.returncode}"
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        return False, str(e)


def _exec_write_file(action: Dict[str, Any]) -> tuple[bool, str]:
    path_s = action.get("path")
    if not path_s:
        return False, "write_file action without path"
    path = Path(str(path_s))
    content = str(action.get("content", ""))
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return True, f"wrote {path}"
    except OSError as e:
        return False, f"write failed: {e}"


def _exec_delete_file(action: Dict[str, Any]) -> tuple[bool, str]:
    path_s = action.get("path")
    if not path_s:
        return False, "delete_file action without path"
    path = Path(str(path_s))
    try:
        if path.is_file() or path.is_symlink():
            path.unlink()
            return True, f"deleted {path}"
        if path.is_dir():
            shutil.rmtree(path)
            return True, f"removed dir {path}"
        return True, f"not present: {path}"
    except OSError as e:
        return False, f"delete failed: {e}"


def _exec_xfconf_set(action: Dict[str, Any]) -> tuple[bool, str]:
    channel = action.get("channel")
    prop = action.get("property")
    value = action.get("value")
    vtype = action.get("value_type") or "string"
    if not (channel and prop):
        return False, "xfconf_set requires channel + property"
    if shutil.which("xfconf-query") is None:
        return False, "xfconf-query not on PATH"
    argv = [
        "xfconf-query",
        "--channel", str(channel),
        "--property", str(prop),
        "--type", str(vtype),
        "--set", str(value),
        "--create",
    ]
    try:
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=10)
        return proc.returncode == 0, (
            (proc.stdout + proc.stderr).strip() or f"rc={proc.returncode}"
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        return False, str(e)


def _exec_xfconf_unset(action: Dict[str, Any]) -> tuple[bool, str]:
    channel = action.get("channel")
    prop = action.get("property")
    if not (channel and prop):
        return False, "xfconf_unset requires channel + property"
    if shutil.which("xfconf-query") is None:
        return False, "xfconf-query not on PATH"
    argv = [
        "xfconf-query",
        "--channel", str(channel),
        "--property", str(prop),
        "--reset",
    ]
    try:
        proc = subprocess.run(argv, capture_output=True, text=True, timeout=10)
        return proc.returncode == 0, (
            (proc.stdout + proc.stderr).strip() or f"rc={proc.returncode}"
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        return False, str(e)


_EXECUTORS: Dict[str, Callable[[Dict[str, Any]], tuple[bool, str]]] = {
    "shell": _exec_shell,
    "write_file": _exec_write_file,
    "delete_file": _exec_delete_file,
    "xfconf_set": _exec_xfconf_set,
    "xfconf_unset": _exec_xfconf_unset,
}


def restore_one(step_name: str) -> List[str]:
    """Reverse a single step. Runs every action in `restore_actions` in
    REVERSE order (last-applied is first-reversed).

    Returns a list of log lines suitable for printing to the user.
    Never raises — failures land in the log list with `FAIL` markers,
    and the next action still runs so partial restores complete the
    parts they can.
    """
    lines: List[str] = []
    step = load_step(step_name)
    if step is None:
        msg = f"rollback: no record for step {step_name!r}"
        log_action(msg)
        lines.append(msg)
        return lines

    lines.append(f"rollback: restoring {step.step_name} "
                 f"(recorded {step.timestamp})")
    log_action(lines[-1])

    # Reverse order — the LAST mutation the step performed is the FIRST
    # we need to undo, so the system passes through every prior good
    # state on the way back to baseline.
    for action in reversed(step.restore_actions):
        kind = str(action.get("kind") or "")
        desc = str(action.get("description") or kind)
        executor = _EXECUTORS.get(kind)
        if executor is None:
            line = f"rollback:  ?  unknown action kind {kind!r} — skipped"
            lines.append(line)
            log_action(line)
            continue
        ok, detail = executor(action)
        marker = "OK" if ok else "FAIL"
        line = f"rollback:  {marker:4s} {desc} — {detail}"
        lines.append(line)
        log_action(line)

    # The record itself stays on disk so a second `mackes recover` is a
    # no-op rather than a fresh restore — this protects users who run
    # the recovery flow twice from accidentally re-applying the actions
    # against a now-good system.
    return lines


def restore_all() -> List[str]:
    """Restore every recorded step, newest first.

    Newest-first matters: if the user ran panel-swap (10.6.1-4) then
    uninstall-legacy-xfce (10.6.6), we need to re-install the packages
    FIRST before the panel-swap restore tries to flip xfce4-panel's
    autostart override back on. Reversing in record-creation order
    would invert that and fail.
    """
    lines: List[str] = []
    records = list_recent(limit=100)
    if not records:
        lines.append("rollback: no records found — nothing to do")
        return lines
    lines.append(f"rollback: restoring {len(records)} step(s)")
    for step in records:
        lines.extend(restore_one(step.step_name))
    return lines


# ---------------------------------------------------------------------------
# Helpers for birthright steps to build their `prior_state` payloads.
# Kept module-local because they're step-specific glue — each one is a
# small wrapper that turns the live system state into the matching record.
# ---------------------------------------------------------------------------


def capture_panel_swap_state() -> tuple[Dict[str, Any], List[Dict[str, Any]]]:
    """Snapshot the pieces apply_panel_swap is about to mutate.

    Returns (prior_state, restore_actions). The restore_actions are
    pre-built so the panel-swap step doesn't have to know the rollback
    schema in detail.
    """
    home = Path(os.path.expanduser("~"))
    autostart = home / ".config" / "autostart" / "xfce4-panel.desktop"

    prior_state: Dict[str, Any] = {
        "xfce4_panel_installed": shutil.which("xfce4-panel") is not None,
        "xfdesktop_installed": shutil.which("xfdesktop") is not None,
        "autostart_existed": autostart.is_file(),
        "autostart_content": "",
        "keybindings": {},
    }
    restore_actions: List[Dict[str, Any]] = []

    # Autostart override: capture prior content so we can re-write it
    # exactly. If the file did not exist, the rollback removes the
    # override that apply_panel_swap is about to drop.
    if autostart.is_file():
        try:
            prior_state["autostart_content"] = autostart.read_text(encoding="utf-8")
            restore_actions.append({
                "kind": "write_file",
                "path": str(autostart),
                "content": prior_state["autostart_content"],
                "description": f"restore prior {autostart}",
            })
        except OSError:
            # Treat unreadable as not-existed; rollback will delete the
            # override panel-swap is about to write.
            prior_state["autostart_existed"] = False
            restore_actions.append({
                "kind": "delete_file",
                "path": str(autostart),
                "description": f"remove panel-swap autostart override at {autostart}",
            })
    else:
        restore_actions.append({
            "kind": "delete_file",
            "path": str(autostart),
            "description": f"remove panel-swap autostart override at {autostart}",
        })

    # Keybindings: <Super>l + <Super>Space — same pair the step rebinds.
    for combo in ("<Super>l", "<Super>Space"):
        prop = f"/commands/custom/{combo}"
        current = _probe_xfconf(prop)
        prior_state["keybindings"][combo] = current
        if current is not None:
            restore_actions.append({
                "kind": "xfconf_set",
                "channel": "xfce4-keyboard-shortcuts",
                "property": prop,
                "value_type": "string",
                "value": current,
                "description": f"restore prior Whisker binding for {combo}",
            })
        else:
            restore_actions.append({
                "kind": "xfconf_unset",
                "channel": "xfce4-keyboard-shortcuts",
                "property": prop,
                "description": f"clear panel-swap override for {combo}",
            })

    # Re-launch xfce4-panel / xfdesktop if they were installed; the
    # original step quit them. Best-effort — runs after package restore.
    if prior_state["xfce4_panel_installed"]:
        restore_actions.append({
            "kind": "shell",
            "argv": ["xfce4-panel"],
            "needs_root": False,
            "description": "relaunch xfce4-panel",
        })
    if prior_state["xfdesktop_installed"]:
        restore_actions.append({
            "kind": "shell",
            "argv": ["xfdesktop"],
            "needs_root": False,
            "description": "relaunch xfdesktop",
        })

    return prior_state, restore_actions


def _probe_xfconf(prop: str) -> Optional[str]:
    """Return the current xfconf value for `prop` on the
    xfce4-keyboard-shortcuts channel, or None when unset / unavailable.
    """
    if shutil.which("xfconf-query") is None:
        return None
    try:
        proc = subprocess.run(
            ["xfconf-query", "--channel", "xfce4-keyboard-shortcuts",
             "--property", prop],
            capture_output=True, text=True, timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    out = proc.stdout.strip()
    return out or None


def capture_panel_archive_state() -> tuple[Dict[str, Any], List[Dict[str, Any]]]:
    """Snapshot for apply_panel_archive: the rollback removes the archive
    directory if the step created it, and leaves a pre-existing archive
    alone (idempotent reruns must not delete it on undo)."""
    home = Path(os.path.expanduser("~"))
    dst = home / ".config" / "mackes-panel" / "legacy-xfce-panel"
    pre_existed = dst.exists()
    prior_state = {
        "archive_dir": str(dst),
        "archive_existed_before": pre_existed,
    }
    restore_actions: List[Dict[str, Any]] = []
    if not pre_existed:
        restore_actions.append({
            "kind": "delete_file",
            "path": str(dst),
            "description": f"remove archive directory at {dst}",
        })
    return prior_state, restore_actions


def capture_uninstall_legacy_state(
    installed_now: List[str],
) -> tuple[Dict[str, Any], List[Dict[str, Any]]]:
    """Snapshot for apply_uninstall_legacy_xfce. `installed_now` is the
    subset of `_LEGACY_XFCE_PACKAGES` rpm currently reports as installed —
    the same list the step is about to pass to dnf.

    Rollback re-installs those packages via `dnf install -y` routed
    through AdminSession.
    """
    prior_state = {
        "installed_packages": list(installed_now),
    }
    restore_actions: List[Dict[str, Any]] = []
    if installed_now:
        restore_actions.append({
            "kind": "shell",
            "argv": ["dnf", "install", "-y", *installed_now],
            "needs_root": True,
            "timeout": 900,
            "description": "re-install legacy XFCE packages: "
                           + ", ".join(installed_now),
        })
    return prior_state, restore_actions
