"""Background-thread probe helper for Workbench panels.

The canonical pattern for getting blocking subprocess calls off a panel's
`__init__` (Phase 11.9 reliability sweep — see `PROJECT_WORKLIST.md`).
Models the 8.6.7 sidebar fix: don't block the GTK main loop on a
firewall-cmd / nmcli / rpm -q / tailscale roundtrip — render a skeleton
synchronously, kick off the probe on a daemon thread, marshal the
result back through `GLib.idle_add`.

Usage:

    from mackes.workbench._async import async_probe

    class FirewallPanel(Gtk.Box):
        def __init__(self):
            super().__init__()
            self._build_skeleton()         # cheap; no probes
            async_probe(
                self._gather_state,        # off-main-thread
                self._apply_state,         # on-main-thread
            )

        def _gather_state(self):
            # Runs on a daemon thread. Any subprocess call goes here.
            return FirewallState(
                zones=_zones(),
                services=_enabled_services(),
            )

        def _apply_state(self, state):
            # Runs on the GTK main thread. Safe to touch widgets.
            self._zone_combo.set_active(...)
            self._refresh_services(state.services)

Why this matters: `__init__` is on the main thread. If it does 30 ms of
real work, the panel switch animation hitches. If it does 5 s of real
work (waiting on `firewall-cmd --list-all` when firewalld is down), the
whole Workbench locks up and looks broken.
"""
from __future__ import annotations

import logging
import sys
import threading
from typing import Any, Callable, TypeVar

from gi.repository import GLib

T = TypeVar("T")

_log = logging.getLogger(__name__)


def async_probe(
    probe: Callable[[], T],
    on_result: Callable[[T], Any],
    *,
    on_error: Callable[[BaseException], Any] | None = None,
    thread_name: str | None = None,
) -> threading.Thread:
    """Run `probe()` on a daemon thread, then call `on_result(value)`
    on the GTK main thread via `GLib.idle_add`.

    If `probe` raises, `on_error(exc)` is called on the GTK main thread
    when provided, otherwise the exception is logged at WARNING. The
    panel's UI stays in its skeleton state — the user sees an empty
    list rather than a crash.

    Returns the spawned thread so callers can join it in tests.
    """
    name = thread_name or f"mackes-probe-{probe.__name__}"

    def _worker() -> None:
        try:
            result = probe()
        except BaseException as exc:  # noqa: BLE001
            if on_error is not None:
                GLib.idle_add(_safe_callback, on_error, exc)
            else:
                _log.warning("async_probe %s failed: %r", name, exc)
                print(f"async_probe {name} failed: {exc!r}", file=sys.stderr)
            return
        GLib.idle_add(_safe_callback, on_result, result)

    thread = threading.Thread(target=_worker, daemon=True, name=name)
    thread.start()
    return thread


def _safe_callback(callback: Callable[[Any], Any], value: Any) -> bool:
    """Run `callback(value)` while swallowing exceptions. Returns False
    so GLib doesn't reschedule us as a recurring idle.

    Without this guard a buggy `on_result` would bubble up into GLib's
    main loop and trigger the "Gtk-WARNING **: Source ID was not found"
    family of confused errors.
    """
    try:
        callback(value)
    except BaseException as exc:  # noqa: BLE001
        _log.exception("async_probe callback failed: %r", exc)
        print(f"async_probe callback failed: {exc!r}", file=sys.stderr)
    return False
