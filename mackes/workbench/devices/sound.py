"""Devices → Sound.

PulseAudio/PipeWire default sink picker. Backed by `pactl` since neither
PA nor PW exposes itself through xfconf.
"""
from __future__ import annotations

import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_header, title_label,
)


def _pactl(*args: str) -> str:
    try:
        return subprocess.check_output(["pactl", *args], text=True, stderr=subprocess.DEVNULL).strip()
    except (FileNotFoundError, subprocess.CalledProcessError):
        return ""


def _list_sinks() -> list[tuple[str, str]]:
    raw = _pactl("list", "short", "sinks")
    out: list[tuple[str, str]] = []
    for line in raw.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            out.append((parts[1], parts[1]))  # name, label
    return out


def _list_sources() -> list[tuple[str, str]]:
    raw = _pactl("list", "short", "sources")
    out: list[tuple[str, str]] = []
    for line in raw.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2 and not parts[1].endswith(".monitor"):
            out.append((parts[1], parts[1]))
    return out


def _default_sink() -> str:
    return _pactl("get-default-sink")


def _default_source() -> str:
    return _pactl("get-default-source")


def _set_default_sink(name: str) -> None:
    _pactl("set-default-sink", name)
    log_action(f"sound: default sink -> {name}")


def _set_default_source(name: str) -> None:
    _pactl("set-default-source", name)
    log_action(f"sound: default source -> {name}")


class SoundPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Sound"), False, False, 0)
        box.pack_start(info_label(
            "Pick which speakers or headphones play sound, and which "
            "microphone records it."
        ), False, False, 0)

        if not _pactl("info"):
            box.pack_start(info_label("pactl not available — install pulseaudio-utils."),
                           False, False, 0)
            return box

        box.pack_start(section_header("Output"), False, False, 0)
        sinks = _list_sinks()
        sink_combo = Gtk.ComboBoxText()
        for _, label in sinks:
            sink_combo.append_text(label)
        cur = _default_sink()
        names = [n for n, _ in sinks]
        if cur in names:
            sink_combo.set_active(names.index(cur))
        elif sinks:
            sink_combo.set_active(0)

        def on_sink(c):
            txt = c.get_active_text()
            if txt:
                _set_default_sink(txt)
        sink_combo.connect("changed", on_sink)
        box.pack_start(labeled_row("Default sink", sink_combo), False, False, 0)

        box.pack_start(section_header("Input"), False, False, 0)
        sources = _list_sources()
        src_combo = Gtk.ComboBoxText()
        for _, label in sources:
            src_combo.append_text(label)
        cur_src = _default_source()
        src_names = [n for n, _ in sources]
        if cur_src in src_names:
            src_combo.set_active(src_names.index(cur_src))
        elif sources:
            src_combo.set_active(0)

        def on_src(c):
            txt = c.get_active_text()
            if txt:
                _set_default_source(txt)
        src_combo.connect("changed", on_src)
        box.pack_start(labeled_row("Default source", src_combo), False, False, 0)

        return box
