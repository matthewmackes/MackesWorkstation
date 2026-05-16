"""Wizard screen 7 — Network."""
from __future__ import annotations

from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.workbench._common import info_label, labeled_row, section_header


FIREWALL_ZONES = ["FedoraWorkstation", "public", "home", "work", "trusted", "block"]


def build(ctx) -> Gtk.Widget:
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
    box.set_margin_top(28); box.set_margin_bottom(28)
    box.set_margin_start(40); box.set_margin_end(40)

    title = Gtk.Label(label="Network")
    title.set_xalign(0); title.get_style_context().add_class("title-1")
    box.pack_start(title, False, False, 0)

    box.pack_start(section_header("Quick Network Mesh"), False, False, 0)
    qnm = Gtk.Switch(); qnm.set_active(ctx.enable_qnm)
    qnm.connect("notify::active",
                lambda s, _g: setattr(ctx, "enable_qnm", s.get_active()))
    box.pack_start(labeled_row("Enable QNM", qnm), False, False, 0)
    box.pack_start(info_label("QNM is a standalone daemon Mackes proxies in the Network tab."),
                   False, False, 0)

    box.pack_start(section_header("Firewall"), False, False, 0)
    fw = Gtk.ComboBoxText()
    for z in FIREWALL_ZONES:
        fw.append_text(z)
    fw.set_active(FIREWALL_ZONES.index(ctx.firewall_zone)
                  if ctx.firewall_zone in FIREWALL_ZONES else 0)
    def on_zone(c):
        txt = c.get_active_text()
        if txt:
            ctx.firewall_zone = txt
    fw.connect("changed", on_zone)
    box.pack_start(labeled_row("Default zone", fw), False, False, 0)

    box.pack_start(section_header("VPN (optional)"), False, False, 0)
    path_label = Gtk.Label(label="(none)"); path_label.set_xalign(0)
    path_label.get_style_context().add_class("dim-label")
    import_btn = Gtk.Button(label="Import .ovpn / .conf …")
    def on_import(_):
        chooser = Gtk.FileChooserNative.new(
            "Import VPN config", None,
            Gtk.FileChooserAction.OPEN, "_Open", "_Cancel",
        )
        if chooser.run() == Gtk.ResponseType.ACCEPT:
            f = chooser.get_filename()
            if f and Path(f).exists():
                ctx.imported_vpn_path = f
                path_label.set_text(f)
        chooser.destroy()
    import_btn.connect("clicked", on_import)
    box.pack_start(labeled_row("Config", import_btn), False, False, 0)
    box.pack_start(path_label, False, False, 0)

    return box
