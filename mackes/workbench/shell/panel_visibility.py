"""Shell → XFCE Panel Visibility.

Toggle whether xfce4-panel autostarts. Mackes' shipped shell stack
(Polybar/Plank/Rofi) replaces it; some users want to flip back. The toggle
also kills the running instance when disabled.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.shell_profiles import set_xfce_panel_enabled, xfce_panel_enabled
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_header, title_label,
)


class PanelVisibilityPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("XFCE Panel Visibility"), False, False, 0)
        box.pack_start(info_label(
            "Mackes ships Polybar as the primary panel. xfce4-panel is hidden by default; "
            "flip this on if you want both, or to fall back temporarily."
        ), False, False, 0)

        box.pack_start(section_header("Autostart"), False, False, 0)

        switch = Gtk.Switch()
        switch.set_active(xfce_panel_enabled())
        status = Gtk.Label(label="")
        status.set_xalign(0); status.get_style_context().add_class("dim-label")

        def on_active(s, _gparam):
            actions = set_xfce_panel_enabled(s.get_active())
            status.set_text(actions[-1] if actions else "")

        switch.connect("notify::active", on_active)
        box.pack_start(labeled_row("xfce4-panel autostart", switch), False, False, 0)
        box.pack_start(status, False, False, 0)
        return box
