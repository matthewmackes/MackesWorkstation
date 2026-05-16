"""Entry point. `python -m mackes` or the installed `mackes` binary lands here.

Recognized flags before delegating to the GTK Application:
  --uninstall  Run the headless uninstall sequence (Q9 lock). The GUI panel
               at Maintain → Uninstall does the same thing with a streaming
               view; this flag exists as the escape hatch when the GUI itself
               is broken.
  --yes        With --uninstall, skip the interactive confirmation (Q28 lock).
"""
from __future__ import annotations

import sys

from mackes.app import MackesApp


def _run_cli_uninstall(yes: bool) -> int:
    from mackes.uninstall import run_uninstall
    if not yes:
        try:
            print("This will remove Mackes Shell and all its files, reset xfconf to")
            print("XFCE defaults, reinstall replaced XFCE components, and clean up")
            print("xfce11-unified v2.2 leftovers. A final snapshot will be tarballed")
            print("to ~/Desktop/. Type 'UNINSTALL' to proceed:")
            line = input("> ").strip()
        except EOFError:
            line = ""
        if line != "UNINSTALL":
            print("Aborted.")
            return 130

    def _emit(line: str) -> None:
        print(line, flush=True)

    report = run_uninstall(progress=_emit)
    print()
    print(f"Failed steps: {report.failed_count} of {len(report.steps)}")
    if report.log_path is not None:
        print(f"Full log: {report.log_path}")
    if report.desktop_tarball is not None:
        print(f"Final snapshot: {report.desktop_tarball}")
    return 0 if report.failed_count == 0 else 1


def main(argv: list[str] | None = None) -> int:
    argv = list(argv if argv is not None else sys.argv)

    # Q9 lock — `mackes --uninstall` runs headless without launching the GUI.
    flags = {a for a in argv[1:] if a.startswith("--")}
    if "--uninstall" in flags:
        return _run_cli_uninstall(yes="--yes" in flags)

    app = MackesApp()
    return app.run(argv)


if __name__ == "__main__":
    raise SystemExit(main())
