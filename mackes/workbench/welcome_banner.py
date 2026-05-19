"""KDE Connect upgrade welcome banner (Phase 13.5.1).

Surfaces a single-shot banner in the Workbench shell the first time
the user opens it after an upgrade. The banner introduces
`Workbench → Network → KDE Connect`, links to
`docs/help/kde-connect.md`, and dismisses persistently.

State file: ``~/.config/mackes-shell/welcome_shown_for.txt`` carries
the version string (`mackes.__version__`) the banner was last
acknowledged for. When that string equals the current version, the
banner doesn't render. When it differs (or the file is missing), the
banner renders until the user clicks Dismiss.

Module shape mirrors the rest of the workbench helpers: pure-logic
helpers (`should_show_for_version`, `mark_shown`) plus a thin GTK
`build_banner_widget()` constructor. The pure helpers run under the
no-pytest shim so the upgrade-detection state machine is covered
without a display.
"""
from __future__ import annotations

import os
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402

from mackes.workbench._common import a11y


def _state_file() -> Path:
    """Path of the persistent ``welcome_shown_for.txt`` marker.

    Honors ``$XDG_CONFIG_HOME`` (tests pass this in via monkeypatch);
    falls back to ``~/.config/mackes-shell/``.
    """
    config_root = os.environ.get("XDG_CONFIG_HOME") or str(
        Path.home() / ".config"
    )
    return Path(config_root) / "mackes-shell" / "welcome_shown_for.txt"


def shown_for_version(state_path: Path | None = None) -> str | None:
    """Return the version string the banner was last acknowledged for,
    or ``None`` if the marker doesn't exist or can't be read."""
    path = state_path or _state_file()
    try:
        return path.read_text(encoding="utf-8").strip() or None
    except (OSError, UnicodeDecodeError):
        return None


def should_show_for_version(current_version: str,
                            state_path: Path | None = None) -> bool:
    """Pure helper: should the banner render?

    True when the marker is missing OR carries a different version
    than ``current_version``. False otherwise. No I/O side effects
    beyond reading the marker.
    """
    recorded = shown_for_version(state_path=state_path)
    return recorded != current_version


def mark_shown(current_version: str,
               state_path: Path | None = None) -> None:
    """Persist the marker so the banner doesn't render again until the
    next version bump. Creates the parent directory as needed."""
    path = state_path or _state_file()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(current_version, encoding="utf-8")


def build_banner_widget(
    current_version: str,
    on_dismiss=None,
    state_path: Path | None = None,
) -> Gtk.Widget | None:
    """Construct the Carbon-styled welcome banner — or return ``None``
    if it shouldn't render this run.

    Caller embeds the returned widget at the top of the Workbench
    content area. Click handlers:

      * **Open KDE Connect** — invokes ``on_dismiss(focus_slug)`` with
        ``mackes://network.kde-connect`` so the parent can navigate.
      * **Dismiss** — persists the marker and removes itself.
    """
    if not should_show_for_version(current_version, state_path=state_path):
        return None

    banner = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
    banner.get_style_context().add_class("mackes-welcome-banner")
    banner.set_margin_top(8); banner.set_margin_start(12)
    banner.set_margin_end(12); banner.set_margin_bottom(4)

    # Left: icon + body text.
    body = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
    body.set_hexpand(True)
    title = Gtk.Label(label=f"What's new in Mackes Shell {current_version}")
    title.set_xalign(0)
    title.get_style_context().add_class("mackes-banner-title")
    body.pack_start(title, False, False, 0)
    text = Gtk.Label(label=(
        "Paired phones and tablets now appear in Workbench → Network →"
        " KDE Connect. The mesh-mDNS bridge keeps them reachable when"
        " they leave your LAN. See docs/help/kde-connect.md for the"
        " setup walkthrough."
    ))
    text.set_xalign(0); text.set_line_wrap(True)
    body.pack_start(text, False, False, 0)
    banner.pack_start(body, True, True, 0)

    # Right: actions.
    open_btn = Gtk.Button(label="Open KDE Connect")
    a11y(open_btn, "Open the Workbench KDE Connect panel", tooltip=None)
    banner.pack_start(open_btn, False, False, 0)
    dismiss_btn = Gtk.Button(label="Dismiss")
    a11y(dismiss_btn, "Hide this banner until the next upgrade",
         tooltip=None)
    banner.pack_start(dismiss_btn, False, False, 0)

    def _on_open(_btn):
        mark_shown(current_version, state_path=state_path)
        if on_dismiss is not None:
            try:
                on_dismiss("network.kde-connect")
            except TypeError:
                on_dismiss()
        banner.hide()

    def _on_dismiss(_btn):
        mark_shown(current_version, state_path=state_path)
        banner.hide()

    open_btn.connect("clicked", _on_open)
    dismiss_btn.connect("clicked", _on_dismiss)

    return banner
