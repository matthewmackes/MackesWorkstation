"""Maintain → Snapshots.

GTK wrapper over the snapshots engine. List existing snapshots, create a new
one (with optional label), restore, delete. Q10 lock: manual snapshots only —
no auto-snapshotting; the user clicks the button.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.snapshots import create_snapshot, delete_snapshot, list_snapshots, restore_snapshot
from mackes.state import MackesState
from mackes.workbench._common import (
    info_label, panel_box, section_header, title_label,
)


class SnapshotsPanel(Gtk.Box):
    def __init__(self, state: MackesState) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.state = state
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Snapshots"), False, False, 0)
        box.pack_start(info_label(
            "Restore points capture your live config (xfconf + Polybar + Plank + "
            "Rofi + xfce4-panel) into a timestamped directory. Restore wipes the "
            "live config and replays the captured one. Take a snapshot before "
            "risky changes."
        ), False, False, 0)

        box.pack_start(section_header("Create"), False, False, 0)
        create_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self._label = Gtk.Entry()
        self._label.set_placeholder_text("Optional label — e.g. before-theme-swap")
        self._label.set_hexpand(True)
        create_row.pack_start(self._label, True, True, 0)
        create_btn = Gtk.Button(label="Create restore point")
        create_btn.get_style_context().add_class("suggested-action")
        create_btn.connect("clicked", lambda *_: self._on_create())
        create_row.pack_start(create_btn, False, False, 0)
        box.pack_start(create_row, False, False, 0)
        self._status = Gtk.Label(label=""); self._status.set_xalign(0)
        self._status.get_style_context().add_class("dim-label")
        box.pack_start(self._status, False, False, 0)

        box.pack_start(section_header("Existing"), False, False, 0)
        self._list_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._list_box, False, False, 0)

        self.add(box)
        self._refresh()

    # ----- handlers -------------------------------------------------------

    def _on_create(self) -> None:
        label = self._label.get_text().strip() or "snapshot"
        snap = create_snapshot(label=label, source_preset=self.state.active_preset)
        self._label.set_text("")
        self._status.set_text(f"Created: {snap.name}")
        self._refresh()

    def _on_restore(self, snap) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self.get_toplevel(), modal=True,
            message_type=Gtk.MessageType.WARNING, buttons=Gtk.ButtonsType.OK_CANCEL,
            text=f"Restore snapshot {snap.name}?",
        )
        dialog.format_secondary_text(
            "This wipes your current Polybar/Plank/Rofi/xfce4-panel config and "
            "replays the captured one. xfconf channels are reloaded. Continue?"
        )
        resp = dialog.run(); dialog.destroy()
        if resp != Gtk.ResponseType.OK:
            return
        actions = restore_snapshot(snap)
        self._status.set_text(actions[-1] if actions else f"Restored {snap.name}")
        self._refresh()

    def _on_delete(self, snap) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self.get_toplevel(), modal=True,
            message_type=Gtk.MessageType.QUESTION, buttons=Gtk.ButtonsType.OK_CANCEL,
            text=f"Delete snapshot {snap.name}?",
        )
        resp = dialog.run(); dialog.destroy()
        if resp != Gtk.ResponseType.OK:
            return
        delete_snapshot(snap)
        self._status.set_text(f"Deleted: {snap.name}")
        self._refresh()

    # ----- rendering ------------------------------------------------------

    def _refresh(self) -> bool:
        for child in list(self._list_box.get_children()):
            self._list_box.remove(child)
        snaps = list_snapshots()
        if not snaps:
            self._list_box.pack_start(info_label("No snapshots yet."), False, False, 0)
        for snap in snaps:
            mf = snap.manifest()
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            text = snap.display_label()
            if mf.get("source_preset"):
                text += f"  (from preset: {mf['source_preset']})"
            lbl = Gtk.Label(label=text)
            lbl.set_xalign(0); lbl.set_line_wrap(True)
            row.pack_start(lbl, True, True, 0)

            restore_btn = Gtk.Button(label="Restore")
            def _on_restore_click(_b, s=snap):
                self._on_restore(s)
            restore_btn.connect("clicked", _on_restore_click)

            del_btn = Gtk.Button(label="Delete")
            del_btn.get_style_context().add_class("destructive-action")
            def _on_delete_click(_b, s=snap):
                self._on_delete(s)
            del_btn.connect("clicked", _on_delete_click)

            row.pack_end(del_btn, False, False, 0)
            row.pack_end(restore_btn, False, False, 0)
            self._list_box.pack_start(row, False, False, 0)
        self._list_box.show_all()
        return False
