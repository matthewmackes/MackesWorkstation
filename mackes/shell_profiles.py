"""Polybar / Plank / Rofi profile apply (Q12 lock: preset picker only).

A profile is a config-file blob shipped under `data/shell-profiles/<tool>/`.
Applying = copy the blob into the live `~/.config/<tool>/`, then signal the
running daemon (or relaunch it). Mackes also owns a tiny launcher script for
Polybar at `~/.local/bin/mackes-polybar-launch.sh`, so profile switches reach
running instances cleanly.
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
from pathlib import Path
from typing import Optional

from mackes.logging import log_action
from mackes.state import CONFIG_DIR, HOME, LOG_DIR


# User-saved profiles live alongside the user's other XDG state. Searched
# *before* the shipped tree so a user-saved profile of the same name wins.
USER_PROFILE_DIR = CONFIG_DIR / "shell-profiles"

SHIPPED_PROFILE_DIRS = [
    USER_PROFILE_DIR,
    Path("/usr/share/mackes-shell/data/shell-profiles"),
    Path(__file__).resolve().parent.parent / "data" / "shell-profiles",
]


def _shipped(tool: str, name: str, suffixes: list[str]) -> Optional[Path]:
    for root in SHIPPED_PROFILE_DIRS:
        for suffix in suffixes:
            candidate = root / tool / f"{name}{suffix}"
            if candidate.exists():
                return candidate
    return None


def _list_profiles(tool: str, suffixes: list[str]) -> list[str]:
    seen: dict[str, None] = {}
    for root in SHIPPED_PROFILE_DIRS:
        d = root / tool
        if not d.is_dir():
            continue
        for entry in sorted(d.iterdir()):
            if any(entry.name.endswith(s) for s in suffixes):
                stem = entry.name
                for s in suffixes:
                    if stem.endswith(s):
                        stem = stem[: -len(s)]
                        break
                seen.setdefault(stem, None)
    return list(seen.keys())


def save_polybar_profile(name: str, text: str) -> Path:
    """Persist a generated polybar config under the user's profile dir.
    Returns the destination path. Caller is expected to apply separately."""
    dest_dir = USER_PROFILE_DIR / "polybar"
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest = dest_dir / f"{name}.ini"
    dest.write_text(text, encoding="utf-8")
    return dest


# ---------------------------------------------------------------------------
# Polybar
# ---------------------------------------------------------------------------


POLYBAR_DIR = HOME / ".config" / "polybar"
POLYBAR_LAUNCHER = HOME / ".local" / "bin" / "mackes-polybar-launch.sh"
POLYBAR_ACTIVE_MARKER = POLYBAR_DIR / ".mackes-active-profile"
POLYBAR_AUTOSTART = HOME / ".config" / "autostart" / "mackes-polybar.desktop"
POLYBAR_STDERR_LOG = LOG_DIR / "polybar.log"

# Regex matching `[bar/<name>]` section headers in a Polybar .ini.
_POLYBAR_BAR_RE = re.compile(r"^\[bar/([^\]]+)\]\s*$", re.MULTILINE)


def list_polybar_profiles() -> list[str]:
    return _list_profiles("polybar", [".ini"])


def current_polybar_profile() -> Optional[str]:
    if POLYBAR_ACTIVE_MARKER.exists():
        return POLYBAR_ACTIVE_MARKER.read_text(encoding="utf-8").strip() or None
    return None


def _detect_bar_names(config_path: Path) -> list[str]:
    """Parse all `[bar/<name>]` section headers from a Polybar config."""
    try:
        text = config_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    return _POLYBAR_BAR_RE.findall(text)


def _write_polybar_launcher(config_path: Path) -> None:
    POLYBAR_LAUNCHER.parent.mkdir(parents=True, exist_ok=True)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    bar_names = _detect_bar_names(config_path) or ["mackes"]
    bar_launch_lines = "\n".join(
        f'polybar --config="$HOME/.config/polybar/config.ini" {name} '
        f'>>"{POLYBAR_STDERR_LOG}" 2>&1 &'
        for name in bar_names
    )
    POLYBAR_LAUNCHER.write_text(
        "#!/usr/bin/env bash\n"
        "# Managed by Mackes Shell. Re-generated on Polybar profile switch.\n"
        f"# Bars detected in config: {', '.join(bar_names)}\n"
        "killall -q polybar\n"
        "while pgrep -x polybar >/dev/null; do sleep 0.2; done\n"
        f'mkdir -p "{POLYBAR_STDERR_LOG.parent}"\n'
        f'echo "--- $(date -Is) polybar launch ---" >>"{POLYBAR_STDERR_LOG}"\n'
        f"{bar_launch_lines}\n",
        encoding="utf-8",
    )
    os.chmod(POLYBAR_LAUNCHER, 0o755)


def _write_polybar_autostart() -> None:
    POLYBAR_AUTOSTART.parent.mkdir(parents=True, exist_ok=True)
    POLYBAR_AUTOSTART.write_text(
        "[Desktop Entry]\n"
        "Type=Application\n"
        "Name=Mackes Polybar\n"
        "Comment=Launch Polybar at session start (managed by Mackes Shell)\n"
        f"Exec=bash {POLYBAR_LAUNCHER}\n"
        "X-GNOME-Autostart-enabled=true\n"
        "X-Mackes-Managed=1\n",
        encoding="utf-8",
    )


def apply_polybar(profile: str) -> list[str]:
    src = _shipped("polybar", profile, [".ini"])
    actions: list[str] = []
    if src is None:
        actions.append(f"polybar: profile not found: {profile}")
        return actions
    POLYBAR_DIR.mkdir(parents=True, exist_ok=True)
    dest_config = POLYBAR_DIR / "config.ini"
    shutil.copy2(src, dest_config)
    POLYBAR_ACTIVE_MARKER.write_text(profile, encoding="utf-8")
    _write_polybar_launcher(dest_config)
    _write_polybar_autostart()
    actions.append(f"polybar: profile -> {profile}")
    return _finalize_polybar(dest_config, actions)


def apply_polybar_text(text: str, marker: str = "editor") -> list[str]:
    """Write a generated polybar config (string) to ~/.config/polybar/config.ini
    and relaunch. Used by the Polybar Editor; bypasses the shipped-profile
    lookup since the config is generated, not copied."""
    actions: list[str] = []
    POLYBAR_DIR.mkdir(parents=True, exist_ok=True)
    dest_config = POLYBAR_DIR / "config.ini"
    dest_config.write_text(text, encoding="utf-8")
    POLYBAR_ACTIVE_MARKER.write_text(marker, encoding="utf-8")
    _write_polybar_launcher(dest_config)
    _write_polybar_autostart()
    actions.append(f"polybar: wrote generated config ({len(text)} bytes) marker={marker}")
    return _finalize_polybar(dest_config, actions)


def _finalize_polybar(dest_config: Path, actions: list[str]) -> list[str]:
    bars = _detect_bar_names(dest_config)
    if bars:
        actions.append(f"polybar: bars detected -> {', '.join(bars)}")
    else:
        actions.append("polybar: no [bar/...] section found; falling back to 'mackes'")
    actions.append(f"polybar: autostart installed at {POLYBAR_AUTOSTART}")
    if shutil.which("polybar") is not None:
        try:
            subprocess.Popen(["bash", str(POLYBAR_LAUNCHER)],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            actions.append("polybar: relaunched")
        except OSError as e:
            actions.append(f"polybar: relaunch failed: {e}")
    else:
        actions.append("polybar: binary not installed (autostart will run after install)")
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# Plank
# ---------------------------------------------------------------------------
#
# Plank has three independent surfaces:
#   * Profile: a full ~/.config/plank/dock1/settings keyfile blob, copied
#     from data/shell-profiles/plank/<name>.dock. One profile sets every
#     dock-level option at once. Useful for "Audio Rig" vs "Workstation".
#   * Theme:   a directory under ~/.local/share/plank/themes/<Name>/dock.theme,
#     selected via the `theme` GSettings key. Themes only style the dock
#     (paddings, colors, rounding, animation timings). Mackes ships the
#     full erikdubois/plankthemes catalog and ensures-installed on first
#     use of the picker.
#   * Live keys: every other GSettings key under
#     net.launchpad.plank.docks.dock1 (position, icon-size, alignment,
#     hide-mode, pressure-reveal, …). Written live via `gsettings set`.
#
# Plank reads GSettings at startup AND watches for changes, so live writes
# take effect without a restart for almost every key.


PLANK_DIR = HOME / ".config" / "plank" / "dock1"
PLANK_ACTIVE_MARKER = HOME / ".config" / "plank" / ".mackes-active-profile"
PLANK_THEMES_INSTALL_DIR = HOME / ".local" / "share" / "plank" / "themes"
PLANK_GSETTINGS_SCHEMA = "net.launchpad.plank.docks.dock1"
PLANK_GSETTINGS_PATH = "/net/launchpad/plank/docks/dock1/"

SHIPPED_PLANK_THEME_DIRS = [
    Path("/usr/share/mackes-shell/data/plank-themes"),
    Path(__file__).resolve().parent.parent / "data" / "plank-themes",
]


def list_plank_profiles() -> list[str]:
    return _list_profiles("plank", [".dock", ".dockini"])


def current_plank_profile() -> Optional[str]:
    if PLANK_ACTIVE_MARKER.exists():
        return PLANK_ACTIVE_MARKER.read_text(encoding="utf-8").strip() or None
    return None


def apply_plank(profile: str) -> list[str]:
    src = _shipped("plank", profile, [".dock", ".dockini"])
    actions: list[str] = []
    if src is None:
        actions.append(f"plank: profile not found: {profile}")
        return actions
    PLANK_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, PLANK_DIR / "settings")
    PLANK_ACTIVE_MARKER.parent.mkdir(parents=True, exist_ok=True)
    PLANK_ACTIVE_MARKER.write_text(profile, encoding="utf-8")
    actions.append(f"plank: profile -> {profile}")
    if shutil.which("plank") is not None:
        subprocess.call(["killall", "-q", "plank"])
        try:
            subprocess.Popen(["plank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            actions.append("plank: relaunched")
        except OSError as e:
            actions.append(f"plank: relaunch failed: {e}")
    for line in actions:
        log_action(line)
    return actions


# ----- Themes ---------------------------------------------------------------


def shipped_plank_themes() -> list[Path]:
    """Theme directories Mackes ships (each contains a dock.theme file)."""
    seen: dict[str, Path] = {}
    for root in SHIPPED_PLANK_THEME_DIRS:
        if not root.is_dir():
            continue
        for entry in sorted(root.iterdir()):
            if entry.is_dir() and (entry / "dock.theme").exists():
                seen.setdefault(entry.name, entry)
    return list(seen.values())


def installed_plank_themes() -> list[str]:
    """Themes Plank can already see — built-ins and ~/.local/share/plank/themes/."""
    names: set[str] = {"Default", "Transparent", "Matte", "Gtk+"}  # plank built-ins
    if PLANK_THEMES_INSTALL_DIR.is_dir():
        for entry in PLANK_THEMES_INSTALL_DIR.iterdir():
            if entry.is_dir() and (entry / "dock.theme").exists():
                names.add(entry.name)
    return sorted(names)


def install_shipped_plank_themes() -> list[str]:
    """Copy every shipped theme directory into ~/.local/share/plank/themes/.
    Idempotent. Returns a log of what was installed/skipped."""
    actions: list[str] = []
    PLANK_THEMES_INSTALL_DIR.mkdir(parents=True, exist_ok=True)
    for src in shipped_plank_themes():
        dest = PLANK_THEMES_INSTALL_DIR / src.name
        try:
            if dest.exists():
                # Re-copy only if shipped dock.theme is newer
                src_mtime = (src / "dock.theme").stat().st_mtime
                dst_mtime = (dest / "dock.theme").stat().st_mtime if (dest / "dock.theme").exists() else 0
                if src_mtime <= dst_mtime:
                    continue
                shutil.rmtree(dest)
            shutil.copytree(src, dest, symlinks=True)
            actions.append(f"plank theme installed: {src.name}")
        except OSError as e:
            actions.append(f"plank theme failed {src.name}: {e}")
    if not actions:
        actions.append("plank themes: all up-to-date")
    for line in actions:
        log_action(line)
    return actions


def list_plank_themes() -> list[str]:
    """The union of installed and shipped (but-not-yet-installed) theme names.

    The Plank panel renders this list and quietly installs the shipped theme
    on selection if it isn't already on disk.
    """
    names: set[str] = set(installed_plank_themes())
    for src in shipped_plank_themes():
        names.add(src.name)
    return sorted(names)


# ----- gsettings live keys --------------------------------------------------


def _have_gsettings() -> bool:
    return shutil.which("gsettings") is not None


def gsettings_get(key: str) -> Optional[str]:
    if not _have_gsettings():
        return None
    try:
        out = subprocess.check_output(
            ["gsettings", "get", PLANK_GSETTINGS_SCHEMA, key],
            text=True, stderr=subprocess.DEVNULL, timeout=5,
        ).strip()
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return None
    # gsettings prints quoted strings and 'true'/'false' for booleans
    if out.startswith("'") and out.endswith("'"):
        return out[1:-1]
    return out


def gsettings_set(key: str, value: str) -> bool:
    if not _have_gsettings():
        return False
    try:
        subprocess.check_call(
            ["gsettings", "set", PLANK_GSETTINGS_SCHEMA, key, value],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5,
        )
        log_action(f"plank gsettings: {key} = {value}")
        return True
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return False


def apply_plank_theme(theme_name: str) -> list[str]:
    """Ensure the theme is installed under ~/.local/share/plank/themes/,
    then set the Plank GSettings `theme` key to its name."""
    actions: list[str] = []
    # If the theme is shipped but not yet installed, install it
    install_dir = PLANK_THEMES_INSTALL_DIR / theme_name
    if not install_dir.exists():
        src_root = None
        for root in SHIPPED_PLANK_THEME_DIRS:
            candidate = root / theme_name
            if candidate.is_dir() and (candidate / "dock.theme").exists():
                src_root = candidate
                break
        if src_root is not None:
            PLANK_THEMES_INSTALL_DIR.mkdir(parents=True, exist_ok=True)
            shutil.copytree(src_root, install_dir, symlinks=True)
            actions.append(f"plank theme installed: {theme_name}")
    if gsettings_set("theme", theme_name):
        actions.append(f"plank theme -> {theme_name}")
    else:
        actions.append(f"plank theme: gsettings unavailable (would set theme={theme_name})")
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# Rofi
# ---------------------------------------------------------------------------


ROFI_DIR = HOME / ".config" / "rofi"
ROFI_ACTIVE_MARKER = ROFI_DIR / ".mackes-active-profile"


def list_rofi_profiles() -> list[str]:
    return _list_profiles("rofi", [".rasi"])


def current_rofi_profile() -> Optional[str]:
    if ROFI_ACTIVE_MARKER.exists():
        return ROFI_ACTIVE_MARKER.read_text(encoding="utf-8").strip() or None
    return None


def apply_rofi(profile: str) -> list[str]:
    src = _shipped("rofi", profile, [".rasi"])
    actions: list[str] = []
    if src is None:
        actions.append(f"rofi: profile not found: {profile}")
        return actions
    ROFI_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, ROFI_DIR / "config.rasi")
    ROFI_ACTIVE_MARKER.write_text(profile, encoding="utf-8")
    actions.append(f"rofi: profile -> {profile}")
    log_action(actions[-1])
    return actions


# ---------------------------------------------------------------------------
# xfce4-panel autostart toggle (per-user .desktop override)
# ---------------------------------------------------------------------------


XFCE_PANEL_AUTOSTART = HOME / ".config" / "autostart" / "xfce4-panel.desktop"


def xfce_panel_enabled() -> bool:
    """True if xfce4-panel is allowed to autostart for this user.

    We treat 'no autostart override file or override Hidden=false' as enabled,
    and 'override file with Hidden=true' as disabled.
    """
    if not XFCE_PANEL_AUTOSTART.exists():
        return True
    try:
        for line in XFCE_PANEL_AUTOSTART.read_text(encoding="utf-8").splitlines():
            if line.strip().lower() == "hidden=true":
                return False
    except OSError:
        pass
    return True


def set_xfce_panel_enabled(enabled: bool) -> list[str]:
    actions: list[str] = []
    XFCE_PANEL_AUTOSTART.parent.mkdir(parents=True, exist_ok=True)
    if enabled:
        if XFCE_PANEL_AUTOSTART.exists():
            XFCE_PANEL_AUTOSTART.unlink()
        actions.append("xfce4-panel: autostart enabled")
    else:
        XFCE_PANEL_AUTOSTART.write_text(
            "[Desktop Entry]\n"
            "Type=Application\n"
            "Name=xfce4-panel (disabled by Mackes)\n"
            "Exec=true\n"
            "Hidden=true\n"
            "X-GNOME-Autostart-enabled=false\n",
            encoding="utf-8",
        )
        subprocess.call(["killall", "-q", "xfce4-panel"])
        actions.append("xfce4-panel: autostart disabled, running instance killed")
    for line in actions:
        log_action(line)
    return actions
