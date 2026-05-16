"""Smoke tests for save_polybar_profile + user-local lookup priority."""
from __future__ import annotations

from pathlib import Path

from mackes import shell_profiles


def test_user_profile_dir_is_searched_first():
    # USER_PROFILE_DIR is first in SHIPPED_PROFILE_DIRS so user-saved
    # profiles shadow shipped ones of the same name.
    assert shell_profiles.SHIPPED_PROFILE_DIRS[0] == shell_profiles.USER_PROFILE_DIR


def test_save_polybar_profile_signature_callable():
    # Don't actually write to ~/.config in this smoke test; just confirm the
    # function exists and accepts the documented signature.
    fn = shell_profiles.save_polybar_profile
    import inspect
    sig = inspect.signature(fn)
    assert list(sig.parameters) == ["name", "text"]


def test_apply_polybar_text_function_present():
    assert callable(shell_profiles.apply_polybar_text)
