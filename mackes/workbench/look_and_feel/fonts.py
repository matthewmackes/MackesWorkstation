"""v2.0.0 Phase F.3 — Fonts panel rewritten through `mde_settings_bridge`.

Replaces the font-selection subsection of the legacy `appearance.py`
panel. Writes the MDE settings keys (`font.name`, `font.monospace`,
`font.hinting`, `font.antialias`) — same keys the Rust appliers in
`crates/mackesd/src/settings/` honor.

Per the MDE schema:

  font.name      → gsettings `font-name`           (Pango FontDescription)
  font.monospace → gsettings `monospace-font-name` (Pango FontDescription)
  font.hinting   → gsettings `font-hinting`        ("none"|"slight"|"medium"|"full")
  font.antialias → gsettings `font-antialiasing`   ("none"|"grayscale"|"rgba")

No xfconf reads / writes; no XfconfBridge import.
"""
from __future__ import annotations

from typing import Optional

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as bridge
from mackes.workbench._common import (
    labeled_row, panel_box, section_header, title_label,
)


_HINTING_LEVELS = ("none", "slight", "medium", "full")
_AA_MODES = ("none", "grayscale", "rgba")


def _font_button(initial: Optional[str], setting_key: str) -> Gtk.FontButton:
    """A `FontButton` wired so changes persist via `set_setting`."""
    btn = Gtk.FontButton()
    if initial:
        btn.set_font_name(initial)
    btn.connect(
        "font-set",
        lambda b: bridge.set_setting(setting_key, b.get_font_name()),
    )
    return btn


def _enum_combo(
    values: tuple[str, ...],
    initial: str,
    setting_key: str,
) -> Gtk.ComboBoxText:
    combo = Gtk.ComboBoxText()
    for v in values:
        combo.append_text(v)
    if initial in values:
        combo.set_active(values.index(initial))
    else:
        combo.set_active(0)
    combo.connect(
        "changed",
        lambda c: bridge.set_setting(
            setting_key, c.get_active_text() or values[0]
        ),
    )
    return combo


class FontsPanel(Gtk.Box):
    """MDE Fonts panel — four controls: default font, monospace,
    hinting, antialiasing. Reads / writes `font.*` keys through the
    bridge."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer = panel_box()
        outer.pack_start(title_label("Fonts"), False, False, 0)

        # Default font.
        outer.pack_start(section_header("Default font"), False, False, 0)
        current = bridge.get_setting("font.name") or "Cantarell 11"
        outer.pack_start(
            labeled_row("Interface font",
                        _font_button(str(current), "font.name")),
            False, False, 0,
        )

        # Monospace font.
        outer.pack_start(section_header("Monospace"), False, False, 0)
        current = bridge.get_setting("font.monospace") or "Source Code Pro 11"
        outer.pack_start(
            labeled_row("Monospace font",
                        _font_button(str(current), "font.monospace")),
            False, False, 0,
        )

        # Hinting.
        outer.pack_start(section_header("Rendering"), False, False, 0)
        current = str(bridge.get_setting("font.hinting") or "slight")
        outer.pack_start(
            labeled_row("Hinting",
                        _enum_combo(_HINTING_LEVELS, current, "font.hinting")),
            False, False, 0,
        )
        current = str(bridge.get_setting("font.antialias") or "rgba")
        outer.pack_start(
            labeled_row("Antialiasing",
                        _enum_combo(_AA_MODES, current, "font.antialias")),
            False, False, 0,
        )

        self.pack_start(outer, True, True, 0)
