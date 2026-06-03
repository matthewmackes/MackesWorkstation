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


# ─── v4.0.1: schema validation gate ────────────────────────────


def test_validate_against_current_clean_returns_no_source_preset_warning(
    isolated_xdg,
):
    """Snapshot created with source_preset set must not surface
    the 'source_preset not recorded' advisory."""
    from mackes.snapshots import (
        create_snapshot, validate_snapshot_against_current,
    )
    snap = create_snapshot(label="clean", source_preset="vanilla")
    warnings = validate_snapshot_against_current(snap)
    src_preset_warns = [w for w in warnings if "source_preset" in w]
    assert src_preset_warns == [], f"unexpected: {src_preset_warns}"


def test_validate_flags_missing_source_preset(isolated_xdg):
    """v1.x snapshots without source_preset get an advisory
    warning so the operator knows the preset-shape check was
    skipped."""
    import json
    from mackes.snapshots import (
        create_snapshot, validate_snapshot_against_current,
    )
    snap = create_snapshot(label="pre-v1.4")
    mf_path = snap.path / "manifest.json"
    manifest = json.loads(mf_path.read_text())
    manifest["source_preset"] = None
    mf_path.write_text(json.dumps(manifest))
    warnings = validate_snapshot_against_current(snap)
    assert any("source_preset not recorded" in w for w in warnings)


def test_validate_flags_keys_only_in_snapshot(isolated_xdg):
    """A snapshot that records a key the current bridge no longer
    knows about surfaces a schema-drift warning so the operator
    knows that key won't round-trip."""
    import json
    from mackes.snapshots import (
        create_snapshot, validate_snapshot_against_current,
    )
    snap = create_snapshot(label="extra-keys", source_preset="vanilla")
    mf_path = snap.path / "manifest.json"
    manifest = json.loads(mf_path.read_text())
    manifest["mde_keys"] = list(manifest.get("mde_keys", [])) + [
        "synthetic.retired.legacy.key"
    ]
    mf_path.write_text(json.dumps(manifest))
    warnings = validate_snapshot_against_current(snap)
    assert any("not in current bridge" in w for w in warnings)


def test_restore_strict_raises_on_validation_warning(isolated_xdg):
    """With strict=True, restore_snapshot refuses to write if
    validation surfaces any warning. Default (non-strict) keeps
    v1.x behavior (log + proceed)."""
    import json
    import pytest
    from mackes.snapshots import create_snapshot, restore_snapshot
    snap = create_snapshot(label="strict-test")
    mf_path = snap.path / "manifest.json"
    manifest = json.loads(mf_path.read_text())
    manifest["source_preset"] = None
    mf_path.write_text(json.dumps(manifest))
    with pytest.raises(ValueError, match="strict schema validation"):
        restore_snapshot(snap, strict=True)
