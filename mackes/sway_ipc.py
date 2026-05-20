"""Tiny `swaymsg` subprocess wrapper (v2.0.0 Phase F.8).

Replaces the v1.x `i3-msg` calls scattered through the Workbench
window-manager panel with one focused module. Every function is a
sync subprocess shell-out — Python on the v1.x line doesn't have an
async runtime, and the panel only needs single-shot commands. The
Phase E.4 panel rewrite will move to the real `swayipc-async` Rust
crate; until then this shim covers the Workbench panel's needs.

Public API:

  is_sway_running()   → True when `swaymsg` is callable AND the
                        compositor responds.
  current_workspace() → integer workspace number (None on failure).
  focus_workspace(n)  → True on success.
  set_layout(name)    → True on success ("splith", "splitv",
                        "tabbed", "stacking", "default").
  kill_focused()      → True on success.
"""
from __future__ import annotations

import json
import shutil
import subprocess
from typing import Optional


def _swaymsg_path() -> Optional[str]:
    return shutil.which("swaymsg")


def is_sway_running() -> bool:
    """True when swaymsg is on PATH AND the compositor responds to
    `-t get_version`. False on any failure (no Wayland, no sway, no
    binary, etc.)."""
    path = _swaymsg_path()
    if path is None:
        return False
    try:
        r = subprocess.run(
            [path, "-t", "get_version"],
            capture_output=True, text=True, timeout=3,
        )
    except (OSError, subprocess.SubprocessError):
        return False
    return r.returncode == 0


def _run(args: list[str]) -> tuple[int, str, str]:
    """Spawn `swaymsg <args>`. Returns (exit_code, stdout, stderr)
    with empty strings on missing binary."""
    path = _swaymsg_path()
    if path is None:
        return (127, "", "swaymsg not on $PATH")
    try:
        r = subprocess.run(
            [path, *args],
            capture_output=True, text=True, timeout=5,
        )
    except (OSError, subprocess.SubprocessError) as e:
        return (1, "", str(e))
    return (r.returncode, r.stdout, r.stderr)


def current_workspace() -> Optional[int]:
    """Return the integer name of the currently-focused workspace,
    or None when the IPC call fails / the workspace isn't named with
    an integer."""
    rc, out, _ = _run(["-t", "get_workspaces"])
    if rc != 0:
        return None
    try:
        workspaces = json.loads(out)
    except json.JSONDecodeError:
        return None
    if not isinstance(workspaces, list):
        return None
    for ws in workspaces:
        if isinstance(ws, dict) and ws.get("focused"):
            name = ws.get("name")
            if isinstance(name, str):
                try:
                    return int(name)
                except ValueError:
                    return None
            if isinstance(name, int):
                return name
    return None


def focus_workspace(n: int) -> bool:
    """Focus workspace number `n` via `swaymsg workspace number N`."""
    rc, _, _ = _run(["workspace", "number", str(n)])
    return rc == 0


def set_layout(layout: str) -> bool:
    """Set the layout on the focused container. Valid values:
    `splith`, `splitv`, `tabbed`, `stacking`, `default`."""
    valid = {"splith", "splitv", "tabbed", "stacking", "default"}
    if layout not in valid:
        return False
    rc, _, _ = _run(["layout", layout])
    return rc == 0


def kill_focused() -> bool:
    """Kill the currently-focused window."""
    rc, _, _ = _run(["kill"])
    return rc == 0


def get_tree() -> Optional[dict]:
    """Return the parsed `get_tree` reply (the full sway tree as
    a nested dict). None on any failure."""
    rc, out, _ = _run(["-t", "get_tree"])
    if rc != 0:
        return None
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return None


def reload_config() -> bool:
    """`swaymsg reload` — re-reads ~/.config/sway/config + its
    include chain. Called by settings::keybinds after writing a
    fresh `mackes-bindings.conf`."""
    rc, _, _ = _run(["reload"])
    return rc == 0


__all__ = [
    "is_sway_running",
    "current_workspace", "focus_workspace",
    "set_layout", "kill_focused",
    "get_tree", "reload_config",
]
