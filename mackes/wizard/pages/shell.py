"""Wizard screen 5 — Shell Layout (live preview)."""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.shell_profiles import (
    apply_plank, apply_polybar, apply_rofi,
    list_plank_profiles, list_polybar_profiles, list_rofi_profiles,
)
from mackes.workbench._common import labeled_row, section_header


def build(ctx) -> Gtk.Widget:
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
    box.set_margin_top(28); box.set_margin_bottom(28)
    box.set_margin_start(40); box.set_margin_end(40)

    title = Gtk.Label(label="Shell Layout")
    title.set_xalign(0); title.get_style_context().add_class("title-1")
    box.pack_start(title, False, False, 0)

    blurb = Gtk.Label(label=(
        "Polybar / Plank / Rofi profiles. Each profile is a config-file blob "
        "we ship; selecting applies it live."
    ))
    blurb.set_xalign(0); blurb.set_line_wrap(True)
    blurb.get_style_context().add_class("dim-label")
    box.pack_start(blurb, False, False, 0)

    overrides = ctx.overrides.setdefault("shell", {})
    preset_shell = ctx.selected_preset.shell if ctx.selected_preset else {}

    def _profile_combo(label, key, lister, applier, preset_key):
        profiles = lister()
        combo = Gtk.ComboBoxText()
        for p in profiles:
            combo.append_text(p)
        initial = preset_shell.get(preset_key)
        if initial in profiles:
            combo.set_active(profiles.index(initial))
        elif profiles:
            combo.set_active(0)
        def on_changed(c):
            chosen = c.get_active_text()
            if chosen:
                applier(chosen)
                overrides[preset_key] = chosen
        combo.connect("changed", on_changed)
        box.pack_start(section_header(label), False, False, 0)
        box.pack_start(labeled_row("Profile", combo), False, False, 0)

    _profile_combo("Polybar", "polybar", list_polybar_profiles, apply_polybar, "polybar_profile")
    _profile_combo("Plank", "plank", list_plank_profiles, apply_plank, "plank_profile")
    _profile_combo("Rofi", "rofi", list_rofi_profiles, apply_rofi, "rofi_profile")

    box.pack_start(section_header("XFCE Panel"), False, False, 0)
    sw = Gtk.Switch()
    sw.set_active(bool(preset_shell.get("xfce_panel_enabled", False)))
    def on_xfce(s, _g):
        from mackes.shell_profiles import set_xfce_panel_enabled
        set_xfce_panel_enabled(s.get_active())
        overrides["xfce_panel_enabled"] = s.get_active()
    sw.connect("notify::active", on_xfce)
    box.pack_start(labeled_row("xfce4-panel autostart", sw), False, False, 0)

    return box
