"""v2.0.0 Phase F.4 — tests for the MDE Displays panel.

Validates the panel's discovery + bridge-contract surface without
constructing the GTK widget (covered by the headless smoke).
"""
from __future__ import annotations

import json
import os
import sys
import tempfile
from unittest import mock

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def test_displays_module_imports():
    from mackes.workbench.devices import displays
    assert hasattr(displays, "DisplaysPanel")
    assert callable(displays._output_names)


def test_output_names_returns_list_when_sway_absent():
    from mackes.workbench.devices import displays
    names = displays._output_names()
    assert isinstance(names, list)
    # In CI / sandbox there's no sway, so we expect an empty list, not
    # an exception.


def test_output_names_skips_inactive_and_empty():
    from mackes.workbench.devices import displays
    fake_outputs = [
        {"name": "eDP-1", "active": True},
        {"name": "HDMI-A-1", "active": False},  # dropped
        {"name": "", "active": True},           # dropped
        {"name": "DP-1", "active": True},
    ]
    with mock.patch("mackes.sway_ipc.get_outputs", return_value=fake_outputs):
        assert displays._output_names() == ["eDP-1", "DP-1"]


def test_displays_module_imports_bridge_only_no_xfconf():
    """F.4 lock — displays.py must NOT import xfconf_bridge."""
    import inspect
    from mackes.workbench.devices import displays
    src = inspect.getsource(displays)
    assert "xfconf_bridge" not in src, "displays.py must not import xfconf_bridge"
    assert "mde_settings_bridge" in src, "displays.py must use mde_settings_bridge"


def test_displays_writes_locked_mde_keys():
    """Displays panel writes every Phase C.3 display.* key."""
    import inspect
    from mackes.workbench.devices import displays
    src = inspect.getsource(displays)
    for key in (
        "display.primary",
        "display.scale",
        "display.night_light",
        "display.night_light_temp",
    ):
        assert key in src, f"displays.py must reference {key}"


def test_displays_reads_outputs_through_sway_ipc():
    """displays.py calls sway_ipc.get_outputs, not subprocess xrandr."""
    import inspect
    from mackes.workbench.devices import displays
    src = inspect.getsource(displays)
    assert "sway_ipc" in src
    assert "get_outputs" in src
    # The xrandr binary is the v1.x backend that this panel replaces;
    # the rewrite must not subprocess it.
    assert "subprocess.run([\"xrandr\"" not in src
    assert "'xrandr'" not in src


def test_sway_ipc_get_outputs_surface_exists():
    """sway_ipc grew get_outputs() in F.4."""
    from mackes import sway_ipc
    assert callable(sway_ipc.get_outputs)
    assert "get_outputs" in sway_ipc.__all__


def test_sway_ipc_get_outputs_returns_list_when_unavailable():
    from mackes import sway_ipc
    # On a system without sway, get_outputs returns [].
    out = sway_ipc.get_outputs()
    assert isinstance(out, list)


def test_sway_ipc_get_outputs_handles_bad_json():
    from mackes import sway_ipc
    with mock.patch("mackes.sway_ipc._run", return_value=(0, "not json", "")):
        assert sway_ipc.get_outputs() == []


def test_sway_ipc_get_outputs_handles_nonlist_response():
    from mackes import sway_ipc
    with mock.patch("mackes.sway_ipc._run",
                    return_value=(0, json.dumps({"oops": True}), "")):
        assert sway_ipc.get_outputs() == []


def test_sway_ipc_get_outputs_parses_array():
    from mackes import sway_ipc
    payload = json.dumps([{"name": "eDP-1", "active": True}])
    with mock.patch("mackes.sway_ipc._run", return_value=(0, payload, "")):
        out = sway_ipc.get_outputs()
        assert out == [{"name": "eDP-1", "active": True}]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
