"""v2.0.0 Phase 0.13 — identifier-presence tests for the rebrand sweep.

Asserts the new MDE-namespaced identifiers actually ship in the
matching artifacts: spec Provides/Obsoletes line, CHANGELOG 2.0.0
header, metainfo component-id surface, bin/ shim presence, man
pages installed. Complements the per-area tests (D-Bus surfaces +
config-path migrator + env-var shim) shipped earlier.
"""
from __future__ import annotations

from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


# --- spec Provides / Obsoletes parse ------------------------------------

def test_spec_advertises_mde_provides():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    assert "Provides:       mde = %{version}-%{release}" in spec, (
        "spec must declare `Provides: mde = ...` so `dnf install mde` "
        "resolves to this RPM during the v1.x → v2.0.0 transition"
    )


def test_spec_keeps_legacy_mackes_shell_provides():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    assert "Provides:       mackes-shell = %{version}-%{release}" in spec, (
        "spec must keep `Provides: mackes-shell` so installs from the "
        "1.0.6+ line resolve cleanly through the rename"
    )


def test_spec_obsoletes_pre_v3_legacy():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    assert "Obsoletes:      mackes-shell < 3.0" in spec


def test_spec_drops_retired_systemd_install_lines():
    """Phase B.13 retired 10 standalone units; their install lines
    should NOT be in the spec anymore."""
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    for unit in (
        "data/systemd/mackes-clipboard-daemon.service",
        "data/systemd/mackes-gvfsd-mesh.service",
        "data/systemd/mackes-mdns-relay.service",
        "data/systemd/mackes-remmina-sync.service",
        "data/systemd/mackes-remmina-sync.timer",
        "data/systemd/mackes-media-sync.service",
        "data/systemd/mackes-media-sync.timer",
        "data/systemd/mackes-ansible-pull.service",
        "data/systemd/mackes-ansible-pull.timer",
        "data/systemd/mackesd-kdc-bridge.service",
    ):
        assert f"install -m 0644 {unit}" not in spec, (
            f"retired unit install line still in spec: {unit}"
        )


def test_spec_installs_new_mde_session_unit():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    assert "install -m 0644 data/systemd/mde-session.service" in spec


def test_spec_installs_new_mde_binaries():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    for bin_name in ("mde", "mde-wm", "mde-enforce-session",
                     "mde-migrate-from-1x", "mde-shell-migrate-v2"):
        assert f"bin/{bin_name}" in spec, (
            f"new mde-* binary {bin_name} missing from spec install lines"
        )


def test_spec_installs_new_man_pages():
    spec = (REPO / "packaging" / "fedora" / "mackes-shell.spec").read_text()
    for man in ("mde.1", "mded.8",
                "mde-migrate-from-1x.1", "mde-shell-migrate-v2.1"):
        assert f"data/man/{man}" in spec, (
            f"man page {man} missing from spec install lines"
        )


# --- CHANGELOG 2.0.0 header --------------------------------------------

def test_changelog_carries_v2_0_0_entry():
    text = (REPO / "CHANGELOG.md").read_text()
    assert "## 2.0.0 — Rebrand to Mackes Desktop Environment (MDE)" in text


def test_changelog_v2_entry_documents_upgrade_path():
    text = (REPO / "CHANGELOG.md").read_text()
    # Three critical upgrade-path elements: Obsoletes/Provides,
    # config-path migrator, env-var shim.
    assert "Obsoletes" in text or "Provides" in text
    assert "mde-migrate-from-1x" in text
    assert "MDE_*" in text and "MACKES_*" in text


def test_changelog_v2_entry_documents_unified_daemon():
    text = (REPO / "CHANGELOG.md").read_text()
    assert "Unified Rust meta-daemon" in text or "unified Rust" in text.lower()


# --- bin/ + man/ presence ----------------------------------------------

def test_every_new_mde_bin_shim_ships():
    for name in ("mde", "mde-wm", "mde-enforce-session",
                 "mde-migrate-from-1x", "mde-shell-migrate-v2"):
        p = REPO / "bin" / name
        assert p.is_file(), f"missing bin shim: {p}"
        assert p.stat().st_mode & 0o111, f"bin shim not executable: {p}"


def test_every_new_man_page_ships():
    for name in ("mde.1", "mde-migrate-from-1x.1", "mde-shell-migrate-v2.1"):
        p = REPO / "data" / "man" / name
        assert p.is_file(), f"missing man page: {p}"
    assert (REPO / "data" / "man" / "mded.8").is_file()


# --- identifier-table reference doc -----------------------------------

def test_identifier_table_doc_ships():
    p = REPO / "docs" / "design" / "v2.0.0-mde-rebrand" / "identifiers.md"
    assert p.is_file()
    text = p.read_text()
    # Spot-check that the canonical mappings are documented.
    assert "mackes-shell" in text and "mde" in text
    assert "mackesd" in text and "mded" in text
    assert ("shell.mackes" in text or "org.mackes" in text) \
        and "dev.mackes.MDE" in text
    assert "MACKES_" in text and "MDE_" in text


def test_cosmic_files_upstream_pin_doc_ships():
    p = REPO / "docs" / "upstream" / "cosmic-files.md"
    assert p.is_file()
    text = p.read_text()
    assert "cosmic-files" in text
    assert "GPL-3.0" in text


def test_cosmic_files_license_attribution_ships():
    p = REPO / "LICENSES" / "COSMIC-FILES.md"
    assert p.is_file()
    text = p.read_text()
    assert "cosmic-files" in text
    assert "System76" in text
    assert "GPL-3.0" in text
