"""Unit tests for `mackes.workbench._async.async_probe`.

The helper is the canonical pattern for getting blocking probes off
the GTK main thread (Phase 11.9). Failure here means every panel that
adopted the pattern is also broken — keep these tests fast and
deterministic.
"""
from __future__ import annotations

import time

import pytest

gi = pytest.importorskip("gi")
gi.require_version("Gtk", "3.0")
try:
    from gi.repository import GLib, Gtk  # noqa: F401, E402
except (ImportError, ValueError) as exc:
    pytest.skip(f"GTK3 typelib unavailable: {exc}", allow_module_level=True)

from mackes.workbench._async import async_probe  # noqa: E402


def _run_main_loop_briefly(seconds: float = 0.3) -> None:
    """Spin GLib's main context for `seconds`. async_probe's callback
    delivery uses `GLib.idle_add`, so we need a running context to
    actually receive callbacks in a test."""
    ctx = GLib.MainContext.default()
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        ctx.iteration(may_block=False)
        time.sleep(0.01)


def test_async_probe_delivers_result_on_main_thread():
    captured: list[int] = []
    thread = async_probe(lambda: 42, captured.append)
    thread.join(timeout=2)
    assert not thread.is_alive(), "probe thread should have finished"
    _run_main_loop_briefly()
    assert captured == [42], f"expected [42], got {captured}"


def test_async_probe_swallows_probe_exception_by_default():
    captured: list[int] = []
    # No on_error → exception goes to the logger; on_result never fires.
    thread = async_probe(_raise_value_error, captured.append)
    thread.join(timeout=2)
    _run_main_loop_briefly()
    assert captured == [], "on_result should NOT fire when probe raises"


def test_async_probe_routes_exception_to_on_error():
    errors: list[BaseException] = []
    results: list[object] = []
    thread = async_probe(
        _raise_value_error,
        results.append,
        on_error=errors.append,
    )
    thread.join(timeout=2)
    _run_main_loop_briefly()
    assert results == []
    assert len(errors) == 1
    assert isinstance(errors[0], ValueError)
    assert str(errors[0]) == "probe-bang"


def test_async_probe_swallows_callback_exception():
    """A buggy on_result must not crash the GTK main loop."""
    fired = []

    def explosive_callback(_value: object) -> None:
        fired.append("called")
        raise RuntimeError("callback-bang")

    thread = async_probe(lambda: "ok", explosive_callback)
    thread.join(timeout=2)
    _run_main_loop_briefly()
    # The callback DID run (it fired) but its exception was swallowed.
    # If `_safe_callback` weren't there, the RuntimeError would bubble
    # up through GLib and corrupt the next idle source.
    assert fired == ["called"]


def test_async_probe_uses_daemon_thread():
    """A non-daemon thread would block process exit."""
    thread = async_probe(lambda: None, lambda _v: None)
    assert thread.daemon, "probe thread must be a daemon"


def test_async_probe_thread_name_includes_probe_name():
    def my_specific_probe():
        return None

    thread = async_probe(my_specific_probe, lambda _v: None)
    assert "my_specific_probe" in thread.name, (
        f"expected probe name in thread name, got {thread.name!r}"
    )


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------


def _raise_value_error():
    raise ValueError("probe-bang")
