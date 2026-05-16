"""Snapshot create/list/delete round-trip (xfconf-query optional)."""
from __future__ import annotations


def test_create_and_list(isolated_xdg):
    from mackes.snapshots import create_snapshot, list_snapshots, delete_snapshot

    snap = create_snapshot(label="round-trip-test")
    assert snap.path.exists()
    assert (snap.path / "manifest.json").exists()
    mf = snap.manifest()
    assert mf["label"] == "round-trip-test"

    found = list_snapshots()
    assert any(s.path == snap.path for s in found)

    delete_snapshot(snap)
    assert not snap.path.exists()


def test_slug_label_normalizes(isolated_xdg):
    from mackes.snapshots import create_snapshot
    snap = create_snapshot(label="my Cool / Snapshot!")
    # Path stem after the timestamp prefix must be a valid filesystem name
    name = snap.path.name
    # Format: YYYY-MM-DDTHHMMSS_<slug>
    assert "_" in name
    slug = name.split("_", 1)[1]
    assert all(c.isalnum() or c in "._-" for c in slug)
    snap.path.rmdir() if not any(snap.path.iterdir()) else None
