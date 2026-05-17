"""Devices → Keyboard."""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


class KeyboardPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Keyboard"), False, False, 0)
        box.pack_start(info_label(
            "Set how fast a held-down key repeats, switch between "
            "keyboard layouts, and check the global shortcuts you have."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Repeat"), False, False, 0)

        repeat_switch = Gtk.Switch()
        repeat_switch.set_active(bool(xf.get("keyboards", "/Default/KeyRepeat", True)))
        def on_repeat(s, _g):
            xf.set("keyboards", "/Default/KeyRepeat", s.get_active())
        repeat_switch.connect("notify::active", on_repeat)
        box.pack_start(labeled_row("Key repeat enabled", repeat_switch), False, False, 0)

        delay = Gtk.SpinButton.new_with_range(100, 2000, 50)
        delay.set_value(int(xf.get("keyboards", "/Default/KeyRepeat/Delay", 500) or 500))
        def on_delay(s):
            xf.set("keyboards", "/Default/KeyRepeat/Delay", int(s.get_value()))
        delay.connect("value-changed", on_delay)
        box.pack_start(labeled_row("Repeat delay (ms)", delay), False, False, 0)

        rate = Gtk.SpinButton.new_with_range(1, 100, 1)
        rate.set_value(int(xf.get("keyboards", "/Default/KeyRepeat/Rate", 25) or 25))
        def on_rate(s):
            xf.set("keyboards", "/Default/KeyRepeat/Rate", int(s.get_value()))
        rate.connect("value-changed", on_rate)
        box.pack_start(labeled_row("Repeat rate (chars/s)", rate), False, False, 0)

        box.pack_start(section_header("Layout"), False, False, 0)

        layout_entry = Gtk.Entry()
        layout_entry.set_text(str(xf.get("keyboard-layout", "/Default/XkbLayout", "us") or "us"))
        layout_entry.set_placeholder_text("us, gb, de, fr …")
        def on_layout(e):
            xf.set("keyboard-layout", "/Default/XkbLayout", e.get_text(), type_hint="string")
        layout_entry.connect("activate", on_layout)
        layout_entry.connect("focus-out-event", lambda e, _evt: (on_layout(e), False)[1])
        box.pack_start(labeled_row("XKB layout", layout_entry), False, False, 0)

        box.pack_start(section_header("Shortcuts"), False, False, 0)
        launch = Gtk.Button(label="Open xfce4-keyboard-settings (full shortcut editor)")
        def on_launch(_):
            subprocess.Popen(["xfce4-keyboard-settings"], stdout=subprocess.DEVNULL,
                             stderr=subprocess.DEVNULL)
        launch.connect("clicked", on_launch)
        box.pack_start(launch, False, False, 0)

        return box
