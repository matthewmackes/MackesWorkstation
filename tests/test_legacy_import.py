"""Tests for ``mackes.legacy_import`` — Phase 10.2 / v3.0.0 Q49.

Covers detection of 2.x leftovers under ``~/.config/mackes-shell/``,
translation into ``~/.config/mackes-panel/panel.toml`` matching the
schema in ``crates/mackes-config/src/lib.rs``, and resilience against
corrupted / partial legacy state.

Every test runs against the ``isolated_xdg`` fixture so neither the
developer's real ``~/.config`` nor the test runner's environment is
ever touched.
"""
from __future__ import annotations

import json
import tomllib
from pathlib import Path


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------


def _legacy_dir(env: dict) -> Path:
    """Return the legacy ~/.config/mackes-shell path inside the fixture."""
    # `env["config"]` IS ~/.config/mackes-shell (the conftest fixture maps
    # `mackes.state.CONFIG_DIR` directly). The real ~/.config root is its
    # parent.
    return env["config"]


def _config_root(env: dict) -> Path:
    return env["config"].parent


def _write_state(env: dict, payload: dict) -> None:
    legacy = _legacy_dir(env)
    legacy.mkdir(parents=True, exist_ok=True)
    (legacy / "state.json").write_text(
        json.dumps(payload, indent=2), encoding="utf-8"
    )


def _write_pinned(env: dict, names: list[str]) -> None:
    pinned = _legacy_dir(env) / "pinned"
    pinned.mkdir(parents=True, exist_ok=True)
    for name in names:
        # Some are real .desktop files; some are bare names (which
        # the resolver should still normalize). Mix both forms.
        target = pinned / name
        if name.endswith(".desktop"):
            target.write_text(
                f"[Desktop Entry]\nName={name[:-8].title()}\n",
                encoding="utf-8",
            )
        else:
            target.write_text("", encoding="utf-8")


def _read_panel_toml(env: dict) -> dict:
    path = _config_root(env) / "mackes-panel" / "panel.toml"
    return tomllib.loads(path.read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# detect()
# ---------------------------------------------------------------------------


def test_no_legacy_state_returns_none(isolated_xdg):
    """Fresh install — no ~/.config/mackes-shell directory at all."""
    from mackes.legacy_import import detect

    # Wipe the dir the state-module fixture pre-created so we have a
    # *truly* fresh environment.
    import shutil
    legacy_dir = isolated_xdg["config"]
    # The conftest fixture calls ensure_dirs() which creates
    # mackes-shell/. Removing it lets us cover the no-dir branch.
    if legacy_dir.exists():
        shutil.rmtree(legacy_dir)

    assert detect() is None


def test_empty_legacy_dir_returns_none(isolated_xdg):
    """Directory present but no recognized files — detect() returns None."""
    from mackes.legacy_import import detect

    # ensure_dirs() already created mackes-shell/. Leave it empty.
    assert (isolated_xdg["config"]).is_dir()
    assert detect() is None


def test_preset_only(isolated_xdg):
    """state.json with a preset_name field but no wallpaper or pins."""
    from mackes.legacy_import import detect

    _write_state(isolated_xdg,{"preset_name": "mackes"})
    legacy = detect()
    assert legacy is not None
    assert legacy.preset_name == "mackes"
    assert legacy.wallpaper_path is None
    assert legacy.pinned_apps == []


def test_wallpaper_only(isolated_xdg):
    """state.json with only a wallpaper field."""
    from mackes.legacy_import import detect

    _write_state(
        isolated_xdg, {"wallpaper": "/home/user/Pictures/sunset.jpg"}
    )
    legacy = detect()
    assert legacy is not None
    assert legacy.preset_name is None
    assert legacy.wallpaper_path == "/home/user/Pictures/sunset.jpg"


def test_pinned_apps_directory_scan(isolated_xdg):
    """Pinned apps come from ~/.config/mackes-shell/pinned/ entries."""
    from mackes.legacy_import import detect

    _write_pinned(
        isolated_xdg,
        ["firefox.desktop", "org.gnome.Terminal.desktop", "gimp"],
    )
    legacy = detect()
    assert legacy is not None
    # Sorted by filename so the order is stable: firefox / gimp / org.gnome.*
    assert legacy.pinned_apps == [
        "firefox.desktop", "gimp.desktop", "org.gnome.Terminal.desktop",
    ]


def test_corrupted_state_json_is_tolerated(isolated_xdg):
    """A malformed state.json should NOT raise — detect() returns
    a LegacyState only if something else is migratable."""
    from mackes.legacy_import import detect

    legacy_dir = isolated_xdg["config"]
    legacy_dir.mkdir(parents=True, exist_ok=True)
    (legacy_dir / "state.json").write_text("{ not valid json", encoding="utf-8")
    # No pins, no other artifacts.
    assert detect() is None

    # When *something else* is present, the corrupted state.json is
    # ignored but the other fields still drive a non-None return.
    _write_pinned(isolated_xdg, ["firefox.desktop"])
    legacy = detect()
    assert legacy is not None
    assert legacy.preset_name is None
    assert legacy.pinned_apps == ["firefox.desktop"]


def test_missing_pinned_subdir_does_not_break_scan(isolated_xdg):
    """Detection works even when pinned/ does not exist."""
    from mackes.legacy_import import detect

    _write_state(isolated_xdg,{"preset_name": "hashbang"})
    # Sanity: confirm pinned dir is absent.
    assert not (isolated_xdg["config"] / "pinned").exists()
    legacy = detect()
    assert legacy is not None
    assert legacy.preset_name == "hashbang"
    assert legacy.pinned_apps == []


def test_drawer_overrides_loaded(isolated_xdg):
    """drawer-overrides.json is read and surfaced verbatim."""
    from mackes.legacy_import import detect

    legacy_dir = isolated_xdg["config"]
    legacy_dir.mkdir(parents=True, exist_ok=True)
    (legacy_dir / "drawer-overrides.json").write_text(
        json.dumps({
            "show_appmenu": False,
            "mesh_drift_seconds": 600,
            "totally_unknown_key": "ignored",
        }),
        encoding="utf-8",
    )
    legacy = detect()
    assert legacy is not None
    assert legacy.drawer_overrides == {
        "show_appmenu": False,
        "mesh_drift_seconds": 600,
        "totally_unknown_key": "ignored",
    }


def test_recents_dropped_in_log_but_captured_in_state(isolated_xdg):
    """recents.json is captured into LegacyState.recents."""
    from mackes.legacy_import import detect

    legacy_dir = isolated_xdg["config"]
    legacy_dir.mkdir(parents=True, exist_ok=True)
    (legacy_dir / "recents.json").write_text(
        json.dumps(["gimp.desktop", "inkscape"]), encoding="utf-8",
    )
    legacy = detect()
    assert legacy is not None
    # `inkscape` got `.desktop` appended; `gimp.desktop` kept verbatim.
    assert legacy.recents == ["gimp.desktop", "inkscape.desktop"]


# ---------------------------------------------------------------------------
# import_to_panel_toml()
# ---------------------------------------------------------------------------


def test_full_migration_produces_parseable_panel_toml(isolated_xdg):
    """A realistic 2.x state round-trips into a panel.toml the Rust
    schema would accept (we re-parse with tomllib to confirm the shape)."""
    from mackes.legacy_import import detect, import_to_panel_toml

    _write_state(isolated_xdg,{
        "preset_name": "hashbang",
        "wallpaper": "/usr/share/backgrounds/sunset.jpg",
    })
    _write_pinned(isolated_xdg,
                  ["firefox.desktop", "org.gnome.Terminal.desktop"])
    legacy_dir = isolated_xdg["config"]
    (legacy_dir / "drawer-overrides.json").write_text(
        json.dumps({"show_appmenu": False, "mesh_drift_seconds": 600}),
        encoding="utf-8",
    )

    legacy = detect()
    assert legacy is not None
    log = import_to_panel_toml(legacy)
    assert log, "expected at least one migration log line"
    assert any("firefox.desktop" in line for line in log)

    doc = _read_panel_toml(isolated_xdg)
    # Schema-faithful structure
    assert doc["top_bar"]["appmenu"] is False
    assert isinstance(doc["top_bar"]["status_items"], list)
    assert doc["mesh"]["drift_check_seconds"] == 600
    assert doc["mesh"]["replicate"] is True  # default preserved
    apps = [i for i in doc["dock"]["items"] if i["kind"] == "app"]
    assert {i["desktop"] for i in apps} == {
        "firefox.desktop", "org.gnome.Terminal.desktop"
    }
    # migration sidecar
    assert doc["migration"]["legacy_preset"] == "hashbang"
    assert doc["migration"]["legacy_wallpaper"] == \
        "/usr/share/backgrounds/sunset.jpg"


def test_import_is_idempotent(isolated_xdg):
    """Re-running the import with the same LegacyState produces the
    same panel.toml byte-for-byte."""
    from mackes.legacy_import import detect, import_to_panel_toml

    _write_state(isolated_xdg,{"preset_name": "daylight"})
    _write_pinned(isolated_xdg,
                  ["firefox.desktop", "gimp.desktop"])

    legacy = detect()
    assert legacy is not None

    import_to_panel_toml(legacy)
    first = (isolated_xdg["config"].parent / "mackes-panel" / "panel.toml").read_text(
        encoding="utf-8"
    )

    # The first import wrote `active_preset` into state.json, so the
    # second detect() reads it back via the same priority chain — but
    # we re-use the same LegacyState object to exercise pure
    # idempotency of `import_to_panel_toml`.
    import_to_panel_toml(legacy)
    second = (isolated_xdg["config"].parent / "mackes-panel" / "panel.toml").read_text(
        encoding="utf-8"
    )
    assert first == second, "panel.toml drifted on re-import"


def test_import_preserves_existing_pins(isolated_xdg):
    """If panel.toml already has pinned apps, legacy pins are appended
    (not duplicated) and existing entries survive."""
    from mackes.legacy_import import detect, import_to_panel_toml

    # Pre-seed panel.toml with an existing firefox pin.
    panel_dir = isolated_xdg["config"].parent / "mackes-panel"
    panel_dir.mkdir(parents=True, exist_ok=True)
    (panel_dir / "panel.toml").write_text(
        "[top_bar]\n"
        'status_items = ["mesh", "clipboard", "volume", '
        '"battery", "notifications", "user"]\n'
        "appmenu = true\n\n"
        "[[dock.items]]\n"
        'kind = "app"\n'
        'desktop = "firefox.desktop"\n\n'
        "[mesh]\n"
        "replicate = true\n"
        "drift_check_seconds = 300\n",
        encoding="utf-8",
    )

    _write_pinned(isolated_xdg,
                  ["firefox.desktop", "gimp.desktop"])

    legacy = detect()
    assert legacy is not None
    log = import_to_panel_toml(legacy)
    assert any("already present" in line for line in log)

    doc = _read_panel_toml(isolated_xdg)
    apps = [i["desktop"] for i in doc["dock"]["items"] if i["kind"] == "app"]
    # firefox once (existing), gimp once (new)
    assert apps == ["firefox.desktop", "gimp.desktop"]


def test_corrupt_panel_toml_falls_back_to_default(isolated_xdg):
    """If the existing panel.toml is unparseable, we don't crash — we
    rewrite from defaults plus the legacy fields."""
    from mackes.legacy_import import import_to_panel_toml, LegacyState

    panel_dir = isolated_xdg["config"].parent / "mackes-panel"
    panel_dir.mkdir(parents=True, exist_ok=True)
    (panel_dir / "panel.toml").write_text(
        "this is = not valid \\n toml [",  # malformed
        encoding="utf-8",
    )

    legacy = LegacyState(pinned_apps=["gimp.desktop"])
    log = import_to_panel_toml(legacy)
    assert any("gimp.desktop" in line for line in log)

    doc = _read_panel_toml(isolated_xdg)
    apps = [i["desktop"] for i in doc["dock"]["items"] if i["kind"] == "app"]
    assert apps == ["gimp.desktop"]
    # Default status_items + appmenu restored.
    assert doc["top_bar"]["appmenu"] is True
    assert "mesh" in doc["top_bar"]["status_items"]


def test_drawer_overrides_partial_application(isolated_xdg):
    """Known keys land; unknown keys are dropped (recorded in log)."""
    from mackes.legacy_import import import_to_panel_toml, LegacyState

    legacy = LegacyState(drawer_overrides={
        "show_appmenu": False,
        "status_items": ["mesh", "battery", "rubbish"],  # rubbish filtered
        "mesh_replicate": False,
        "mesh_drift_seconds": 0,
        "weird_key": "x",
    })
    log = import_to_panel_toml(legacy)
    assert any("dropped" in line and "weird_key" in line for line in log)

    doc = _read_panel_toml(isolated_xdg)
    assert doc["top_bar"]["appmenu"] is False
    assert doc["top_bar"]["status_items"] == ["mesh", "battery"]
    assert doc["mesh"]["replicate"] is False
    assert doc["mesh"]["drift_check_seconds"] == 0


def test_import_writes_active_preset_to_state_json(isolated_xdg):
    """After import the legacy state.json gains an `active_preset` key
    so a wizard re-run sees the machine as provisioned."""
    from mackes.legacy_import import import_to_panel_toml, LegacyState

    legacy = LegacyState(preset_name="vanilla")
    import_to_panel_toml(legacy)
    state_path = isolated_xdg["config"] / "state.json"
    doc = json.loads(state_path.read_text(encoding="utf-8"))
    assert doc["active_preset"] == "vanilla"


def test_round_trip_through_python_tomllib(isolated_xdg):
    """The emitted file parses cleanly with the same toml dialect
    the Rust serde model uses (toml 1.0). We re-parse with tomllib
    and verify every required schema field lands."""
    from mackes.legacy_import import import_to_panel_toml, LegacyState

    legacy = LegacyState(
        preset_name="hashbang",
        wallpaper_path="/tmp/wp.png",
        pinned_apps=["a.desktop", "b.desktop", "c.desktop"],
    )
    import_to_panel_toml(legacy)
    doc = _read_panel_toml(isolated_xdg)

    # Required top-level keys
    assert {"top_bar", "dock", "mesh", "migration"} <= set(doc.keys())
    # top_bar shape
    assert isinstance(doc["top_bar"]["status_items"], list)
    assert isinstance(doc["top_bar"]["appmenu"], bool)
    # dock shape — every item carries the required kind-discriminator
    for item in doc["dock"]["items"]:
        assert item["kind"] in ("app", "mesh")
        if item["kind"] == "app":
            assert isinstance(item["desktop"], str)
        else:
            assert isinstance(item["id"], str)
    # mesh shape
    assert isinstance(doc["mesh"]["replicate"], bool)
    assert isinstance(doc["mesh"]["drift_check_seconds"], int)


def test_pinned_via_symlink_to_system_desktop(isolated_xdg, tmp_path):
    """A pinned/ entry that's a symlink to a real .desktop resolves to
    that file's basename."""
    from mackes.legacy_import import detect

    # Create a fake system .desktop somewhere outside the legacy dir.
    system_app = tmp_path / "applications" / "vlc.desktop"
    system_app.parent.mkdir(parents=True, exist_ok=True)
    system_app.write_text("[Desktop Entry]\nName=VLC\n", encoding="utf-8")

    pinned = isolated_xdg["config"] / "pinned"
    pinned.mkdir(parents=True, exist_ok=True)
    (pinned / "vlc-link").symlink_to(system_app)

    legacy = detect()
    assert legacy is not None
    assert legacy.pinned_apps == ["vlc.desktop"]
