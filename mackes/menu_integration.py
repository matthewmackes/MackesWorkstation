"""Hide xfce4-settings menu entries (Q19 lock).

Strategy: write a per-user `.desktop` override with `NoDisplay=true` for each
of the xfce4-settings .desktop entries we want hidden. This is the standard
XDG mechanism for hiding system desktop entries without modifying the system
file. The originals stay installed so xfsettingsd and any tool that calls
`xfce4-display-settings` etc. still work.

Mackes also installs its own top-level Settings entry (`mackes-shell.desktop`).
Repair/uninstall restores the original visibility by deleting the overrides.
"""
from __future__ import annotations

import shutil
from pathlib import Path

from mackes.logging import log_action
from mackes.state import HOME, CONFIG_DIR


# xfce4-settings ships these .desktop entries on a typical Fedora install. We
# hide them all when Mackes is the active control panel.
XFCE_SETTINGS_DESKTOPS = [
    "xfce-settings-manager.desktop",
    "xfce4-settings-manager.desktop",
    "xfce-display-settings.desktop",
    "xfce4-display-settings.desktop",
    "xfce-keyboard-settings.desktop",
    "xfce4-keyboard-settings.desktop",
    "xfce-mouse-settings.desktop",
    "xfce4-mouse-settings.desktop",
    "xfce-appearance-settings.desktop",
    "xfce4-appearance-settings.desktop",
    "xfwm4-settings.desktop",
    "xfwm4-tweaks-settings.desktop",
    "xfwm4-workspace-settings.desktop",
    "xfce4-session-settings.desktop",
    "xfce4-power-manager-settings.desktop",
    "xfce4-notifyd-config.desktop",
    "thunar-volman-settings.desktop",
    "xfce4-mime-settings.desktop",
    "xfce4-accessibility-settings.desktop",
]

USER_APPLICATIONS_DIR = HOME / ".local" / "share" / "applications"
OVERRIDES_BACKUP_DIR = CONFIG_DIR / "overrides"


def _system_desktop(name: str) -> Path | None:
    for root in (Path("/usr/share/applications"), Path("/usr/local/share/applications")):
        candidate = root / name
        if candidate.exists():
            return candidate
    return None


def _override_text() -> str:
    return (
        "[Desktop Entry]\n"
        "Hidden=true\n"
        "NoDisplay=true\n"
        "X-Mackes-Hidden=1\n"
    )


def hide_xfce_settings_entries() -> list[str]:
    USER_APPLICATIONS_DIR.mkdir(parents=True, exist_ok=True)
    OVERRIDES_BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    actions: list[str] = []
    for name in XFCE_SETTINGS_DESKTOPS:
        system_path = _system_desktop(name)
        if system_path is None:
            continue
        # Back up the system file's path (not contents — file stays in place)
        backup_marker = OVERRIDES_BACKUP_DIR / f"{name}.original-path"
        if not backup_marker.exists():
            backup_marker.write_text(str(system_path), encoding="utf-8")
        override = USER_APPLICATIONS_DIR / name
        if not override.exists() or "X-Mackes-Hidden" not in override.read_text(encoding="utf-8"):
            override.write_text(_override_text(), encoding="utf-8")
            actions.append(f"hidden: {name}")
    if not actions:
        actions.append("no xfce4-settings menu entries needed hiding")
    for line in actions:
        log_action(line)
    return actions


def restore_xfce_settings_entries() -> list[str]:
    actions: list[str] = []
    if not USER_APPLICATIONS_DIR.exists():
        return ["no overrides directory; nothing to restore"]
    for name in XFCE_SETTINGS_DESKTOPS:
        override = USER_APPLICATIONS_DIR / name
        if override.exists():
            try:
                content = override.read_text(encoding="utf-8")
            except OSError:
                content = ""
            if "X-Mackes-Hidden" in content:
                override.unlink()
                actions.append(f"restored: {name}")
        marker = OVERRIDES_BACKUP_DIR / f"{name}.original-path"
        if marker.exists():
            marker.unlink()
    if not actions:
        actions.append("no Mackes-hidden entries found")
    for line in actions:
        log_action(line)
    return actions


def install_mackes_menu_entry(source_desktop: Path) -> list[str]:
    """Copy the shipped mackes-shell.desktop into the user applications dir."""
    actions: list[str] = []
    USER_APPLICATIONS_DIR.mkdir(parents=True, exist_ok=True)
    target = USER_APPLICATIONS_DIR / "mackes-shell.desktop"
    if source_desktop.exists():
        shutil.copy2(source_desktop, target)
        actions.append(f"installed menu entry: {target}")
    else:
        actions.append(f"menu entry source missing: {source_desktop}")
    for line in actions:
        log_action(line)
    return actions
