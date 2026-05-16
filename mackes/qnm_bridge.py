"""Bridge to the Quick Network Mesh (QNM) daemon.

Per the migration doc (§1.2): QNM stays as its own binary tree. Mackes' Network
→ QNM panel is a thin proxy — it reads status from `qnmctl status`, exposes
start/stop/restart, and launches the QNM GUI. No QNM logic moves into Mackes.
"""
from __future__ import annotations

import shutil
import subprocess
from typing import Optional

from mackes.logging import log_action


QNMCTL = "qnmctl"
QNM_GUI_CANDIDATES = ["qnm-gui", "qnm-gui.sh"]


def have_qnm() -> bool:
    return shutil.which(QNMCTL) is not None


def status() -> dict[str, str]:
    """Return parsed status dict from `qnmctl status`. Empty dict if unavailable."""
    if not have_qnm():
        return {"installed": "no"}
    try:
        out = subprocess.check_output([QNMCTL, "status"], stderr=subprocess.STDOUT, text=True,
                                      timeout=5)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
        return {"installed": "yes", "error": str(e)}
    parsed: dict[str, str] = {"installed": "yes", "raw": out.strip()}
    for line in out.splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            parsed[k.strip().lower().replace(" ", "_")] = v.strip()
    return parsed


def _run(args: list[str]) -> str:
    if not have_qnm():
        return "qnmctl not installed"
    try:
        out = subprocess.check_output(args, stderr=subprocess.STDOUT, text=True, timeout=10)
        return out.strip() or "ok"
    except subprocess.CalledProcessError as e:
        return f"failed: {e.output.strip() if e.output else e}"
    except subprocess.TimeoutExpired:
        return "timeout"


def start() -> str:
    msg = _run([QNMCTL, "start"])
    log_action(f"qnm start: {msg}")
    return msg


def stop() -> str:
    msg = _run([QNMCTL, "stop"])
    log_action(f"qnm stop: {msg}")
    return msg


def restart() -> str:
    msg = _run([QNMCTL, "restart"])
    log_action(f"qnm restart: {msg}")
    return msg


def set_qnm_enabled(enabled: bool) -> list[str]:
    if not have_qnm():
        return ["qnm: qnmctl not installed; skipping"]
    if enabled:
        return [f"qnm enable: {start()}"]
    return [f"qnm disable: {stop()}"]


def gui_launcher() -> Optional[str]:
    for name in QNM_GUI_CANDIDATES:
        p = shutil.which(name)
        if p:
            return p
    return None


def launch_gui() -> str:
    p = gui_launcher()
    if p is None:
        return "QNM GUI not installed"
    try:
        subprocess.Popen([p], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        log_action(f"qnm gui launched: {p}")
        return "launched"
    except OSError as e:
        return f"failed to launch: {e}"
