"""Network → VPN.

NetworkManager VPN connection list, plus an import button for .ovpn files.
WireGuard import is `nmcli connection import type wireguard file <path>`;
OpenVPN is the same with `type openvpn`.
"""
from __future__ import annotations

import subprocess
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import (
    info_label, panel_box, section_description, section_header, title_label,
)


def _nmcli(*args: str) -> str:
    try:
        return subprocess.check_output(["nmcli", *args], text=True, stderr=subprocess.STDOUT,
                                       timeout=8).strip()
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return ""


def _list_vpns() -> list[dict[str, str]]:
    raw = _nmcli("-t", "-f", "NAME,TYPE,DEVICE,STATE", "connection", "show")
    out = []
    for line in raw.splitlines():
        parts = line.split(":")
        if len(parts) >= 4 and parts[1] in ("vpn", "wireguard"):
            out.append({"name": parts[0], "type": parts[1], "device": parts[2], "state": parts[3]})
    return out


def _import_path(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".ovpn":
        vpn_type = "openvpn"
    elif suffix in (".conf", ".wg"):
        vpn_type = "wireguard"
    else:
        return f"unknown VPN file type: {suffix}"
    out = _nmcli("connection", "import", "type", vpn_type, "file", str(path))
    log_action(f"vpn import {path.name}: {out[:80]}")
    return out or f"imported {path.name}"


class VpnPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("VPN"), False, False, 0)
        box.pack_start(info_label(
            "Connect to a private network through a third-party VPN. "
            "Import a config file you got from your VPN provider, then "
            "switch it on or off here."
        ), False, False, 0)
        box.pack_start(section_description(
            "This is for commercial or work VPNs (OpenVPN, WireGuard). "
            "For Mackes' own mesh, use the Mesh VPN panel instead."
        ), False, False, 0)

        if not _nmcli("--version"):
            box.pack_start(info_label("nmcli not available."), False, False, 0)
            self.add(box); return

        box.pack_start(section_header("Configured VPNs"), False, False, 0)
        self._list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._list, False, False, 0)

        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        imp = Gtk.Button(label="Import .ovpn / .conf …")
        imp.connect("clicked", lambda *_: self._import_dialog())
        actions.pack_start(imp, False, False, 0)
        rfr = Gtk.Button(label="Refresh")
        rfr.connect("clicked", lambda *_: self._refresh())
        actions.pack_start(rfr, False, False, 0)
        box.pack_start(actions, False, False, 0)

        self.add(box)
        self._refresh()

    def _import_dialog(self) -> None:
        chooser = Gtk.FileChooserNative.new(
            "Import VPN config", self.get_toplevel(),
            Gtk.FileChooserAction.OPEN, "_Open", "_Cancel",
        )
        f = Gtk.FileFilter()
        f.set_name("VPN configs (.ovpn, .conf, .wg)")
        for p in ("*.ovpn", "*.conf", "*.wg"):
            f.add_pattern(p)
        chooser.add_filter(f)
        if chooser.run() == Gtk.ResponseType.ACCEPT:
            path = Path(chooser.get_filename() or "")
            if path.exists():
                _import_path(path)
                GLib.idle_add(self._refresh)
        chooser.destroy()

    def _refresh(self) -> bool:
        for child in list(self._list.get_children()):
            self._list.remove(child)
        vpns = _list_vpns()
        if not vpns:
            self._list.pack_start(info_label("No VPN connections configured."), False, False, 0)
        for v in vpns:
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            label = Gtk.Label(label=f"{v['name']}   [{v['type']}]   ({v['state'] or 'inactive'})")
            label.set_xalign(0)
            row.pack_start(label, True, True, 0)
            up = Gtk.Button(label="Up")
            down = Gtk.Button(label="Down")
            def _up(_b, name=v["name"]):
                _nmcli("connection", "up", name)
                log_action(f"vpn up: {name}")
                GLib.idle_add(self._refresh)
            def _down(_b, name=v["name"]):
                _nmcli("connection", "down", name)
                log_action(f"vpn down: {name}")
                GLib.idle_add(self._refresh)
            up.connect("clicked", _up); down.connect("clicked", _down)
            row.pack_end(down, False, False, 0)
            row.pack_end(up, False, False, 0)
            self._list.pack_start(row, False, False, 0)
        self._list.show_all()
        return False
