"""Tests for ``mackes.mackesd_bridge`` (Phase 12.13.3 cutover).

The production path of every public function shells out to the real
``mackesd`` binary. Here we mock ``subprocess.run`` so the tests are
hermetic — `test_mackesd_bridge_smoke_real_binary` covers the
production code path against the actual binary when one is on
``PATH``.
"""
from __future__ import annotations

import json
import logging
import shutil
import subprocess
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Shared fixture: a clean bridge module per test.
# ---------------------------------------------------------------------------


@pytest.fixture
def bridge(monkeypatch, tmp_path):
    """Re-import ``mackesd_bridge`` with a private XDG_CONFIG_HOME.

    Each test gets an empty config home so the [migration] flag
    resolution starts at the default, plus a reset deprecation-log
    dedupe set and a reset availability cache.

    We also force the ``mackes`` parent logger to propagate so
    pytest's ``caplog`` (which subscribes via the root logger) can
    observe records emitted on ``mackes.mackesd_bridge``. The shared
    ``mackes.logging.get_logger()`` sets ``propagate=False`` on the
    rotating-file log so production builds don't leak our log lines
    into other handlers — but in-process tests need them to surface
    on the root chain to hit caplog.
    """
    import logging as _logging

    config = tmp_path / "config"
    config.mkdir(parents=True, exist_ok=True)
    monkeypatch.setenv("XDG_CONFIG_HOME", str(config))
    # Make sure the env override doesn't bleed in from the host shell.
    monkeypatch.delenv("MACKES_USE_MACKESD", raising=False)

    parent = _logging.getLogger("mackes")
    saved_propagate = parent.propagate
    parent.propagate = True

    import mackes.mackesd_bridge as mb
    mb._invalidate_availability_cache()
    mb._reset_deprecation_log_for_tests()
    try:
        yield mb
    finally:
        parent.propagate = saved_propagate
        mb._reset_deprecation_log_for_tests()


# ---------------------------------------------------------------------------
# 1. Bridge availability detection
# ---------------------------------------------------------------------------


def test_mackesd_available_finds_binary_on_path(bridge, monkeypatch):
    """`_mackesd_available()` returns True when ``shutil.which`` resolves."""
    monkeypatch.setattr(bridge.shutil, "which",
                        lambda name: "/usr/bin/mackesd" if name == "mackesd" else None)
    bridge._invalidate_availability_cache()
    assert bridge._mackesd_available() is True


def test_mackesd_available_returns_false_when_missing(bridge, monkeypatch):
    """`_mackesd_available()` returns False + caches the answer."""
    monkeypatch.setattr(bridge.shutil, "which", lambda name: None)
    bridge._invalidate_availability_cache()
    assert bridge._mackesd_available() is False
    # Cached — flipping `which` afterwards still reports False until
    # the cache is invalidated.
    monkeypatch.setattr(bridge.shutil, "which", lambda name: "/x/mackesd")
    assert bridge._mackesd_available() is False
    bridge._invalidate_availability_cache()
    assert bridge._mackesd_available() is True


# ---------------------------------------------------------------------------
# 2. JSON parsing — HealthReport
# ---------------------------------------------------------------------------


_HEALTHZ_FIXTURE = (
    '{"schema":1,"is_leader":false,"applied_revision":null,'
    '"node_count":3,"healthy_nodes":2,"degraded_nodes":1,'
    '"unreachable_nodes":0,"audit_chain_intact":true,'
    '"version":"0.0.0"}'
)


def test_health_report_from_json_parses_valid_payload(bridge):
    """`HealthReport.from_json` deserializes the canonical fixture."""
    report = bridge.HealthReport.from_json(_HEALTHZ_FIXTURE)
    assert report.schema == 1
    assert report.is_leader is False
    assert report.applied_revision is None
    assert report.node_count == 3
    assert report.healthy_nodes == 2
    assert report.degraded_nodes == 1
    assert report.unreachable_nodes == 0
    assert report.audit_chain_intact is True
    assert report.version == "0.0.0"


def test_health_report_from_json_rejects_invalid_json(bridge):
    """Malformed JSON raises ``ValueError`` (caller falls back)."""
    with pytest.raises(ValueError):
        bridge.HealthReport.from_json("not-json")


def test_health_report_from_json_rejects_missing_field(bridge):
    """Missing required keys raise ``ValueError``."""
    with pytest.raises(ValueError):
        bridge.HealthReport.from_json('{"schema": 1}')


# ---------------------------------------------------------------------------
# 3. Feature-flag on/off behavior
# ---------------------------------------------------------------------------


def test_health_returns_none_when_flag_off(bridge, monkeypatch, caplog):
    """Flag-off path returns None + emits a [deprecated] warning."""
    # Make sure the bridge would otherwise shell out — binary available,
    # subprocess returns the fixture.
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0, stdout=_HEALTHZ_FIXTURE, stderr=""
        ),
    )
    # Flag defaults to False on 1.x; verify no panel.toml exists.
    caplog.set_level(logging.WARNING, logger=bridge.logger.name)
    assert bridge.health() is None
    assert any(
        "[deprecated]" in rec.message and "use_mackesd = false" in rec.message
        for rec in caplog.records
    )


def test_health_flag_on_via_env_override_shells_out(bridge, monkeypatch):
    """`MACKES_USE_MACKESD=1` flips the flag without touching panel.toml."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0, stdout=_HEALTHZ_FIXTURE, stderr=""
        ),
    )
    report = bridge.health()
    assert report is not None
    assert report.node_count == 3
    assert report.healthy_nodes == 2


def test_health_flag_on_via_panel_toml(bridge, tmp_path, monkeypatch):
    """`[migration].use_mackesd = true` in panel.toml flips the flag."""
    panel_toml = bridge._panel_toml_path()
    panel_toml.parent.mkdir(parents=True, exist_ok=True)
    panel_toml.write_text(
        "[top_bar]\n"
        'status_items = ["mesh"]\n'
        "appmenu = true\n\n"
        "[mesh]\n"
        "replicate = true\n"
        "drift_check_seconds = 300\n\n"
        "[migration]\n"
        "use_mackesd = true\n",
        encoding="utf-8",
    )
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0, stdout=_HEALTHZ_FIXTURE, stderr=""
        ),
    )
    assert bridge._read_use_mackesd_flag() is True
    report = bridge.health()
    assert report is not None


# ---------------------------------------------------------------------------
# 4. Deprecation log emission
# ---------------------------------------------------------------------------


def test_deprecated_warning_emitted_once_per_reason(bridge, monkeypatch, caplog):
    """Multiple flag-off calls log exactly one [deprecated] line per reason."""
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    caplog.set_level(logging.WARNING, logger=bridge.logger.name)

    for _ in range(5):
        assert bridge.health() is None

    deprecated_lines = [
        rec for rec in caplog.records if "[deprecated]" in rec.message
    ]
    assert len(deprecated_lines) == 1


# ---------------------------------------------------------------------------
# 5. Fallback on bridge-unavailable (binary not on PATH)
# ---------------------------------------------------------------------------


def test_health_returns_none_when_binary_missing(bridge, monkeypatch, caplog):
    """Flag on, but binary missing → None + deprecated log."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: False)
    caplog.set_level(logging.WARNING, logger=bridge.logger.name)
    assert bridge.health() is None
    assert any(
        "mackesd not on PATH" in rec.message and "[deprecated]" in rec.message
        for rec in caplog.records
    )


def test_health_returns_none_on_subprocess_failure(bridge, monkeypatch):
    """Flag on, subprocess exits non-zero → None (caller falls back)."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=2, stdout="", stderr="boom"
        ),
    )
    assert bridge.health() is None


def test_health_returns_none_on_timeout(bridge, monkeypatch):
    """Subprocess timeout falls back gracefully."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)

    def boom(args):
        raise subprocess.TimeoutExpired(cmd=["mackesd", *args], timeout=5.0)

    monkeypatch.setattr(bridge, "_run_mackesd", boom)
    assert bridge.health() is None


# ---------------------------------------------------------------------------
# 6. Valid JSON shape per HealthReport (every field is present + typed)
# ---------------------------------------------------------------------------


def test_health_report_has_every_expected_field(bridge):
    """Parsed report's dataclass shape matches mackesd_core::health."""
    report = bridge.HealthReport.from_json(_HEALTHZ_FIXTURE)
    expected_fields = {
        "schema", "is_leader", "applied_revision", "node_count",
        "healthy_nodes", "degraded_nodes", "unreachable_nodes",
        "audit_chain_intact", "version",
    }
    # dataclasses.fields preserves declaration order; convert to set
    # for membership checks.
    import dataclasses
    actual_fields = {f.name for f in dataclasses.fields(report)}
    assert actual_fields == expected_fields


# ---------------------------------------------------------------------------
# 7. peers_why + audit_verify + paired_inventory paths
# ---------------------------------------------------------------------------


def test_peers_why_returns_stdout(bridge, monkeypatch):
    """`peers_why` returns the stripped stdout of the subprocess."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0,
            stdout="peer:anvil: reason chain ...\n", stderr="",
        ),
    )
    out = bridge.peers_why("peer:anvil")
    assert out == "peer:anvil: reason chain ..."


def test_audit_verify_intact(bridge, monkeypatch):
    """`audit_verify` returns `intact=True` when mackesd exits 0."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0,
            stdout="verified 42 events  ·  chain intact\n", stderr="",
        ),
    )
    outcome = bridge.audit_verify()
    assert outcome is not None
    assert outcome.intact is True
    assert outcome.exit_code == 0
    assert outcome.is_empty is False


def test_audit_verify_break(bridge, monkeypatch):
    """`audit_verify` returns `intact=False` when mackesd exits 1."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=1,
            stdout="", stderr="audit chain BREAK at event 7",
        ),
    )
    outcome = bridge.audit_verify()
    assert outcome is not None
    assert outcome.intact is False
    assert outcome.exit_code == 1
    assert "BREAK" in outcome.message


def test_paired_inventory_parses_json_array(bridge, monkeypatch):
    """`paired_inventory` returns a typed list of LegacyArtifact."""
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    monkeypatch.setattr(bridge, "_mackesd_available", lambda: True)
    fixture = json.dumps([
        {
            "path": "/home/mm/.config/mackes-shell/state.json",
            "size_bytes": 256,
            "mtime_ms": 1700000000000,
            "artifact_kind": "json_config",
            "mesh_data": False,
        },
        {
            "path": "/home/mm/.config/mackes-shell/mesh-peers.json",
            "size_bytes": 1024,
            "mtime_ms": 1700000000001,
            "artifact_kind": "json_config",
            "mesh_data": True,
        },
    ])
    monkeypatch.setattr(
        bridge, "_run_mackesd",
        lambda args: subprocess.CompletedProcess(
            args=args, returncode=0, stdout=fixture, stderr=""
        ),
    )
    inventory = bridge.paired_inventory()
    assert inventory is not None
    assert len(inventory) == 2
    assert inventory[1].mesh_data is True
    assert inventory[1].path == Path("/home/mm/.config/mackes-shell/mesh-peers.json")
    assert inventory[1].artifact_kind == "json_config"


# ---------------------------------------------------------------------------
# 8. set_use_mackesd_flag round-trips through panel.toml
# ---------------------------------------------------------------------------


def test_set_use_mackesd_flag_round_trips(bridge):
    """`set_use_mackesd_flag(True)` makes `_read_use_mackesd_flag()` True."""
    written = bridge.set_use_mackesd_flag(True)
    assert written == bridge._panel_toml_path()
    assert bridge._read_use_mackesd_flag() is True

    bridge.set_use_mackesd_flag(False)
    assert bridge._read_use_mackesd_flag() is False


# ---------------------------------------------------------------------------
# 9. Real-binary smoke test — only when mackesd is on PATH.
# ---------------------------------------------------------------------------


def _resolve_real_mackesd() -> str | None:
    """Find a ``mackesd`` binary new enough to expose ``healthz``.

    Preference order: ``target/release/mackesd`` under the repo (the
    dev build) before any system-wide binary, since the dev build is
    what the current branch's CLI surface matches. Returns ``None``
    when no candidate supports the modern subcommand set.
    """
    repo_release = (
        Path(__file__).resolve().parent.parent
        / "target" / "release" / "mackesd"
    )
    candidates = []
    if repo_release.is_file():
        candidates.append(str(repo_release))
    on_path = shutil.which("mackesd")
    if on_path:
        candidates.append(on_path)
    for cand in candidates:
        try:
            proc = subprocess.run(  # noqa: S603
                [cand, "healthz"],
                capture_output=True, text=True, timeout=5.0, check=False,
            )
        except (OSError, subprocess.TimeoutExpired):
            continue
        if proc.returncode == 0 and proc.stdout.lstrip().startswith("{"):
            return cand
    return None


def test_real_mackesd_healthz_shells_out_and_parses(bridge, monkeypatch, tmp_path):
    """End-to-end smoke against the actual binary.

    Production code path: env override flips the flag on, the bridge
    invokes the binary for real, parses the JSON, and returns a
    populated HealthReport. The CLI's empty-baseline output (a fresh
    install with no events) is enough to satisfy the schema check.
    """
    real = _resolve_real_mackesd()
    if real is None:
        pytest.skip(
            "no `mackesd healthz`-capable binary on PATH or in "
            "target/release — install the RPM or run `make rust`."
        )

    # Make the bridge use the resolved binary regardless of what's
    # on the host PATH. We arrange a one-entry bin/ dir on PATH that
    # symlinks to the dev build.
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    (bin_dir / "mackesd").symlink_to(real)
    monkeypatch.setenv("PATH", str(bin_dir))
    monkeypatch.setenv("MACKES_USE_MACKESD", "1")
    bridge._invalidate_availability_cache()
    report = bridge.health()
    assert report is not None
    assert report.schema == 1
    assert isinstance(report.version, str) and report.version
