"""Pure-helper tests for the Phase 12.8 mesh GUI panels.

Tests live in the no-GTK helpers (slug lookup, diff builder, passcode
validator) so they run under the `_run_without_pytest.py` shim and
real pytest alike.
"""
from __future__ import annotations


def test_passcode_validator_accepts_16_char_url_safe():
    from mackes.wizard.pages.mesh_passcode import passcode_is_valid
    assert passcode_is_valid("aB3-_xyz12345678")
    assert passcode_is_valid("0123456789abcdef")
    assert passcode_is_valid("ABCDEFGHIJKLMNOP")
    assert passcode_is_valid("------__________")


def test_passcode_validator_rejects_wrong_length():
    from mackes.wizard.pages.mesh_passcode import passcode_is_valid
    assert not passcode_is_valid("")
    assert not passcode_is_valid("short")
    assert not passcode_is_valid("a" * 15)
    assert not passcode_is_valid("a" * 17)
    assert not passcode_is_valid("a" * 32)


def test_passcode_validator_rejects_non_url_safe():
    from mackes.wizard.pages.mesh_passcode import passcode_is_valid
    # Space, plus, slash, equals are NOT in the URL-safe alphabet.
    assert not passcode_is_valid("aaaaaaaaaaaaaa b")
    assert not passcode_is_valid("a/aaaaaaaaaaaa+a")
    assert not passcode_is_valid("aaaaaaaaaaaaaa=a")
    # Multi-byte chars are out.
    assert not passcode_is_valid("aaaaaaaaaaaaaaña")


def test_mesh_control_tab_lookup_round_trip():
    from mackes.workbench.network.mesh_control import (
        TABS, slug_for_tab, tab_index_for_slug,
    )
    for i, (slug, _label, _mod, _cls) in enumerate(TABS):
        assert tab_index_for_slug(slug) == i, f"{slug} -> {i}"
        assert slug_for_tab(i) == slug, f"{i} -> {slug}"


def test_mesh_control_tab_lookup_falls_back_safely():
    from mackes.workbench.network.mesh_control import (
        slug_for_tab, tab_index_for_slug,
    )
    # Unknown slugs map to index 0 (the Health tab); out-of-range
    # indexes map to the first slug.
    assert tab_index_for_slug("does-not-exist") == 0
    assert tab_index_for_slug("") == 0
    assert slug_for_tab(-1) == "health"
    assert slug_for_tab(10_000) == "health"


def test_mesh_history_diff_emits_unified_diff_when_payloads_differ():
    from mackes.workbench.network.mesh_history import build_diff_lines
    diff = build_diff_lines(
        {"a": 1, "b": 2},
        {"a": 1, "b": 3},
        "rev-1", "rev-2",
    )
    assert diff, "expected non-empty diff for differing payloads"
    joined = "\n".join(diff)
    assert "rev-1" in joined
    assert "rev-2" in joined
    assert any("-" in line or "+" in line for line in diff)


def test_mesh_history_diff_handles_identical_payloads():
    from mackes.workbench.network.mesh_history import build_diff_lines
    same = {"k": "v"}
    diff = build_diff_lines(same, same, "rev-a", "rev-b")
    assert diff == []


def test_mesh_history_diff_handles_non_json_payloads():
    """Non-JSON-serializable payloads fall back to ``str()`` without
    raising — keeps the diff viewer robust against legacy revisions."""
    from mackes.workbench.network.mesh_history import build_diff_lines

    class Unserializable:
        def __repr__(self):
            return "Unser()"

    diff = build_diff_lines(Unserializable(), Unserializable(),
                            "rev-1", "rev-2")
    # Identical repr -> empty diff (but no exception).
    assert diff == []
