"""Shell profile listing, application (file copy), and Plank theme machinery."""
from __future__ import annotations

from pathlib import Path


def test_list_polybar_profiles(isolated_xdg):
    from mackes.shell_profiles import list_polybar_profiles
    profiles = list_polybar_profiles()
    # After preset cleanup the only shipped profile is chupre-custom.
    assert "chupre-custom" in profiles


def test_list_plank_themes_includes_shipped(isolated_xdg):
    from mackes.shell_profiles import list_plank_themes, shipped_plank_themes
    shipped = {p.name for p in shipped_plank_themes()}
    themes = set(list_plank_themes())
    # Plank built-ins + at least the shipped Mackes catalog
    assert "Default" in themes
    assert shipped <= themes


def test_install_shipped_plank_themes(isolated_xdg):
    from mackes.shell_profiles import (
        install_shipped_plank_themes, PLANK_THEMES_INSTALL_DIR, shipped_plank_themes,
    )
    actions = install_shipped_plank_themes()
    assert actions
    # Every shipped theme should now exist under ~/.local/share/plank/themes
    for theme in shipped_plank_themes():
        assert (PLANK_THEMES_INSTALL_DIR / theme.name / "dock.theme").exists()


def test_apply_polybar_writes_config_and_autostart(isolated_xdg):
    from mackes.shell_profiles import (
        POLYBAR_DIR, POLYBAR_ACTIVE_MARKER, POLYBAR_AUTOSTART, POLYBAR_LAUNCHER,
        apply_polybar,
    )
    actions = apply_polybar("chupre-custom")
    # Active marker reflects the chosen profile
    assert POLYBAR_ACTIVE_MARKER.exists()
    assert POLYBAR_ACTIVE_MARKER.read_text(encoding="utf-8").strip() == "chupre-custom"
    # Config + launcher script + autostart .desktop (P1 lock) all present
    assert (POLYBAR_DIR / "config.ini").exists()
    assert POLYBAR_LAUNCHER.exists()
    assert POLYBAR_AUTOSTART.exists()
    autostart_text = POLYBAR_AUTOSTART.read_text(encoding="utf-8")
    assert "X-Mackes-Managed=1" in autostart_text
    # Launcher invokes by the detected bar name, not a hardcoded one.
    launcher_text = POLYBAR_LAUNCHER.read_text(encoding="utf-8")
    assert "polybar --config=" in launcher_text
    assert any("polybar: profile -> chupre-custom" in a for a in actions)
    assert any("bars detected" in a for a in actions)


def test_xfce_panel_enable_disable(isolated_xdg):
    from mackes.shell_profiles import (
        xfce_panel_enabled, set_xfce_panel_enabled, XFCE_PANEL_AUTOSTART,
    )
    assert xfce_panel_enabled() is True  # no override = enabled

    set_xfce_panel_enabled(False)
    assert XFCE_PANEL_AUTOSTART.exists()
    assert xfce_panel_enabled() is False

    set_xfce_panel_enabled(True)
    assert not XFCE_PANEL_AUTOSTART.exists()
    assert xfce_panel_enabled() is True
