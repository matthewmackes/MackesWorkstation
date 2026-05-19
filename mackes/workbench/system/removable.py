"""System → Removable Media (thunar-volman xfconf channel).

Controls what happens when removable storage is plugged in, when a camera
is attached, when an audio CD is inserted, etc. All driven by the
`thunar-volman` xfconf channel.

11.9 reliability sweep: 13× synchronous `xfconf-query --get` calls in
`__init__` (one per switch) summed to ~250 ms. The panel now renders the
switches immediately (default-off) and pulls the real boolean state via
`mackes.workbench._async.async_probe` — switches snap to their correct
position when the probe lands.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.workbench._async import async_probe
from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    a11y, error_label, info_label, labeled_row, panel_box, section_header, title_label,
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


def _gather_switch_states() -> dict[str, bool]:
    """Off-main-thread: one xfconf-query per switch in parallel-ish (the
    bridge is sync but each call is fast — running them on the probe
    thread keeps the GTK main loop free)."""
    try:
        xf = get_bridge()
    except XfconfError:
        return {}
    out: dict[str, bool] = {}
    for _section, key, _label in _SWITCHES:
        out[key] = bool(xf.get(CHANNEL, key, False))
    return out


class RemovablePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._switches: dict[str, Gtk.Switch] = {}
        self.add(self._build_skeleton())
        async_probe(_gather_switch_states, self._apply_state)

    def _build_skeleton(self) -> Gtk.Widget:
        """Sync — render every section + switch row but skip the
        xfconf bind. Switches default to off; `_apply_state` snaps them
        to the live xfconf value (and wires up `notify::active` then)."""
        box = panel_box()
        box.pack_start(title_label("Removable Media"), False, False, 0)
        box.pack_start(info_label(
            "What your machine should do when you plug in a USB drive, "
            "camera, audio CD, or other removable device."
        ), False, False, 0)

        try:
            get_bridge()  # verify xfconf-query is installed; cheap.
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        current_section = None
        for section, key, label in _SWITCHES:
            if section != current_section:
                box.pack_start(section_header(section), False, False, 0)
                current_section = section
            sw = Gtk.Switch()
            # Real value + signal binding land in `_apply_state` once
            # the probe completes — keeps the constructor sync-cheap.
            a11y(sw, name=f"Removable-media auto-action: {label}",
                 tooltip=f"Toggle the xfce4-volumed action {label}")
            self._switches[key] = sw
            box.pack_start(labeled_row(label, sw), False, False, 0)

        return box

    def _apply_state(self, states: dict[str, bool]) -> None:
        """Snap each switch to its xfconf value, then wire its writeback."""
        try:
            xf = get_bridge()
        except XfconfError:
            return
        for key, sw in self._switches.items():
            sw.set_active(states.get(key, False))

            def _on_active(s, _gp, _k=key):
                xf.set(CHANNEL, _k, s.get_active())

            sw.connect("notify::active", _on_active)
