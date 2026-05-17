"""System → Removable Media (thunar-volman xfconf channel).

Controls what happens when removable storage is plugged in, when a camera
is attached, when an audio CD is inserted, etc. All driven by the
`thunar-volman` xfconf channel.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


CHANNEL = "thunar-volman"


_SWITCHES: list[tuple[str, str, str]] = [
    ("Storage",  "/automount-drives/enabled",      "Auto-mount drives on connect"),
    ("Storage",  "/automount-media/enabled",       "Auto-mount removable media"),
    ("Storage",  "/autobrowse/enabled",            "Auto-browse new media in Thunar"),
    ("Storage",  "/autorun/enabled",               "Auto-run scripts on inserted media"),
    ("Devices",  "/autoipod/enabled",              "iPod / portable music player"),
    ("Devices",  "/autophoto/enabled",             "Auto-import photos from cameras"),
    ("Devices",  "/autoprinter/enabled",           "Configure printers on connect"),
    ("Devices",  "/autoscanner/enabled",           "Recognize scanners on connect"),
    ("Devices",  "/autoinput/enabled",             "Configure new input devices"),
    ("Devices",  "/autotablet/enabled",            "Configure graphics tablets"),
    ("Optical",  "/autoplay-audio-cd/enabled",     "Auto-play audio CDs"),
    ("Optical",  "/autoplay-video-cd/enabled",     "Auto-play video CDs"),
    ("Optical",  "/autoplay-dvd/enabled",          "Auto-play DVDs"),
]


class RemovablePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Removable Media"), False, False, 0)
        box.pack_start(info_label(
            "What your machine should do when you plug in a USB drive, "
            "camera, audio CD, or other removable device."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        current_section = None
        for section, key, label in _SWITCHES:
            if section != current_section:
                box.pack_start(section_header(section), False, False, 0)
                current_section = section
            sw = Gtk.Switch()
            xf.bind_switch(sw, CHANNEL, key, False)
            box.pack_start(labeled_row(label, sw), False, False, 0)

        return box
