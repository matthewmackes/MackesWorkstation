"""System → Workspaces (xfwm4 workspace count + names)."""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


CHANNEL = "xfwm4"


class WorkspacesPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Workspaces"), False, False, 0)
        box.pack_start(info_label(
            "Virtual desktops let you spread out your windows. Choose "
            "how many you want and how to switch between them."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Count"), False, False, 0)
        count = Gtk.SpinButton.new_with_range(1, 16, 1)
        xf.bind_spin(count, CHANNEL, "/general/workspace_count", 4)
        box.pack_start(labeled_row("Workspaces", count), False, False, 0)

        box.pack_start(section_header("Behavior"), False, False, 0)

        wrap = Gtk.Switch()
        wrap.set_active(bool(xf.get(CHANNEL, "/general/wrap_workspaces", False)))
        def on_wrap(s, _g):
            xf.set(CHANNEL, "/general/wrap_workspaces", s.get_active())
        wrap.connect("notify::active", on_wrap)
        box.pack_start(labeled_row("Wrap around at edges", wrap), False, False, 0)

        cycle = Gtk.Switch()
        cycle.set_active(bool(xf.get(CHANNEL, "/general/cycle_workspaces", False)))
        def on_cycle(s, _g):
            xf.set(CHANNEL, "/general/cycle_workspaces", s.get_active())
        cycle.connect("notify::active", on_cycle)
        box.pack_start(labeled_row("Cycle when switching", cycle), False, False, 0)

        return box
