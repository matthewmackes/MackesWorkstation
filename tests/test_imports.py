"""Smoke-test: every Mackes module imports without GTK on the system.

Modules that require gi/GTK are skipped via try/except. We're verifying
syntax, import graph, and absence of accidental cycles.
"""
from __future__ import annotations

import importlib
import pkgutil


def test_every_non_gui_module_imports():
    failures: list[str] = []
    # Walk the mackes package and try to import everything.
    import mackes
    for finder, name, ispkg in pkgutil.walk_packages(mackes.__path__, prefix="mackes."):
        try:
            importlib.import_module(name)
        except ImportError as e:
            # GUI deps (gi.repository, etc.) are acceptable to miss
            msg = str(e)
            if "gi" in msg or "Gtk" in msg or "PyGObject" in msg:
                continue
            failures.append(f"{name}: {e}")
        except Exception as e:  # noqa: BLE001
            failures.append(f"{name}: {type(e).__name__}: {e}")
    assert not failures, "Import failures:\n" + "\n".join(failures)
