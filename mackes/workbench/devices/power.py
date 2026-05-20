"""Devices → Power (v2.0.0 Phase F.1 — switched from XfconfBridge to MDE
settings bridge).

Reads + writes:
  * Power profile → `powerprofilesctl get/set` (Phase C.4 applier
    also routes through this).
  * Lid-close action → `power.lid_action` sidecar key
    (`$XDG_CACHE_HOME/mde/power-prefs.json`); honored by
    mde-session's logind drop-in writer.
  * Idle-suspend timeout (battery) → `power.suspend_idle_battery_s`
    sidecar key (same file).

The lid-close per-power-source distinction (battery vs AC) collapses
to a single key in the MDE model — the locked Phase C.4 schema only
exposes one lid_action since modern logind handles AC vs battery via
a single drop-in. Idle-suspend keeps two keys (battery + AC) per the
schema.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as _b
from mackes.workbench._common import (
    a11y, info_label, labeled_row, panel_box, section_header, title_label,
)


PROFILES = ["performance", "balanced", "power-saver"]
LID_ACTIONS = ["nothing", "suspend", "hibernate", "poweroff"]
LID_LABELS = ["Do nothing", "Suspend", "Hibernate", "Shut down"]


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

        # ---- Profile (powerprofilesctl) -----------------------
        box.pack_start(section_header("Profile"), False, False, 0)
        profile_combo = Gtk.ComboBoxText()
        for p in PROFILES:
            profile_combo.append_text(p)
        current = _b.power_profile_get() or "balanced"
        if current in PROFILES:
            profile_combo.set_active(PROFILES.index(current))
        else:
            profile_combo.set_active(PROFILES.index("balanced"))

        def on_profile_changed(c):
            i = c.get_active()
            if i >= 0:
                _b.power_profile_set(PROFILES[i])

        profile_combo.connect("changed", on_profile_changed)
        a11y(profile_combo, name="Power profile",
             tooltip="Pick performance, balanced, or power-saver")
        box.pack_start(labeled_row("Power profile", profile_combo),
                       False, False, 0)

        # ---- Lid close action (MDE sidecar) -------------------
        box.pack_start(section_header("Lid close"), False, False, 0)
        lid_combo = Gtk.ComboBoxText()
        for lbl in LID_LABELS:
            lid_combo.append_text(lbl)
        cur_lid = _b.get_setting("power.lid_action") or "suspend"
        if cur_lid in LID_ACTIONS:
            lid_combo.set_active(LID_ACTIONS.index(cur_lid))
        else:
            lid_combo.set_active(LID_ACTIONS.index("suspend"))

        def on_lid_changed(c):
            i = c.get_active()
            if i >= 0:
                _b.set_setting("power.lid_action", LID_ACTIONS[i])

        lid_combo.connect("changed", on_lid_changed)
        a11y(lid_combo, name="Lid-close action",
             tooltip="What happens when the laptop lid closes")
        box.pack_start(labeled_row("On lid close", lid_combo),
                       False, False, 0)

        # ---- Idle suspend timeouts (MDE sidecar) -------------
        box.pack_start(section_header("Idle"), False, False, 0)
        suspend_bat = Gtk.SpinButton.new_with_range(0, 7200, 60)
        cur_bat = _b.get_setting("power.suspend_idle_battery_s") or 900
        suspend_bat.set_value(int(cur_bat))

        def on_bat(s):
            _b.set_setting("power.suspend_idle_battery_s", int(s.get_value()))

        suspend_bat.connect("value-changed", on_bat)
        a11y(suspend_bat, name="Idle-suspend timeout (battery, seconds)",
             tooltip="Seconds of inactivity before suspending on battery "
                     "— 0 disables (0–7200)")
        box.pack_start(labeled_row("Suspend after (battery, s)",
                                   suspend_bat), False, False, 0)

        suspend_ac = Gtk.SpinButton.new_with_range(0, 7200, 60)
        cur_ac = _b.get_setting("power.suspend_idle_ac_s") or 0
        suspend_ac.set_value(int(cur_ac))

        def on_ac(s):
            _b.set_setting("power.suspend_idle_ac_s", int(s.get_value()))

        suspend_ac.connect("value-changed", on_ac)
        a11y(suspend_ac, name="Idle-suspend timeout (AC power, seconds)",
             tooltip="Seconds of inactivity before suspending on AC "
                     "power — 0 disables (0–7200)")
        box.pack_start(labeled_row("Suspend after (AC, s)", suspend_ac),
                       False, False, 0)

        return box
