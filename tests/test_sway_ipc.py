"""Tests for `mackes.sway_ipc` (Phase F.8)."""
from __future__ import annotations


def _with_no_swaymsg(fn):
    """Force shutil.which('swaymsg') to return None for the duration
    of `fn`."""
    import shutil
    saved = shutil.which
    shutil.which = lambda c: None if c == "swaymsg" else saved(c)
    try:
        return fn()
    finally:
        shutil.which = saved


def test_is_sway_running_false_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import is_sway_running
        assert not is_sway_running()
    _with_no_swaymsg(body)


def test_current_workspace_returns_none_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import current_workspace
        assert current_workspace() is None
    _with_no_swaymsg(body)


def test_focus_workspace_returns_false_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import focus_workspace
        assert not focus_workspace(2)
    _with_no_swaymsg(body)


def test_set_layout_rejects_invalid_value():
    """The validity check is pure (no swaymsg needed for the
    rejection path), so it always returns False for bogus values."""
    from mackes.sway_ipc import set_layout
    assert not set_layout("not-a-real-layout")
    assert not set_layout("")


def test_set_layout_returns_false_when_swaymsg_absent_for_valid_value():
    def body():
        from mackes.sway_ipc import set_layout
        # Valid layout name, but no swaymsg to spawn.
        assert not set_layout("splith")
    _with_no_swaymsg(body)


def test_kill_focused_returns_false_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import kill_focused
        assert not kill_focused()
    _with_no_swaymsg(body)


def test_get_tree_returns_none_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import get_tree
        assert get_tree() is None
    _with_no_swaymsg(body)


def test_reload_config_returns_false_when_swaymsg_absent():
    def body():
        from mackes.sway_ipc import reload_config
        assert not reload_config()
    _with_no_swaymsg(body)
