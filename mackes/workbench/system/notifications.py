"""System → Notifications (xfce4-notifyd xfconf channel)."""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    a11y, error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


CHANNEL = "xfce4-notifyd"

# xfce4-notifyd places notifications by integer corner index.
LOCATION_LABELS = ["Top right", "Bottom right", "Bottom left", "Top left"]
LOCATION_VALUES = [1, 2, 3, 4]


class NotificationsPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Notifications"), False, False, 0)
        box.pack_start(info_label(
            "Where pop-up notifications appear on your screen, how "
            "long they stick around, and how transparent they look."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Placement"), False, False, 0)
        loc_combo = Gtk.ComboBoxText()
        for lbl in LOCATION_LABELS:
            loc_combo.append_text(lbl)
        cur = int(xf.get(CHANNEL, "/notify-location", 1) or 1)
        loc_combo.set_active(LOCATION_VALUES.index(cur) if cur in LOCATION_VALUES else 0)
        def on_loc(c):
            i = c.get_active()
            if i >= 0:
                xf.set(CHANNEL, "/notify-location", LOCATION_VALUES[i])
        loc_combo.connect("changed", on_loc)
        a11y(loc_combo, name="Screen corner for notification pop-ups",
             tooltip="Which corner of the screen notifications appear in")
        box.pack_start(labeled_row("Corner", loc_combo), False, False, 0)

        box.pack_start(section_header("Behavior"), False, False, 0)

        do_fade = Gtk.Switch()
        do_fade.set_active(bool(xf.get(CHANNEL, "/do-fadeout", True)))
        do_fade.connect("notify::active",
                        lambda s, _g: xf.set(CHANNEL, "/do-fadeout", s.get_active()))
        a11y(do_fade, name="Fade notifications out when dismissing",
             tooltip="Animate notification dismissal with a fade effect")
        box.pack_start(labeled_row("Fade out", do_fade), False, False, 0)

        do_slideout = Gtk.Switch()
        do_slideout.set_active(bool(xf.get(CHANNEL, "/do-slideout", False)))
        do_slideout.connect("notify::active",
                            lambda s, _g: xf.set(CHANNEL, "/do-slideout", s.get_active()))
        a11y(do_slideout, name="Slide notifications out when dismissing",
             tooltip="Animate notification dismissal with a slide effect")
        box.pack_start(labeled_row("Slide out", do_slideout), False, False, 0)

        primary_only = Gtk.Switch()
        primary_only.set_active(bool(xf.get(CHANNEL, "/primary-monitor", False)))
        primary_only.connect("notify::active",
                             lambda s, _g: xf.set(CHANNEL, "/primary-monitor", s.get_active()))
        a11y(primary_only, name="Show notifications only on the primary monitor",
             tooltip="Restrict notification placement to the primary display")
        box.pack_start(labeled_row("Primary monitor only", primary_only), False, False, 0)

        notify_dnd = Gtk.Switch()
        notify_dnd.set_active(bool(xf.get(CHANNEL, "/applications/suppress-fullscreen", True)))
        notify_dnd.connect("notify::active",
                           lambda s, _g: xf.set(CHANNEL, "/applications/suppress-fullscreen",
                                                s.get_active()))
        a11y(notify_dnd,
             name="Suppress notifications while a fullscreen app is focused",
             tooltip="Auto-quiet notifications during movies, games, presentations")
        box.pack_start(labeled_row("Hide while a fullscreen window is focused",
                                   notify_dnd), False, False, 0)

        box.pack_start(section_header("Theme"), False, False, 0)
        theme_entry = Gtk.Entry()
        theme_entry.set_text(str(xf.get(CHANNEL, "/theme", "Default") or "Default"))
        def on_theme(e):
            xf.set(CHANNEL, "/theme", e.get_text(), type_hint="string")
        theme_entry.connect("activate", on_theme)
        theme_entry.connect("focus-out-event", lambda e, _evt: (on_theme(e), False)[1])
        a11y(theme_entry, name="Notification theme name",
             tooltip="xfce4-notifyd theme name — Default, Smoke, etc.")
        box.pack_start(labeled_row("Notification theme", theme_entry), False, False, 0)

        return box
