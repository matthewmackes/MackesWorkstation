"""v2.0.0 Phase F.4 — Displays panel through `mde_settings_bridge`.

Replaces the v1.x xrandr-only Displays panel. Reads the live output
list from `swaymsg -t get_outputs` (the compositor-aware path) and
writes user-facing preferences through `mde_settings_bridge`.

Per the MDE schema (mackes/mde_settings_bridge.py:160-163):

  display.primary           → sidecar `display.json` `primary`     (output-name string)
  display.scale             → sidecar `display.json` `scale`       (float)
  display.night_light       → sidecar `display.json` `night_light` (bool)
  display.night_light_temp  → sidecar `display.json` `night_light_temp` (int Kelvin)

Brightness (display.brightness via brightnessctl) is handled by a
separate worker in `crates/mackesd/src/workers/`; this panel just
surfaces the user-facing preferences. The detection of connected
outputs uses `mackes.sway_ipc.get_outputs()` so the panel survives
on a TTY (returns an empty state) and on a non-sway compositor
(same — empty state).
"""
from __future__ import annotations

from typing import List

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as bridge
from mackes import sway_ipc
from mackes.workbench._common import (
    empty_state, labeled_row, panel_box, section_header, title_label,
)


def _output_names() -> List[str]:
    """Names of every connected output, in the order swaymsg
    reports them. Returns `[]` if sway isn't running."""
    return [
        o.get("name", "")
        for o in sway_ipc.get_outputs()
        if o.get("active", True) and o.get("name")
    ]


def _primary_combo(outputs: List[str], current: str) -> Gtk.ComboBoxText:
    combo = Gtk.ComboBoxText()
    for o in outputs:
        combo.append_text(o)
    if current in outputs:
        combo.set_active(outputs.index(current))
    elif outputs:
        combo.set_active(0)
    combo.connect(
        "changed",
        lambda c: bridge.set_setting(
            "display.primary",
            c.get_active_text() or outputs[0] if outputs else "",
        ),
    )
    return combo


def _scale_spin(current: float) -> Gtk.SpinButton:
    adj = Gtk.Adjustment(value=current, lower=0.5, upper=4.0,
                         step_increment=0.25, page_increment=0.5)
    spin = Gtk.SpinButton(adjustment=adj, digits=2, climb_rate=0.25)
    spin.connect(
        "value-changed",
        lambda s: bridge.set_setting("display.scale", float(s.get_value())),
    )
    return spin


def _night_light_switch(initial: bool) -> Gtk.Switch:
    sw = Gtk.Switch()
    sw.set_active(initial)
    sw.connect(
        "notify::active",
        lambda s, _g: bridge.set_setting("display.night_light",
                                         bool(s.get_active())),
    )
    return sw


def _night_light_temp_spin(initial: int) -> Gtk.SpinButton:
    adj = Gtk.Adjustment(value=initial, lower=1000, upper=10000,
                         step_increment=100, page_increment=500)
    spin = Gtk.SpinButton(adjustment=adj, digits=0)
    spin.connect(
        "value-changed",
        lambda s: bridge.set_setting("display.night_light_temp",
                                     int(s.get_value())),
    )
    return spin


class DisplaysPanel(Gtk.Box):
    """MDE Displays panel — four controls (primary / scale /
    night-light on/off / night-light temp). Reads `display.*` keys
    through the bridge; enumerates outputs through sway IPC."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer = panel_box()
        outer.pack_start(title_label("Displays"), False, False, 0)

        outputs = _output_names()
        if not outputs:
            outer.pack_start(
                empty_state(
                    "No displays detected",
                    "MDE reads displays from sway. If you're on a TTY or a "
                    "different compositor, this panel won't have outputs to "
                    "configure.",
                ),
                True, True, 0,
            )
            self.pack_start(outer, True, True, 0)
            return

        outer.pack_start(section_header("Primary"), False, False, 0)
        current = str(bridge.get_setting("display.primary") or "")
        outer.pack_start(
            labeled_row("Primary display",
                        _primary_combo(outputs, current)),
            False, False, 0,
        )

        outer.pack_start(section_header("Scale"), False, False, 0)
        current_scale = bridge.get_setting("display.scale")
        try:
            current_scale = float(current_scale) if current_scale else 1.0
        except (TypeError, ValueError):
            current_scale = 1.0
        outer.pack_start(
            labeled_row("Scale factor", _scale_spin(current_scale)),
            False, False, 0,
        )

        outer.pack_start(section_header("Night light"), False, False, 0)
        nl_on = bool(bridge.get_setting("display.night_light"))
        outer.pack_start(
            labeled_row("Enable night light", _night_light_switch(nl_on)),
            False, False, 0,
        )
        nl_temp = bridge.get_setting("display.night_light_temp")
        try:
            nl_temp = int(nl_temp) if nl_temp else 4500
        except (TypeError, ValueError):
            nl_temp = 4500
        outer.pack_start(
            labeled_row("Color temperature (K)",
                        _night_light_temp_spin(nl_temp)),
            False, False, 0,
        )

        self.pack_start(outer, True, True, 0)
