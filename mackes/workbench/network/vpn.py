"""Network → VPN.

NetworkManager VPN connection list, plus an import button for .ovpn files.
WireGuard import is `nmcli connection import type wireguard file <path>`;
OpenVPN is the same with `type openvpn`.

11.9 reliability sweep: `nmcli --version` + the initial `nmcli connection
show` were synchronous in `__init__` (~120 ms combined; much worse when
NetworkManager is starting). Both now run via
`mackes.workbench._async.async_probe`; the VPN list shows a "Loading…"
placeholder until the probe completes.
"""
from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._async import async_probe
from mackes.workbench._common import (
    a11y, empty_state, error_state, format_probe_error,
    info_label, panel_box, section_description, section_header, title_label,
)


class _NmcliError(RuntimeError):
    """nmcli ran but exited non-zero. Wrapped so the panel can show the
    real error text instead of silently rendering an empty list."""


def _nmcli(*args: str) -> str:
    """Run nmcli and return stdout. Raises ``_NmcliError`` on probe
    failure so call sites can distinguish "no VPNs" from "nmcli is
    broken" — Phase 11.5."""
    try:
        return subprocess.check_output(
            ["nmcli", *args], text=True, stderr=subprocess.STDOUT, timeout=8,
        ).strip()
    except (FileNotFoundError, subprocess.CalledProcessError,
            subprocess.TimeoutExpired) as exc:
        raise _NmcliError(format_probe_error(exc)) from exc


def _nmcli_or_blank(*args: str) -> str:
    """Convenience for sites that legitimately want the empty-string
    fallback (e.g. the `--version` reachability probe)."""
    try:
        return _nmcli(*args)
    except _NmcliError:
        return ""


def _list_vpns() -> list[dict[str, str]]:
    raw = _nmcli("-t", "-f", "NAME,TYPE,DEVICE,STATE", "connection", "show")
    out = []
    for line in raw.splitlines():
        parts = line.split(":")
        if len(parts) >= 4 and parts[1] in ("vpn", "wireguard"):
            out.append({"name": parts[0], "type": parts[1], "device": parts[2], "state": parts[3]})
    return out


def _import_path(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".ovpn":
        vpn_type = "openvpn"
    elif suffix in (".conf", ".wg"):
        vpn_type = "wireguard"
    else:
        return f"unknown VPN file type: {suffix}"
    try:
        out = _nmcli("connection", "import", "type", vpn_type, "file", str(path))
    except _NmcliError as exc:
        log_action(f"vpn import {path.name}: failed — {exc}")
        return f"import failed: {exc}"
    log_action(f"vpn import {path.name}: {out[:80]}")
    return out or f"imported {path.name}"


@dataclass(frozen=True)
class _VpnProbe:
    """Result of the off-main-thread probe: nmcli reachability + (on
    success) the parsed connection list, or the wrapped error."""
    has_nmcli: bool
    vpns: list[dict[str, str]]
    error: _NmcliError | None


def _gather_vpn_state() -> _VpnProbe:
    """Off-main-thread: every nmcli probe in one place."""
    if not _nmcli_or_blank("--version"):
        return _VpnProbe(has_nmcli=False, vpns=[], error=None)
    try:
        return _VpnProbe(has_nmcli=True, vpns=_list_vpns(), error=None)
    except _NmcliError as exc:
        return _VpnProbe(has_nmcli=True, vpns=[], error=exc)


class VpnPanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._list: Gtk.Box | None = None
        self._content_root: Gtk.Box | None = None
        self._build_skeleton()
        async_probe(_gather_vpn_state, self._apply_state)

    def _build_skeleton(self) -> None:
        """Sync — chrome + a "Loading…" placeholder. The list + actions
        get added by `_apply_state` once nmcli has answered."""
        box = panel_box()
        box.pack_start(title_label("VPN"), False, False, 0)
        box.pack_start(info_label(
            "Connect to a private network through a third-party VPN. "
            "Import a config file you got from your VPN provider, then "
            "switch it on or off here."
        ), False, False, 0)
        box.pack_start(section_description(
            "This is for commercial or work VPNs (OpenVPN, WireGuard). "
            "For Mackes' own mesh, use the Mesh VPN panel instead."
        ), False, False, 0)

        self._loading = info_label("Checking NetworkManager…")
        box.pack_start(self._loading, False, False, 0)

        self._content_root = box
        self.add(box)

    def _apply_state(self, probe: _VpnProbe) -> None:
        """Main thread — discharge the placeholder and build sections."""
        assert self._content_root is not None
        if self._loading is not None and self._loading.get_parent() is not None:
            self._content_root.remove(self._loading)
            self._loading = None
        box = self._content_root

        if not probe.has_nmcli:
            box.pack_start(error_state(
                "NetworkManager not available",
                "`nmcli` isn't installed or isn't on $PATH. Install "
                "NetworkManager (Maintain → Dependencies) and reopen "
                "this panel.",
                retry_label=None,
            ), True, True, 0)
            box.show_all()
            return

        box.pack_start(section_header("Configured VPNs"), False, False, 0)
        self._list = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.pack_start(self._list, False, False, 0)

        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        imp = Gtk.Button(label="Import .ovpn / .conf …")
        imp.connect("clicked", lambda *_: self._import_dialog())
        a11y(imp, name="Import an OpenVPN or WireGuard config file",
             tooltip="Pick a .ovpn / .conf / .wg file from disk to import")
        actions.pack_start(imp, False, False, 0)
        rfr = Gtk.Button(label="Refresh")
        rfr.connect("clicked", lambda *_: self._refresh())
        a11y(rfr, name="Refresh the VPN connection list",
             tooltip="Re-run nmcli to refresh configured VPN connections")
        actions.pack_start(rfr, False, False, 0)
        box.pack_start(actions, False, False, 0)

        self._render_vpns(probe)
        box.show_all()

    def _import_dialog(self) -> None:
        chooser = Gtk.FileChooserNative.new(
            "Import VPN config", self.get_toplevel(),
            Gtk.FileChooserAction.OPEN, "_Open", "_Cancel",
        )
        f = Gtk.FileFilter()
        f.set_name("VPN configs (.ovpn, .conf, .wg)")
        for p in ("*.ovpn", "*.conf", "*.wg"):
            f.add_pattern(p)
        chooser.add_filter(f)
        if chooser.run() == Gtk.ResponseType.ACCEPT:
            path = Path(chooser.get_filename() or "")
            if path.exists():
                _import_path(path)
                self._refresh()
        chooser.destroy()

    def _refresh(self) -> bool:
        """User-triggered refresh — async so nmcli latency never blocks
        the main loop."""
        async_probe(_gather_vpn_state, self._render_vpns)
        return GLib.SOURCE_REMOVE

    def _render_vpns(self, probe: _VpnProbe) -> None:
        """Re-render the connection list from a fresh probe payload.

        Skipped before the panel's `_apply_state` has set up `_list`
        (theoretically possible if `_refresh` somehow races construction).
        """
        if self._list is None:
            return
        for child in list(self._list.get_children()):
            self._list.remove(child)

        # Phase 11.5: distinguish "probe failed" from "no VPNs configured"
        # so the user sees an actionable next step on nmcli breakage.
        if probe.error is not None:
            self._list.pack_start(error_state(
                "Couldn't read VPN list",
                f"nmcli returned an error: {probe.error}",
                on_retry=self._refresh,
            ), True, True, 0)
            self._list.show_all()
            return

        if not probe.vpns:
            self._list.pack_start(empty_state(
                "No VPN connections configured",
                "Click Import to bring in a .ovpn or WireGuard config "
                "file you got from your VPN provider.",
                icon_name="network-vpn-symbolic",
                cta_label="Import .ovpn / .conf …",
                on_cta=self._import_dialog,
            ), True, True, 0)
            self._list.show_all()
            return

        for v in probe.vpns:
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            label = Gtk.Label(label=f"{v['name']}   [{v['type']}]   ({v['state'] or 'inactive'})")
            label.set_xalign(0)
            row.pack_start(label, True, True, 0)
            up = Gtk.Button(label="Up")
            a11y(up, name=f"Bring {v['name']} VPN connection up",
                 tooltip=f"Activate the {v['name']} VPN ({v['type']})")
            down = Gtk.Button(label="Down")
            a11y(down, name=f"Take {v['name']} VPN connection down",
                 tooltip=f"Deactivate the {v['name']} VPN")

            def _up(_b, name=v["name"]):
                # nmcli connection up is slow when a VPN is misconfigured;
                # push the call to the probe thread and refresh on return.
                def _do() -> None:
                    try:
                        _nmcli("connection", "up", name)
                        log_action(f"vpn up: {name}")
                    except _NmcliError as exc:
                        log_action(f"vpn up {name} failed: {exc}")

                async_probe(_do, lambda _v: self._refresh())

            def _down(_b, name=v["name"]):
                def _do() -> None:
                    try:
                        _nmcli("connection", "down", name)
                        log_action(f"vpn down: {name}")
                    except _NmcliError as exc:
                        log_action(f"vpn down {name} failed: {exc}")

                async_probe(_do, lambda _v: self._refresh())

            up.connect("clicked", _up); down.connect("clicked", _down)
            row.pack_end(down, False, False, 0)
            row.pack_end(up, False, False, 0)
            self._list.pack_start(row, False, False, 0)
        self._list.show_all()
