"""Apps → Installed — searchable RPM list."""
from __future__ import annotations

import threading

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.app_mgmt import list_installed_packages, remove_packages
from mackes.workbench._common import (
    info_label, panel_box, section_header, title_label,
)


class AppsInstalledPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Installed packages"), False, False, 0)
        box.pack_start(info_label(
            "Every RPM on this machine. Filter by name; remove individual "
            "packages via the per-row button. Removals run under sudo."
        ), False, False, 0)

        self._filter = Gtk.SearchEntry(); self._filter.set_placeholder_text("Filter…")
        self._filter.connect("search-changed", lambda *_: self._refresh_view())
        box.pack_start(self._filter, False, False, 0)

        box.pack_start(section_header("Packages"), False, False, 0)
        self._list_store = Gtk.ListStore(str, str)  # name, version
        self._view = Gtk.TreeView(model=self._list_store)
        for i, title in enumerate(("Name", "Version")):
            col = Gtk.TreeViewColumn(title, Gtk.CellRendererText(), text=i)
            col.set_resizable(True); col.set_sort_column_id(i)
            self._view.append_column(col)
        sw = Gtk.ScrolledWindow(); sw.set_min_content_height(380); sw.add(self._view)
        box.pack_start(sw, True, True, 0)

        bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        remove_btn = Gtk.Button(label="Remove selected")
        remove_btn.get_style_context().add_class("destructive-action")
        remove_btn.connect("clicked", lambda *_: self._remove_selected())
        bar.pack_start(remove_btn, False, False, 0)
        reload_btn = Gtk.Button(label="Reload list")
        reload_btn.connect("clicked", lambda *_: self._reload())
        bar.pack_start(reload_btn, False, False, 0)
        box.pack_start(bar, False, False, 0)

        box.pack_start(section_header("Log"), False, False, 0)
        self._log = Gtk.TextView(); self._log.set_editable(False); self._log.set_monospace(True)
        log_scroll = Gtk.ScrolledWindow(); log_scroll.set_min_content_height(100)
        log_scroll.add(self._log)
        box.pack_start(log_scroll, False, False, 0)

        self.add(box)
        self._all: list[tuple[str, str]] = []
        self._reload()

    def _reload(self) -> None:
        self._append_log("loading rpm -qa…")
        def worker() -> None:
            pairs = list_installed_packages()
            GLib.idle_add(self._set_all, pairs)
        threading.Thread(target=worker, daemon=True).start()

    def _set_all(self, pairs: list[tuple[str, str]]) -> bool:
        self._all = pairs
        self._refresh_view()
        self._append_log(f"loaded {len(pairs)} packages")
        return False

    def _refresh_view(self) -> None:
        needle = (self._filter.get_text() or "").strip().lower()
        self._list_store.clear()
        for name, version in self._all:
            if needle and needle not in name.lower():
                continue
            self._list_store.append([name, version])

    def _remove_selected(self) -> None:
        selection = self._view.get_selection()
        model, paths = selection.get_selected_rows()
        names = [model[p][0] for p in paths]
        if not names:
            self._append_log("(no selection)")
            return
        self._append_log(f"--- removing: {', '.join(names)} ---")
        def worker() -> None:
            actions = remove_packages(names, category="bloat")
            GLib.idle_add(self._finish, actions)
        threading.Thread(target=worker, daemon=True).start()

    def _finish(self, actions: list[str]) -> bool:
        for line in actions:
            self._append_log(line)
        self._reload()
        return False

    def _append_log(self, line: str) -> None:
        buf = self._log.get_buffer()
        end = buf.get_end_iter()
        buf.insert(end, line + "\n")
