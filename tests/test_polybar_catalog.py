"""Tests for the polybar catalog + generator."""
from __future__ import annotations

import pytest

from mackes import polybar_catalog as cat
from mackes import polybar_gen as gen


def test_families_discovered():
    families = cat.list_families()
    assert len(families) > 10, "expected at least the bulk of adi1090x families"
    keys = {f.key for f in families}
    # spot-check a few known families ship in both variants
    assert "simple/forest" in keys
    assert "simple/material" in keys
    assert "simple/shapes" in keys
    assert "bitmap/shapes" in keys


def test_family_key_lookup():
    assert cat.get_family("forest") is not None
    assert cat.get_family("simple/forest") is not None
    assert cat.get_family("definitely/notreal") is None
    assert cat.get_family("notreal") is None


def test_modules_extracted():
    fam = cat.get_family("forest")
    assert fam is not None
    mods = cat.list_modules(fam)
    names = {m.name for m in mods}
    # forest is one of the standard families; should expose common modules
    assert "workspaces" in names
    assert "date" in names


def test_palette_extracted():
    fam = cat.get_family("forest")
    assert fam is not None
    pal = cat.palette(fam)
    assert "background" in pal
    assert "foreground" in pal
    assert pal["background"].startswith("#")


def test_bar_layout_has_modules():
    fam = cat.get_family("forest")
    assert fam is not None
    layout = cat.bar_layout(fam)
    # forest's default bar should populate left + right at least
    assert len(layout.modules_left) > 0
    assert len(layout.modules_right) > 0


def test_generate_returns_self_contained_ini():
    out = gen.generate(gen.GenOptions(family_key="forest"))
    assert "[bar/mackes]" in out
    # palette inlined, not included by reference
    assert "include-file" not in out
    # honors family-default layout
    assert "modules-left = " in out
    assert "modules-right = " in out


def test_generate_overrides_geometry():
    opts = gen.GenOptions(family_key="forest", position="bottom", height=22, radius=4)
    out = gen.generate(opts)
    assert "bottom = true" in out
    assert "height = 22" in out
    assert "radius = 4" in out


def test_generate_overrides_modules():
    opts = gen.GenOptions(
        family_key="forest",
        use_family_layout=False,
        modules_left=("workspaces",),
        modules_center=("date",),
        modules_right=("volume", "battery"),
    )
    out = gen.generate(opts)
    assert "modules-left = workspaces" in out
    assert "modules-center = date" in out
    assert "modules-right = volume battery" in out


def test_generate_unknown_family_raises():
    with pytest.raises(ValueError, match="unknown family"):
        gen.generate(gen.GenOptions(family_key="not-a-real-family"))
