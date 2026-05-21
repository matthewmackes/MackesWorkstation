"""i3 layout quick-menu — system-tray popup.

A GTK3 status-icon that opens a floating popup with one-click layout
commands against the focused i3 container.  Left-click the tray icon
to toggle the popup; right-click for Quit.

Usage:
    python3 -m mackes.i3_menu
"""
from __future__ import annotations

import shutil
import subprocess

import gi

gi.require_version("Gdk", "3.0")
gi.require_version("Gtk", "3.0")
from gi.repository import Gdk, Gtk


# ---------------------------------------------------------------------------
# i3-msg helper
# ---------------------------------------------------------------------------

def _i3(cmd: str) -> None:
    if not shutil.which("i3-msg"):
        return
    try:
        subprocess.run(["i3-msg", cmd], capture_output=True, timeout=3, check=False)
    except (OSError, subprocess.TimeoutExpired):
        pass


# ---------------------------------------------------------------------------
# Layout actions  (label, i3-msg command)
# ---------------------------------------------------------------------------

_ACTIONS: list[tuple[str, str]] = [
    ("Split H",       "split h"),
    ("Split V",       "split v"),
    ("Tabbed",        "layout tabbed"),
    ("Stacked",       "layout stacking"),
    ("Toggle Float",  "floating toggle"),
    ("Fullscreen",    "fullscreen toggle"),
]

_CSS = b"""
window#i3menu {
    background-color: #1c1c1c;
    border: 1px solid #444;
    border-radius: 8px;
}
box {
    padding: 10px;
}
label.section-title {
    color: #8d8d8d;
    font-size: 11px;
    font-weight: bold;
    letter-spacing: 0.08em;
    margin-bottom: 4px;
}
button {
    background: #2d2d2d;
    color: #e0e0e0;
    border: 1px solid #4a4a4a;
    border-radius: 5px;
    padding: 7px 0;
    font-size: 12px;
    min-width: 116px;
}
button:hover {
    background: #0f62fe;
    border-color: #0f62fe;
    color: #ffffff;
}
button:active {
    background: #0353e9;
}
"""


# ---------------------------------------------------------------------------
# Popup window
# ---------------------------------------------------------------------------

class _Popup(Gtk.Window):

    def __init__(self) -> None:
        super().__init__(type=Gtk.WindowType.POPUP)
        self.set_name("i3menu")
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_resizable(False)

        provider = Gtk.CssProvider()
        provider.load_from_data(_CSS)
        Gtk.StyleContext.add_provider_for_screen(
            Gdk.Screen.get_default(),
            provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        outer.set_border_width(10)
        self.add(outer)

        title = Gtk.Label(label="I3 LAYOUT")
        title.get_style_context().add_class("section-title")
        outer.pack_start(title, False, False, 0)

        sep = Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL)
        outer.pack_start(sep, False, False, 2)

        grid = Gtk.Grid(column_spacing=6, row_spacing=6, column_homogeneous=True)
        outer.pack_start(grid, False, False, 0)

        for i, (label, cmd) in enumerate(_ACTIONS):
            btn = Gtk.Button(label=label)
            btn.connect("clicked", self._on_action, cmd)
            grid.attach(btn, i % 2, i // 2, 1, 1)

        # Hide on focus loss
        self.connect("focus-out-event", lambda *_: self.hide())

    def _on_action(self, _btn: Gtk.Button, cmd: str) -> None:
        self.hide()
        _i3(cmd)

    def show_near(self, x: int, y: int) -> None:
        self.show_all()
        # nudge left so the popup doesn't clip off the right edge
        self.move(max(0, x - 50), y)
        self.present()
        self.grab_focus()


# ---------------------------------------------------------------------------
# Tray icon
# ---------------------------------------------------------------------------

class _Tray:

    def __init__(self) -> None:
        self._popup = _Popup()
        icon = Gtk.StatusIcon()
        icon.set_from_icon_name("preferences-system-windows")
        icon.set_tooltip_text("i3 layout menu")
        icon.set_visible(True)
        icon.connect("activate", self._on_click)
        icon.connect("popup-menu", self._on_right_click)
        self._icon = icon

    def _on_click(self, icon: Gtk.StatusIcon) -> None:
        if self._popup.is_visible():
            self._popup.hide()
            return
        ok, area, _orient = icon.get_geometry()
        if ok:
            self._popup.show_near(area.x, area.y + area.height + 4)
        else:
            display = Gdk.Display.get_default()
            _seat, x, y = display.get_default_seat().get_pointer().get_position()
            self._popup.show_near(x, y - 160)

    def _on_right_click(self, icon: Gtk.StatusIcon, button: int, time: int) -> None:
        menu = Gtk.Menu()
        quit_item = Gtk.MenuItem(label="Quit")
        quit_item.connect("activate", lambda *_: Gtk.main_quit())
        menu.append(quit_item)
        menu.show_all()
        menu.popup(None, None, Gtk.StatusIcon.position_menu, icon, button, time)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> None:
    _Tray()
    Gtk.main()


if __name__ == "__main__":
    main()
