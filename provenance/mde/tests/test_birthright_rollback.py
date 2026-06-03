"""Tests for the Phase 10.6.8 rollback ledger.

The module under test (`mackes/birthright_rollback.py`) writes JSON
records to `<rollback_dir>/<step>.json` and replays them in reverse on
`restore_one` / `restore_all`. These tests verify:

  1. `record()` writes a valid file with the expected schema.
  2. `list_recent` returns records newest-first, honoring the JSON
     `timestamp` field rather than mtime.
  3. `restore_one` executes the recorded actions in REVERSE order and
     skips unknown action kinds without raising.
  4. `restore_all` chains every step, also newest-first.
  5. Missing-step requests return a sentinel "no record" log line
     instead of raising.
  6. Corrupted JSON files are skipped by `list_recent` without taking
     the well-formed records down with them.
  7. The action executors actually mutate the filesystem (write_file,
     delete_file) in a temp dir — confirms the rollback is not a stub.

Every test scopes `XDG_CONFIG_HOME` into a `tempfile.TemporaryDirectory`
so the developer's real `~/.config/mackes-panel/rollback/` is never
touched. Compatible with the bare `_run_without_pytest` runner — no
fixtures are used.
"""
from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path
from typing import Any, Dict, List

from mackes import birthright_rollback as rb


# ---------------------------------------------------------------------------
# Test helpers
# ---------------------------------------------------------------------------


class _TempLedger:
    """Context-manager that points the rollback ledger at a TemporaryDirectory.

    Equivalent to the pytest `tmp_path` + monkeypatch idiom, but works in
    the runner-without-pytest path the rest of `tests/` follows.
    """

    def __init__(self) -> None:
        self._td: tempfile.TemporaryDirectory | None = None
        self._prev_env: str | None = None

    def __enter__(self) -> Path:
        self._td = tempfile.TemporaryDirectory()
        path = Path(self._td.name) / "config" / "mackes-panel" / "rollback"
        # Set the explicit override so both the path probe and the
        # tests see the same dir. Also export XDG_CONFIG_HOME so the
        # Rust-side reader (cargo test) would resolve the same place
        # if it ever ran from this process.
        self._prev_env = os.environ.get("XDG_CONFIG_HOME")
        os.environ["XDG_CONFIG_HOME"] = str(Path(self._td.name) / "config")
        rb.set_rollback_dir_override(path)
        return path

    def __exit__(self, *exc: Any) -> None:
        rb.set_rollback_dir_override(None)
        if self._prev_env is None:
            os.environ.pop("XDG_CONFIG_HOME", None)
        else:
            os.environ["XDG_CONFIG_HOME"] = self._prev_env
        if self._td is not None:
            self._td.cleanup()


def _make_record(step_name: str, timestamp: str,
                 actions: List[Dict[str, Any]]) -> rb.RollbackStep:
    """Build a RollbackStep with a fixed timestamp — used to control
    sort order in `list_recent` tests."""
    return rb.RollbackStep(
        step_name=step_name,
        timestamp=timestamp,
        prior_state={"synthetic": True},
        restore_actions=actions,
    )


# ---------------------------------------------------------------------------
# 1. record() writes JSON
# ---------------------------------------------------------------------------


def test_record_writes_file_with_expected_schema():
    with _TempLedger() as path:
        out = rb.record(
            "apply_panel_swap",
            prior_state={"autostart_existed": True},
            restore_actions=[
                {"kind": "shell", "argv": ["dnf", "install", "-y", "xfce4-panel"],
                 "needs_root": True, "description": "reinstall xfce4-panel"},
            ],
        )
        assert out == path / "apply_panel_swap.json", \
            f"unexpected output path: {out}"
        assert out.is_file(), "record file was not written"

        data = json.loads(out.read_text(encoding="utf-8"))
        assert data["step_name"] == "apply_panel_swap"
        assert data["prior_state"] == {"autostart_existed": True}
        assert len(data["restore_actions"]) == 1
        assert data["restore_actions"][0]["argv"] == [
            "dnf", "install", "-y", "xfce4-panel"]
        assert data["timestamp"].endswith("Z"), "timestamp not UTC-Z"


# ---------------------------------------------------------------------------
# 2. list_recent() — newest first by JSON timestamp
# ---------------------------------------------------------------------------


def test_list_recent_orders_by_timestamp_desc():
    with _TempLedger() as path:
        path.mkdir(parents=True, exist_ok=True)
        # Write three records with deliberately scrambled mtimes vs.
        # timestamps to prove `list_recent` keys on the JSON field.
        a = _make_record("step_a", "2025-01-01T00:00:00Z", [])
        b = _make_record("step_b", "2026-05-19T12:00:00Z", [])
        c = _make_record("step_c", "2024-06-15T08:30:00Z", [])
        (path / "step_a.json").write_text(a.to_json(), encoding="utf-8")
        (path / "step_b.json").write_text(b.to_json(), encoding="utf-8")
        (path / "step_c.json").write_text(c.to_json(), encoding="utf-8")

        records = rb.list_recent(limit=10)
        assert [r.step_name for r in records] == ["step_b", "step_a", "step_c"], \
            f"expected newest-first ordering, got {[r.step_name for r in records]}"

        # `limit` truncates after the sort.
        first_two = rb.list_recent(limit=2)
        assert [r.step_name for r in first_two] == ["step_b", "step_a"]


# ---------------------------------------------------------------------------
# 3. restore_one() reverses actions
# ---------------------------------------------------------------------------


def test_restore_one_executes_actions_in_reverse_order():
    with _TempLedger() as path:
        td = path.parent  # the temp config dir
        td.mkdir(parents=True, exist_ok=True)
        target_file = td / "target.txt"
        target_file.write_text("ORIGINAL", encoding="utf-8")
        scratch = td / "scratch.txt"

        # Two actions: (1) create scratch with content X, (2) write
        # target with content Y. Reverse order means scratch creation
        # happens AFTER target write — so target ends with Y, scratch
        # exists. Both files prove the executor ran.
        actions = [
            {"kind": "write_file", "path": str(target_file),
             "content": "RESTORED-Y",
             "description": "restore target.txt"},
            {"kind": "write_file", "path": str(scratch),
             "content": "RESTORED-X",
             "description": "create scratch.txt"},
        ]
        rb.record("synthetic_step", {"trace": "test"}, actions)

        lines = rb.restore_one("synthetic_step")

        assert target_file.read_text(encoding="utf-8") == "RESTORED-Y", \
            "write_file action did not restore target.txt"
        assert scratch.read_text(encoding="utf-8") == "RESTORED-X", \
            "write_file action did not create scratch.txt"

        # The first OK line should be the scratch write (last action in
        # the record == first reversed). We check by looking for action
        # descriptions in the log order.
        ok_lines = [ln for ln in lines if "OK" in ln]
        assert len(ok_lines) == 2, f"expected 2 OK lines, got {ok_lines}"
        assert "create scratch.txt" in ok_lines[0], \
            f"expected scratch action first, got: {ok_lines}"
        assert "restore target.txt" in ok_lines[1], \
            f"expected target action second, got: {ok_lines}"


# ---------------------------------------------------------------------------
# 4. restore_all() walks every record newest-first
# ---------------------------------------------------------------------------


def test_restore_all_processes_every_record_newest_first():
    with _TempLedger() as path:
        path.mkdir(parents=True, exist_ok=True)
        marker_a = path.parent / "marker_a.txt"
        marker_b = path.parent / "marker_b.txt"

        older = _make_record(
            "older_step", "2025-01-01T00:00:00Z",
            [{"kind": "write_file", "path": str(marker_a),
              "content": "OLDER", "description": "write marker_a"}],
        )
        newer = _make_record(
            "newer_step", "2026-05-19T12:00:00Z",
            [{"kind": "write_file", "path": str(marker_b),
              "content": "NEWER", "description": "write marker_b"}],
        )
        (path / "older_step.json").write_text(older.to_json(), encoding="utf-8")
        (path / "newer_step.json").write_text(newer.to_json(), encoding="utf-8")

        lines = rb.restore_all()

        # Both markers exist (both records ran).
        assert marker_a.is_file()
        assert marker_b.is_file()
        # Newer step should be mentioned BEFORE older step in the log.
        newer_idx = next(i for i, ln in enumerate(lines) if "newer_step" in ln)
        older_idx = next(i for i, ln in enumerate(lines) if "older_step" in ln)
        assert newer_idx < older_idx, (
            f"expected newer_step before older_step, got order: {lines}"
        )


# ---------------------------------------------------------------------------
# 5. Missing step name surfaces a clean message
# ---------------------------------------------------------------------------


def test_restore_one_missing_step_returns_message_does_not_raise():
    with _TempLedger():
        lines = rb.restore_one("never_recorded")
        assert any("no record" in ln for ln in lines), \
            f"expected 'no record' line, got: {lines}"
        # Definitely did NOT raise — we got back a list.
        assert isinstance(lines, list)


def test_restore_all_empty_ledger_surfaces_clean_message():
    with _TempLedger():
        lines = rb.restore_all()
        assert any("no records found" in ln for ln in lines), \
            f"expected 'no records found' line, got: {lines}"


# ---------------------------------------------------------------------------
# 6. Corrupted JSON files are tolerated
# ---------------------------------------------------------------------------


def test_list_recent_skips_corrupt_json_keeps_valid_records():
    with _TempLedger() as path:
        path.mkdir(parents=True, exist_ok=True)
        # One valid record.
        good = _make_record("good_step", "2026-05-19T12:00:00Z", [])
        (path / "good_step.json").write_text(good.to_json(), encoding="utf-8")
        # One file that's valid JSON but missing required fields — should
        # still parse via the `.get()` defaults; that's not corrupt.
        (path / "partial.json").write_text('{"step_name": "partial"}',
                                           encoding="utf-8")
        # One outright corrupt file.
        (path / "broken.json").write_text("this is not json at all{",
                                          encoding="utf-8")

        records = rb.list_recent(limit=10)
        names = [r.step_name for r in records]
        assert "good_step" in names
        assert "partial" in names
        # Broken file did NOT raise, just got dropped silently (logged).
        assert all(n != "" for n in names if n is not None)


def test_load_step_returns_none_for_corrupt_file():
    with _TempLedger() as path:
        path.mkdir(parents=True, exist_ok=True)
        (path / "wrecked.json").write_text("{ not-real-json", encoding="utf-8")
        assert rb.load_step("wrecked") is None


# ---------------------------------------------------------------------------
# 7. delete_file executor reverses a creation
# ---------------------------------------------------------------------------


def test_delete_file_action_removes_file_on_disk():
    with _TempLedger():
        with tempfile.TemporaryDirectory() as td:
            victim = Path(td) / "removed-by-rollback.txt"
            victim.write_text("kept", encoding="utf-8")
            actions = [
                {"kind": "delete_file", "path": str(victim),
                 "description": "drop the file the step created"},
            ]
            rb.record("delete_test", {}, actions)
            rb.restore_one("delete_test")
            assert not victim.exists(), \
                "delete_file action did not remove the target"


# ---------------------------------------------------------------------------
# 8. Unknown action kinds are skipped with a marker, not raised
# ---------------------------------------------------------------------------


def test_unknown_action_kind_is_skipped_not_raised():
    with _TempLedger():
        actions = [
            {"kind": "something_we_dont_handle", "description": "future kind"},
        ]
        rb.record("unknown_kind", {}, actions)
        lines = rb.restore_one("unknown_kind")
        joined = "\n".join(lines)
        assert "unknown action kind" in joined, \
            f"expected unknown-kind notice, got: {joined!r}"


# ---------------------------------------------------------------------------
# 9. capture_panel_swap_state produces a valid record (smoke)
# ---------------------------------------------------------------------------


def test_capture_panel_swap_state_returns_serializable_payload():
    with _TempLedger():
        prior, actions = rb.capture_panel_swap_state()
        # prior must be JSON-serializable straight out.
        encoded = json.dumps({"prior": prior, "actions": actions})
        assert "xfce4_panel_installed" in encoded
        # Restore actions must reference the autostart .desktop path
        # apply_panel_swap is about to write — proves the rollback
        # actually targets the right file.
        joined = json.dumps(actions)
        assert "xfce4-panel.desktop" in joined, \
            "panel-swap rollback should reference xfce4-panel.desktop"
