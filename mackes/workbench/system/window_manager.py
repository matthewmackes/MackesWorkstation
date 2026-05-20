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
from gi.repository import Gtk  # noqa: E402

from mackes.workbench._common import (
    a11y, info_label, panel_box, section_header, title_label,
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
    """Return the running WM's `Name:`.

    v2.0.0 Phase F.8 — checks sway via `mackes.sway_ipc.is_sway_running()`
    first (Wayland path, X11-only wmctrl is unreliable on Wayland);
    falls back to `wmctrl -m` for the v1.x X11 line.
    """
    try:
        from mackes import sway_ipc
        if sway_ipc.is_sway_running():
            return "sway"
    except Exception:  # noqa: BLE001
        pass
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


def _wm_msg(*args: str) -> None:
    """Fire a one-shot window-manager IPC command.

    v2.0.0 Phase F.8 — routes through `mackes.sway_ipc` when sway is
    the active compositor; falls back to `i3-msg` for the v1.x X11
    line. Specifically handles the layout-toggle vocabulary the
    panel emits:

       layout splith / splitv / tabbed / stacking / default
       kill

    Anything else passes through unchanged to the legacy i3-msg
    spawn path.
    """
    try:
        from mackes import sway_ipc
        if sway_ipc.is_sway_running():
            if len(args) >= 2 and args[0] == "layout":
                sway_ipc.set_layout(args[1])
                return
            if args == ("kill",):
                sway_ipc.kill_focused()
                return
            # Other commands fall through to swaymsg with the same
            # argv shape (sway accepts every i3-msg layout verb).
            try:
                subprocess.run(
                    ["swaymsg", *args],
                    capture_output=True, timeout=2, check=False,
                )
            except (OSError, subprocess.TimeoutExpired):
                pass
            return
    except Exception:  # noqa: BLE001
        pass
    if not shutil.which("i3-msg"):
        return
    try:
        subprocess.run(
            ["i3-msg", *args], capture_output=True, timeout=2, check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        pass


# v1.x callers used _i3_msg; keep the name as an alias so existing
# call sites continue to work without an audit pass.
_i3_msg = _wm_msg


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
            "Mackes uses i3 to tile your apps automatically into clean "
            "grids, tabs, and stacks. Pick a layout below to apply it "
            "to the current workspace."
        ), False, False, 0)

        # Phase 8.8 (1.0.7) — i3 is the only WM; the WM-toggle row is
        # retired. Surface the active WM as a one-line status only.
        active = _detect_wm() or "unknown"
        if active.lower() != "i3":
            # Defensive: should only ever happen during the brief
            # window between RPM upgrade and apply_enforce_i3 firing.
            # Show a banner pointing the user at the migration step.
            banner = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            banner.pack_start(info_label(
                f"Current WM is {active}, not i3. Re-run the Mackes "
                "setup wizard to switch."
            ), True, True, 0)
            btn = Gtk.Button(label="Run wizard")
            btn.get_style_context().add_class("suggested-action")
            btn.connect("clicked", lambda *_: subprocess.Popen(
                ["mackes", "--wizard"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            ))
            a11y(btn,
                 name="Launch the Mackes Setup Wizard to switch to i3",
                 tooltip="Open the Mackes wizard to re-run the WM migration")
            banner.pack_end(btn, False, False, 0)
            box.pack_start(banner, False, False, 0)

        # ---- i3 body (always rendered now) --------------------------
        box.pack_start(self._build_i3_body(), True, True, 0)

        return box

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

        # 1.1.0 — Gaps profile picker (5 popular configs, user lock
        # 2026-05-19). Lives below the Layouts grid since it's a more
        # persistent decision than a single-shot layout apply.
        body.pack_start(section_header("Gaps"), False, False, 0)
        body.pack_start(info_label(
            "Pick a gaps profile — applies immediately via i3-msg reload. "
            "Picks land in ~/.config/i3/config.d/mackes-gaps.conf so "
            "they survive `mackes-wm reset`."
        ), False, False, 0)
        body.pack_start(self._build_gaps_grid(), False, False, 0)

        # Helpful footer with the reload + reset actions.
        body.pack_start(section_header("Config"), False, False, 0)
        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        reload_btn = Gtk.Button(label="Reload i3 config")
        reload_btn.connect("clicked", lambda *_: _i3_msg("reload"))
        a11y(reload_btn, name="Reload i3 window-manager configuration",
             tooltip="Tell i3 to reload its config (i3-msg reload)")
        actions.pack_start(reload_btn, False, False, 0)
        reset_btn = Gtk.Button(label="Reset to Mackes default")
        reset_btn.connect("clicked", self._on_reset_i3_config)
        a11y(reset_btn, name="Reset i3 configuration to the Mackes default",
             tooltip="Restore the shipped Mackes i3 config — clears user overrides")
        actions.pack_start(reset_btn, False, False, 0)
        body.pack_start(actions, False, False, 0)

        return body

    def _build_gaps_grid(self) -> Gtk.Widget:
        """5-tile grid for the 1.1.0 gaps profiles."""
        from mackes import i3_gaps as _gaps

        current = _gaps.detect_current()
        grid = Gtk.Grid()
        grid.set_row_spacing(8)
        grid.set_column_spacing(8)
        grid.set_margin_top(8)
        grid.set_margin_bottom(8)
        grid.set_column_homogeneous(True)
        for i, profile in enumerate(_gaps.PROFILES):
            grid.attach(
                self._gaps_tile(profile, is_current=(profile.key == current)),
                i % 3, i // 3, 1, 1,
            )
        return grid

    def _gaps_tile(self, profile, is_current: bool) -> Gtk.Widget:
        from mackes import i3_gaps as _gaps

        btn = Gtk.Button()
        btn.set_relief(Gtk.ReliefStyle.NORMAL)
        btn.set_tooltip_text(
            f"{profile.description}\n"
            f"inner={profile.inner}px · outer={profile.outer}px"
        )
        if is_current:
            btn.get_style_context().add_class("suggested-action")
        col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        col.set_margin_top(8); col.set_margin_bottom(8)
        col.set_margin_start(12); col.set_margin_end(12)
        title = Gtk.Label(label=profile.label)
        title.get_style_context().add_class("mackes-tile-title")
        title.set_xalign(0)
        col.pack_start(title, False, False, 0)
        sub = Gtk.Label(label=f"{profile.inner}px inner · {profile.outer}px outer")
        sub.get_style_context().add_class("mackes-tile-sub")
        sub.set_xalign(0)
        col.pack_start(sub, False, False, 0)
        btn.add(col)
        key = profile.key
        btn.connect("clicked", lambda *_: (
            _gaps.apply_profile(key), self._render(),
        ))
        return btn

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
    # xfwm4 panel removed in Phase 8.8 (1.0.7) — i3 is the only WM.
    # The xfwm4 theme/focus/title-bar settings are no longer
    # authoritative. CHANNEL / FOCUS_MODES / TITLE_LAYOUTS constants
    # below are kept as references for any future inspection-only
    # surface (not currently rendered).
    # ----------------------------------------------------------------
