"""System → Default Apps (mimeapps.list).

Edits ~/.config/mimeapps.list — the XDG-standard user MIME-to-application
mapping. Discovers installed handlers by scanning .desktop files for
MimeType= declarations. Mackes exposes a handful of common MIME categories
(web browser, mail, terminal, file manager, text editor, image viewer,
video player, audio player) with a dropdown per category.

11.9 reliability sweep: `_discover_handlers()` walks every .desktop file
in three directories (~300 files on a typical Fedora desktop) and parses
each with ConfigParser — ~340 ms at construction time. The walk now
happens off-main-thread via `mackes.workbench._async.async_probe`; the
category combos are inserted by `_apply_state` once the scan lands.
"""
from __future__ import annotations

import configparser
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.state import HOME
from mackes.workbench._async import async_probe
from mackes.workbench._common import (
    a11y, info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


MIMEAPPS = HOME / ".config" / "mimeapps.list"
DESKTOP_DIRS = [
    HOME / ".local" / "share" / "applications",
    Path("/usr/local/share/applications"),
    Path("/usr/share/applications"),
]

# Curated map of category-label -> list of canonical MIME types we'll bind to.
CATEGORIES: list[tuple[str, list[str]]] = [
    ("Web browser",     ["x-scheme-handler/http", "x-scheme-handler/https",
                         "text/html"]),
    ("Email",           ["x-scheme-handler/mailto", "message/rfc822"]),
    ("File manager",    ["inode/directory"]),
    ("Terminal",        ["x-scheme-handler/terminal"]),
    ("Text editor",     ["text/plain"]),
    ("Image viewer",    ["image/png", "image/jpeg", "image/webp"]),
    ("Video player",    ["video/mp4", "video/x-matroska"]),
    ("Audio player",    ["audio/mpeg", "audio/flac", "audio/ogg"]),
    ("PDF viewer",      ["application/pdf"]),
]


def _discover_handlers() -> dict[str, list[tuple[str, str]]]:
    """For each MIME type we care about, list (desktop_id, display_name).

    Scans every .desktop file in DESKTOP_DIRS for MimeType= declarations.
    Later directories shadow earlier ones (so user-local overrides system).
    """
    all_mimes = {m for _, mimes in CATEGORIES for m in mimes}
    by_mime: dict[str, dict[str, str]] = {m: {} for m in all_mimes}

    for root in DESKTOP_DIRS:
        if not root.is_dir():
            continue
        for path in root.glob("*.desktop"):
            try:
                parser = configparser.RawConfigParser(interpolation=None, strict=False)
                parser.read(path, encoding="utf-8")
            except (OSError, configparser.Error):
                continue
            if not parser.has_section("Desktop Entry"):
                continue
            entry = parser["Desktop Entry"]
            if entry.get("NoDisplay", "false").lower() == "true":
                continue
            if entry.get("Hidden", "false").lower() == "true":
                continue
            mimes = [m.strip() for m in entry.get("MimeType", "").split(";") if m.strip()]
            if not mimes:
                continue
            name = entry.get("Name", path.stem)
            for m in mimes:
                if m in by_mime:
                    by_mime[m][path.name] = name

    # Return sorted list per MIME
    result: dict[str, list[tuple[str, str]]] = {}
    for mime, handlers in by_mime.items():
        result[mime] = sorted(handlers.items(), key=lambda kv: kv[1].lower())
    return result


def _load_mimeapps() -> configparser.RawConfigParser:
    parser = configparser.RawConfigParser(interpolation=None, strict=False)
    parser.optionxform = str  # preserve case
    if MIMEAPPS.exists():
        try:
            parser.read(MIMEAPPS, encoding="utf-8")
        except (OSError, configparser.Error):
            pass
    for section in ("Default Applications", "Added Associations"):
        if not parser.has_section(section):
            parser.add_section(section)
    return parser


def _current_default(mime: str) -> str:
    parser = _load_mimeapps()
    return parser.get("Default Applications", mime, fallback="")


def _set_default(mimes: Iterable[str], desktop_id: str) -> None:
    parser = _load_mimeapps()
    for m in mimes:
        if desktop_id:
            parser.set("Default Applications", m, desktop_id)
        else:
            parser.remove_option("Default Applications", m)
    MIMEAPPS.parent.mkdir(parents=True, exist_ok=True)
    if MIMEAPPS.exists():
        shutil.copy2(MIMEAPPS, MIMEAPPS.with_suffix(".list.bak"))
    with MIMEAPPS.open("w", encoding="utf-8") as f:
        parser.write(f, space_around_delimiters=False)
    log_action(f"default apps: {','.join(mimes)} -> {desktop_id or '(cleared)'}")


@dataclass(frozen=True)
class _DefaultAppsState:
    """Off-main-thread snapshot of the .desktop scan plus the current
    default for each category's first MIME type."""
    handlers: dict[str, list[tuple[str, str]]]
    current_defaults: dict[str, str]  # category-key MIME -> desktop_id


def _gather_default_apps_state() -> _DefaultAppsState:
    handlers = _discover_handlers()
    # Pre-resolve the "current default" lookup for every category so the
    # GTK builder doesn't re-parse mimeapps.list nine times.
    current_defaults: dict[str, str] = {}
    for _, mimes in CATEGORIES:
        if mimes:
            current_defaults[mimes[0]] = _current_default(mimes[0])
    return _DefaultAppsState(
        handlers=handlers, current_defaults=current_defaults,
    )


class DefaultAppsPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build_skeleton()
        async_probe(_gather_default_apps_state, self._apply_state)

    def _build_skeleton(self) -> None:
        """Render chrome immediately; sections fill in after the probe."""
        box = panel_box()
        box.pack_start(title_label("Default Apps"), False, False, 0)
        box.pack_start(info_label(
            "Choose which app opens when you double-click a web link, "
            "image, video, or other file type."
        ), False, False, 0)
        box.pack_start(section_description(
            "Only apps already installed on your machine show up here. "
            "Install new ones from the Apps panel."
        ), False, False, 0)

        self._loading = info_label("Scanning installed applications…")
        box.pack_start(self._loading, False, False, 0)

        self._content_root = box
        self.add(box)

    def _apply_state(self, state: _DefaultAppsState) -> None:
        if self._loading is not None and self._loading.get_parent() is not None:
            self._content_root.remove(self._loading)
            self._loading = None

        box = self._content_root
        handlers = state.handlers

        for label, mimes in CATEGORIES:
            box.pack_start(section_header(label), False, False, 0)
            # Union of handlers across the category's MIME types
            seen: dict[str, str] = {}
            for m in mimes:
                for desktop_id, name in handlers.get(m, []):
                    seen.setdefault(desktop_id, name)
            options = sorted(seen.items(), key=lambda kv: kv[1].lower())

            combo = Gtk.ComboBoxText()
            combo.append_text("(none)")
            for _, name in options:
                combo.append_text(name)

            ids = [None] + [oid for oid, _ in options]
            current = state.current_defaults.get(mimes[0], "")
            try:
                idx = ids.index(current) if current in ids else 0
            except ValueError:
                idx = 0
            combo.set_active(idx)

            def on_changed(c, _mimes=mimes, _ids=ids):
                i = c.get_active()
                if i < 0:
                    return
                desktop_id = _ids[i] or ""
                _set_default(_mimes, desktop_id)

            combo.connect("changed", on_changed)
            a11y(combo, name=f"Default app for {label}",
                 tooltip=f"Pick which app opens {label.lower()} files / URLs by default")
            box.pack_start(labeled_row("Handler", combo), False, False, 0)

            mlbl = info_label("Applies to: " + ", ".join(mimes))
            box.pack_start(mlbl, False, False, 0)

        box.show_all()
