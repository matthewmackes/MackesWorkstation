"""System → Window Manager.

1.0.7+: two-mode panel. When xfwm4 is the active WM, surfaces xfwm4's
theme/focus/title-bar settings (the legacy panel). When i3 is the
active WM, surfaces a grid of layout presets that apply via `i3-msg`
to the current workspace. A top "Active window manager" row toggles
between the two via `mackes-wm i3 | xfwm4`.
"""
from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

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


def _detect_wm() -> str:
    """Return the running WM's `Name:` from `wmctrl -m`, or empty string."""
    try:
        out = subprocess.run(
            ["wmctrl", "-m"], capture_output=True, text=True,
            timeout=2, check=False,
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return ""
    for line in out.splitlines():
        if line.startswith("Name:"):
            return line.split(":", 1)[1].strip()
    return ""


def _i3_msg(*args: str) -> None:
    """Fire a one-shot i3-msg with no error feedback to the UI.
    Caller is expected to refresh the panel afterwards if state matters."""
    if not shutil.which("i3-msg"):
        return
    try:
        subprocess.run(
            ["i3-msg", *args], capture_output=True, timeout=2, check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        pass


def _mackes_wm(target: str) -> None:
    """Call /usr/bin/mackes-wm to swap window managers live."""
    if not shutil.which("mackes-wm"):
        return
    try:
        subprocess.Popen(
            ["mackes-wm", target],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
    except OSError:
        pass


# ---------------------------------------------------------------------------
# Layout presets (i3 only)
# ---------------------------------------------------------------------------
#
# Each preset is a (label, description, command-sequence) tuple. The
# command sequence runs via i3-msg; multi-step layouts chain commands
# with semicolons (i3's native sequence separator). Layouts that
# require interaction beyond `i3-msg` (the 2×2 grid) are flagged
# `interactive=True` and pop a short instructions toast.

_LAYOUT_PRESETS: list[tuple[str, str, str, bool]] = [
    # User-requested core three:
    ("Maximized",
     "Fullscreen the focused window. Top and bottom bars stay visible.",
     "fullscreen enable",
     False),
    ("Side by Side",
     "Two windows split horizontally. New windows append to the right.",
     "layout splith; focus parent; layout splith",
     False),
    ("Split in 4",
     "2×2 grid. Place the four windows you want tiled, then click. "
     "i3 splits the workspace horizontally, then each half vertically.",
     "split h; layout splith",  # placeholder; the click handler does the multi-step dance
     True),
    # Suggested five:
    ("Master + Stack",
     "One big window on the left, the rest stacked on the right "
     "(the KDE / Plasma 'master + stack' pattern).",
     "layout splith; focus right; layout stacking",
     False),
    ("Tabbed",
     "All windows in the container become tabs across the top. "
     "Click a tab to switch — like browser tabs but for the desktop.",
     "layout tabbed",
     False),
    ("Stacking",
     "Like Tabbed but the title list is vertical. Saves horizontal "
     "space when you have many concurrent windows.",
     "layout stacking",
     False),
    ("Focus Mode",
     "Single window centered with large gaps around it. Distraction-"
     "free writing / reading mode.",
     "gaps inner current set 60; gaps outer current set 40",
     False),
    ("Floating",
     "Toggle the focused window to floating (free-drag) mode. "
     "Click again to re-tile.",
     "floating toggle",
     False),
]


class WindowManagerPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._content_holder = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL, spacing=0,
        )
        self.add(self._content_holder)
        self._render()

    def _render(self) -> None:
        for c in self._content_holder.get_children():
            self._content_holder.remove(c)
        self._content_holder.pack_start(self._build(), True, True, 0)
        self._content_holder.show_all()

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Window Manager"), False, False, 0)
        box.pack_start(info_label(
            "Pick the window manager that lays out your apps. Xfwm4 is "
            "the classic XFCE manager — floating windows you arrange "
            "by hand. i3 tiles automatically into clean grids, tabs, "
            "and stacks."
        ), False, False, 0)

        # ---- Active window manager -----------------------------------
        active = _detect_wm() or "unknown"
        box.pack_start(section_header("Active window manager"), False, False, 0)
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        row.pack_start(info_label(f"Currently running: {active}"),
                       True, True, 0)
        if active.lower().startswith("xfwm"):
            btn = Gtk.Button(label="Switch to i3")
            btn.get_style_context().add_class("suggested-action")
            btn.connect("clicked", self._on_switch_to_i3)
        elif active.lower().startswith("i3"):
            btn = Gtk.Button(label="Switch to Xfwm4")
            btn.connect("clicked", self._on_switch_to_xfwm4)
        else:
            btn = Gtk.Button(label="Refresh")
            btn.connect("clicked", lambda *_: self._render())
        row.pack_end(btn, False, False, 0)
        box.pack_start(row, False, False, 0)

        # ---- WM-specific body ----------------------------------------
        if active.lower().startswith("i3"):
            box.pack_start(self._build_i3_body(), True, True, 0)
        else:
            box.pack_start(self._build_xfwm_body(), True, True, 0)

        return box

    def _on_switch_to_i3(self, _btn: Gtk.Button) -> None:
        _mackes_wm("i3")
        # Give the WM a moment to swap, then re-render.
        GLib.timeout_add_seconds(1, lambda: (self._render(), False)[1])

    def _on_switch_to_xfwm4(self, _btn: Gtk.Button) -> None:
        _mackes_wm("xfwm4")
        GLib.timeout_add_seconds(1, lambda: (self._render(), False)[1])

    # ----------------------------------------------------------------
    # i3 — layout buttons
    # ----------------------------------------------------------------

    def _build_i3_body(self) -> Gtk.Widget:
        body = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        body.pack_start(section_header("Layouts"), False, False, 0)
        body.pack_start(info_label(
            "Click a layout to apply it to the focused workspace. "
            "Layouts work on the current container — open the windows "
            "you want tiled first, then pick a layout."
        ), False, False, 0)

        grid = Gtk.Grid()
        grid.set_row_spacing(8); grid.set_column_spacing(8)
        grid.set_margin_top(8); grid.set_margin_bottom(8)
        grid.set_column_homogeneous(True)

        for i, (label, desc, cmd, interactive) in enumerate(_LAYOUT_PRESETS):
            tile = self._layout_tile(label, desc, cmd, interactive)
            grid.attach(tile, i % 3, i // 3, 1, 1)
        body.pack_start(grid, False, False, 0)

        # Helpful footer with the reload + reset actions.
        body.pack_start(section_header("Config"), False, False, 0)
        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        reload_btn = Gtk.Button(label="Reload i3 config")
        reload_btn.connect("clicked", lambda *_: _i3_msg("reload"))
        actions.pack_start(reload_btn, False, False, 0)
        reset_btn = Gtk.Button(label="Reset to Mackes default")
        reset_btn.connect("clicked", self._on_reset_i3_config)
        actions.pack_start(reset_btn, False, False, 0)
        body.pack_start(actions, False, False, 0)

        return body

    def _layout_tile(self, label: str, desc: str, cmd: str,
                     interactive: bool) -> Gtk.Widget:
        btn = Gtk.Button()
        btn.set_relief(Gtk.ReliefStyle.NORMAL)
        btn.set_tooltip_text(desc)
        col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        col.set_margin_top(8); col.set_margin_bottom(8)
        col.set_margin_start(12); col.set_margin_end(12)
        title = Gtk.Label(label=label)
        title.get_style_context().add_class("mackes-tile-title")
        title.set_xalign(0)
        col.pack_start(title, False, False, 0)
        sub = Gtk.Label(label=desc)
        sub.get_style_context().add_class("mackes-tile-sub")
        sub.set_xalign(0)
        sub.set_line_wrap(True)
        sub.set_max_width_chars(40)
        col.pack_start(sub, False, False, 0)
        btn.add(col)

        if interactive and label == "Split in 4":
            btn.connect("clicked", self._on_split_in_4)
        else:
            btn.connect("clicked", lambda *_, c=cmd: _i3_msg(c))
        return btn

    def _on_split_in_4(self, _btn: Gtk.Button) -> None:
        """Best-effort 2x2 tile of the current workspace's windows.
        Sequence: split horizontally at top, then split each half
        vertically. i3 walks the tree from the focused container,
        so we focus the workspace root first."""
        # Step 1: set the workspace to splith
        _i3_msg("focus parent; focus parent; layout splith")
        # Step 2: split right half into two rows
        _i3_msg("focus right; split v")
        # Step 3: split left half into two rows
        _i3_msg("focus left; split v")

    def _on_reset_i3_config(self, _btn: Gtk.Button) -> None:
        src = Path("/usr/share/mackes-shell/i3/config")
        dst = Path.home() / ".config" / "i3" / "config"
        if not src.is_file():
            return
        try:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_text(src.read_text(encoding="utf-8"), encoding="utf-8")
        except OSError:
            return
        _i3_msg("reload")

    # ----------------------------------------------------------------
    # xfwm4 — legacy panel (unchanged from 1.0.6)
    # ----------------------------------------------------------------

    def _build_xfwm_body(self) -> Gtk.Widget:
        body = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)

        try:
            xf = get_bridge()
        except XfconfError as e:
            body.pack_start(error_label(str(e)), False, False, 0)
            return body

        body.pack_start(section_header("Theme"), False, False, 0)
        themes = _xfwm_themes()
        theme_combo = Gtk.ComboBoxText()
        for t in themes:
            theme_combo.append_text(t)
        xf.bind_combo(theme_combo, CHANNEL, "/general/theme", themes, themes[0])
        body.pack_start(labeled_row("Decoration theme", theme_combo),
                        False, False, 0)

        body.pack_start(section_header("Focus"), False, False, 0)
        focus_combo = Gtk.ComboBoxText()
        for f in FOCUS_MODES:
            focus_combo.append_text(f)
        xf.bind_combo(focus_combo, CHANNEL, "/general/focus_mode",
                      FOCUS_MODES, "click")
        body.pack_start(labeled_row("Focus mode", focus_combo),
                        False, False, 0)

        raise_focus = Gtk.Switch()
        raise_focus.set_active(bool(xf.get(CHANNEL, "/general/raise_on_focus", True)))
        def on_raise(s, _g):
            xf.set(CHANNEL, "/general/raise_on_focus", s.get_active())
        raise_focus.connect("notify::active", on_raise)
        body.pack_start(labeled_row("Raise on focus", raise_focus),
                        False, False, 0)

        body.pack_start(section_header("Title bar"), False, False, 0)
        layout_combo = Gtk.ComboBoxText()
        for layout in TITLE_LAYOUTS:
            layout_combo.append_text(layout)
        xf.bind_combo(layout_combo, CHANNEL, "/general/button_layout",
                      TITLE_LAYOUTS, "O|HMC")
        body.pack_start(labeled_row("Button layout", layout_combo),
                        False, False, 0)

        return body
