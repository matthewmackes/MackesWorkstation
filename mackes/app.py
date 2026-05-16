"""Gtk.Application — the single binary entry point.

Implements Q4 lock: one binary that detects state and routes to either the
first-run wizard (state.provisioned is False) or the daily workbench
(state.provisioned is True).
"""
from __future__ import annotations

import sys

from pathlib import Path
from typing import Optional

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gio, GLib, Gdk  # noqa: E402

from mackes import __version__
from mackes.state import MackesState, ensure_dirs


_CSS_ROOTS = (
    Path("/usr/share/mackes-shell/data/css"),
    Path(__file__).resolve().parent.parent / "data" / "css",
)


def _resolve_css(*rel_parts: str) -> Optional[Path]:
    for root in _CSS_ROOTS:
        p = root.joinpath(*rel_parts)
        if p.is_file():
            return p
    return None


def _load_provider(path: Path, priority: int) -> None:
    provider = Gtk.CssProvider()
    try:
        provider.load_from_path(str(path))
    except GLib.Error:
        return
    screen = Gdk.Screen.get_default()
    if screen is None:
        return
    Gtk.StyleContext.add_provider_for_screen(screen, provider, priority)


def _install_css(active_preset: Optional[str]) -> None:
    base = _resolve_css("mackes.css")
    if base is not None:
        _load_provider(base, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    if active_preset:
        accent = _resolve_css("accents", f"{active_preset}.css")
        if accent is not None:
            _load_provider(accent, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION + 1)


APP_ID = "shell.mackes.Mackes"


class MackesApp(Gtk.Application):
    def __init__(self) -> None:
        super().__init__(
            application_id=APP_ID,
            flags=Gio.ApplicationFlags.HANDLES_COMMAND_LINE,
        )
        self.add_main_option(
            "wizard", ord("w"), GLib.OptionFlags.NONE, GLib.OptionArg.NONE,
            "Force the first-run wizard regardless of state", None,
        )
        self.add_main_option(
            "version", ord("V"), GLib.OptionFlags.NONE, GLib.OptionArg.NONE,
            "Print version and exit", None,
        )
        self._force_wizard = False

    # ---- Application lifecycle -------------------------------------------

    def do_command_line(self, command_line: Gio.ApplicationCommandLine) -> int:  # type: ignore[override]
        opts = command_line.get_options_dict().end().unpack()
        if opts.get("version"):
            print(f"mackes {__version__}")
            return 0
        self._force_wizard = bool(opts.get("wizard"))
        self.activate()
        return 0

    def do_activate(self) -> None:  # type: ignore[override]
        ensure_dirs()
        state = MackesState.load()
        _install_css(state.active_preset)
        if self._force_wizard or not state.provisioned:
            self._open_wizard(state)
        else:
            self._open_workbench(state)

    # ---- Routing ---------------------------------------------------------

    def _open_wizard(self, state: MackesState) -> None:
        # Import locally so wizard/workbench dependencies are lazy
        from mackes.wizard.window import WizardWindow

        win = WizardWindow(application=self, state=state)
        win.connect("destroy", lambda *_: self.quit())
        win.show_all()

    def _open_workbench(self, state: MackesState) -> None:
        from mackes.workbench.window import WorkbenchWindow

        win = WorkbenchWindow(application=self, state=state)
        win.connect("destroy", lambda *_: self.quit())
        win.show_all()


def main(argv: list[str] | None = None) -> int:
    return MackesApp().run(argv if argv is not None else sys.argv)
