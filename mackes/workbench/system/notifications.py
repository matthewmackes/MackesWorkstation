"""System → Notifications (v2.0.0 Phase F.5 — switched to MDE settings bridge).

The v1.x panel exposed xfce4-notifyd-specific knobs (fade-out
animation, slide-out animation, primary-monitor-only, theme name).
The v2.0.0 notifications server (workers/notifications_server.rs) +
Iced applet replaces xfce4-notifyd; those animation + theme knobs
are now controlled by libcosmic theme tokens, not user toggles. So
this panel narrows to the two settings the MDE schema exposes:

  * notification.do_not_disturb — flag file at
    $XDG_CACHE_HOME/mde/notifications-dnd
  * notification.location — corner string in
    $XDG_CACHE_HOME/mde/notifications-prefs.json

Default-expire-ms is operator-tunable via the bridge too but isn't
surfaced in the panel today; the in-app per-notification expire
hint overrides it anyway. mde_settings_bridge.set_setting handles
the persistence.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as _b
from mackes.workbench._common import (
    a11y, info_label, labeled_row, panel_box, section_header, title_label,
)


LOCATION_LABELS = ["Top right", "Top left", "Bottom right", "Bottom left", "Center"]
LOCATION_VALUES = ["top-right", "top-left", "bottom-right", "bottom-left", "center"]


class NotificationsPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Notifications"), False, False, 0)
        box.pack_start(info_label(
            "Where pop-up notifications appear on screen and whether "
            "to suppress them while you're heads-down."
        ), False, False, 0)

        # ---- Placement (notification.location) ----------------
        box.pack_start(section_header("Placement"), False, False, 0)
        loc_combo = Gtk.ComboBoxText()
        for lbl in LOCATION_LABELS:
            loc_combo.append_text(lbl)
        current = _b.get_setting("notification.location") or "top-right"
        if current in LOCATION_VALUES:
            loc_combo.set_active(LOCATION_VALUES.index(current))
        else:
            loc_combo.set_active(0)

        def on_loc(c):
            i = c.get_active()
            if i >= 0:
                _b.set_setting("notification.location", LOCATION_VALUES[i])

        loc_combo.connect("changed", on_loc)
        a11y(loc_combo, name="Screen corner for notification pop-ups",
             tooltip="Which corner of the screen notifications appear in")
        box.pack_start(labeled_row("Corner", loc_combo), False, False, 0)

        # ---- DND (flag file under $XDG_CACHE_HOME/mde/) -------
        box.pack_start(section_header("Do Not Disturb"), False, False, 0)
        from mackes.mde_settings_bridge import sidecar_path

        dnd_switch = Gtk.Switch()
        dnd_path = sidecar_path("notifications-dnd")
        dnd_switch.set_active(dnd_path.exists())

        def on_dnd(s, _g):
            from pathlib import Path
            if s.get_active():
                dnd_path.parent.mkdir(parents=True, exist_ok=True)
                Path(dnd_path).write_text("")
            elif dnd_path.exists():
                dnd_path.unlink()

        dnd_switch.connect("notify::active", on_dnd)
        a11y(dnd_switch, name="Suppress all notifications",
             tooltip="Mute every notification until you turn this off")
        box.pack_start(labeled_row("Suppress all notifications", dnd_switch),
                       False, False, 0)

        # ---- Default expire timeout (notification.default_expire_ms)
        box.pack_start(section_header("Default duration"), False, False, 0)
        expire_spin = Gtk.SpinButton.new_with_range(-1, 60_000, 500)
        cur_expire = _b.get_setting("notification.default_expire_ms")
        expire_spin.set_value(int(cur_expire) if cur_expire is not None else -1)

        def on_expire(s):
            _b.set_setting("notification.default_expire_ms", int(s.get_value()))

        expire_spin.connect("value-changed", on_expire)
        a11y(expire_spin, name="Default notification duration in milliseconds",
             tooltip="How long notifications stick around when the caller "
                     "doesn't specify (-1 = follow the spec default)")
        box.pack_start(labeled_row("Default duration (ms; -1 = spec default)",
                                   expire_spin), False, False, 0)

        return box
