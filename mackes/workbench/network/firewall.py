"""Network → Firewall (firewalld via firewall-cmd).

1.0.7+ (Phase 11.9): every `firewall-cmd` probe is now off-main-thread.
The panel renders a skeleton on construct, then fills in via
`async_probe` from `mackes.workbench._async`. firewalld being slow or
absent never blocks the Workbench main loop.
"""
from __future__ import annotations

import subprocess
from dataclasses import dataclass

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._async import async_probe
from mackes.workbench._common import (
    info_label, labeled_row, panel_box, section_description, section_header, title_label,
)


# Shorter timeout than before (8 s → 2 s). The smoke test caught
# `firewall-cmd --list-all` hanging for ≥5 s when firewalld is down;
# 2 s is enough to gather state when the daemon is alive and gives up
# fast otherwise.
_FW_TIMEOUT_S = 2


def _fw(*args: str) -> str:
    try:
        return subprocess.check_output(
            ["firewall-cmd", *args],
            text=True,
            stderr=subprocess.STDOUT,
            timeout=_FW_TIMEOUT_S,
        ).strip()
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
        return getattr(e, "output", "") or ""


def _zones() -> list[str]:
    out = _fw("--get-zones")
    return out.split() if out else []


def _default_zone() -> str:
    return _fw("--get-default-zone")


def _enabled_services() -> list[str]:
    out = _fw("--list-services")
    return out.split() if out else []


def _set_default_zone(zone: str) -> str:
    msg = _fw("--set-default-zone", zone)
    log_action(f"firewall: default zone -> {zone}")
    return msg


def _toggle_service(service: str, enable: bool) -> str:
    flag = "--add-service" if enable else "--remove-service"
    msg = _fw(flag, service, "--permanent")
    _fw("--reload")
    log_action(f"firewall: {'enabled' if enable else 'disabled'} {service}")
    return msg


COMMON_SERVICES = ["ssh", "http", "https", "samba", "samba-client", "mdns", "dhcpv6-client"]


@dataclass(frozen=True)
class _FirewallState:
    """Snapshot of firewalld state gathered off the main thread."""
    has_fw_cmd: bool
    zones: list[str]
    default_zone: str
    enabled_services: set[str]


def _gather_firewall_state() -> _FirewallState:
    """Off-main-thread: every firewall-cmd probe in one place. Each one
    has its own 2 s timeout; failure modes are surfaced via empty
    fields, not exceptions."""
    has = bool(_fw("--version"))
    if not has:
        return _FirewallState(
            has_fw_cmd=False, zones=[], default_zone="", enabled_services=set(),
        )
    return _FirewallState(
        has_fw_cmd=True,
        zones=_zones(),
        default_zone=_default_zone(),
        enabled_services=set(_enabled_services()),
    )


class FirewallPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._build_skeleton()
        async_probe(_gather_firewall_state, self._apply_state)

    def _build_skeleton(self) -> None:
        """Sync — never touches firewall-cmd. Renders headings + empty
        containers that `_apply_state` populates once the probe lands."""
        box = panel_box()
        box.pack_start(title_label("Firewall"), False, False, 0)
        box.pack_start(info_label(
            "Control what other computers are allowed to reach on this "
            "machine. The firewall blocks everything by default — flip "
            "on only the services you trust."
        ), False, False, 0)
        box.pack_start(section_description(
            "Changes need an admin password. If unsure, leave the "
            "default settings — they're safe for most home networks."
        ), False, False, 0)

        # The "loading" placeholder gets replaced by the real content
        # in _apply_state. Keeping it as a child of `self` (not a
        # local) so we can find + remove it.
        self._loading = info_label("Checking firewalld…")
        box.pack_start(self._loading, False, False, 0)

        self._content_root = box
        # Slots for content the probe fills in:
        self._zone_section: Gtk.Widget | None = None
        self._service_section: Gtk.Widget | None = None
        self._service_box: Gtk.Box | None = None

        self.add(box)

    def _apply_state(self, state: _FirewallState) -> None:
        """Main thread — safe to touch widgets."""
        self._content_root.remove(self._loading)

        if not state.has_fw_cmd:
            self._content_root.pack_start(
                info_label("firewall-cmd not available — install firewalld."),
                False, False, 0,
            )
            self._content_root.show_all()
            return

        # Default-zone selector.
        self._content_root.pack_start(
            section_header("Default zone"), False, False, 0,
        )
        zones = state.zones or ["public"]
        zone_combo = Gtk.ComboBoxText()
        for z in zones:
            zone_combo.append_text(z)
        cur = state.default_zone
        zone_combo.set_active(zones.index(cur) if cur in zones else 0)

        def on_zone(combo: Gtk.ComboBoxText) -> None:
            txt = combo.get_active_text()
            if txt:
                # Mutator + refresh — both off-main-thread.
                async_probe(
                    lambda: (_set_default_zone(txt), _gather_firewall_state())[1],
                    self._reapply_state,
                )

        zone_combo.connect("changed", on_zone)
        self._content_root.pack_start(
            labeled_row("Active default zone", zone_combo), False, False, 0,
        )

        self._content_root.pack_start(
            section_header("Enabled services (current zone)"), False, False, 0,
        )
        self._service_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        self._content_root.pack_start(self._service_box, False, False, 0)
        self._fill_services(state.enabled_services)

        self._content_root.show_all()

    def _reapply_state(self, state: _FirewallState) -> None:
        """A mutator just landed; re-render just the service rows."""
        if self._service_box is None:
            return
        self._fill_services(state.enabled_services)

    def _fill_services(self, enabled: set[str]) -> None:
        assert self._service_box is not None
        for child in list(self._service_box.get_children()):
            self._service_box.remove(child)

        for svc in COMMON_SERVICES:
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            lbl = Gtk.Label(label=svc)
            lbl.set_xalign(0)
            lbl.set_size_request(180, -1)
            row.pack_start(lbl, False, False, 0)
            sw = Gtk.Switch()
            sw.set_active(svc in enabled)

            def _on_toggle(switch: Gtk.Switch, _g: object, name: str = svc) -> None:
                enable = switch.get_active()
                # Don't block — toggle off-main-thread and refresh
                # service state when the dust settles.
                async_probe(
                    lambda: (_toggle_service(name, enable), _gather_firewall_state())[1],
                    self._reapply_state,
                )

            sw.connect("notify::active", _on_toggle)
            row.pack_start(sw, False, False, 0)
            self._service_box.pack_start(row, False, False, 0)
        self._service_box.show_all()

    # Legacy entry point — sidebar code may still call this. Forward to
    # the probe path so callers keep working without main-thread blocks.
    def _refresh_services(self) -> bool:
        async_probe(_gather_firewall_state, self._apply_state)
        return GLib.SOURCE_REMOVE
