# xfce-settings hide overrides

This directory is reserved for shipping XDG `.desktop` overrides as data files
(rather than constructing them at runtime). Currently `mackes/menu_integration.py`
generates the overrides at install time, so this directory is intentionally
empty.

Kept in tree so the RPM spec can reference the path if a future revision
ships pre-baked overrides.
