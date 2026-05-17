"""Devices → Mouse & Touchpad."""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


def _list_pointer_devices() -> list[str]:
    try:
        out = subprocess.check_output(["xinput", "--list", "--name-only"],
                                      text=True, stderr=subprocess.DEVNULL)
    except (FileNotFoundError, subprocess.CalledProcessError):
        return []
    return [n for n in out.splitlines() if n and not n.startswith("Virtual core")]


class MousePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Mouse & Touchpad"), False, False, 0)
        box.pack_start(info_label(
            "How fast your cursor moves, how forgiving double-click "
            "timing is, and whether to enable any extras on each "
            "pointing device."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Pointer"), False, False, 0)

        accel = Gtk.SpinButton.new_with_range(0.1, 10.0, 0.1)
        accel.set_digits(1)
        accel.set_value(float(xf.get("pointers", "/Default/Acceleration", 1.0) or 1.0))
        def on_accel(s):
            xf.set("pointers", "/Default/Acceleration", float(s.get_value()), type_hint="double")
        accel.connect("value-changed", on_accel)
        box.pack_start(labeled_row("Acceleration", accel), False, False, 0)

        threshold = Gtk.SpinButton.new_with_range(1, 100, 1)
        threshold.set_value(int(xf.get("pointers", "/Default/Threshold", 4) or 4))
        def on_threshold(s):
            xf.set("pointers", "/Default/Threshold", int(s.get_value()))
        threshold.connect("value-changed", on_threshold)
        box.pack_start(labeled_row("Threshold", threshold), False, False, 0)

        click = Gtk.SpinButton.new_with_range(100, 800, 50)
        click.set_value(int(xf.get("xsettings", "/Net/DoubleClickTime", 250) or 250))
        def on_click(s):
            xf.set("xsettings", "/Net/DoubleClickTime", int(s.get_value()))
        click.connect("value-changed", on_click)
        box.pack_start(labeled_row("Double-click time (ms)", click), False, False, 0)

        box.pack_start(section_header("Detected devices"), False, False, 0)
        devs = _list_pointer_devices()
        if devs:
            for name in devs:
                box.pack_start(info_label(f"• {name}"), False, False, 0)
        else:
            box.pack_start(info_label("No xinput devices detected (xinput not installed?)"),
                           False, False, 0)

        return box
