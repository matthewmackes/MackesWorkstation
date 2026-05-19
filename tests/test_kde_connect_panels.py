"""Pure-helper tests for the Phase 13.3 KDE Connect panels."""
from __future__ import annotations


def test_format_device_label_uses_glyph_name_kind_status():
    from mackes.workbench.network.kde_connect import format_device_label
    label = format_device_label({
        "id": "abc", "name": "My Phone", "kind": "phone", "reachable": True,
    })
    assert "📱" in label
    assert "My Phone" in label
    assert "phone" in label
    assert "reachable" in label


def test_format_device_label_handles_offline():
    from mackes.workbench.network.kde_connect import format_device_label
    label = format_device_label({
        "id": "abc", "name": "Tablet", "kind": "tablet", "reachable": False,
    })
    assert "offline" in label
    assert "tablet" in label


def test_format_device_label_unknown_kind_uses_question_glyph():
    from mackes.workbench.network.kde_connect import format_device_label
    label = format_device_label({"id": "abc", "name": "X", "kind": "unknown"})
    assert "❓" in label


def test_format_last_seen_just_now():
    from mackes.workbench.network.kde_connect import format_last_seen
    assert format_last_seen(1_000_000, now=1_000_010) == "just now"


def test_format_last_seen_minutes():
    from mackes.workbench.network.kde_connect import format_last_seen
    assert format_last_seen(1_000_000, now=1_000_000 + 5 * 60) == "5m ago"


def test_format_last_seen_hours():
    from mackes.workbench.network.kde_connect import format_last_seen
    assert format_last_seen(1_000_000, now=1_000_000 + 3 * 3600) == "3h ago"


def test_format_last_seen_days():
    from mackes.workbench.network.kde_connect import format_last_seen
    assert format_last_seen(1_000_000, now=1_000_000 + 2 * 86400) == "2d ago"


def test_format_last_seen_never_when_zero():
    from mackes.workbench.network.kde_connect import format_last_seen
    assert format_last_seen(0) == "never"
    assert format_last_seen(-5) == "never"


def test_paired_device_records_empty_when_no_config():
    """When ~/.config/kdeconnect/ doesn't exist (or HOME is wrong),
    the scan returns []. No exceptions."""
    import os
    import tempfile
    tmp = tempfile.TemporaryDirectory()
    try:
        original_home = os.environ.get("HOME")
        os.environ["HOME"] = tmp.name  # config dir won't exist under here
        try:
            from mackes.workbench.network.kde_connect import (
                paired_device_records,
            )
            result = paired_device_records()
            assert result == []
        finally:
            if original_home is None:
                del os.environ["HOME"]
            else:
                os.environ["HOME"] = original_home
    finally:
        tmp.cleanup()


def test_paired_device_records_reads_uuid_subdirs():
    """Confirm the scanner picks up UUID-shaped subdirectories and
    skips junk dirs."""
    import json
    import os
    import tempfile
    from pathlib import Path
    tmp = tempfile.TemporaryDirectory()
    try:
        original_home = os.environ.get("HOME")
        os.environ["HOME"] = tmp.name
        cfg = Path(tmp.name) / ".config" / "kdeconnect"
        cfg.mkdir(parents=True)
        # Valid UUID dir.
        uuid = "abcdef0123456789abcdef0123456789"
        (cfg / uuid).mkdir()
        (cfg / uuid / "identity.json").write_text(
            json.dumps({"name": "Pixel 9", "deviceType": "phone"}),
        )
        # Junk dirs that should be skipped.
        (cfg / "trustdb").mkdir()
        (cfg / "settings.conf").write_text("x")
        try:
            from mackes.workbench.network.kde_connect import (
                paired_device_records,
            )
            result = paired_device_records()
            assert len(result) == 1
            assert result[0]["id"] == uuid
            assert result[0]["name"] == "Pixel 9"
            assert result[0]["kind"] == "phone"
        finally:
            if original_home is None:
                del os.environ["HOME"]
            else:
                os.environ["HOME"] = original_home
    finally:
        tmp.cleanup()
