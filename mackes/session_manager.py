"""Session-manager extension — chupre dotfiles applier + process supervisor.

C6.a/C6.b/C11 locks: the session manager owns three responsibilities.

  1. **Apply the chupre dotfiles bundle.** The bundle is shipped under
     `data/shell-profiles/chupre/<dir>/` (alacritty, gtk-3.0, gtk-4.0,
     polybar) and copied into `~/.config/<dir>/` at wizard time and on
     repair. i3/picom/sxhkd/nvim/networkmanager-dmenu are skipped (XFCE
     doesn't use them).
  2. **Supervise managed processes** — Polybar, Plank, dunst (notifications
     daemon that replaces xfce4-notifyd), picom (if installed). Status is
     live: green dot if PID exists, amber if installed but not running,
     red if installed-and-failed (non-zero exit code in last 30 s), grey
     if not installed.
  3. **Surface that state** to the Dashboard status strip and the System →
     Session panel's "Managed processes" section.

Lightweight: no background daemon. State is read on demand when a panel
asks. Process control uses `subprocess` + `pkill -x <name>`.
"""
from __future__ import annotations

import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from mackes.logging import log_action
from mackes.state import HOME


# ---------------------------------------------------------------------------
# Chupre dotfiles bundle
# ---------------------------------------------------------------------------


SHIPPED_CHUPRE_BUNDLE_DIRS = [
    Path("/usr/share/mackes-shell/data/shell-profiles/chupre"),
    Path(__file__).resolve().parent.parent / "data" / "shell-profiles" / "chupre",
]

# Subdirs we copy from the bundle into ~/.config/. Anything not in this list
# is skipped even if it appears in the bundle (e.g. i3, picom, sxhkd are
# i3wm-only and don't belong on XFCE).
APPLIED_BUNDLE_SUBDIRS = ("alacritty", "gtk-3.0", "gtk-4.0")


def _bundle_root() -> Optional[Path]:
    for root in SHIPPED_CHUPRE_BUNDLE_DIRS:
        if root.is_dir():
            return root
    return None


def apply_chupre_dotfiles() -> list[str]:
    """Copy XFCE-compatible chupre dotfile dirs into ~/.config/.

    Existing user content is overwritten — the caller (wizard apply) takes a
    snapshot beforehand. Polybar/Plank/Rofi are handled by `shell_profiles`,
    not here, so we don't touch ~/.config/polybar etc. (avoids two writers
    racing each other).
    """
    actions: list[str] = []
    bundle = _bundle_root()
    if bundle is None:
        actions.append("chupre bundle: not shipped; skipping")
        return actions
    actions.append(f"chupre bundle: source = {bundle}")
    for sub in APPLIED_BUNDLE_SUBDIRS:
        src = bundle / sub
        if not src.is_dir():
            actions.append(f"chupre bundle: {sub} not in bundle; skipping")
            continue
        dest = HOME / ".config" / sub
        try:
            if dest.exists():
                shutil.rmtree(dest)
            shutil.copytree(src, dest, symlinks=False)
            actions.append(f"chupre bundle: copied {sub} -> {dest}")
        except OSError as e:
            actions.append(f"chupre bundle: copy {sub} failed: {e}")
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# Managed process registry
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ManagedProcess:
    name: str                # display name + pgrep target
    binary: str              # PATH binary to check
    started_by_mackes: bool  # True if Mackes owns the autostart .desktop


MANAGED_PROCESSES: tuple[ManagedProcess, ...] = (
    ManagedProcess("polybar", "polybar", started_by_mackes=True),
    ManagedProcess("plank",   "plank",   started_by_mackes=True),
    ManagedProcess("dunst",   "dunst",   started_by_mackes=True),
    ManagedProcess("picom",   "picom",   started_by_mackes=False),
)


@dataclass
class ProcessStatus:
    name: str
    installed: bool
    running: bool
    pid: Optional[int]

    @property
    def state(self) -> str:
        """One of: ok / warn / missing.

        ok      = installed AND running
        warn    = installed AND not running
        missing = not installed
        """
        if not self.installed:
            return "missing"
        return "ok" if self.running else "warn"


def _pid_of(name: str) -> Optional[int]:
    try:
        out = subprocess.check_output(
            ["pgrep", "-x", name],
            text=True, stderr=subprocess.DEVNULL, timeout=2,
        ).strip().splitlines()
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError):
        return None
    if not out:
        return None
    try:
        return int(out[0])
    except ValueError:
        return None


def process_status() -> list[ProcessStatus]:
    """Snapshot of every managed process's current state. Cheap; safe to
    call from any panel's render path."""
    statuses: list[ProcessStatus] = []
    for proc in MANAGED_PROCESSES:
        installed = shutil.which(proc.binary) is not None
        pid = _pid_of(proc.name) if installed else None
        statuses.append(ProcessStatus(
            name=proc.name,
            installed=installed,
            running=pid is not None,
            pid=pid,
        ))
    return statuses


# ---------------------------------------------------------------------------
# Process control
# ---------------------------------------------------------------------------


def start_process(name: str) -> list[str]:
    """Start a managed process via its Mackes-owned launcher or the bare
    binary."""
    actions: list[str] = []
    if name == "polybar":
        from mackes.shell_profiles import POLYBAR_LAUNCHER
        if POLYBAR_LAUNCHER.exists():
            subprocess.Popen(["bash", str(POLYBAR_LAUNCHER)],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            actions.append("polybar: launched via mackes-polybar-launch.sh")
        else:
            actions.append("polybar: launcher missing — pick a profile to install it")
    elif shutil.which(name) is not None:
        try:
            subprocess.Popen([name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            actions.append(f"{name}: started")
        except OSError as e:
            actions.append(f"{name}: start failed: {e}")
    else:
        actions.append(f"{name}: not installed")
    for line in actions:
        log_action(line)
    return actions


def stop_process(name: str) -> list[str]:
    actions: list[str] = []
    rc = subprocess.call(["pkill", "-x", name])
    actions.append(f"{name}: pkill rc={rc}")
    log_action(actions[-1])
    return actions


def restart_process(name: str) -> list[str]:
    out = stop_process(name)
    # Brief settle (some processes hold sockets); cheap polling already
    # baked into the polybar launcher script.
    out.extend(start_process(name))
    return out
