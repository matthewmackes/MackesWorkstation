"""Devices → Power (xfce4-power-manager xfconf channel)."""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    a11y, error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


CHANNEL = "xfce4-power-manager"
PROFILES = ["performance", "balanced", "power-saver"]


class PowerPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Power"), False, False, 0)
        box.pack_start(info_label(
            "What should happen when you close the laptop lid or step "
            "away, and which power profile to keep."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Profile"), False, False, 0)
        profile_combo = Gtk.ComboBoxText()
        for p in PROFILES:
            profile_combo.append_text(p)
        xf.bind_combo(profile_combo, CHANNEL, "/xfce4-power-manager/power-profile",
                      PROFILES, "balanced")
        a11y(profile_combo, name="Power profile (XFCE power manager)",
             tooltip="Pick performance, balanced, or power-saver")
        box.pack_start(labeled_row("Power profile", profile_combo), False, False, 0)

        box.pack_start(section_header("Lid close"), False, False, 0)

        # 0 nothing, 1 suspend, 2 hibernate, 3 shutdown — xfce4-power-manager values
        LID_VALUES = ["0", "1", "2", "3"]
        LID_LABELS = ["Do nothing", "Suspend", "Hibernate", "Shut down"]
        lid_battery = Gtk.ComboBoxText()
        for lbl in LID_LABELS:
            lid_battery.append_text(lbl)
        cur = str(xf.get(CHANNEL, "/xfce4-power-manager/lid-action-on-battery", "1") or "1")
        lid_battery.set_active(LID_VALUES.index(cur) if cur in LID_VALUES else 1)
        def on_lid_battery(c):
            i = c.get_active()
            if i >= 0:
                xf.set(CHANNEL, "/xfce4-power-manager/lid-action-on-battery", int(LID_VALUES[i]))
        lid_battery.connect("changed", on_lid_battery)
        a11y(lid_battery, name="Lid-close action when on battery",
             tooltip="What happens when the laptop lid closes on battery power")
        box.pack_start(labeled_row("On battery", lid_battery), False, False, 0)

        lid_ac = Gtk.ComboBoxText()
        for lbl in LID_LABELS:
            lid_ac.append_text(lbl)
        cur = str(xf.get(CHANNEL, "/xfce4-power-manager/lid-action-on-ac", "0") or "0")
        lid_ac.set_active(LID_VALUES.index(cur) if cur in LID_VALUES else 0)
        def on_lid_ac(c):
            i = c.get_active()
            if i >= 0:
                xf.set(CHANNEL, "/xfce4-power-manager/lid-action-on-ac", int(LID_VALUES[i]))
        lid_ac.connect("changed", on_lid_ac)
        a11y(lid_ac, name="Lid-close action when on AC power",
             tooltip="What happens when the laptop lid closes while plugged in")
        box.pack_start(labeled_row("On AC power", lid_ac), False, False, 0)

        box.pack_start(section_header("Idle"), False, False, 0)
        suspend = Gtk.SpinButton.new_with_range(0, 120, 5)
        suspend.set_value(int(xf.get(CHANNEL, "/xfce4-power-manager/inactivity-on-battery", 15) or 15))
        def on_susp(s):
            xf.set(CHANNEL, "/xfce4-power-manager/inactivity-on-battery", int(s.get_value()))
        suspend.connect("value-changed", on_susp)
        a11y(suspend, name="Idle-suspend timeout in minutes (on battery)",
             tooltip="Minutes of inactivity before suspending — 0 disables (0–120)")
        box.pack_start(labeled_row("Suspend after (min)", suspend), False, False, 0)

        return box
