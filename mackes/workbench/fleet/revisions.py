"""Fleet → Revisions panel (v2.0.0 Phase F.12).

Lists every desired_config revision via `mded revisions list --json`
(shipped alongside this panel; see the matching mded subcommand in
crates/mackesd/src/bin/mackesd.rs). Each row offers a Rollback
button that invokes `mded revisions rollback <id>` per-peer or
fleet-wide.
"""
from __future__ import annotations

import json
import shutil
import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.workbench._common import (
    a11y, empty_state, error_state, info_label,
    panel_box, section_header, title_label,
)


def _mded_on_path() -> bool:
    return shutil.which("mded") is not None


def list_revisions() -> tuple[list[dict], str | None]:
    """Pure-helper: invoke `mded revisions list --json`. Returns
    (rows, error_msg). On any failure: ([], message). Lifted for
    unit-test coverage without spawning subprocesses."""
    if not _mded_on_path():
        return ([], "mded binary not on $PATH")
    try:
        r = subprocess.run(
            ["mded", "revisions", "list", "--json"],
            capture_output=True, text=True, timeout=5,
        )
    except (OSError, subprocess.SubprocessError) as e:
        return ([], f"mded invocation failed: {e}")
    if r.returncode != 0:
        return ([], r.stderr.strip() or f"exit code {r.returncode}")
    try:
        data = json.loads(r.stdout)
    except json.JSONDecodeError as e:
        return ([], f"mded returned non-JSON output: {e}")
    if not isinstance(data, list):
        return ([], "mded returned non-list output")
    return (data, None)


def rollback_to(revision_id: str, peers: str = "all") -> tuple[bool, str]:
    """Pure-helper: invoke `mded revisions rollback`. Returns
    (ok, message)."""
    if not _mded_on_path():
        return (False, "mded not installed")
    try:
        r = subprocess.run(
            ["mded", "revisions", "rollback", revision_id, "--peers", peers],
            capture_output=True, text=True, timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as e:
        return (False, str(e))
    return (
        (r.returncode == 0, r.stdout.strip() or "ok")
        if r.returncode == 0
        else (False, r.stderr.strip() or f"exit {r.returncode}")
    )


def format_revision_row(rev: dict) -> str:
    """Pure-helper: render a one-line summary for the list row.
    Unit-test-friendly."""
    rid = rev.get("revision_id", "?")
    author = rev.get("author", "?")
    state = rev.get("state", "?")
    created = rev.get("created_at", "?")
    summary = rev.get("summary", "")
    return f"{rid}  [{state}]  by {author}  ·  {created}  ·  {summary}"


class FleetRevisionsPanel(Gtk.Box):
    """List desired-config revisions + roll back per-peer or
    fleet-wide."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Configuration revisions"), False, False, 0)
        box.pack_start(info_label(
            "Every desired-config revision in descending order. Click "
            "Rollback to make a prior revision the new applied row."
        ), False, False, 0)

        if not _mded_on_path():
            box.pack_start(error_state(
                "mded not installed",
                "Install the mackesd RPM to list / rollback revisions.",
                retry_label=None,
            ), False, False, 0)
            return box

        revisions, err = list_revisions()
        if err is not None and not revisions:
            box.pack_start(error_state(
                "Couldn't load revisions", err, retry_label=None,
            ), False, False, 0)
            return box

        if not revisions:
            box.pack_start(empty_state(
                "No revisions yet",
                "Push a setting through the Fleet → Push panel "
                "(or via `mded fleet push-setting`) to create one.",
                retry_label=None,
            ), False, False, 0)
            return box

        box.pack_start(section_header("Revisions"), False, False, 0)
        list_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)

        for rev in revisions:
            row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            row.set_margin_top(6); row.set_margin_bottom(6)
            label = Gtk.Label(label=format_revision_row(rev))
            label.set_xalign(0); label.set_hexpand(True)
            row.pack_start(label, True, True, 0)

            rid = rev.get("revision_id", "")
            rollback_btn = Gtk.Button(label="Rollback")
            a11y(rollback_btn, name=f"Roll back to {rid}",
                 tooltip=f"Re-apply {rid} as a fresh revision (peers=all)")

            def on_rollback(_btn, revision_id=rid):
                rollback_to(revision_id, "all")
                # Caller refreshes manually for now; the live
                # refresh wires through when 12.9.1 ships the
                # interactive surface.

            rollback_btn.connect("clicked", on_rollback)
            row.pack_start(rollback_btn, False, False, 0)
            list_box.pack_start(row, False, False, 0)

        box.pack_start(list_box, True, True, 0)
        return box
