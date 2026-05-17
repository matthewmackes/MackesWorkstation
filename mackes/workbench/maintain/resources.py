"""Maintain → Resources.

Lightweight live CPU / RAM / disk view — fourth (and final) tool in the
MaintenanceKit. Deliberately small: three numbers, three progress bars,
refresh every 1.5 s. No process tree, no per-core breakdown, no nethogs —
that's what htop is for.

Reads /proc directly so there's no psutil dependency.
"""
from __future__ import annotations

import shutil
import time

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.workbench._common import info_label, panel_box, section_description, section_header, title_label


_REFRESH_MS = 1500


# ---- /proc readers ---------------------------------------------------------


def _read_proc_stat_cpu() -> tuple[int, int]:
    """Return (idle, total) jiffies for the aggregate 'cpu' line."""
    try:
        with open("/proc/stat", "r") as f:
            first = f.readline()
    except OSError:
        return 0, 0
    parts = first.split()
    if not parts or parts[0] != "cpu":
        return 0, 0
    fields = [int(x) for x in parts[1:] if x.isdigit()]
    if len(fields) < 4:
        return 0, 0
    idle = fields[3]
    total = sum(fields)
    return idle, total


def _read_meminfo() -> tuple[int, int]:
    """Return (used_kb, total_kb)."""
    info: dict[str, int] = {}
    try:
        with open("/proc/meminfo", "r") as f:
            for line in f:
                k, _, rest = line.partition(":")
                v = rest.strip().split(" ", 1)[0]
                if v.isdigit():
                    info[k] = int(v)
    except OSError:
        return 0, 0
    total = info.get("MemTotal", 0)
    avail = info.get("MemAvailable", info.get("MemFree", 0))
    return max(0, total - avail), total


def _read_disk() -> tuple[int, int]:
    """Return (used_bytes, total_bytes) for `/`."""
    try:
        usage = shutil.disk_usage("/")
        return usage.used, usage.total
    except OSError:
        return 0, 0


# ---- Card widget ------------------------------------------------------------


class _Card(Gtk.Frame):
    def __init__(self, title: str) -> None:
        super().__init__()
        self.get_style_context().add_class("view")

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.set_margin_top(12); box.set_margin_bottom(12)
        box.set_margin_start(16); box.set_margin_end(16)

        hdr = Gtk.Label(label=title.upper())
        hdr.set_xalign(0)
        hdr.get_style_context().add_class("mackes-section-header")
        box.pack_start(hdr, False, False, 0)

        self._headline = Gtk.Label(label="—")
        self._headline.set_xalign(0)
        self._headline.get_style_context().add_class("title-2")
        box.pack_start(self._headline, False, False, 0)

        self._detail = Gtk.Label(label="")
        self._detail.set_xalign(0)
        self._detail.get_style_context().add_class("dim-label")
        box.pack_start(self._detail, False, False, 0)

        self._bar = Gtk.ProgressBar()
        self._bar.set_margin_top(8)
        box.pack_start(self._bar, False, False, 0)

        self.add(box)

    def set(self, headline: str, detail: str, frac: float) -> None:
        self._headline.set_text(headline)
        self._detail.set_text(detail)
        self._bar.set_fraction(max(0.0, min(1.0, frac)))


# ---- Panel ----------------------------------------------------------------


class ResourcesPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._last_idle, self._last_total = _read_proc_stat_cpu()
        self._last_t = time.monotonic()
        self._build()
        self._refresh()
        GLib.timeout_add(_REFRESH_MS, self._refresh)

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Resources"), False, False, 0)
        box.pack_start(info_label(
            "A live look at how busy your machine is — how much "
            "processor, memory, and disk space you're using right now."
        ), False, False, 0)
        box.pack_start(section_description(
            "Updates a couple of times per second. If a number stays "
            "near 100%, something heavy is running."
        ), False, False, 0)

        box.pack_start(section_header("Live"), False, False, 0)
        cards = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        cards.set_homogeneous(True)
        self._cpu = _Card("CPU")
        self._ram = _Card("Memory")
        self._dsk = _Card("Disk (/)")
        cards.pack_start(self._cpu, True, True, 0)
        cards.pack_start(self._ram, True, True, 0)
        cards.pack_start(self._dsk, True, True, 0)
        box.pack_start(cards, False, False, 0)

        self.add(box)

    def _refresh(self) -> bool:
        idle, total = _read_proc_stat_cpu()
        d_idle = idle - self._last_idle
        d_total = total - self._last_total
        if d_total > 0:
            usage = 1.0 - (d_idle / d_total)
        else:
            usage = 0.0
        self._last_idle, self._last_total = idle, total
        self._cpu.set(
            headline=f"{usage * 100:.0f}%",
            detail=f"{time.strftime('%H:%M:%S')}",
            frac=usage,
        )

        used_kb, total_kb = _read_meminfo()
        if total_kb > 0:
            ram_frac = used_kb / total_kb
            self._ram.set(
                headline=f"{ram_frac * 100:.0f}%",
                detail=f"{used_kb // 1024} / {total_kb // 1024} MiB used",
                frac=ram_frac,
            )
        else:
            self._ram.set("—", "(unable to read /proc/meminfo)", 0.0)

        used_b, total_b = _read_disk()
        if total_b > 0:
            d_frac = used_b / total_b
            self._dsk.set(
                headline=f"{d_frac * 100:.0f}%",
                detail=f"{used_b // (1024**3)} / {total_b // (1024**3)} GiB used",
                frac=d_frac,
            )
        else:
            self._dsk.set("—", "(unable to read disk_usage)", 0.0)
        return True  # keep the timeout firing
