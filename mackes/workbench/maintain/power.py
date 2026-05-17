"""Maintain → Power.

Power-profile selector — third tool in the MaintenanceKit. Uses
power-profiles-daemon (`powerprofilesctl`) when available; falls back to a
read-only display of `tlp-stat` info if tlp is the system's power tool.

Today's `devices/power.py` panel is a single xfconf setting for the power
manager's GUI preference. This panel goes one level deeper: live profile
switching with auto-detection of which daemon owns the surface.
"""
from __future__ import annotations

import shutil
import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


def _ppd_get_profiles() -> tuple[list[str], str | None]:
    if shutil.which("powerprofilesctl") is None:
        return [], None
    try:
        listing = subprocess.check_output(
            ["powerprofilesctl", "list"], text=True, timeout=5,
        )
        active = subprocess.check_output(
            ["powerprofilesctl", "get"], text=True, timeout=5,
        ).strip()
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError):
        return [], None
    profiles: list[str] = []
    for line in listing.splitlines():
        # ppd output: "  performance:" / "  balanced:" / etc.; bare colon-prefixed names
        line = line.strip()
        if line.endswith(":"):
            profiles.append(line[:-1].strip().lstrip("*").strip())
        elif line.startswith("* "):
            profiles.append(line[2:].rstrip(":").strip())
    profiles = [p for p in profiles if p]
    return profiles, active or None


def _ppd_set(profile: str) -> str:
    try:
        subprocess.check_output(
            ["powerprofilesctl", "set", profile],
            stderr=subprocess.STDOUT, text=True, timeout=5,
        )
        return f"set profile -> {profile}"
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as e:
        return f"set failed: {e}"


def _tlp_summary() -> str | None:
    if shutil.which("tlp-stat") is None:
        return None
    try:
        out = subprocess.check_output(
            ["tlp-stat", "-s"], text=True, timeout=5,
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as e:
        return f"tlp-stat error: {e}"
    # condense to first 8 informative lines
    interesting = [ln for ln in out.splitlines()
                   if ln and not ln.startswith(("--- ", "+++ ", "==="))]
    return "\n".join(interesting[:8])


class PowerPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build()
        GLib.idle_add(self._refresh)

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("Power"), False, False, 0)
        box.pack_start(info_label(
            "Choose how your machine balances speed and battery life. "
            "Switch to Performance when you need everything, "
            "Power-saver when you don't."
        ), False, False, 0)
        box.pack_start(section_description(
            "Most laptops should stay on Balanced. Performance can run "
            "fans harder; Power-saver may slow heavy apps."
        ), False, False, 0)

        box.pack_start(section_header("Profile"), False, False, 0)
        self._combo = Gtk.ComboBoxText()
        self._combo.connect("changed", self._on_changed)
        box.pack_start(labeled_row("Active", self._combo), False, False, 0)

        self._status = Gtk.Label(label="")
        self._status.set_xalign(0)
        self._status.get_style_context().add_class("dim-label")
        box.pack_start(self._status, False, False, 0)

        box.pack_start(section_header("tlp (if present)"), False, False, 0)
        self._tlp = Gtk.Label(label="")
        self._tlp.set_xalign(0)
        self._tlp.set_line_wrap(True)
        box.pack_start(self._tlp, False, False, 0)

        refresh = Gtk.Button(label="Refresh")
        refresh.connect("clicked", lambda *_: self._refresh())
        box.pack_start(refresh, False, False, 0)

        self.add(box)

    def _refresh(self) -> bool:
        profiles, active = _ppd_get_profiles()
        self._combo.handler_block_by_func(self._on_changed)
        self._combo.remove_all()
        if profiles:
            for p in profiles:
                self._combo.append(p, p.title())
            if active and active in profiles:
                self._combo.set_active_id(active)
            self._combo.set_sensitive(True)
            self._status.set_text(f"power-profiles-daemon: active = {active or '(unknown)'}")
        else:
            self._combo.set_sensitive(False)
            self._status.set_text(
                "power-profiles-daemon not available "
                "(install power-profiles-daemon for live switching)."
            )
        self._combo.handler_unblock_by_func(self._on_changed)

        tlp = _tlp_summary()
        self._tlp.set_text(tlp or "(tlp not installed)")
        return False

    def _on_changed(self, *_) -> None:
        profile = self._combo.get_active_id()
        if not profile:
            return
        msg = _ppd_set(profile)
        log_action(f"power: {msg}")
        self._status.set_text(msg)
