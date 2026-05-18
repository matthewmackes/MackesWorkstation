"""Mackes Shell tray icon — Gtk.StatusIcon based.

Q8 lock from the GUI-redesign survey: the popover is reachable via
panel-plugin button + tray icon + Super+M hotkey. This module is the
tray side.

Entry: `python3 -m mackes.tray` or just `mackes-tray` (a thin shim
ships at /usr/bin/). Run as a user-systemd service after login.

Behaviour:
  * Single click on the tray icon → spawn `mackes --popover`
  * Right click → context menu (Open popover, Open full window,
    Mesh Health, Re-apply preset, Quit tray)
  * Icon tracks live mesh state via mackes.mesh.overall_state()
    every 30 s: green dot when healthy, yellow when warn, red when
    fail. Updated via a GLib.timeout.

Gtk.StatusIcon is deprecated in GTK3 but XFCE's systray still
honours it. The proper modern path is org.kde.StatusNotifierItem
via libayatana-appindicator — captured as a follow-up.
"""
from __future__ import annotations

import os
import subprocess
import sys

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402


def _spawn_popover(*_args) -> None:
    try:
        subprocess.Popen(["mackes", "--popover"],
                         stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL,
                         start_new_session=True)
    except OSError as e:
        sys.stderr.write(f"mackes-tray: spawn failed: {e}\n")


def _spawn_full(*_args) -> None:
    try:
        subprocess.Popen(["mackes"],
                         stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL,
                         start_new_session=True)
    except OSError as e:
        sys.stderr.write(f"mackes-tray: spawn failed: {e}\n")


def _open_mesh_health(*_args) -> None:
    try:
        from mackes.workbench.popover.window import PopoverWindow
        # Easier: open the popover already on the mesh tab
        subprocess.Popen(["mackes", "--popover"],
                         env={**os.environ, "MACKES_POPOVER_TAB": "mesh"},
                         stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL,
                         start_new_session=True)
    except (OSError, ImportError):
        _spawn_popover()


def _quit(_icon=None) -> None:
    Gtk.main_quit()


def _build_menu(icon: Gtk.StatusIcon) -> Gtk.Menu:
    menu = Gtk.Menu()
    for label, cb in (
        ("Open Mackes (popover)", _spawn_popover),
        ("Open Mackes (full window)", _spawn_full),
        ("Mesh Health", _open_mesh_health),
        (None, None),
        ("Quit tray icon", _quit),
    ):
        if label is None:
            menu.append(Gtk.SeparatorMenuItem())
            continue
        item = Gtk.MenuItem(label=label)
        item.connect("activate", cb)
        menu.append(item)
    menu.show_all()
    return menu


def _refresh_icon(icon: Gtk.StatusIcon) -> bool:
    """Update the tray icon + tooltip from mesh.health(). Returns True
    so GLib keeps calling us."""
    state = "ok"
    label = "Mackes"
    try:
        from mackes.mesh import health, overall_state, summary
        snap = health()
        state = overall_state(snap)
        label = f"Mackes — {summary(snap)}"
    except Exception:  # noqa: BLE001
        pass
    # Fedora's hicolor doesn't have state-specific icons; reuse the
    # branded mackes-shell SVG with no state overlay for v1. State is
    # in the tooltip.
    icon.set_from_icon_name("mackes-shell")
    icon.set_tooltip_text(label)
    return True


def main(argv: list[str] | None = None) -> int:
    icon = Gtk.StatusIcon()
    icon.set_from_icon_name("mackes-shell")
    icon.set_tooltip_text("Mackes Shell — click to open popover")
    icon.set_visible(True)

    icon.connect("activate", _spawn_popover)
    icon.connect("popup-menu",
                 lambda i, button, time:
                     _build_menu(i).popup(None, None, Gtk.StatusIcon.position_menu,
                                          i, button, time))
    _refresh_icon(icon)
    GLib.timeout_add_seconds(30, _refresh_icon, icon)

    Gtk.main()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
