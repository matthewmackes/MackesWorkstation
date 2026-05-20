"""v2.0.0 Phase F.3 — Themes panel rewritten through `mde_settings_bridge`.

Replaces the GTK-theme + icon-theme + dark-variant subsections of the
legacy `appearance.py` panel with a small, focused panel that reads /
writes the MDE settings keys (`theme.name`, `theme.icon_set`,
`theme.mode`) — same keys the Rust appliers in
`crates/mackesd/src/settings/` honor.

Per the MDE schema:

  theme.name     → gsettings `gtk-theme`         (string)
  theme.icon_set → gsettings `icon-theme`        (string)
  theme.mode     → gsettings `color-scheme`      ("default" / "dark" / "light")
  theme.accent   → gsettings `accent-color`      (#RRGGBB, surfaced via appearance.py)

No xfconf reads / writes; no XfconfBridge import. The sub-millisecond
`gsettings_get` calls let us build the panel synchronously without an
`async_probe`. Discovery of installed themes still walks the standard
GTK locations (handled by helpers shared with `appearance.py`).
"""
from __future__ import annotations

import os
from pathlib import Path
from typing import List

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as bridge
from mackes.workbench._common import (
    error_label, labeled_row, panel_box, section_header, title_label,
)


# ---- theme discovery -------------------------------------------------------

_GTK_THEME_DIRS = (
    "/usr/share/themes",
    "/usr/local/share/themes",
    os.path.expanduser("~/.themes"),
    os.path.expanduser("~/.local/share/themes"),
)

_ICON_THEME_DIRS = (
    "/usr/share/icons",
    "/usr/local/share/icons",
    os.path.expanduser("~/.icons"),
    os.path.expanduser("~/.local/share/icons"),
)


def discover_gtk_themes() -> List[str]:
    """Names of installed GTK3 themes — every directory under any
    `themes/` root that ships `gtk-3.0/`."""
    seen: dict[str, None] = {}
    for root in _GTK_THEME_DIRS:
        if not os.path.isdir(root):
            continue
        for entry in sorted(os.listdir(root)):
            p = Path(root) / entry / "gtk-3.0"
            if p.is_dir():
                seen.setdefault(entry, None)
    return list(seen.keys())


def discover_icon_themes() -> List[str]:
    """Names of installed icon themes — every directory under any
    `icons/` root that ships `index.theme`."""
    seen: dict[str, None] = {}
    for root in _ICON_THEME_DIRS:
        if not os.path.isdir(root):
            continue
        for entry in sorted(os.listdir(root)):
            if (Path(root) / entry / "index.theme").is_file():
                seen.setdefault(entry, None)
    return list(seen.keys())


# ---- helpers --------------------------------------------------------------

def _combo_for(values: List[str], current: str) -> Gtk.ComboBoxText:
    combo = Gtk.ComboBoxText()
    for v in values:
        combo.append_text(v)
    if current in values:
        combo.set_active(values.index(current))
    elif values:
        combo.set_active(0)
    return combo


def _save_on_change(setting_key: str):
    def on_changed(combo: Gtk.ComboBoxText) -> None:
        text = combo.get_active_text()
        if text:
            bridge.set_setting(setting_key, text)
    return on_changed


# ---- panel ---------------------------------------------------------------

class ThemesPanel(Gtk.Box):
    """MDE Themes panel — three controls: GTK theme, icon theme, color
    mode. All three write through `mde_settings_bridge.set_setting`."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer = panel_box()
        outer.pack_start(title_label("Themes"), False, False, 0)

        gtk_themes = discover_gtk_themes()
        icon_themes = discover_icon_themes()

        if not gtk_themes:
            outer.pack_start(error_label("No GTK themes found"),
                             False, False, 0)
            self.pack_start(outer, True, True, 0)
            return

        # GTK theme.
        outer.pack_start(section_header("Widget theme"), False, False, 0)
        current = str(bridge.get_setting("theme.name") or "")
        combo = _combo_for(gtk_themes, current)
        combo.connect("changed", _save_on_change("theme.name"))
        outer.pack_start(labeled_row("GTK theme", combo), False, False, 0)

        # Icon theme.
        if icon_themes:
            outer.pack_start(section_header("Icons"), False, False, 0)
            current = str(bridge.get_setting("theme.icon_set") or "")
            combo = _combo_for(icon_themes, current)
            combo.connect("changed", _save_on_change("theme.icon_set"))
            outer.pack_start(labeled_row("Icon theme", combo), False, False, 0)

        # Color mode.
        outer.pack_start(section_header("Mode"), False, False, 0)
        modes = ["default", "light", "dark"]
        current = str(bridge.get_setting("theme.mode") or "default")
        combo = _combo_for(modes, current)
        combo.connect("changed", _save_on_change("theme.mode"))
        outer.pack_start(labeled_row("Color scheme", combo), False, False, 0)

        self.pack_start(outer, True, True, 0)
