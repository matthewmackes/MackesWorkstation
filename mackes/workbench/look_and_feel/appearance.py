"""Look & Feel → Appearance.

Reference panel for the architecture. Shows:
  - Q13 lock: one unified Appearance panel with internal sections
  - Q16 lock: panels bind directly to xfconf keys via xfconf_bridge
  - Q9  lock: immediate apply — every widget change writes through

xfconf channels used:
  xsettings  /Net/ThemeName        GTK theme name
  xsettings  /Net/IconThemeName    Icon theme
  xsettings  /Gtk/CursorThemeName  Cursor theme
  xsettings  /Gtk/CursorThemeSize  Cursor size
  xsettings  /Gtk/FontName         UI font (e.g. "Droid Sans 10")
  xsettings  /Gtk/MonospaceFontName  Monospace font

Wallpaper lives in the xfce4-desktop channel; per-monitor and per-workspace
properties make the key path dynamic (see _wallpaper_section).

Theme discovery scans /usr/share/themes (and ~/.themes) for entries with a
gtk-3.0 subdirectory, /usr/share/icons (and ~/.icons) for entries with an
index.theme file, and /usr/share/icons for entries with a cursors subdir.
"""
from __future__ import annotations

from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import get_bridge, XfconfError
from mackes.workbench._common import (
    error_label, info_label, labeled_row, section_header, title_label,
)


def _discover_gtk_themes() -> list[str]:
    seen: set[str] = set()
    for root in (Path("/usr/share/themes"), Path("/usr/local/share/themes"),
                 Path.home() / ".themes"):
        if not root.is_dir():
            continue
        for entry in root.iterdir():
            if (entry / "gtk-3.0").is_dir():
                seen.add(entry.name)
    return sorted(seen) or ["Adwaita"]


def _discover_icon_themes() -> list[str]:
    seen: set[str] = set()
    for root in (Path("/usr/share/icons"), Path("/usr/local/share/icons"),
                 Path.home() / ".icons", Path.home() / ".local/share/icons"):
        if not root.is_dir():
            continue
        for entry in root.iterdir():
            if (entry / "index.theme").exists():
                seen.add(entry.name)
    return sorted(seen) or ["Adwaita"]


def _discover_cursor_themes() -> list[str]:
    seen: set[str] = set()
    for root in (Path("/usr/share/icons"), Path("/usr/local/share/icons"),
                 Path.home() / ".icons", Path.home() / ".local/share/icons"):
        if not root.is_dir():
            continue
        for entry in root.iterdir():
            if (entry / "cursors").is_dir() or (entry / "cursor.theme").exists():
                seen.add(entry.name)
    return sorted(seen) or ["Adwaita"]


def _list_monitors() -> list[str]:
    """Best-effort list of connected monitor names from xrandr."""
    import subprocess
    try:
        out = subprocess.check_output(["xrandr", "--query"], text=True,
                                      stderr=subprocess.DEVNULL, timeout=4)
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return ["monitor0"]
    mons = []
    for line in out.splitlines():
        if " connected" in line:
            mons.append(line.split(" ", 1)[0])
    return mons or ["monitor0"]


class AppearancePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        self.set_margin_top(24); self.set_margin_bottom(24)
        self.set_margin_start(28); self.set_margin_end(28)

        try:
            self.xf = get_bridge()
        except XfconfError as e:
            self.pack_start(error_label(str(e)), False, False, 0)
            return

        self.pack_start(title_label("Appearance"), False, False, 0)
        self.pack_start(info_label(
            "Theme, icons, cursor, fonts, and wallpaper — all backed by xfconf. "
            "Changes apply immediately."
        ), False, False, 0)

        self.pack_start(self._theme_section(), False, False, 0)
        self.pack_start(self._icons_section(), False, False, 0)
        self.pack_start(self._cursor_section(), False, False, 0)
        self.pack_start(self._fonts_section(), False, False, 0)
        self.pack_start(self._antialiasing_section(), False, False, 0)
        self.pack_start(self._wallpaper_section(), False, False, 0)

    # ---- Theme ------------------------------------------------------------

    def _theme_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Theme"), False, False, 0)

        themes = _discover_gtk_themes()
        combo = Gtk.ComboBoxText()
        for t in themes:
            combo.append_text(t)
        self.xf.bind_combo(combo, "xsettings", "/Net/ThemeName", themes,
                           "Adwaita" if "Adwaita" in themes else themes[0])
        box.pack_start(labeled_row("GTK theme", combo), False, False, 0)

        dark = Gtk.Switch()
        dark.set_active(bool(self.xf.get("xsettings", "/Net/ThemeName", "")).__class__.__name__ != ""
                        and "dark" in str(self.xf.get("xsettings", "/Net/ThemeName", "")).lower())
        # Actually wire to /Settings/Gtk/ApplicationPreferDarkTheme
        dark.set_active(bool(self.xf.get("xsettings", "/Gtk/ApplicationPreferDarkTheme", False)))
        def on_dark(s, _g):
            self.xf.set("xsettings", "/Gtk/ApplicationPreferDarkTheme", s.get_active())
        dark.connect("notify::active", on_dark)
        box.pack_start(labeled_row("Prefer dark variant", dark), False, False, 0)

        return box

    # ---- Icons ------------------------------------------------------------

    def _icons_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Icons"), False, False, 0)

        icons = _discover_icon_themes()
        combo = Gtk.ComboBoxText()
        for t in icons:
            combo.append_text(t)
        self.xf.bind_combo(combo, "xsettings", "/Net/IconThemeName", icons,
                           "Adwaita" if "Adwaita" in icons else icons[0])
        box.pack_start(labeled_row("Icon theme", combo), False, False, 0)
        return box

    # ---- Cursor -----------------------------------------------------------

    def _cursor_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Cursor"), False, False, 0)

        cursors = _discover_cursor_themes()
        combo = Gtk.ComboBoxText()
        for t in cursors:
            combo.append_text(t)
        self.xf.bind_combo(combo, "xsettings", "/Gtk/CursorThemeName", cursors,
                           "Adwaita" if "Adwaita" in cursors else cursors[0])
        box.pack_start(labeled_row("Cursor theme", combo), False, False, 0)

        spin = Gtk.SpinButton.new_with_range(16, 96, 4)
        self.xf.bind_spin(spin, "xsettings", "/Gtk/CursorThemeSize", 24)
        box.pack_start(labeled_row("Cursor size", spin), False, False, 0)
        return box

    # ---- Fonts ------------------------------------------------------------

    def _fonts_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Fonts"), False, False, 0)

        ui = Gtk.FontButton()
        self.xf.bind_font(ui, "xsettings", "/Gtk/FontName", "Droid Sans 10")
        box.pack_start(labeled_row("Interface", ui), False, False, 0)

        mono = Gtk.FontButton()
        mono.set_filter_func(lambda family, _face: family.is_monospace())
        self.xf.bind_font(mono, "xsettings", "/Gtk/MonospaceFontName", "JetBrains Mono 10")
        box.pack_start(labeled_row("Monospace", mono), False, False, 0)

        return box

    # ---- Antialiasing / hinting ------------------------------------------

    def _antialiasing_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Font rendering"), False, False, 0)

        aa = Gtk.Switch()
        aa.set_active(bool(self.xf.get("xsettings", "/Xft/Antialias", 1)))
        aa.connect("notify::active",
                   lambda s, _g: self.xf.set("xsettings", "/Xft/Antialias",
                                              1 if s.get_active() else 0))
        box.pack_start(labeled_row("Antialiasing", aa), False, False, 0)

        HINTING = ["none", "slight", "medium", "full"]
        hinting = Gtk.ComboBoxText()
        for h in HINTING:
            hinting.append_text(h)
        self.xf.bind_combo(hinting, "xsettings", "/Xft/HintStyle", HINTING, "slight")
        box.pack_start(labeled_row("Hinting", hinting), False, False, 0)

        RGBA = ["none", "rgb", "bgr", "vrgb", "vbgr"]
        rgba = Gtk.ComboBoxText()
        for r in RGBA:
            rgba.append_text(r)
        self.xf.bind_combo(rgba, "xsettings", "/Xft/RGBA", RGBA, "rgb")
        box.pack_start(labeled_row("Sub-pixel order", rgba), False, False, 0)

        return box

    # ---- Wallpaper --------------------------------------------------------

    def _wallpaper_section(self) -> Gtk.Widget:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.pack_start(section_header("Wallpaper"), False, False, 0)

        monitors = _list_monitors()
        monitor_combo = Gtk.ComboBoxText()
        for m in monitors:
            monitor_combo.append_text(m)
        monitor_combo.set_active(0)
        box.pack_start(labeled_row("Monitor", monitor_combo), False, False, 0)

        chooser = Gtk.FileChooserButton(title="Wallpaper", action=Gtk.FileChooserAction.OPEN)
        filt = Gtk.FileFilter()
        filt.set_name("Images")
        for ext in ("png", "jpg", "jpeg", "webp", "svg"):
            filt.add_pattern(f"*.{ext}")
        chooser.add_filter(filt)

        STYLES = ["0 — None", "1 — Centered", "2 — Tiled",
                  "3 — Stretched", "4 — Scaled", "5 — Zoomed"]
        style_combo = Gtk.ComboBoxText()
        for s in STYLES:
            style_combo.append_text(s)

        def key_for(prop: str) -> str:
            idx = monitor_combo.get_active()
            mon = monitors[idx] if idx >= 0 else "monitor0"
            return f"/backdrop/screen0/{mon}/workspace0/{prop}"

        def refresh():
            current = self.xf.get("xfce4-desktop", key_for("last-image"), "")
            if current and Path(str(current)).exists():
                chooser.set_filename(str(current))
            style = int(self.xf.get("xfce4-desktop", key_for("image-style"), 5) or 5)
            style_combo.set_active(min(max(style, 0), len(STYLES) - 1))

        def on_monitor_changed(_):
            refresh()
        monitor_combo.connect("changed", on_monitor_changed)
        refresh()

        def on_set(b):
            f = b.get_filename()
            if f:
                self.xf.set("xfce4-desktop", key_for("last-image"), f, type_hint="string")
        chooser.connect("file-set", on_set)
        box.pack_start(labeled_row("Image", chooser), False, False, 0)

        def on_style(c):
            i = c.get_active()
            if i >= 0:
                self.xf.set("xfce4-desktop", key_for("image-style"), int(i))
        style_combo.connect("changed", on_style)
        box.pack_start(labeled_row("Style", style_combo), False, False, 0)

        return box
