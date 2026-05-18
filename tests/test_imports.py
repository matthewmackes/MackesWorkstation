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
            # GUI deps (gi.repository / GTK typelibs) are acceptable to miss
            # on minimal CI containers. Real ImportError-related strings to
            # skip: the explicit "gi"/"Gtk"/"PyGObject" matches, plus the
            # less-obvious "Typelib file for namespace 'xlib' / 'cairo' /
            # 'Gtk' …" that PyGObject raises when a typelib is absent.
            msg = str(e)
            if any(kw in msg for kw in
                   ("gi", "Gtk", "PyGObject", "Typelib", "namespace")):
                continue
            failures.append(f"{name}: {e}")
        except Exception as e:  # noqa: BLE001
            # Same GUI-dep tolerance for non-ImportError exceptions PyGObject
            # may raise when a typelib is missing (e.g. ValueError).
            msg = str(e)
            if any(kw in msg for kw in
                   ("Typelib", "namespace", "gi.repository")):
                continue
            failures.append(f"{name}: {type(e).__name__}: {e}")
    assert not failures, "Import failures:\n" + "\n".join(failures)
