"""Phase 13.4 — drawer KDE Connect notification merge tests."""
from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path


def _with_isolated_cache(fn):
    """Run `fn(cache_dir)` with XDG_CACHE_HOME pointed at a tempdir.

    Doubles as a fixture-substitute for the no-pytest shim.
    """
    tmp = tempfile.TemporaryDirectory()
    try:
        old = os.environ.get("XDG_CACHE_HOME")
        os.environ["XDG_CACHE_HOME"] = tmp.name
        try:
            return fn(Path(tmp.name))
        finally:
            if old is None:
                del os.environ["XDG_CACHE_HOME"]
            else:
                os.environ["XDG_CACHE_HOME"] = old
    finally:
        tmp.cleanup()


def test_drawer_load_notifications_handles_missing_caches():
    def body(cache):
        from mackes.drawer import _load_pending_notifications
        assert _load_pending_notifications() == []
    _with_isolated_cache(body)


def test_drawer_load_notifications_reads_legacy_file():
    def body(cache):
        notes_dir = cache / "mackes"
        notes_dir.mkdir()
        (notes_dir / "notifications.json").write_text(json.dumps([
            {"app": "X", "title": "T", "body": "B"},
        ]))
        from mackes.drawer import _load_pending_notifications
        out = _load_pending_notifications()
        assert len(out) == 1
        assert out[0]["app"] == "X"
    _with_isolated_cache(body)


def test_drawer_load_notifications_merges_phone_origin():
    """KDE Connect mirrored notifications get `origin: "phone"`
    auto-applied so the drawer renders the phone badge."""
    def body(cache):
        notes_dir = cache / "mackes"
        notes_dir.mkdir()
        (notes_dir / "kdeconnect-notifications.json").write_text(json.dumps([
            {"device": "Pixel-9", "title": "Hey", "text": "How are you?"},
        ]))
        from mackes.drawer import _load_pending_notifications
        out = _load_pending_notifications()
        assert len(out) == 1
        assert out[0]["origin"] == "phone"
        assert out[0]["app"] == "Pixel-9"  # device falls through as app
        assert out[0]["title"] == "Hey"
        assert out[0]["body"] == "How are you?"
    _with_isolated_cache(body)


def test_drawer_load_notifications_concatenates_when_both_exist():
    def body(cache):
        notes_dir = cache / "mackes"
        notes_dir.mkdir()
        (notes_dir / "notifications.json").write_text(json.dumps([
            {"app": "mackes", "title": "Mesh joined"},
        ]))
        (notes_dir / "kdeconnect-notifications.json").write_text(json.dumps([
            {"device": "Phone", "title": "MMS arrived"},
        ]))
        from mackes.drawer import _load_pending_notifications
        out = _load_pending_notifications()
        assert len(out) == 2
        # Legacy notification first, phone notification appended.
        assert out[0].get("origin") != "phone"
        assert out[1]["origin"] == "phone"
    _with_isolated_cache(body)


def test_drawer_load_notifications_skips_non_dict_phone_entries():
    """Garbled phone payloads don't crash the merge — non-dict
    entries are skipped, valid ones still surface."""
    def body(cache):
        notes_dir = cache / "mackes"
        notes_dir.mkdir()
        (notes_dir / "kdeconnect-notifications.json").write_text(json.dumps([
            "not-a-dict",
            {"device": "Phone", "title": "valid"},
            42,
        ]))
        from mackes.drawer import _load_pending_notifications
        out = _load_pending_notifications()
        assert len(out) == 1
        assert out[0]["title"] == "valid"
    _with_isolated_cache(body)


def test_drawer_load_notifications_handles_corrupt_phone_json():
    def body(cache):
        notes_dir = cache / "mackes"
        notes_dir.mkdir()
        (notes_dir / "kdeconnect-notifications.json").write_text("{not json")
        from mackes.drawer import _load_pending_notifications
        # Corrupt phone file -> empty merge, no exception.
        assert _load_pending_notifications() == []
    _with_isolated_cache(body)
