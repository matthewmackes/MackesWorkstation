"""Shell → Rofi Launcher (Q12 lock: preset picker only)."""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.shell_profiles import (
    apply_rofi, current_rofi_profile, list_rofi_profiles,
)
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_header, title_label,
)


class RofiPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Rofi Launcher"), False, False, 0)
        box.pack_start(info_label(
            "Pick a Rofi style. Rofi has no daemon, so the new style takes effect "
            "the next time you open the launcher."
        ), False, False, 0)

        box.pack_start(section_header("Style"), False, False, 0)
        profiles = list_rofi_profiles()
        if not profiles:
            box.pack_start(info_label("No Rofi profiles found."), False, False, 0)
            return box

        combo = Gtk.ComboBoxText()
        for p in profiles:
            combo.append_text(p)
        active = current_rofi_profile()
        if active in profiles:
            combo.set_active(profiles.index(active))
        else:
            combo.set_active(0)

        status = Gtk.Label(label=""); status.set_xalign(0)
        status.get_style_context().add_class("dim-label")

        def on_changed(c):
            chosen = c.get_active_text()
            if chosen:
                actions = apply_rofi(chosen)
                status.set_text(actions[-1] if actions else "")

        combo.connect("changed", on_changed)
        box.pack_start(labeled_row("Style", combo), False, False, 0)
        box.pack_start(status, False, False, 0)

        # Test button — open Rofi so the user can preview the change.
        test = Gtk.Button(label="Open Rofi to preview")
        def on_test(_):
            try:
                subprocess.Popen(["rofi", "-show", "drun"])
            except FileNotFoundError:
                status.set_text("rofi not installed")
        test.connect("clicked", on_test)
        box.pack_start(test, False, False, 0)
        return box
