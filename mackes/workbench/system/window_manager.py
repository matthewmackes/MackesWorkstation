"""System → Window Manager (xfwm4)."""
from __future__ import annotations

from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import (
    error_label, info_label, labeled_row, panel_box, section_header, title_label,
)


CHANNEL = "xfwm4"
FOCUS_MODES = ["click", "sloppy", "mouse"]
TITLE_LAYOUTS = ["O|HMC", "O|SHMC", "C|HMO", "OSC|HM"]


def _xfwm_themes() -> list[str]:
    seen: set[str] = set()
    for root in (Path("/usr/share/themes"), Path.home() / ".themes"):
        if not root.is_dir():
            continue
        for entry in root.iterdir():
            if (entry / "xfwm4").is_dir():
                seen.add(entry.name)
    return sorted(seen) or ["Default"]


class WindowManagerPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Window Manager"), False, False, 0)
        box.pack_start(info_label(
            "How your windows look and behave: title-bar style, which "
            "window gets the keyboard when you move your mouse around, "
            "and where the close button lives."
        ), False, False, 0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            box.pack_start(error_label(str(e)), False, False, 0)
            return box

        box.pack_start(section_header("Theme"), False, False, 0)
        themes = _xfwm_themes()
        theme_combo = Gtk.ComboBoxText()
        for t in themes:
            theme_combo.append_text(t)
        xf.bind_combo(theme_combo, CHANNEL, "/general/theme", themes, themes[0])
        box.pack_start(labeled_row("Decoration theme", theme_combo), False, False, 0)

        box.pack_start(section_header("Focus"), False, False, 0)
        focus_combo = Gtk.ComboBoxText()
        for f in FOCUS_MODES:
            focus_combo.append_text(f)
        xf.bind_combo(focus_combo, CHANNEL, "/general/focus_mode", FOCUS_MODES, "click")
        box.pack_start(labeled_row("Focus mode", focus_combo), False, False, 0)

        raise_focus = Gtk.Switch()
        raise_focus.set_active(bool(xf.get(CHANNEL, "/general/raise_on_focus", True)))
        def on_raise(s, _g):
            xf.set(CHANNEL, "/general/raise_on_focus", s.get_active())
        raise_focus.connect("notify::active", on_raise)
        box.pack_start(labeled_row("Raise on focus", raise_focus), False, False, 0)

        box.pack_start(section_header("Title bar"), False, False, 0)
        layout_combo = Gtk.ComboBoxText()
        for layout in TITLE_LAYOUTS:
            layout_combo.append_text(layout)
        xf.bind_combo(layout_combo, CHANNEL, "/general/button_layout",
                      TITLE_LAYOUTS, "O|HMC")
        box.pack_start(labeled_row("Button layout", layout_combo), False, False, 0)

        return box
