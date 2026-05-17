"""Network → Firewall (firewalld via firewall-cmd)."""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


def _fw(*args: str) -> str:
    try:
        return subprocess.check_output(["firewall-cmd", *args], text=True,
                                       stderr=subprocess.STDOUT, timeout=8).strip()
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
        return getattr(e, "output", "") or ""


def _zones() -> list[str]:
    out = _fw("--get-zones")
    return out.split() if out else []


def _default_zone() -> str:
    return _fw("--get-default-zone")


def _enabled_services() -> list[str]:
    out = _fw("--list-services")
    return out.split() if out else []


def _set_default_zone(zone: str) -> str:
    msg = _fw("--set-default-zone", zone)
    log_action(f"firewall: default zone -> {zone}")
    return msg


def _toggle_service(service: str, enable: bool) -> str:
    flag = "--add-service" if enable else "--remove-service"
    msg = _fw(flag, service, "--permanent")
    _fw("--reload")
    log_action(f"firewall: {'enabled' if enable else 'disabled'} {service}")
    return msg


COMMON_SERVICES = ["ssh", "http", "https", "samba", "samba-client", "mdns", "dhcpv6-client"]


class FirewallPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Firewall"), False, False, 0)
        box.pack_start(info_label(
            "Control what other computers are allowed to reach on this "
            "machine. The firewall blocks everything by default — flip "
            "on only the services you trust."
        ), False, False, 0)
        box.pack_start(section_description(
            "Changes need an admin password. If unsure, leave the "
            "default settings — they're safe for most home networks."
        ), False, False, 0)

        if not _fw("--version"):
            box.pack_start(info_label("firewall-cmd not available — install firewalld."),
                           False, False, 0)
            self.add(box); return

        box.pack_start(section_header("Default zone"), False, False, 0)
        zones = _zones() or ["public"]
        zone_combo = Gtk.ComboBoxText()
        for z in zones:
            zone_combo.append_text(z)
        cur = _default_zone()
        if cur in zones:
            zone_combo.set_active(zones.index(cur))
        else:
            zone_combo.set_active(0)
        def on_zone(c):
            txt = c.get_active_text()
            if txt:
                _set_default_zone(txt)
                GLib.idle_add(self._refresh_services)
        zone_combo.connect("changed", on_zone)
        box.pack_start(labeled_row("Active default zone", zone_combo), False, False, 0)

        box.pack_start(section_header("Enabled services (current zone)"), False, False, 0)
        self._service_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._service_box, False, False, 0)
        self._refresh_services()

        self.add(box)

    def _refresh_services(self) -> bool:
        for child in list(self._service_box.get_children()):
            self._service_box.remove(child)
        enabled = set(_enabled_services())
        for svc in COMMON_SERVICES:
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            lbl = Gtk.Label(label=svc); lbl.set_xalign(0); lbl.set_size_request(180, -1)
            row.pack_start(lbl, False, False, 0)
            sw = Gtk.Switch(); sw.set_active(svc in enabled)
            def _on(s, _g, name=svc):
                _toggle_service(name, s.get_active())
                GLib.idle_add(self._refresh_services)
            sw.connect("notify::active", _on)
            row.pack_start(sw, False, False, 0)
            self._service_box.pack_start(row, False, False, 0)
        self._service_box.show_all()
        return False
