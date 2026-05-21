"""Fleet → Push settings panel (v2.0.0 Phase F.11).

UI flow: pick a setting key (every entry from `mde_settings_bridge
._KEY_MAP`), enter a JSON-encoded value, pick a peer selector
(`all` or a comma-separated list of node ids), click Apply. Shells
out to `mded fleet push-setting <key> <value> --peers <sel>`
(Phase G.4) which writes one desired_config row + one
fleet_settings_apply_log row per targeted peer.

Pre-flight diff: shows what the local current value is so the
operator sees "this overwrites theme.accent='#2b9af3' on every
peer" before they click Apply.
"""
from __future__ import annotations

import json
import shutil
import subprocess

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes import mde_settings_bridge as _b
from mackes.workbench._common import (
    a11y, error_state, info_label, labeled_row,
    panel_box, section_header, title_label,
)


def _mded_on_path() -> bool:
    return shutil.which("mded") is not None


def push_setting(key: str, value_json: str, peers: str) -> tuple[bool, str]:
    """Pure-helper: invoke `mded fleet push-setting`. Returns
    (ok, message). Lifted for unit-test coverage."""
    if not _mded_on_path():
        return (False, "mded binary not on $PATH — install the mackesd RPM")
    try:
        r = subprocess.run(
            ["mded", "fleet", "push-setting", key, value_json,
             "--peers", peers],
            capture_output=True, text=True, timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as e:
        return (False, f"mded invocation failed: {e}")
    if r.returncode == 0:
        return (True, r.stdout.strip() or "ok")
    return (False, r.stderr.strip() or f"exit code {r.returncode}")


class FleetSettingsPanel(Gtk.Box):
    """Push a setting revision to the fleet."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(self._build())

    def _build(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(title_label("Push settings to fleet"), False, False, 0)
        box.pack_start(info_label(
            "Send one setting value to every peer (or a subset). The "
            "reconcile loop on each peer picks up the revision on its "
            "next tick (~30 s)."
        ), False, False, 0)

        if not _mded_on_path():
            box.pack_start(error_state(
                "mded not installed",
                "Install the mackesd RPM to push fleet revisions.",
                retry_label=None,
            ), False, False, 0)
            return box

        # Key picker.
        box.pack_start(section_header("Setting key"), False, False, 0)
        key_combo = Gtk.ComboBoxText()
        keys = sorted(_b._KEY_MAP.keys())
        for k in keys:
            key_combo.append_text(k)
        if keys:
            key_combo.set_active(0)
        a11y(key_combo, name="Setting key to push",
             tooltip="Dot-notated MDE settings key")
        box.pack_start(labeled_row("Key", key_combo), False, False, 0)

        # Current-value preview.
        cur_label = Gtk.Label(label="(pick a key to see its current value)")
        cur_label.set_xalign(0); cur_label.set_line_wrap(True)
        cur_label.get_style_context().add_class("mackes-row-meta")
        box.pack_start(labeled_row("Current value", cur_label),
                       False, False, 0)

        def on_key(c):
            i = c.get_active()
            if i < 0:
                return
            current = _b.get_setting(keys[i])
            cur_label.set_text(
                f"{json.dumps(current)}" if current is not None
                else "(unset)"
            )
        key_combo.connect("changed", on_key)
        on_key(key_combo)

        # Value entry.
        box.pack_start(section_header("New value (JSON-encoded)"),
                       False, False, 0)
        value_entry = Gtk.Entry()
        value_entry.set_placeholder_text('"#ff00aa"  or  42  or  true')
        a11y(value_entry, name="New value as JSON",
             tooltip="JSON-encoded value — strings need quotes")
        box.pack_start(labeled_row("Value", value_entry), False, False, 0)

        # Peer selector.
        box.pack_start(section_header("Target peers"), False, False, 0)
        peers_entry = Gtk.Entry()
        peers_entry.set_text("all")
        peers_entry.set_placeholder_text("all  or  peer:anvil,peer:birch")
        a11y(peers_entry, name="Peer selector",
             tooltip="'all' for every healthy peer, or a comma-list")
        box.pack_start(labeled_row("Peers", peers_entry), False, False, 0)

        # Apply + status.
        actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        apply_btn = Gtk.Button(label="Apply")
        status = Gtk.Label(label=""); status.set_xalign(0)

        def on_apply(_btn):
            i = key_combo.get_active()
            if i < 0:
                status.set_text("Pick a key first.")
                return
            key = keys[i]
            value = value_entry.get_text().strip()
            peers = peers_entry.get_text().strip() or "all"
            if not value:
                status.set_text("Enter a JSON-encoded value (e.g. \"x\" or 42).")
                return
            ok, msg = push_setting(key, value, peers)
            status.set_text(("✓ " if ok else "✗ ") + msg)

        apply_btn.connect("clicked", on_apply)
        a11y(apply_btn, name="Push the revision",
             tooltip="Shells out to `mded fleet push-setting`")
        actions.pack_start(apply_btn, False, False, 0)
        actions.pack_start(status, True, True, 0)
        box.pack_start(actions, False, False, 12)

        return box
