"""Wizard screen 4 — Appearance (live preview).

The preset's appearance values pre-populate the controls. Edits are stored
into ctx.overrides['appearance']; the Apply page merges them on top of the
preset.
"""
from __future__ import annotations

from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.xfconf_bridge import XfconfError, get_bridge
from mackes.workbench._common import labeled_row, section_header


def _discover(names_dir: str, marker_subdir: str) -> list[str]:
    seen: set[str] = set()
    for root in (Path("/usr/share") / names_dir, Path.home() / f".{names_dir}"):
        if not root.is_dir():
            continue
        for entry in root.iterdir():
            if (entry / marker_subdir).is_dir():
                seen.add(entry.name)
    return sorted(seen) or ["Adwaita"]


def build(ctx) -> Gtk.Widget:
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
    box.set_margin_top(28); box.set_margin_bottom(28)
    box.set_margin_start(40); box.set_margin_end(40)

    title = Gtk.Label(label="Appearance")
    title.set_xalign(0); title.get_style_context().add_class("title-1")
    box.pack_start(title, False, False, 0)

    blurb = Gtk.Label(label=(
        "Preset defaults are shown. Changes write through immediately so "
        "you can preview, and are also stored as overrides for the final apply."
    ))
    blurb.set_xalign(0); blurb.set_line_wrap(True)
    blurb.get_style_context().add_class("dim-label")
    box.pack_start(blurb, False, False, 0)

    try:
        xf = get_bridge()
    except XfconfError as e:
        err = Gtk.Label(label=str(e))
        err.get_style_context().add_class("error")
        box.pack_start(err, False, False, 0)
        return box

    overrides = ctx.overrides.setdefault("appearance", {})
    preset_app = ctx.selected_preset.appearance if ctx.selected_preset else {}

    def record(field: str, value):
        overrides[field] = value

    box.pack_start(section_header("Theme"), False, False, 0)
    themes = _discover("themes", "gtk-3.0")
    theme_combo = Gtk.ComboBoxText()
    for t in themes:
        theme_combo.append_text(t)
    initial = preset_app.get("gtk_theme") or xf.get("xsettings", "/Net/ThemeName", "Adwaita")
    if initial in themes:
        theme_combo.set_active(themes.index(initial))
    else:
        themes.append(str(initial)); theme_combo.append_text(str(initial))
        theme_combo.set_active(len(themes) - 1)
    def on_theme(c):
        txt = c.get_active_text()
        if txt:
            xf.set("xsettings", "/Net/ThemeName", txt, type_hint="string")
            record("gtk_theme", txt)
    theme_combo.connect("changed", on_theme)
    box.pack_start(labeled_row("GTK theme", theme_combo), False, False, 0)

    box.pack_start(section_header("Icons"), False, False, 0)
    icons = _discover("icons", "")  # any /usr/share/icons/<name> dir
    icons = [i for i in (Path("/usr/share/icons").iterdir() if Path("/usr/share/icons").is_dir() else [])
             if i.is_dir()]
    icon_names = sorted({i.name for i in icons}) or ["Adwaita"]
    icon_combo = Gtk.ComboBoxText()
    for i in icon_names:
        icon_combo.append_text(i)
    initial = preset_app.get("icon_theme") or xf.get("xsettings", "/Net/IconThemeName", "Adwaita")
    if initial in icon_names:
        icon_combo.set_active(icon_names.index(initial))
    else:
        icon_names.append(str(initial)); icon_combo.append_text(str(initial))
        icon_combo.set_active(len(icon_names) - 1)
    def on_icon(c):
        txt = c.get_active_text()
        if txt:
            xf.set("xsettings", "/Net/IconThemeName", txt, type_hint="string")
            record("icon_theme", txt)
    icon_combo.connect("changed", on_icon)
    box.pack_start(labeled_row("Icon theme", icon_combo), False, False, 0)

    box.pack_start(section_header("Fonts"), False, False, 0)
    ui_font = Gtk.FontButton()
    ui_font.set_font_name(str(preset_app.get("font_ui")
                              or xf.get("xsettings", "/Gtk/FontName", "Droid Sans 10")))
    def on_ui(b):
        xf.set("xsettings", "/Gtk/FontName", b.get_font_name(), type_hint="string")
        record("font_ui", b.get_font_name())
    ui_font.connect("font-set", on_ui)
    box.pack_start(labeled_row("Interface font", ui_font), False, False, 0)

    mono_font = Gtk.FontButton()
    mono_font.set_filter_func(lambda family, _f: family.is_monospace())
    mono_font.set_font_name(str(preset_app.get("font_monospace")
                                or xf.get("xsettings", "/Gtk/MonospaceFontName", "JetBrains Mono 10")))
    def on_mono(b):
        xf.set("xsettings", "/Gtk/MonospaceFontName", b.get_font_name(), type_hint="string")
        record("font_monospace", b.get_font_name())
    mono_font.connect("font-set", on_mono)
    box.pack_start(labeled_row("Monospace font", mono_font), False, False, 0)

    return box
