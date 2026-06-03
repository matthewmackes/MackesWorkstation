"""Tests for the recovery CLI."""
from __future__ import annotations

import io
import sys

from mackes import recover


def test_main_with_no_snapshots_returns_1_on_latest(monkeypatch=None, capsys=None):
    # When --latest is requested but no snapshots exist, exit code is 1.
    # (No fixtures used; we monkey-patch via setattr so test_runner runs it.)
    saved = recover.list_snapshots
    recover.list_snapshots = lambda: []
    try:
        rc = recover.main(["--latest"])
    finally:
        recover.list_snapshots = saved
    assert rc == 1


def test_main_list_with_no_snapshots_returns_0():
    saved = recover.list_snapshots
    recover.list_snapshots = lambda: []
    try:
        rc = recover.main(["--list"])
    finally:
        recover.list_snapshots = saved
    assert rc == 0


def test_argparse_rejects_unknown_arg():
    saved_err = sys.stderr
    sys.stderr = io.StringIO()
    try:
        try:
            recover.main(["--not-a-flag"])
        except SystemExit as e:
            assert e.code == 2
        else:
            raise AssertionError("expected SystemExit")
    finally:
        sys.stderr = saved_err
