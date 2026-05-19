"""Phase 11.5 — empty + error state coverage.

Drives a handful of panels through their failure paths and asserts the
labeled empty/error states the audit replaced silent `pass`-on-error
with actually render. Each test mocks the underlying probe to raise so
no real subprocess fires.

Skipped when GTK or `$DISPLAY` (or Xvfb) is unavailable — same gate as
`test_panel_instantiation_smoke.py`.
"""
from __future__ import annotations

import os
import subprocess

import pytest


# Skip the whole module if GTK isn't importable or we have no display.
gi = pytest.importorskip("gi")
gi.require_version("Gtk", "3.0")
try:
    from gi.repository import Gtk  # noqa: E402
except (ImportError, ValueError) as exc:
    pytest.skip(f"GTK3 typelib unavailable: {exc}", allow_module_level=True)

if not (os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY")):
    pytest.skip("no $DISPLAY (run under Xvfb)", allow_module_level=True)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _walk(widget: Gtk.Widget):
    """Depth-first walk over a widget tree."""
    yield widget
    if isinstance(widget, Gtk.Container):
        for child in widget.get_children():
            yield from _walk(child)


def _find_label_with(widget: Gtk.Widget, needle: str) -> Gtk.Label | None:
    """Return the first Gtk.Label in the tree containing ``needle`` in
    its visible text. Case-insensitive."""
    needle_low = needle.lower()
    for w in _walk(widget):
        if isinstance(w, Gtk.Label):
            text = (w.get_text() or "")
            if needle_low in text.lower():
                return w
    return None


def _has_class(widget: Gtk.Widget, klass: str) -> bool:
    """Recursive: is any descendant carrying the given CSS class?"""
    for w in _walk(widget):
        ctx = w.get_style_context()
        if ctx is not None and ctx.has_class(klass):
            return True
    return False


# ---------------------------------------------------------------------------
# Helper-level tests — the public empty_state / error_state primitives
# ---------------------------------------------------------------------------


def test_empty_state_renders_title_body_and_cta():
    from mackes.workbench._common import empty_state

    clicks: list[bool] = []
    widget = empty_state(
        "No snapshots yet",
        "Take one before you change something risky.",
        cta_label="Create restore point",
        on_cta=lambda: clicks.append(True),
    )

    assert _find_label_with(widget, "No snapshots yet") is not None, (
        "empty_state() must render its title as a visible Gtk.Label"
    )
    assert _find_label_with(widget, "before you change") is not None, (
        "empty_state() must render the body paragraph"
    )

    # Find the CTA button and click it.
    btn = next(
        (w for w in _walk(widget) if isinstance(w, Gtk.Button)),
        None,
    )
    assert btn is not None, "empty_state() with cta_label must produce a button"
    assert btn.get_label() == "Create restore point"
    btn.clicked()
    assert clicks == [True], "Clicking the CTA must invoke on_cta"


def test_error_state_renders_reason_and_retry():
    from mackes.workbench._common import error_state

    retries: list[bool] = []
    widget = error_state(
        "Couldn't load snapshots",
        "Permission denied: /var/lib/mackes",
        on_retry=lambda: retries.append(True),
    )

    assert _find_label_with(widget, "Couldn't load snapshots") is not None
    assert _find_label_with(widget, "Permission denied") is not None
    assert _has_class(widget, "error"), (
        "error_state() must carry the `error` style class for theming"
    )

    btn = next(
        (w for w in _walk(widget) if isinstance(w, Gtk.Button)),
        None,
    )
    assert btn is not None and btn.get_label() == "Retry"
    btn.clicked()
    assert retries == [True], "Retry button must invoke on_retry"


def test_error_state_omits_button_when_unactionable():
    """A non-recoverable error (e.g. firewall-cmd missing) should still
    render a label tile, just without a Retry button."""
    from mackes.workbench._common import error_state

    widget = error_state(
        "firewalld not available",
        "Install firewalld and reopen this panel.",
        retry_label=None,
    )
    assert _find_label_with(widget, "firewalld not available") is not None
    buttons = [w for w in _walk(widget) if isinstance(w, Gtk.Button)]
    assert buttons == [], (
        "error_state(retry_label=None) must not render a button"
    )


# ---------------------------------------------------------------------------
# Panel-level tests — mock the probe to raise, assert the error label
# ---------------------------------------------------------------------------


def test_apps_installed_panel_renders_error_on_rpm_failure(monkeypatch):
    """When `list_installed_packages()` raises PackageProbeError the
    panel must render an "Couldn't list installed packages" tile, not
    just an empty TreeView. Phase 11.5 — the failure path that motivated
    the audit."""
    # Import the panel module first so we can pin the exception class
    # to the same instance the panel's `except` clause references. The
    # `isolated_xdg` conftest fixture purges `mackes.app_mgmt` mid-suite;
    # creating the panel here forces a clean import so the class
    # identities line up.
    from mackes.workbench.apps import installed as installed_mod
    PackageProbeError = installed_mod.PackageProbeError

    def _boom() -> list[tuple[str, str]]:
        raise PackageProbeError("rpm exited 1")

    # Patch the import-binding inside the panel module so the worker
    # thread hits our mock.
    monkeypatch.setattr(installed_mod, "list_installed_packages", _boom)

    panel = installed_mod.AppsInstalledPanel()
    # `_reload` spawned a daemon thread; pump GLib until it lands.
    # GLib.idle_add fires our `_set_error` once the worker is done.
    from gi.repository import GLib
    deadline = GLib.get_monotonic_time() + 3_000_000  # 3 s
    while GLib.get_monotonic_time() < deadline:
        ctx = GLib.MainContext.default()
        while ctx.iteration(False):
            pass
        if _find_label_with(panel, "Couldn't list installed packages"):
            break

    assert _find_label_with(panel, "Couldn't list installed packages"), (
        "AppsInstalledPanel must surface PackageProbeError as a labeled "
        "error state, not a blank list"
    )
    assert _find_label_with(panel, "rpm exited 1"), (
        "The probe's error message must appear so the user knows what failed"
    )


def test_vpn_panel_renders_error_when_nmcli_missing(monkeypatch):
    """When nmcli isn't installed the VPN panel must render the
    'NetworkManager not available' tile instead of an empty list of
    VPN connections."""
    from mackes.workbench.network import vpn as vpn_mod

    # Force the off-thread probe path to return has_nmcli=False
    # without spawning a subprocess. We patch the gather function so
    # it returns synchronously when the worker thread calls it.
    fake = vpn_mod._VpnProbe(has_nmcli=False, vpns=[], error=None)
    monkeypatch.setattr(vpn_mod, "_gather_vpn_state", lambda: fake)

    panel = vpn_mod.VpnPanel()

    # async_probe handed off to a worker thread; pump GLib until the
    # apply lands.
    from gi.repository import GLib
    deadline = GLib.get_monotonic_time() + 3_000_000  # 3 s
    while GLib.get_monotonic_time() < deadline:
        ctx = GLib.MainContext.default()
        while ctx.iteration(False):
            pass
        if _find_label_with(panel, "NetworkManager not available"):
            break

    assert _find_label_with(panel, "NetworkManager not available"), (
        "VpnPanel must render an error tile when nmcli is unavailable"
    )


def test_vpn_panel_renders_empty_state_when_no_vpns(monkeypatch):
    """nmcli works but no VPNs configured: the panel must show the
    "No VPN connections configured" empty state with a CTA."""
    from mackes.workbench.network import vpn as vpn_mod

    fake = vpn_mod._VpnProbe(has_nmcli=True, vpns=[], error=None)
    monkeypatch.setattr(vpn_mod, "_gather_vpn_state", lambda: fake)

    panel = vpn_mod.VpnPanel()

    from gi.repository import GLib
    deadline = GLib.get_monotonic_time() + 3_000_000
    while GLib.get_monotonic_time() < deadline:
        ctx = GLib.MainContext.default()
        while ctx.iteration(False):
            pass
        if _find_label_with(panel, "No VPN connections configured"):
            break

    assert _find_label_with(panel, "No VPN connections configured"), (
        "VpnPanel must render an empty state when nmcli returns zero VPNs"
    )


def test_vpn_panel_renders_error_when_nmcli_lists_fail(monkeypatch):
    """nmcli `--version` works but `nmcli connection show` fails: the
    panel must render an error tile carrying the wrapped subprocess
    error message, not a blank list that's indistinguishable from "no
    VPNs configured"."""
    from mackes.workbench.network import vpn as vpn_mod

    err = vpn_mod._NmcliError("nmcli connection show timed out after 8 s")
    fake = vpn_mod._VpnProbe(has_nmcli=True, vpns=[], error=err)
    monkeypatch.setattr(vpn_mod, "_gather_vpn_state", lambda: fake)

    panel = vpn_mod.VpnPanel()

    from gi.repository import GLib
    deadline = GLib.get_monotonic_time() + 3_000_000
    while GLib.get_monotonic_time() < deadline:
        ctx = GLib.MainContext.default()
        while ctx.iteration(False):
            pass
        if _find_label_with(panel, "Couldn't read VPN list"):
            break

    assert _find_label_with(panel, "Couldn't read VPN list"), (
        "VpnPanel must distinguish a probe failure from an empty list"
    )
    assert _find_label_with(panel, "timed out after 8 s"), (
        "The wrapped subprocess error must be surfaced in the tile"
    )


def test_format_probe_error_drops_class_prefix_on_oserror():
    from mackes.workbench._common import format_probe_error

    msg = format_probe_error(FileNotFoundError("rpm: not found"))
    assert "rpm: not found" in msg
    assert "FileNotFoundError" not in msg


def test_format_probe_error_keeps_class_for_unknown_exception():
    from mackes.workbench._common import format_probe_error

    class _Weird(Exception):
        pass

    msg = format_probe_error(_Weird("nope"))
    assert "_Weird" in msg and "nope" in msg
