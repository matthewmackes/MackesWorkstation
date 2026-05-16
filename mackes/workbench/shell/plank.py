"""Shell → Plank.

Full Plank control surface. Three sections:

  * Profile     — pick a Mackes-shipped dock profile (.dock blob → settings file)
  * Theme       — pick a Plank theme. Mackes ships the entire
                  erikdubois/plankthemes catalog plus Plank's own built-ins.
                  Selecting a shipped theme installs it under
                  ~/.local/share/plank/themes/ on demand.
  * Live keys   — every other net.launchpad.plank.docks.dock1 GSettings key.
                  Each row is bound via `gsettings get`/`gsettings set` so the
                  running Plank picks it up instantly.

Q12 lock said "preset picker only" originally — that was the v0.1 scope
scrubbing. Users explicitly want full control, so we expose it. Profile
remains the headline action because it sets a coherent layout at once.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.shell_profiles import (
    _have_gsettings,
    apply_plank,
    apply_plank_theme,
    current_plank_profile,
    gsettings_get,
    gsettings_set,
    install_shipped_plank_themes,
    list_plank_profiles,
    list_plank_themes,
)
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_header, title_label,
)


# Enum-valued keys (gsettings strings, exact spelling matters)
POSITIONS = ["top", "bottom", "left", "right"]
ALIGNMENTS = ["panel-mode", "right", "left", "center"]
ITEMS_ALIGNMENTS = ["center", "fill", "start", "end"]
HIDE_MODES = ["none", "intelligent", "auto", "window-dodge", "universal", "window"]


class PlankPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Plank"), False, False, 0)
        box.pack_start(info_label(
            "Plank dock — profile, theme, and every live GSettings key. "
            "Changes apply immediately to the running dock."
        ), False, False, 0)

        self._status = Gtk.Label(label=""); self._status.set_xalign(0)
        self._status.get_style_context().add_class("dim-label")

        # ---- Profile -----------------------------------------------------
        box.pack_start(section_header("Profile"), False, False, 0)
        profiles = list_plank_profiles()
        if profiles:
            combo = Gtk.ComboBoxText()
            for p in profiles:
                combo.append_text(p)
            active = current_plank_profile()
            combo.set_active(profiles.index(active) if active in profiles else 0)
            def on_profile(c):
                chosen = c.get_active_text()
                if chosen:
                    actions = apply_plank(chosen)
                    self._status.set_text(actions[-1] if actions else "")
            combo.connect("changed", on_profile)
            box.pack_start(labeled_row("Mackes profile", combo), False, False, 0)
        else:
            box.pack_start(info_label("No Plank profiles shipped."), False, False, 0)

        # ---- Theme -------------------------------------------------------
        box.pack_start(section_header("Theme"), False, False, 0)

        themes = list_plank_themes()
        theme_combo = Gtk.ComboBoxText()
        for t in themes:
            theme_combo.append_text(t)
        cur_theme = gsettings_get("theme") or "Default"
        if cur_theme not in themes:
            theme_combo.append_text(cur_theme)
            themes_list = themes + [cur_theme]
            theme_combo.set_active(len(themes_list) - 1)
        else:
            theme_combo.set_active(themes.index(cur_theme))

        def on_theme(c):
            txt = c.get_active_text()
            if txt:
                actions = apply_plank_theme(txt)
                self._status.set_text(actions[-1] if actions else "")
        theme_combo.connect("changed", on_theme)
        box.pack_start(labeled_row("Active theme", theme_combo), False, False, 0)

        install_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        install_all = Gtk.Button(label="Install all shipped themes to ~/.local/share/plank/themes")
        def on_install_all(_):
            actions = install_shipped_plank_themes()
            self._status.set_text(f"{len(actions)} theme actions; last: {actions[-1]}")
        install_all.connect("clicked", on_install_all)
        install_row.pack_start(install_all, False, False, 0)
        box.pack_start(install_row, False, False, 0)
        box.pack_start(info_label(
            f"{len(themes)} themes available (includes the erikdubois/plankthemes catalog)."
        ), False, False, 0)

        # ---- Live GSettings keys -----------------------------------------
        if not _have_gsettings():
            box.pack_start(info_label("gsettings not installed — live controls disabled."),
                           False, False, 0)
            box.pack_start(self._status, False, False, 0)
            self.add(box); return

        box.pack_start(section_header("Layout"), False, False, 0)
        box.pack_start(labeled_row("Position",
                                   self._enum_combo("position", POSITIONS, "bottom")),
                       False, False, 0)
        box.pack_start(labeled_row("Alignment on screen edge",
                                   self._enum_combo("alignment", ALIGNMENTS, "center")),
                       False, False, 0)
        box.pack_start(labeled_row("Items alignment",
                                   self._enum_combo("items-alignment", ITEMS_ALIGNMENTS, "center")),
                       False, False, 0)
        box.pack_start(labeled_row("Icon size (px)",
                                   self._int_spin("icon-size", 16, 128, 48)),
                       False, False, 0)
        box.pack_start(labeled_row("Offset (% from anchor)",
                                   self._int_spin("offset", -100, 100, 0)),
                       False, False, 0)
        box.pack_start(labeled_row("Monitor (blank = primary)",
                                   self._text_entry("monitor", "")),
                       False, False, 0)

        box.pack_start(section_header("Hiding"), False, False, 0)
        box.pack_start(labeled_row("Hide mode",
                                   self._enum_combo("hide-mode", HIDE_MODES, "intelligent")),
                       False, False, 0)
        box.pack_start(labeled_row("Unhide delay (ms)",
                                   self._uint_spin("unhide-delay", 0, 5000, 0)),
                       False, False, 0)
        box.pack_start(labeled_row("Hide delay (ms)",
                                   self._uint_spin("hide-delay", 0, 5000, 0)),
                       False, False, 0)
        box.pack_start(labeled_row("Pressure reveal",
                                   self._bool_switch("pressure-reveal", False)),
                       False, False, 0)

        box.pack_start(section_header("Behavior"), False, False, 0)
        box.pack_start(labeled_row("Pinned items only",
                                   self._bool_switch("pinned-only", False)),
                       False, False, 0)
        box.pack_start(labeled_row("Auto-pin running apps",
                                   self._bool_switch("auto-pinning", False)),
                       False, False, 0)
        box.pack_start(labeled_row("Lock items (no drag)",
                                   self._bool_switch("lock-items", False)),
                       False, False, 0)
        box.pack_start(labeled_row("Show Plank menu item",
                                   self._bool_switch("show-dock-item", False)),
                       False, False, 0)
        box.pack_start(labeled_row("Tooltips enabled",
                                   self._bool_switch("tooltips-enabled", True)),
                       False, False, 0)
        box.pack_start(labeled_row("Current workspace only",
                                   self._bool_switch("current-workspace-only", False)),
                       False, False, 0)

        box.pack_start(section_header("Zoom"), False, False, 0)
        box.pack_start(labeled_row("Zoom on hover",
                                   self._bool_switch("zoom-enabled", False)),
                       False, False, 0)
        box.pack_start(labeled_row("Zoom percent",
                                   self._uint_spin("zoom-percent", 100, 200, 150)),
                       False, False, 0)

        box.pack_start(self._status, False, False, 0)
        self.add(box)

    # ----- Live binding helpers --------------------------------------------

    def _enum_combo(self, key: str, options: list[str], default: str) -> Gtk.ComboBoxText:
        combo = Gtk.ComboBoxText()
        for o in options:
            combo.append_text(o)
        current = gsettings_get(key) or default
        combo.set_active(options.index(current) if current in options else 0)
        def on_changed(c, _k=key, _o=options):
            i = c.get_active()
            if i >= 0:
                gsettings_set(_k, _o[i])
        combo.connect("changed", on_changed)
        return combo

    def _bool_switch(self, key: str, default: bool) -> Gtk.Switch:
        sw = Gtk.Switch()
        raw = (gsettings_get(key) or "").lower()
        cur = (raw == "true") if raw in ("true", "false") else default
        sw.set_active(cur)
        def on_active(s, _g, _k=key):
            gsettings_set(_k, "true" if s.get_active() else "false")
        sw.connect("notify::active", on_active)
        return sw

    def _int_spin(self, key: str, lo: int, hi: int, default: int) -> Gtk.SpinButton:
        spin = Gtk.SpinButton.new_with_range(lo, hi, 1)
        raw = gsettings_get(key)
        try:
            spin.set_value(int(raw)) if raw is not None else spin.set_value(default)
        except ValueError:
            spin.set_value(default)
        def on_changed(s, _k=key):
            gsettings_set(_k, str(int(s.get_value())))
        spin.connect("value-changed", on_changed)
        return spin

    def _uint_spin(self, key: str, lo: int, hi: int, default: int) -> Gtk.SpinButton:
        return self._int_spin(key, max(0, lo), hi, default)

    def _text_entry(self, key: str, default: str) -> Gtk.Entry:
        entry = Gtk.Entry()
        entry.set_text(gsettings_get(key) or default)
        def commit(e, _k=key):
            gsettings_set(_k, e.get_text())
        entry.connect("activate", commit)
        entry.connect("focus-out-event", lambda e, _evt: (commit(e), False)[1])
        return entry
