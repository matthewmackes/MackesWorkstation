"""Tests for `mackes.mde_settings_bridge` (Phase F.1 + helpers)."""
from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path


def _with_xdg_cache(fn):
    tmp = tempfile.TemporaryDirectory()
    try:
        old = os.environ.get("XDG_CACHE_HOME")
        os.environ["XDG_CACHE_HOME"] = tmp.name
        try:
            return fn(Path(tmp.name))
        finally:
            if old is None:
                del os.environ["XDG_CACHE_HOME"]
            else:
                os.environ["XDG_CACHE_HOME"] = old
    finally:
        tmp.cleanup()


def test_sidecar_path_honors_xdg_cache_home():
    def body(cache):
        from mackes.mde_settings_bridge import sidecar_path
        p = sidecar_path("foo.json")
        assert p == cache / "mde" / "foo.json"
    _with_xdg_cache(body)


def test_read_sidecar_returns_default_when_missing():
    def body(cache):
        from mackes.mde_settings_bridge import read_sidecar
        assert read_sidecar("never.json") == {}
        assert read_sidecar("never.json", default={"k": 1}) == {"k": 1}
    _with_xdg_cache(body)


def test_read_sidecar_handles_malformed_json():
    def body(cache):
        from mackes.mde_settings_bridge import sidecar_path, read_sidecar
        path = sidecar_path("bad.json")
        path.parent.mkdir(parents=True)
        path.write_text("{not json")
        assert read_sidecar("bad.json") == {}
    _with_xdg_cache(body)


def test_read_sidecar_handles_non_dict_json():
    def body(cache):
        from mackes.mde_settings_bridge import sidecar_path, read_sidecar
        path = sidecar_path("list.json")
        path.parent.mkdir(parents=True)
        path.write_text("[1,2,3]")
        assert read_sidecar("list.json") == {}
    _with_xdg_cache(body)


def test_write_then_read_round_trip():
    def body(cache):
        from mackes.mde_settings_bridge import write_sidecar, read_sidecar
        write_sidecar("test.json", {"a": 1, "b": "two"})
        got = read_sidecar("test.json")
        assert got == {"a": 1, "b": "two"}
    _with_xdg_cache(body)


def test_update_sidecar_overlays_only_changed_keys():
    def body(cache):
        from mackes.mde_settings_bridge import write_sidecar, update_sidecar, read_sidecar
        write_sidecar("u.json", {"a": 1, "b": 2, "c": 3})
        update_sidecar("u.json", b=99)
        got = read_sidecar("u.json")
        assert got == {"a": 1, "b": 99, "c": 3}
    _with_xdg_cache(body)


def test_get_setting_routes_sidecar_keys():
    def body(cache):
        from mackes.mde_settings_bridge import write_sidecar, get_setting
        write_sidecar("power-prefs.json", {"lid_action": "suspend"})
        assert get_setting("power.lid_action") == "suspend"
    _with_xdg_cache(body)


def test_set_setting_writes_to_correct_sidecar():
    def body(cache):
        from mackes.mde_settings_bridge import set_setting, read_sidecar
        assert set_setting("power.lid_action", "hibernate")
        data = read_sidecar("power-prefs.json")
        assert data == {"lid_action": "hibernate"}
    _with_xdg_cache(body)


def test_set_setting_preserves_other_keys_in_same_sidecar():
    def body(cache):
        from mackes.mde_settings_bridge import set_setting, read_sidecar, write_sidecar
        write_sidecar("power-prefs.json", {
            "lid_action": "suspend",
            "suspend_idle_battery_s": 1800,
        })
        set_setting("power.lid_action", "poweroff")
        data = read_sidecar("power-prefs.json")
        assert data["lid_action"] == "poweroff"
        assert data["suspend_idle_battery_s"] == 1800
    _with_xdg_cache(body)


def test_get_setting_unknown_key_returns_none():
    from mackes.mde_settings_bridge import get_setting
    assert get_setting("does.not.exist") is None


def test_set_setting_unknown_key_returns_false():
    from mackes.mde_settings_bridge import set_setting
    assert set_setting("does.not.exist", "x") is False


def test_key_map_covers_every_implemented_phase_c_key():
    """Smoke that the bridge knows about every key the Phase C
    appliers ship (theme/font/power/display/automount/wallpaper/
    notification-non-DND). The lock list below must match the
    appliers."""
    from mackes.mde_settings_bridge import _KEY_MAP
    expected = {
        # theme
        "theme.name", "theme.icon_set", "theme.accent", "theme.mode",
        # font
        "font.name", "font.monospace", "font.hinting", "font.antialias",
        # power (sidecar — profile via powerprofilesctl is separate)
        "power.lid_action", "power.suspend_idle_battery_s", "power.suspend_idle_ac_s",
        # display (sidecar — brightness via brightnessctl is separate)
        "display.primary", "display.scale",
        "display.night_light", "display.night_light_temp",
        # automount
        "automount.on_insert", "automount.open_on_mount", "automount.autorun",
        # wallpaper
        "wallpaper.path", "wallpaper.mode",
        # notification (DND is a flag file, not a sidecar)
        "notification.location", "notification.default_expire_ms",
    }
    assert set(_KEY_MAP.keys()) == expected
