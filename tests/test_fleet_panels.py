"""Pure-helper tests for the Phase F.11 + F.12 fleet panels.

The GTK panel classes themselves require a display; only the pure
shell-out helpers are unit-testable from here.
"""
from __future__ import annotations


def test_format_revision_row_renders_canonical_shape():
    from mackes.workbench.fleet.revisions import format_revision_row
    rev = {
        "revision_id": "r-2026-05-19-0007",
        "author":      "alice",
        "state":       "applied",
        "created_at":  "2026-05-19T18:30:00Z",
        "summary":     "fleet push: theme.accent",
    }
    line = format_revision_row(rev)
    assert "r-2026-05-19-0007" in line
    assert "alice" in line
    assert "applied" in line
    assert "2026-05-19T18:30:00Z" in line
    assert "theme.accent" in line


def test_format_revision_row_handles_missing_fields():
    from mackes.workbench.fleet.revisions import format_revision_row
    line = format_revision_row({})
    # Each missing field falls back to "?"; row is still well-formed.
    assert "?" in line
    assert "  by ?  " in line


def test_list_revisions_returns_empty_when_mded_absent(monkeypatch=None):
    """If `mded` isn't on $PATH, list_revisions returns ([], message).
    Run via monkeypatched shutil.which."""
    import importlib
    import shutil as real_shutil
    saved = real_shutil.which
    real_shutil.which = lambda _cmd: None
    try:
        # Force reimport since the module captured shutil.which at
        # import time? It doesn't — uses shutil.which dynamically.
        # Reimport for safety anyway.
        mod = importlib.import_module("mackes.workbench.fleet.revisions")
        rows, err = mod.list_revisions()
        assert rows == []
        assert err and "mded" in err
    finally:
        real_shutil.which = saved


def test_push_setting_returns_error_when_mded_absent():
    import shutil as real_shutil
    saved = real_shutil.which
    real_shutil.which = lambda _cmd: None
    try:
        from mackes.workbench.fleet.settings import push_setting
        ok, msg = push_setting("theme.name", '"x"', "all")
        assert not ok
        assert "mded" in msg
    finally:
        real_shutil.which = saved


def test_rollback_to_returns_error_when_mded_absent():
    import shutil as real_shutil
    saved = real_shutil.which
    real_shutil.which = lambda _cmd: None
    try:
        from mackes.workbench.fleet.revisions import rollback_to
        ok, msg = rollback_to("r-2026-05-19-0007")
        assert not ok
        assert "mded" in msg
    finally:
        real_shutil.which = saved
