"""CB-2.4 — mde-firstboot.target + two migrator services smoke tests."""
from __future__ import annotations

import sys
from pathlib import Path

import pytest


REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO))


TARGET = REPO / "data/systemd/mde-firstboot.target"
MIGRATE_1X = REPO / "data/systemd/mde-migrate-from-1x.service"
MIGRATE_V2 = REPO / "data/systemd/mde-shell-migrate-v2.service"
SESSION = REPO / "data/systemd/mde-session.service"
SPEC = REPO / "packaging/fedora/mackes-shell.spec"


def test_firstboot_target_ships():
    assert TARGET.is_file()


def test_firstboot_target_is_a_target_not_a_service():
    src = TARGET.read_text()
    # Must NOT have [Service] section — targets are pure sync points.
    assert "[Service]" not in src
    assert "[Install]" in src


def test_firstboot_target_short_circuits_when_markers_exist():
    """Both completion-marker conditions in the [Unit] section so
    the target activates only when at least one migrator hasn't
    yet run."""
    src = TARGET.read_text()
    assert "ConditionPathExists=|!%h/.cache/mde/.migrate-from-1x.done" in src
    assert "ConditionPathExists=|!%h/.cache/mde/.shell-migrate-v2.done" in src


def test_migrate_from_1x_service_is_oneshot_and_marker_gated():
    src = MIGRATE_1X.read_text()
    assert "Type=oneshot" in src
    assert "RemainAfterExit=true" in src
    assert "ConditionPathExists=!%h/.cache/mde/.migrate-from-1x.done" in src
    assert "PartOf=mde-firstboot.target" in src
    assert "ExecStart=/usr/bin/mde-migrate-from-1x" in src


def test_migrate_shell_v2_service_is_oneshot_and_marker_gated():
    src = MIGRATE_V2.read_text()
    assert "Type=oneshot" in src
    assert "RemainAfterExit=true" in src
    assert "ConditionPathExists=!%h/.cache/mde/.shell-migrate-v2.done" in src
    assert "PartOf=mde-firstboot.target" in src
    assert "ExecStart=/usr/bin/mde-shell-migrate-v2" in src


def test_migrate_shell_v2_orders_after_migrate_from_1x():
    """The xfconf-replay migrator must run AFTER the config-tree
    move so the new paths exist before the replay writes to
    them."""
    src = MIGRATE_V2.read_text()
    assert "After=local-fs.target mde-migrate-from-1x.service" in src


def test_session_service_gates_on_firstboot_target():
    src = SESSION.read_text()
    assert "Wants=mde-firstboot.target" in src
    assert "After=mde-firstboot.target" in src


def test_session_service_drops_direct_after_on_migrate_from_1x():
    """The Wants= mde-firstboot.target now covers what the old
    direct After= on the migrator did; the direct ordering line
    should be gone."""
    src = SESSION.read_text()
    # The old line was the bare `After=mde-migrate-from-1x.service`
    # at the top of [Unit]; it lives only inside the firstboot
    # target now.
    direct_after_lines = [
        line for line in src.splitlines()
        if line.strip() == "After=mde-migrate-from-1x.service"
    ]
    assert not direct_after_lines, (
        "session.service must not have a direct After= on the "
        "migrator — let the firstboot target order it"
    )


def test_spec_installs_target_and_migrators():
    src = SPEC.read_text()
    assert "data/systemd/mde-firstboot.target" in src
    assert "data/systemd/mde-migrate-from-1x.service" in src
    assert "data/systemd/mde-shell-migrate-v2.service" in src
    assert "%{_userunitdir}/mde-firstboot.target" in src
    assert "%{_userunitdir}/mde-migrate-from-1x.service" in src
    assert "%{_userunitdir}/mde-shell-migrate-v2.service" in src


def test_target_uses_default_target_for_install():
    """Target is WantedBy=default.target so the user systemd
    instance pulls it in at login without needing per-account
    `systemctl --user enable`."""
    src = TARGET.read_text()
    assert "WantedBy=default.target" in src


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
