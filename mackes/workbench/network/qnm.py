"""Network → Quick Network Mesh.

Thin proxy over `qnmctl` and the QNM GUI launcher; per the migration doc QNM
itself is unchanged.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib  # noqa: E402

from mackes import qnm_bridge
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


class QnmPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Quick Network Mesh"), False, False, 0)
        box.pack_start(info_label(
            "Run, stop, and check on the QNM background helper. "
            "QNM has its own window — Mackes just makes it easy to "
            "reach from here."
        ), False, False, 0)
        box.pack_start(section_description(
            "Most people won't need this panel. Open the QNM window "
            "below to actually use QNM."
        ), False, False, 0)

        if not qnm_bridge.have_qnm():
            box.pack_start(info_label("qnmctl not installed. Use Maintain → Dependencies "
                                     "to install QNM."), False, False, 0)
            self.add(box); return

        box.pack_start(section_header("Status"), False, False, 0)
        self._status = Gtk.TextView()
        self._status.set_editable(False); self._status.set_monospace(True)
        self._status.set_size_request(-1, 100)
        scroll = Gtk.ScrolledWindow(); scroll.add(self._status)
        scroll.set_size_request(-1, 100)
        box.pack_start(scroll, False, False, 0)

        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        for label, fn in [
            ("Start", qnm_bridge.start),
            ("Stop", qnm_bridge.stop),
            ("Restart", qnm_bridge.restart),
            ("Refresh status", lambda: ""),
        ]:
            b = Gtk.Button(label=label)
            def _on(_btn, f=fn):
                f()
                GLib.idle_add(self._refresh)
            b.connect("clicked", _on)
            actions.pack_start(b, False, False, 0)
        box.pack_start(actions, False, False, 0)

        box.pack_start(section_header("GUI"), False, False, 0)
        gui_btn = Gtk.Button(label="Open QNM GUI")
        def _on_gui(_):
            qnm_bridge.launch_gui()
        gui_btn.connect("clicked", _on_gui)
        box.pack_start(labeled_row("Launcher", gui_btn), False, False, 0)

        self.add(box)
        self._refresh()

    def _refresh(self) -> bool:
        s = qnm_bridge.status()
        text = s.get("raw") or "\n".join(f"{k}: {v}" for k, v in s.items())
        self._status.get_buffer().set_text(text or "(no output)")
        return False
