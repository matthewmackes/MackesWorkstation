"""Maintain → System Update.

A single-screen wrapper around `dnf upgrade` with a streaming log. Closes the
"how do I keep my Fedora patched" gap without dragging in PackageKit. The
authentication step uses `pkexec`, which surfaces a polkit auth dialog.

This is one of four micro-tools in the MaintenanceKit (Option 9 of the
platform review). The others (mackes-fonts, mackes-power, mackes-resources)
are TBD.
"""
from __future__ import annotations

import shutil
import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.logging import log_action
from mackes.workbench._common import info_label, panel_box, section_description, section_header, title_label


class SystemUpdatePanel(Gtk.Box):
    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._proc: subprocess.Popen | None = None
        self._io_watch_id: int | None = None
        self._build()
        GLib.idle_add(self._refresh_summary)

    # ---- UI ------------------------------------------------------------

    def _build(self) -> None:
        box = panel_box()
        box.pack_start(title_label("System Update"), False, False, 0)
        box.pack_start(info_label(
            "Install the latest fixes and updates for your machine. "
            "This may take a few minutes."
        ), False, False, 0)
        box.pack_start(section_description(
            "You'll be asked for your password before any change is "
            "made. You can cancel at any time — updates will pick up "
            "where they left off."
        ), False, False, 0)

        box.pack_start(section_header("Summary"), False, False, 0)
        self._summary = Gtk.Label(label="(checking…)")
        self._summary.set_xalign(0)
        self._summary.set_line_wrap(True)
        box.pack_start(self._summary, False, False, 0)

        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self._check_btn = Gtk.Button(label="Check for updates")
        self._check_btn.connect("clicked", lambda *_: self._run(["dnf", "check-update"], auth=False))
        actions.pack_start(self._check_btn, False, False, 0)

        self._install_btn = Gtk.Button(label="Install all updates")
        self._install_btn.connect("clicked",
                                  lambda *_: self._run(["dnf", "upgrade", "-y", "--refresh"], auth=True))
        actions.pack_start(self._install_btn, False, False, 0)

        self._cancel_btn = Gtk.Button(label="Cancel")
        self._cancel_btn.set_sensitive(False)
        self._cancel_btn.connect("clicked", lambda *_: self._cancel())
        actions.pack_start(self._cancel_btn, False, False, 0)
        box.pack_start(actions, False, False, 0)

        box.pack_start(section_header("Output"), False, False, 0)
        self._output = Gtk.TextView()
        self._output.set_editable(False)
        self._output.set_monospace(True)
        scroll = Gtk.ScrolledWindow()
        scroll.add(self._output)
        scroll.set_size_request(-1, 320)
        box.pack_start(scroll, True, True, 0)

        self.add(box)

    # ---- Subprocess plumbing -------------------------------------------

    def _run(self, argv: list[str], *, auth: bool) -> None:
        if self._proc is not None and self._proc.poll() is None:
            self._append("[busy — finish or cancel the current run first]\n")
            return
        if auth:
            if shutil.which("pkexec") is None:
                self._append("[pkexec not installed — install polkit to enable privileged updates]\n")
                return
            argv = ["pkexec", *argv]
        self._append(f"$ {' '.join(argv)}\n")
        log_action(f"system_update: launching {argv}")
        try:
            self._proc = subprocess.Popen(
                argv,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                bufsize=1, text=True,
            )
        except OSError as e:
            self._append(f"[launch failed: {e}]\n")
            return
        self._toggle_busy(True)
        self._io_watch_id = GLib.io_add_watch(
            self._proc.stdout.fileno(),
            GLib.IO_IN | GLib.IO_HUP,
            self._on_io,
        )

    def _on_io(self, fd, condition) -> bool:
        if self._proc is None:
            return False
        line = self._proc.stdout.readline() if self._proc.stdout else ""
        if line:
            self._append(line)
            return True
        # EOF / hup
        rc = self._proc.wait()
        self._append(f"[exit {rc}]\n")
        log_action(f"system_update: exit rc={rc}")
        self._toggle_busy(False)
        self._proc = None
        self._io_watch_id = None
        GLib.idle_add(self._refresh_summary)
        return False

    def _cancel(self) -> None:
        if self._proc is None or self._proc.poll() is not None:
            return
        try:
            self._proc.terminate()
            self._append("[cancel requested]\n")
        except OSError as e:
            self._append(f"[cancel failed: {e}]\n")

    def _toggle_busy(self, busy: bool) -> None:
        self._check_btn.set_sensitive(not busy)
        self._install_btn.set_sensitive(not busy)
        self._cancel_btn.set_sensitive(busy)

    # ---- Helpers -------------------------------------------------------

    def _append(self, text: str) -> None:
        buf = self._output.get_buffer()
        buf.insert(buf.get_end_iter(), text)
        mark = buf.create_mark(None, buf.get_end_iter(), False)
        self._output.scroll_to_mark(mark, 0.0, False, 0.0, 1.0)

    def _refresh_summary(self) -> bool:
        # quick, non-privileged availability check (no metadata refresh)
        try:
            out = subprocess.check_output(
                ["dnf", "list", "--upgrades", "-q"],
                stderr=subprocess.STDOUT, text=True, timeout=15,
            )
            count = max(0, sum(1 for ln in out.splitlines()
                               if ln and not ln.startswith(("Last metadata", "Available", "Upgrad"))))
            self._summary.set_text(f"{count} package(s) have upgrades available."
                                   if count else "Up to date.")
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as e:
            self._summary.set_text(f"(couldn't check: {e})")
        return False
