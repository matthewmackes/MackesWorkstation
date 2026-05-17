"""Maintain → Logs.

Tail mackes.log and the xfsettingsd journal. Tail length is bounded; the
panel polls the log file size on a 2-second interval and re-renders only
when the file grows.
"""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.state import LOG_DIR
from mackes.workbench._common import (
    info_label, panel_box, section_description, section_header, title_label,
)


MACKES_LOG = LOG_DIR / "mackes.log"
TAIL_LINES = 400


def _read_tail(n: int) -> str:
    if not MACKES_LOG.exists():
        return "No log yet — mackes hasn't recorded any actions."
    try:
        text = MACKES_LOG.read_text(encoding="utf-8", errors="ignore")
    except OSError as e:
        return f"(failed to read log: {e})"
    lines = text.splitlines()
    return "\n".join(lines[-n:]) if lines else "(log is empty)"


def _journal_xfsettingsd(n: int) -> str:
    try:
        out = subprocess.check_output(
            ["journalctl", "--user", "-u", "xfsettingsd", "-n", str(n), "--no-pager"],
            text=True, stderr=subprocess.STDOUT, timeout=5,
        )
        return out.strip() or "(journal returned nothing)"
    except FileNotFoundError:
        return "journalctl not found."
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
        return getattr(e, "output", "") or str(e)


class LogsPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._last_size = -1
        self._poll_id = 0
        self._build()
        self.connect("destroy", self._on_destroy)

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Logs"), False, False, 0)
        box.pack_start(info_label(
            "Recent activity from Mackes and your desktop settings "
            "service. Useful when something went wrong and you want to "
            "see why."
        ), False, False, 0)
        box.pack_start(section_description(
            f"Shows the last {TAIL_LINES} lines of each log and "
            "refreshes itself every couple of seconds."
        ), False, False, 0)

        bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        copy = Gtk.Button(label="Copy mackes.log path")
        copy.connect("clicked", lambda *_: self._copy_path())
        refresh = Gtk.Button(label="Refresh now")
        refresh.connect("clicked", lambda *_: self._refresh(force=True))
        bar.pack_start(refresh, False, False, 0)
        bar.pack_start(copy, False, False, 0)
        box.pack_start(bar, False, False, 0)

        box.pack_start(section_header("mackes.log"), False, False, 0)
        self._mackes_view = Gtk.TextView()
        self._mackes_view.set_editable(False); self._mackes_view.set_monospace(True)
        self._mackes_view.set_cursor_visible(False)
        scroll1 = Gtk.ScrolledWindow(); scroll1.add(self._mackes_view)
        scroll1.set_size_request(-1, 260)
        box.pack_start(scroll1, True, True, 0)

        box.pack_start(section_header("xfsettingsd journal"), False, False, 0)
        self._journal_view = Gtk.TextView()
        self._journal_view.set_editable(False); self._journal_view.set_monospace(True)
        self._journal_view.set_cursor_visible(False)
        scroll2 = Gtk.ScrolledWindow(); scroll2.add(self._journal_view)
        scroll2.set_size_request(-1, 200)
        box.pack_start(scroll2, True, True, 0)

        self.add(box)
        # Visibility-gated poll: only tick the file-stat refresh while the
        # panel is actually shown. Saves a per-2s wake on every other panel.
        self._poll_id: int | None = None
        self.connect("map",   lambda *_: self._start_poll())
        self.connect("unmap", lambda *_: self._stop_poll())
        # First paint happens right after add(), force a refresh now so the
        # views aren't empty when the user first lands.
        self._refresh(force=True)

    def _start_poll(self) -> None:
        if self._poll_id is None:
            self._refresh(force=True)
            self._poll_id = GLib.timeout_add_seconds(
                2, lambda: (self._refresh(), True)[1])

    def _stop_poll(self) -> None:
        if self._poll_id is not None:
            GLib.source_remove(self._poll_id)
            self._poll_id = None

    def _refresh(self, *, force: bool = False) -> None:
        try:
            size = MACKES_LOG.stat().st_size if MACKES_LOG.exists() else 0
        except OSError:
            size = -1
        if force or size != self._last_size:
            self._mackes_view.get_buffer().set_text(_read_tail(TAIL_LINES))
            self._scroll_to_end(self._mackes_view)
            self._last_size = size
        if force:
            self._journal_view.get_buffer().set_text(_journal_xfsettingsd(TAIL_LINES))
            self._scroll_to_end(self._journal_view)

    @staticmethod
    def _scroll_to_end(view: Gtk.TextView) -> None:
        buf = view.get_buffer()
        end = buf.get_end_iter()
        mark = buf.create_mark(None, end, False)
        view.scroll_to_mark(mark, 0, False, 0, 1)

    def _copy_path(self) -> None:
        from gi.repository import Gdk
        clip = Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD)
        clip.set_text(str(MACKES_LOG), -1)

    def _on_destroy(self, *_):
        if self._poll_id:
            GLib.source_remove(self._poll_id)
            self._poll_id = 0
