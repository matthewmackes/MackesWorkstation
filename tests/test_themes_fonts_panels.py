"""v2.0.0 Phase F.3 — tests for the MDE Themes + Fonts panels.

Exercises the pure-helper surface (`discover_*_themes`) and the
import-side-effects + bridge-key contract. GTK construction itself is
covered by the headless smoke (xvfb) — these tests just guard the
non-GTK contracts.
"""
from __future__ import annotations

import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def test_themes_module_imports():
    from mackes.workbench.look_and_feel import themes
    assert callable(themes.discover_gtk_themes)
    assert callable(themes.discover_icon_themes)
    assert hasattr(themes, "ThemesPanel")


def test_fonts_module_imports():
    from mackes.workbench.look_and_feel import fonts
    assert hasattr(fonts, "FontsPanel")
    assert fonts._HINTING_LEVELS == ("none", "slight", "medium", "full")
    assert fonts._AA_MODES == ("none", "grayscale", "rgba")


def test_discover_gtk_themes_returns_strings():
    from mackes.workbench.look_and_feel import themes
    found = themes.discover_gtk_themes()
    assert isinstance(found, list)
    for t in found:
        assert isinstance(t, str)
        assert t  # no empty names


def test_discover_icon_themes_returns_strings():
    from mackes.workbench.look_and_feel import themes
    found = themes.discover_icon_themes()
    assert isinstance(found, list)
    for t in found:
        assert isinstance(t, str)
        assert t


def test_themes_module_imports_bridge_only_no_xfconf():
    """F.3 lock — themes.py must NOT import xfconf_bridge."""
    import inspect
    from mackes.workbench.look_and_feel import themes
    src = inspect.getsource(themes)
    assert "xfconf_bridge" not in src, "themes.py must not import xfconf_bridge"
    assert "mde_settings_bridge" in src, "themes.py must use mde_settings_bridge"


def test_fonts_module_imports_bridge_only_no_xfconf():
    """F.3 lock — fonts.py must NOT import xfconf_bridge."""
    import inspect
    from mackes.workbench.look_and_feel import fonts
    src = inspect.getsource(fonts)
    assert "xfconf_bridge" not in src, "fonts.py must not import xfconf_bridge"
    assert "mde_settings_bridge" in src, "fonts.py must use mde_settings_bridge"


def test_themes_writes_locked_mde_keys():
    """Themes panel writes exactly theme.name / theme.icon_set /
    theme.mode through the bridge."""
    import inspect
    from mackes.workbench.look_and_feel import themes
    src = inspect.getsource(themes)
    assert "theme.name" in src
    assert "theme.icon_set" in src
    assert "theme.mode" in src


def test_fonts_writes_locked_mde_keys():
    """Fonts panel writes exactly font.name / font.monospace /
    font.hinting / font.antialias through the bridge."""
    import inspect
    from mackes.workbench.look_and_feel import fonts
    src = inspect.getsource(fonts)
    for key in ("font.name", "font.monospace", "font.hinting", "font.antialias"):
        assert key in src, f"fonts.py must reference {key}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
