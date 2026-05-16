"""Shell → Polybar Editor.

Theme picker + geometry knobs + 3-zone module editor + live debounced apply.
Builds on the vendored adi1090x catalog (`mackes.polybar_catalog`) and the
pure-function generator (`mackes.polybar_gen`).

Modules can be added (from a popover listing every module the active family
defines), removed, and reordered within a zone via up/down buttons. Moving
across zones = delete from source, add in destination.

Full GTK3 drag-and-drop is deferred to a polish pass — up/down buttons
deliver the same outcome with a tenth of the code.
"""
from __future__ import annotations

from typing import Optional

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib, Gdk  # noqa: E402

from mackes import polybar_catalog as cat
from mackes import polybar_gen as gen
from mackes.shell_profiles import apply_polybar_text, save_polybar_profile
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_header, title_label,
)


_APPLY_DEBOUNCE_MS = 300
_ZONES = ("left", "center", "right")
_DND_TARGET = "application/x-mackes-polybar-module"
_DND_TARGETS = [Gtk.TargetEntry.new(_DND_TARGET, Gtk.TargetFlags.SAME_APP, 0)]


class _ZoneList(Gtk.Box):
    """One of the three module zones (modules-left / -center / -right).

    Rows are drag-and-drop sources AND the listbox is a drop target. Drop
    data format: f"{source_zone_key}|{index}". Cross-zone moves are
    coordinated by the parent panel's `_move_module` callback.
    """

    def __init__(self, zone_key: str, on_change, on_cross_zone_pluck) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        self.zone_key = zone_key
        self._on_change = on_change
        # Called by other zones when they want to pluck a module out of THIS
        # zone during a cross-zone drag. Signature: (idx) -> module_name | None.
        self._on_cross_zone_pluck = on_cross_zone_pluck
        self._modules: list[str] = []

        hdr = Gtk.Label(label=zone_key.upper())
        hdr.set_xalign(0)
        hdr.get_style_context().add_class("mackes-section-header")
        self.pack_start(hdr, False, False, 0)

        self._listbox = Gtk.ListBox()
        self._listbox.set_selection_mode(Gtk.SelectionMode.NONE)
        self._listbox.get_style_context().add_class("frame")
        self._listbox.drag_dest_set(
            Gtk.DestDefaults.ALL, _DND_TARGETS, Gdk.DragAction.MOVE,
        )
        self._listbox.connect("drag-data-received", self._on_drag_received)
        self.pack_start(self._listbox, False, False, 0)

        self._add_btn = Gtk.Button.new_from_icon_name("list-add-symbolic", Gtk.IconSize.BUTTON)
        self._add_btn.set_label(" Add module")
        self._add_btn.set_always_show_image(True)
        self._add_btn.set_halign(Gtk.Align.START)
        self.pack_start(self._add_btn, False, False, 0)

    # ---- Public API ----------------------------------------------------

    @property
    def modules(self) -> tuple[str, ...]:
        return tuple(self._modules)

    def set_modules(self, mods: tuple[str, ...]) -> None:
        self._modules = list(mods)
        self._rerender()

    def connect_add(self, callback) -> None:
        self._add_btn.connect("clicked", callback)

    def append(self, module_name: str) -> None:
        self._modules.append(module_name)
        self._rerender()
        self._on_change()

    def pluck(self, idx: int) -> str | None:
        """Remove the module at idx and return its name (for cross-zone DnD)."""
        if 0 <= idx < len(self._modules):
            name = self._modules.pop(idx)
            self._rerender()
            self._on_change()
            return name
        return None

    # ---- Rendering -----------------------------------------------------

    def _rerender(self) -> None:
        for child in list(self._listbox.get_children()):
            self._listbox.remove(child)
        for idx, mod in enumerate(self._modules):
            self._listbox.add(self._build_row(idx, mod))
        self._listbox.show_all()

    def _build_row(self, idx: int, name: str) -> Gtk.ListBoxRow:
        row = Gtk.ListBoxRow()
        h = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        h.set_margin_top(4); h.set_margin_bottom(4)
        h.set_margin_start(8); h.set_margin_end(4)

        handle = Gtk.Image.new_from_icon_name("view-list-symbolic", Gtk.IconSize.BUTTON)
        handle.set_tooltip_text("Drag to reorder")
        h.pack_start(handle, False, False, 0)

        lbl = Gtk.Label(label=name)
        lbl.set_xalign(0)
        h.pack_start(lbl, True, True, 0)

        rm = Gtk.Button.new_from_icon_name("edit-delete-symbolic", Gtk.IconSize.BUTTON)
        rm.set_relief(Gtk.ReliefStyle.NONE)
        rm.set_tooltip_text("Remove")
        rm.connect("clicked", lambda *_: self._remove_at(idx))
        h.pack_start(rm, False, False, 0)

        row.add(h)

        # Make the row draggable. The drag payload is "<source_zone>|<index>".
        row.drag_source_set(
            Gdk.ModifierType.BUTTON1_MASK, _DND_TARGETS, Gdk.DragAction.MOVE,
        )
        row.connect("drag-data-get", self._on_drag_data_get, idx)
        row.connect("drag-begin", lambda w, ctx, _i=idx: Gtk.drag_set_icon_name(
            ctx, "view-list-symbolic", 8, 8,
        ))
        return row

    def _remove_at(self, idx: int) -> None:
        if 0 <= idx < len(self._modules):
            self._modules.pop(idx)
            self._rerender()
            self._on_change()

    # ---- Drag-and-drop callbacks ---------------------------------------

    def _on_drag_data_get(self, _widget, _ctx, sel_data, _info, _time, idx):
        sel_data.set(sel_data.get_target(), 8,
                     f"{self.zone_key}|{idx}".encode("utf-8"))

    def _on_drag_received(self, _widget, _ctx, x, y, sel_data, _info, _time):
        payload = sel_data.get_data().decode("utf-8")
        try:
            source_zone, src_idx_str = payload.split("|", 1)
            src_idx = int(src_idx_str)
        except ValueError:
            return
        # Compute the destination index from drop coordinates.
        dest_row = self._listbox.get_row_at_y(y)
        dest_idx = dest_row.get_index() if dest_row is not None else len(self._modules)

        if source_zone == self.zone_key:
            # Same-zone reorder
            if src_idx == dest_idx:
                return
            mod = self._modules.pop(src_idx)
            # Account for the pop shifting indices when moving down
            if dest_idx > src_idx:
                dest_idx -= 1
            self._modules.insert(dest_idx, mod)
        else:
            # Cross-zone: pluck from the source zone, insert here
            mod = self._on_cross_zone_pluck(source_zone, src_idx)
            if mod is None:
                return
            self._modules.insert(dest_idx, mod)
        self._rerender()
        self._on_change()


class PolybarPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._families = cat.list_families()
        self._apply_pending_id: Optional[int] = None
        self._zones: dict[str, _ZoneList] = {}
        self._use_family_layout = True
        self.add(self._build())
        self._reload_family_defaults()

    # ---- UI -------------------------------------------------------------

    def _build(self) -> Gtk.Widget:
        outer = Gtk.ScrolledWindow()
        outer.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)

        box = panel_box()
        box.pack_start(title_label("Polybar Editor"), False, False, 0)
        box.pack_start(info_label(
            "Pick a theme family, geometry, and per-zone module set. Changes "
            "apply live (~300 ms debounce) — polybar relaunches with the "
            "generated config."
        ), False, False, 0)

        if not self._families:
            box.pack_start(info_label(
                "No vendored themes found. Expected upstream tree at "
                "data/shell-profiles/polybar/upstream/."
            ), False, False, 0)
            outer.add(box)
            return outer

        # --- Family selector -----------------------------------------------
        box.pack_start(section_header("Theme"), False, False, 0)
        self._family_combo = Gtk.ComboBoxText()
        for f in self._families:
            self._family_combo.append(f.key, f.key)
        self._family_combo.set_active(0)
        self._family_combo.connect("changed", self._on_family_changed)
        box.pack_start(labeled_row("Family", self._family_combo), False, False, 0)

        # --- Geometry ------------------------------------------------------
        box.pack_start(section_header("Geometry"), False, False, 0)
        self._position_combo = Gtk.ComboBoxText()
        for pos in ("top", "bottom"):
            self._position_combo.append(pos, pos.title())
        self._position_combo.set_active_id("top")
        self._position_combo.connect("changed", self._on_changed)
        box.pack_start(labeled_row("Position", self._position_combo), False, False, 0)

        self._height_spin = Gtk.SpinButton.new_with_range(16, 64, 1)
        self._height_spin.set_value(32)
        self._height_spin.connect("value-changed", self._on_changed)
        box.pack_start(labeled_row("Height (px)", self._height_spin), False, False, 0)

        self._radius_spin = Gtk.SpinButton.new_with_range(0, 20, 1)
        self._radius_spin.set_value(0)
        self._radius_spin.connect("value-changed", self._on_changed)
        box.pack_start(labeled_row("Corner radius", self._radius_spin), False, False, 0)

        # --- Modules -------------------------------------------------------
        box.pack_start(section_header("Modules"), False, False, 0)

        self._use_default_check = Gtk.CheckButton(label="Use the family's default bar layout")
        self._use_default_check.set_active(True)
        self._use_default_check.connect("toggled", self._on_use_default_toggled)
        box.pack_start(self._use_default_check, False, False, 0)

        # 3-column zone display
        zones_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        zones_row.set_homogeneous(True)
        for z in _ZONES:
            zl = _ZoneList(
                z,
                on_change=self._on_changed,
                on_cross_zone_pluck=self._pluck_from_zone,
            )
            zl.connect_add(lambda btn, zone=z: self._open_add_popover(btn, zone))
            self._zones[z] = zl
            zones_row.pack_start(zl, True, True, 0)
        box.pack_start(zones_row, False, False, 0)

        # --- Status --------------------------------------------------------
        self._status = Gtk.Label(label="")
        self._status.set_xalign(0)
        self._status.get_style_context().add_class("dim-label")
        box.pack_start(self._status, False, False, 0)

        apply_btn = Gtk.Button(label="Apply now")
        apply_btn.connect("clicked", lambda *_: self._apply_immediately())
        save_btn = Gtk.Button(label="Save as profile…")
        save_btn.connect("clicked", lambda *_: self._save_as_profile())
        copy_btn = Gtk.Button(label="Copy generated config")
        copy_btn.connect("clicked", lambda *_: self._copy_to_clipboard())
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        row.pack_start(apply_btn, False, False, 0)
        row.pack_start(save_btn, False, False, 0)
        row.pack_start(copy_btn, False, False, 0)
        box.pack_start(row, False, False, 0)

        # --- Show-generated disclosure ------------------------------------
        self._gen_view = Gtk.TextView()
        self._gen_view.set_editable(False); self._gen_view.set_monospace(True)
        self._gen_view.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        gen_scroll = Gtk.ScrolledWindow(); gen_scroll.add(self._gen_view)
        gen_scroll.set_size_request(-1, 240)
        expander = Gtk.Expander(label="Show generated config")
        expander.add(gen_scroll)
        expander.connect("notify::expanded", lambda *_: self._refresh_gen_view())
        box.pack_start(expander, False, False, 0)
        self._gen_expander = expander

        outer.add(box)
        return outer

    # ---- Module-add popover --------------------------------------------

    def _open_add_popover(self, button: Gtk.Button, zone: str) -> None:
        fam = cat.get_family(self._family_combo.get_active_id() or self._families[0].key)
        if fam is None:
            return
        mods = sorted({m.name for m in cat.list_modules(fam)})

        popover = Gtk.Popover.new(button)
        popover.set_position(Gtk.PositionType.BOTTOM)

        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroll.set_min_content_height(280)
        scroll.set_min_content_width(220)

        listbox = Gtk.ListBox()
        for name in mods:
            r = Gtk.ListBoxRow()
            lbl = Gtk.Label(label=name)
            lbl.set_xalign(0)
            lbl.set_margin_top(6); lbl.set_margin_bottom(6)
            lbl.set_margin_start(12); lbl.set_margin_end(12)
            r.add(lbl)
            listbox.add(r)

        def on_activated(_box, row):
            idx = row.get_index()
            self._zones[zone].append(mods[idx])
            self._use_default_check.set_active(False)  # toggling to custom mode
            popover.popdown()

        listbox.connect("row-activated", on_activated)
        scroll.add(listbox)
        popover.add(scroll)
        popover.show_all()
        popover.popup()

    # ---- Family / default-layout reloading -----------------------------

    def _on_family_changed(self, *_args) -> None:
        if self._use_family_layout:
            self._reload_family_defaults()
        self._on_changed()

    def _on_use_default_toggled(self, btn) -> None:
        self._use_family_layout = btn.get_active()
        if self._use_family_layout:
            self._reload_family_defaults()
        self._on_changed()

    def _pluck_from_zone(self, source_zone: str, idx: int) -> str | None:
        zl = self._zones.get(source_zone)
        return zl.pluck(idx) if zl is not None else None

    def _reload_family_defaults(self) -> None:
        key = self._family_combo.get_active_id() or self._families[0].key
        fam = cat.get_family(key)
        if fam is None:
            return
        layout = cat.bar_layout(fam)
        self._zones["left"].set_modules(layout.modules_left)
        self._zones["center"].set_modules(layout.modules_center)
        self._zones["right"].set_modules(layout.modules_right)

    # ---- Apply / debounce ----------------------------------------------

    def _current_opts(self) -> gen.GenOptions:
        return gen.GenOptions(
            family_key=self._family_combo.get_active_id() or self._families[0].key,
            position=self._position_combo.get_active_id() or "top",
            height=int(self._height_spin.get_value()),
            radius=int(self._radius_spin.get_value()),
            modules_left=self._zones["left"].modules,
            modules_center=self._zones["center"].modules,
            modules_right=self._zones["right"].modules,
            use_family_layout=self._use_family_layout,
        )

    def _on_changed(self, *_args) -> None:
        if self._apply_pending_id is not None:
            GLib.source_remove(self._apply_pending_id)
        self._apply_pending_id = GLib.timeout_add(_APPLY_DEBOUNCE_MS, self._apply_immediately)
        self._status.set_text("pending apply…")

    def _apply_immediately(self) -> bool:
        self._apply_pending_id = None
        try:
            opts = self._current_opts()
            text = gen.generate(opts)
            self._last_generated = text
            actions = apply_polybar_text(text, marker=opts.family_key.replace("/", "-"))
            self._status.set_text(actions[-1] if actions else "applied")
            if getattr(self, "_gen_expander", None) is not None and self._gen_expander.get_expanded():
                self._refresh_gen_view()
        except Exception as e:  # noqa: BLE001 — surface to the user verbatim
            self._status.set_text(f"error: {e}")
        return False

    def _refresh_gen_view(self) -> None:
        text = getattr(self, "_last_generated", None)
        if text is None:
            try:
                text = gen.generate(self._current_opts())
            except Exception as e:  # noqa: BLE001
                text = f";; generation failed: {e}"
        self._gen_view.get_buffer().set_text(text)

    def _save_as_profile(self) -> None:
        parent = self.get_toplevel() if isinstance(self.get_toplevel(), Gtk.Window) else None
        dlg = Gtk.Dialog(title="Save polybar profile", transient_for=parent, modal=True)
        dlg.add_buttons("Cancel", Gtk.ResponseType.CANCEL, "Save", Gtk.ResponseType.OK)
        dlg.set_default_response(Gtk.ResponseType.OK)
        content = dlg.get_content_area()
        content.set_margin_top(12); content.set_margin_bottom(12)
        content.set_margin_start(16); content.set_margin_end(16)
        content.set_spacing(8)
        lbl = Gtk.Label(label="Name for this profile (filename slug):")
        lbl.set_xalign(0); content.pack_start(lbl, False, False, 0)
        entry = Gtk.Entry()
        opts = self._current_opts()
        entry.set_text(opts.family_key.split("/")[-1] + "-custom")
        entry.set_activates_default(True)
        content.pack_start(entry, False, False, 0)
        dlg.show_all()
        resp = dlg.run()
        slug = entry.get_text().strip()
        dlg.destroy()
        if resp != Gtk.ResponseType.OK or not slug:
            return
        try:
            text = gen.generate(opts)
            path = save_polybar_profile(slug, text)
            self._status.set_text(f"saved profile -> {path}")
        except Exception as e:  # noqa: BLE001
            self._status.set_text(f"save failed: {e}")

    def _copy_to_clipboard(self) -> None:
        from gi.repository import Gdk
        try:
            text = gen.generate(self._current_opts())
        except Exception as e:  # noqa: BLE001
            self._status.set_text(f"copy: generation failed: {e}")
            return
        clip = Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD)
        clip.set_text(text, -1)
        self._status.set_text(f"copied generated config ({len(text)} bytes) to clipboard")
