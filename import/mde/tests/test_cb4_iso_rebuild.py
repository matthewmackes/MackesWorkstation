"""CB-4.1/4.2/4.4 — ISO rebuild smoke tests."""
from __future__ import annotations

import sys
from pathlib import Path

import pytest


REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO))


KS = REPO / "packaging/iso/mde.ks"
LEGACY_KS = REPO / "packaging/iso/mackes-xfce.ks"
MAKEFILE = REPO / "Makefile"
ISO_README = REPO / "packaging/iso/README.md"


def test_legacy_kickstart_deleted():
    """CB-4.1 — mackes-xfce.ks must be gone."""
    assert not LEGACY_KS.exists(), "v1.x kickstart must be removed at CB-4.1"


def test_mde_kickstart_ships():
    """CB-4.2 — mde.ks must ship."""
    assert KS.is_file()


def test_mde_kickstart_pulls_wayland_stack():
    src = KS.read_text()
    for pkg in (
        "sway",
        "swaylock",
        "swayidle",
        "swaybg",
        "foot",
        "bemenu",
        "brightnessctl",
        "pipewire",
        "wireplumber",
        "grim",
        "slurp",
        "kanshi",
        "wl-clipboard",
        "wlr-randr",
        "lightdm",
        "mde",
    ):
        assert pkg in src, f"kickstart must install {pkg}"


def test_mde_kickstart_drops_xfce_group():
    src = KS.read_text()
    # Only look at the non-comment lines in the %packages block —
    # the lock comment intentionally mentions @xfce-desktop-environment
    # as "what we no longer install."
    pkgs_start = src.find("%packages")
    pkgs_end = src.find("%end", pkgs_start)
    pkg_lines = [
        ln.strip()
        for ln in src[pkgs_start:pkgs_end].splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    ]
    for ln in pkg_lines:
        assert "@xfce-desktop-environment" != ln, (
            f"kickstart still installs {ln!r}"
        )
        assert not ln.startswith("xfce4-"), (
            f"kickstart still installs xfce4 pkg {ln!r}"
        )


def test_mde_kickstart_seeds_state_json_at_new_path():
    src = KS.read_text()
    # New v2.0.0 path is ~/.config/mde/, not ~/.config/mackes-shell/.
    assert "/etc/skel/.config/mde/state.json" in src
    assert "/etc/skel/.config/mackes-shell/state.json" not in src


def test_mde_kickstart_seeds_lightdm_user_session():
    """CB-2.3 — kickstart writes the user-session=mde drop-in."""
    src = KS.read_text()
    assert "/etc/lightdm/lightdm.conf.d/50-mde.conf" in src
    assert "user-session=mde" in src


def test_mde_kickstart_registers_comps_group():
    """CB-3.4 — kickstart marks the comps group at install time."""
    src = KS.read_text()
    assert "dnf groups mark install mackes-desktop-environment" in src


def test_mde_kickstart_stages_wallpaper_at_new_path():
    """CB-4.3 — wallpaper lands at /usr/share/backgrounds/mde-default.png
    (was mackes-default.png on the v1.x ISO)."""
    src = KS.read_text()
    assert "/usr/share/backgrounds/mde-default.png" in src
    assert "/usr/share/backgrounds/mackes-default.png" not in src


def test_makefile_iso_points_at_mde_kickstart():
    """CB-4.4 — make iso targets mde.ks + MDE volid + project name."""
    src = MAKEFILE.read_text()
    assert "packaging/iso/mde.ks" in src
    assert "--volid \"MDE\"" in src
    assert "Mackes Desktop Environment" in src
    # Old kickstart reference must be gone.
    assert "packaging/iso/mackes-xfce.ks" not in src
    assert "MACKES_XFCE" not in src


def test_iso_readme_reflects_mde_rebrand():
    src = ISO_README.read_text()
    # Starts with MDE title (CB-6.x rebrand sweep covers helper READMEs).
    assert "Mackes Desktop Environment" in src
    # References mde.ks (not mackes-xfce.ks).
    assert "mde.ks" in src
    assert "mackes-xfce.ks" not in src


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
