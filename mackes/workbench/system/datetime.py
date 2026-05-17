"""System → Date & Time (timedatectl wrapper).

Shows current time, timezone, and NTP state. Lets the user pick a timezone
and toggle NTP. Setting the time manually is intentionally not exposed —
that's almost always wrong on a networked machine and falls through to
shell access if someone really needs it.
"""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


def _timedatectl(*args: str) -> str:
    try:
        return subprocess.check_output(["timedatectl", *args], text=True,
                                       stderr=subprocess.STDOUT, timeout=8).strip()
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
        return getattr(e, "output", "") or ""


def _status() -> dict[str, str]:
    out = _timedatectl("status")
    parsed: dict[str, str] = {"raw": out}
    for line in out.splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            parsed[k.strip().lower().replace(" ", "_")] = v.strip()
    return parsed


def _list_timezones() -> list[str]:
    # Timezone list is static across the life of a session; cache it
    # forever so re-opening this panel doesn't shell out to timedatectl
    # (which is fast but compounds with other panel-construct probes).
    from mackes.probe_cache import cached
    def _probe():
        out = _timedatectl("list-timezones")
        return [line for line in out.splitlines() if line.strip()]
    return cached("datetime.timezones", factory=_probe, ttl_s=None)


def _set_timezone(tz: str) -> str:
    msg = _timedatectl("set-timezone", tz)
    log_action(f"datetime: timezone -> {tz}")
    return msg


def _set_ntp(enable: bool) -> str:
    msg = _timedatectl("set-ntp", "true" if enable else "false")
    log_action(f"datetime: ntp -> {'on' if enable else 'off'}")
    return msg


class DateTimePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Date & Time"), False, False, 0)
        box.pack_start(info_label(
            "Pick your timezone and decide whether your machine should "
            "keep itself in sync with the internet."
        ), False, False, 0)
        box.pack_start(section_description(
            "If your clock looks wrong, the easiest fix is to turn on "
            "Network time below."
        ), False, False, 0)

        try:
            subprocess.check_output(["timedatectl", "--version"], stderr=subprocess.DEVNULL)
        except (FileNotFoundError, subprocess.CalledProcessError):
            box.pack_start(info_label("timedatectl not available — install systemd."),
                           False, False, 0)
            self.add(box); return

        st = _status()

        box.pack_start(section_header("Current"), False, False, 0)
        self._summary = Gtk.TextView()
        self._summary.set_editable(False); self._summary.set_monospace(True)
        self._summary.set_size_request(-1, 110)
        self._summary.get_buffer().set_text(st.get("raw", ""))
        scroll = Gtk.ScrolledWindow(); scroll.add(self._summary)
        scroll.set_size_request(-1, 110)
        box.pack_start(scroll, False, False, 0)

        refresh = Gtk.Button(label="Refresh")
        refresh.connect("clicked", lambda *_: self._refresh())
        box.pack_start(refresh, False, False, 0)

        box.pack_start(section_header("Network time"), False, False, 0)
        ntp_switch = Gtk.Switch()
        ntp_switch.set_active(st.get("ntp_service", "").lower() == "active"
                              or st.get("system_clock_synchronized", "").lower() == "yes")
        def on_ntp(s, _g):
            _set_ntp(s.get_active())
            GLib.idle_add(self._refresh)
        ntp_switch.connect("notify::active", on_ntp)
        box.pack_start(labeled_row("NTP enabled", ntp_switch), False, False, 0)

        box.pack_start(section_header("Timezone"), False, False, 0)
        zones = _list_timezones()
        combo = Gtk.ComboBoxText()
        combo.set_entry_text_column(0)
        for z in zones:
            combo.append_text(z)
        cur_tz = st.get("time_zone", "").split(" ", 1)[0] or "UTC"
        if cur_tz in zones:
            combo.set_active(zones.index(cur_tz))
        def on_tz(c):
            txt = c.get_active_text()
            if txt:
                _set_timezone(txt)
                GLib.idle_add(self._refresh)
        combo.connect("changed", on_tz)
        box.pack_start(labeled_row("Timezone", combo), False, False, 0)

        self.add(box)

    def _refresh(self) -> bool:
        st = _status()
        self._summary.get_buffer().set_text(st.get("raw", ""))
        return False
