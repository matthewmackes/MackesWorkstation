"""Pure-helper tests for the Phase 13.5.1 welcome banner.

Covers the upgrade-detection state machine without GTK so the
no-pytest shim picks them up.
"""
from __future__ import annotations

from pathlib import Path


def test_should_show_when_marker_missing(tmp_path=None):
    if tmp_path is None:
        # Running under _run_without_pytest.py shim — synthesize a
        # tmpdir manually.
        import tempfile
        tmp = tempfile.TemporaryDirectory()
        try:
            from mackes.workbench.welcome_banner import should_show_for_version
            marker = Path(tmp.name) / "no.txt"
            assert should_show_for_version("1.2.0", state_path=marker)
        finally:
            tmp.cleanup()
        return
    from mackes.workbench.welcome_banner import should_show_for_version
    marker = tmp_path / "no.txt"
    assert should_show_for_version("1.2.0", state_path=marker)


def test_should_hide_when_marker_matches_version():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import (
            should_show_for_version, mark_shown,
        )
        marker = Path(tmp.name) / "marker.txt"
        mark_shown("1.2.0", state_path=marker)
        assert not should_show_for_version("1.2.0", state_path=marker)
    finally:
        tmp.cleanup()


def test_should_show_after_version_bump():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import (
            should_show_for_version, mark_shown,
        )
        marker = Path(tmp.name) / "marker.txt"
        mark_shown("1.1.0", state_path=marker)
        assert should_show_for_version("1.2.0", state_path=marker)
    finally:
        tmp.cleanup()


def test_mark_shown_creates_parent_dir():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import (
            mark_shown, shown_for_version,
        )
        # Nested subdir that doesn't exist yet.
        marker = Path(tmp.name) / "a" / "b" / "c" / "marker.txt"
        mark_shown("1.3.0", state_path=marker)
        assert marker.exists()
        assert shown_for_version(state_path=marker) == "1.3.0"
    finally:
        tmp.cleanup()


def test_shown_for_version_returns_none_when_unreadable():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import shown_for_version
        marker = Path(tmp.name) / "missing.txt"
        assert shown_for_version(state_path=marker) is None
    finally:
        tmp.cleanup()


def test_shown_for_version_treats_empty_file_as_none():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import shown_for_version
        marker = Path(tmp.name) / "empty.txt"
        marker.write_text("")
        assert shown_for_version(state_path=marker) is None
    finally:
        tmp.cleanup()


def test_mark_shown_then_overwrite():
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        from mackes.workbench.welcome_banner import (
            mark_shown, shown_for_version,
        )
        marker = Path(tmp.name) / "m.txt"
        mark_shown("1.0.0", state_path=marker)
        mark_shown("2.0.0", state_path=marker)
        assert shown_for_version(state_path=marker) == "2.0.0"
    finally:
        tmp.cleanup()
