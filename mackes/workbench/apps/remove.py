"""Apps → Remove — curated bloat removal + Lean XFCE.

C4, C9, X1, X2 locks. Two sub-sections in one panel:
  • Fedora bloat — GNOME-on-XFCE apps + LibreOffice
  • XFCE components replaced by Mackes — visually separated, with the
    'replaced by' relationship called out per row.
"""
from __future__ import annotations

import threading

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.app_mgmt import (
    is_dnf_installed, remove_lean_xfce, remove_packages,
)
from mackes.presets import default_preset, load_preset
from mackes.session_manager import process_status
from mackes.state import MackesState
from mackes.workbench._common import (
    info_label, panel_box, section_header, title_label,
)


def _active_preset():
    state = MackesState.load()
    if state.active_preset:
        p = load_preset(state.active_preset)
        if p is not None:
            return p
    return default_preset()


class AppsRemovePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Remove apps"), False, False, 0)
        box.pack_start(info_label(
            "Two groups: Fedora Workstation bloat (GNOME-on-XFCE apps + "
            "LibreOffice) and XFCE components Mackes replaces. The XFCE "
            "components only show as removable when their replacement "
            "daemon is running — Mackes never leaves you panel-less."
        ), False, False, 0)

        # ----- Fedora bloat ------------------------------------------------
        box.pack_start(section_header("Fedora bloat"), False, False, 0)
        self._bloat_rows_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._bloat_rows_box, False, False, 0)
        self._bloat_checks: list[tuple[Gtk.CheckButton, str]] = []

        bloat_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        bloat_btn = Gtk.Button(label="Remove selected bloat")
        bloat_btn.get_style_context().add_class("destructive-action")
        bloat_btn.connect("clicked", lambda *_: self._remove_bloat_selected())
        bloat_bar.pack_start(bloat_btn, False, False, 0)
        box.pack_start(bloat_bar, False, False, 0)

        # ----- Lean XFCE ---------------------------------------------------
        box.pack_start(section_header("XFCE components replaced by Mackes"), False, False, 0)
        self._lean_rows_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._lean_rows_box, False, False, 0)

        lean_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        lean_btn = Gtk.Button(label="Remove eligible XFCE components")
        lean_btn.get_style_context().add_class("destructive-action")
        lean_btn.connect("clicked", lambda *_: self._remove_lean())
        lean_bar.pack_start(lean_btn, False, False, 0)
        refresh = Gtk.Button(label="Refresh")
        refresh.connect("clicked", lambda *_: self._refresh())
        lean_bar.pack_start(refresh, False, False, 0)
        box.pack_start(lean_bar, False, False, 0)

        # ----- Log ---------------------------------------------------------
        box.pack_start(section_header("Log"), False, False, 0)
        self._log = Gtk.TextView(); self._log.set_editable(False)
        self._log.set_monospace(True); self._log.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        log_scroll = Gtk.ScrolledWindow(); log_scroll.set_min_content_height(160)
        log_scroll.add(self._log)
        box.pack_start(log_scroll, True, True, 0)

        self.add(box)
        self._refresh()

    # ---- render ------------------------------------------------------------

    def _refresh(self) -> None:
        preset = _active_preset()
        bloat = list((preset.apps.get("remove_bloat") if preset else None) or [])
        lean = list((preset.apps.get("lean_xfce_remove") if preset else None) or [])

        for child in list(self._bloat_rows_box.get_children()):
            self._bloat_rows_box.remove(child)
        self._bloat_checks = []
        if not bloat:
            self._bloat_rows_box.pack_start(info_label(
                "Active preset declares no `apps.remove_bloat` list."
            ), False, False, 0)
        else:
            for pkg in bloat:
                installed = "*" in pkg or is_dnf_installed(pkg)
                row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
                check = Gtk.CheckButton(); check.set_active(installed)
                check.set_sensitive(installed)
                row.pack_start(check, False, False, 0)
                self._bloat_checks.append((check, pkg))
                name_lbl = Gtk.Label(label=pkg); name_lbl.set_xalign(0)
                name_lbl.set_size_request(260, -1)
                row.pack_start(name_lbl, False, False, 0)
                state_lbl = Gtk.Label(label="installed" if installed else "not installed")
                state_lbl.set_xalign(0)
                state_lbl.get_style_context().add_class(
                    "warning" if installed else "dim-label",
                )
                row.pack_start(state_lbl, True, True, 0)
                self._bloat_rows_box.pack_start(row, False, False, 0)
        self._bloat_rows_box.show_all()

        statuses = {p.name: p for p in process_status()}
        for child in list(self._lean_rows_box.get_children()):
            self._lean_rows_box.remove(child)
        if not lean:
            self._lean_rows_box.pack_start(info_label(
                "Active preset declares no `apps.lean_xfce_remove` list."
            ), False, False, 0)
        else:
            for entry in lean:
                if not isinstance(entry, dict):
                    continue
                pkg = entry.get("package", "")
                repl = entry.get("replaced_by", "")
                installed = is_dnf_installed(pkg)
                repl_status = statuses.get(repl)
                eligible = installed and repl_status is not None and repl_status.running
                row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
                dot = Gtk.Label(label="●")
                dot.get_style_context().add_class(
                    "success" if eligible else ("warning" if installed else "dim-label")
                )
                row.pack_start(dot, False, False, 0)

                name_lbl = Gtk.Label(label=pkg); name_lbl.set_xalign(0)
                name_lbl.set_size_request(180, -1)
                row.pack_start(name_lbl, False, False, 0)

                repl_lbl = Gtk.Label(label=f"replaced by {repl}")
                repl_lbl.set_xalign(0); repl_lbl.set_size_request(200, -1)
                repl_lbl.get_style_context().add_class("dim-label")
                row.pack_start(repl_lbl, False, False, 0)

                if not installed:
                    state_text = "not installed"
                elif repl_status is None:
                    state_text = f"replacement {repl!r} not installed — install first"
                elif not repl_status.running:
                    state_text = f"replacement {repl!r} not running — start it first"
                else:
                    state_text = "eligible for removal"
                state_lbl = Gtk.Label(label=state_text); state_lbl.set_xalign(0)
                state_lbl.set_line_wrap(True)
                state_lbl.get_style_context().add_class(
                    "success" if eligible else "dim-label"
                )
                row.pack_start(state_lbl, True, True, 0)
                self._lean_rows_box.pack_start(row, False, False, 0)
        self._lean_rows_box.show_all()

    # ---- actions -----------------------------------------------------------

    def _remove_bloat_selected(self) -> None:
        pkgs = [name for c, name in self._bloat_checks if c.get_active() and c.get_sensitive()]
        if not pkgs:
            self._append_log("(nothing selected)")
            return
        self._append_log(f"--- removing bloat: {', '.join(pkgs)} ---")
        self._run_async(lambda: remove_packages(pkgs, category="bloat"))

    def _remove_lean(self) -> None:
        preset = _active_preset()
        lean = list((preset.apps.get("lean_xfce_remove") if preset else None) or [])
        if not lean:
            self._append_log("(no lean_xfce_remove list)")
            return
        self._append_log("--- removing eligible XFCE components ---")
        self._run_async(lambda: remove_lean_xfce(lean))

    def _run_async(self, fn) -> None:
        def worker() -> None:
            try:
                actions = fn()
            except Exception as e:  # noqa: BLE001
                actions = [f"error: {e}"]
            GLib.idle_add(self._finish, actions)

        threading.Thread(target=worker, daemon=True).start()

    def _finish(self, actions: list[str]) -> bool:
        for line in actions:
            self._append_log(line)
        self._refresh()
        return False

    def _append_log(self, line: str) -> None:
        buf = self._log.get_buffer()
        end = buf.get_end_iter()
        buf.insert(end, line + "\n")
