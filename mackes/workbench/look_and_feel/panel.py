"""Look & Feel → Panel.

Surfaces `~/.config/mackes-panel/panel.toml` to the user and shows the
mesh-sync drift status. The actual config write/read happens in the
Rust mackes-panel binary; this panel is read-only for now (live edit
via Look & Feel ships in a later cut — Phase 5.7+ adds drag-to-pin
inside the dock itself).

Drift detection mirrors `mackes_panel::mesh_sync::compute_drift`
(crates/mackes-panel/src/mesh_sync.rs). We read the same paths,
compute the same SHA-256 hashes, and surface the result.
"""
from __future__ import annotations

import hashlib
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402


MIRROR_ROOT = Path.home() / ".qnm-sync" / "mackes-panel"
LOCAL_FILE = Path.home() / ".config" / "mackes-panel" / "panel.toml"


def _hash_file(path: Path) -> bytes | None:
    try:
        return hashlib.sha256(path.read_bytes()).digest()
    except (OSError, IOError):
        return None


def _compute_drift() -> dict[str, str]:
    """Return {peer_name: status} where status ∈ {in-sync, drifted,
    missing}. Empty dict when the mirror tree isn't set up yet."""
    local_hash = _hash_file(MIRROR_ROOT / "panel.toml")
    if local_hash is None:
        return {}
    peers_root = MIRROR_ROOT / "peers"
    if not peers_root.is_dir():
        return {}
    out: dict[str, str] = {}
    for entry in sorted(peers_root.iterdir()):
        if not entry.is_dir():
            continue
        peer_hash = _hash_file(entry / "panel.toml")
        if peer_hash is None:
            out[entry.name] = "missing"
        elif peer_hash == local_hash:
            out[entry.name] = "in-sync"
        else:
            out[entry.name] = "drifted"
    return out


def _section_title(text: str) -> Gtk.Widget:
    lab = Gtk.Label(label=text)
    lab.set_xalign(0)
    lab.get_style_context().add_class("mackes-section-title")
    return lab


def _muted(text: str) -> Gtk.Widget:
    lab = Gtk.Label(label=text)
    lab.set_xalign(0)
    lab.set_line_wrap(True)
    lab.get_style_context().add_class("mackes-section-description")
    return lab


def _sync_row(drift: dict[str, str]) -> Gtk.Widget:
    """Single-line "Sync status" row. Click-through TODO when an
    inspector panel exists (Phase 2.6 follow-up)."""
    drifted = sum(1 for v in drift.values() if v == "drifted")
    missing = sum(1 for v in drift.values() if v == "missing")
    in_sync = sum(1 for v in drift.values() if v == "in-sync")

    if not drift:
        label = "Not replicated (no mesh peers)"
        cls = "muted"
    elif drifted == 0 and missing == 0:
        label = f"In sync with {in_sync} peer{'' if in_sync == 1 else 's'}"
        cls = "ok"
    elif drifted > 0:
        label = (
            f"Drifted from {drifted} peer{'' if drifted == 1 else 's'} · "
            f"{in_sync} in sync · {missing} missing"
        )
        cls = "warn"
    else:
        label = f"{missing} peer{'' if missing == 1 else 's'} missing mirror"
        cls = "muted"

    row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
    glyph = Gtk.Label(label="●")
    glyph.get_style_context().add_class(cls)
    text = Gtk.Label(label=label)
    text.set_xalign(0)
    row.pack_start(glyph, False, False, 0)
    row.pack_start(text, True, True, 0)
    return row


class PanelLookFeelPanel(Gtk.Box):
    """Look & Feel → Panel. Read-only summary of mackes-panel state."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer.set_margin_top(12); outer.set_margin_bottom(12)
        outer.set_margin_start(16); outer.set_margin_end(16)

        # Title — the rest of the look_and_feel package handles
        # breadcrumb + page_title via the sidebar shell; this widget
        # only renders the panel body.
        outer.pack_start(
            _section_title("Mackes panel — sync status"),
            False, False, 0,
        )
        outer.pack_start(
            _muted(
                "Your panel layout lives in "
                "~/.config/mackes-panel/panel.toml and replicates "
                "across the mesh via QNM-Shared. This row shows "
                "whether your peers are seeing the same file."
            ),
            False, False, 0,
        )
        outer.pack_start(_sync_row(_compute_drift()), False, False, 8)

        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroller.add(outer)
        self.pack_start(scroller, True, True, 0)


__all__ = ["PanelLookFeelPanel"]
