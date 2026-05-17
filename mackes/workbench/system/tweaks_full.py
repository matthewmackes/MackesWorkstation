"""System → Tweaks — full-page panel covering the floating Tweaks drawer
plus the toggles for maximize-all, Thunar autostart, and Mesh clipboard
daemon (previously birthright-only).

Mirrors the canonical mesh_ssh.py Carbon layout: breadcrumb + page_title
+ page_subtitle + section_title sections.

State source: ~/.config/mackes-shell/tweaks.json (same file the floating
drawer uses, via mackes.workbench.shell.sidebar_window._load_tweaks /
_save_tweaks). Changes here propagate to the drawer and vice-versa.

Systemd unit state is queried per-call (cheap user-bus dbus reads).
"""
from __future__ import annotations

import subprocess
from pathlib import Path

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk  # noqa: E402

from mackes.workbench._common import (
    info_label,
    panel_box,
    section_description,
    section_header,
    title_label,
)


# ---- shared helpers (breadcrumb + subtitle) -------------------------------


def _breadcrumb(parts: list[str]) -> Gtk.Widget:
    bc = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=4)
    bc.get_style_context().add_class("mackes-breadcrumb")
    for i, p in enumerate(parts):
        lab = Gtk.Label(label=p); lab.set_xalign(0)
        bc.pack_start(lab, False, False, 0)
        if i != len(parts) - 1:
            sep = Gtk.Label(label="/"); sep.set_xalign(0)
            sep.get_style_context().add_class("mackes-dot")
            bc.pack_start(sep, False, False, 0)
    return bc


def _page_subtitle(text: str) -> Gtk.Widget:
    lab = Gtk.Label(label=text)
    lab.set_xalign(0); lab.set_line_wrap(True)
    lab.get_style_context().add_class("mackes-page-subtitle")
    return lab


# ---- systemd-user helpers -------------------------------------------------


def _systemctl_user(args: list[str], *, timeout: float = 5.0) -> tuple[int, str]:
    """Run `systemctl --user <args>`. Returns (rc, combined-stdout-stderr)."""
    try:
        r = subprocess.run(
            ["systemctl", "--user", *args],
            capture_output=True, text=True, timeout=timeout,
        )
        return r.returncode, (r.stdout or "") + (r.stderr or "")
    except (OSError, subprocess.TimeoutExpired) as e:
        return 1, str(e)


def _unit_active(unit: str) -> bool:
    rc, out = _systemctl_user(["is-active", unit])
    return rc == 0 and out.strip() == "active"


def _set_unit_enabled(unit: str, enabled: bool) -> None:
    """Enable+start or stop+disable a user unit. Idempotent."""
    if enabled:
        _systemctl_user(["enable", "--now", unit])
    else:
        _systemctl_user(["disable", "--now", unit])


# ---- tweaks.json read/write -----------------------------------------------


def _tweaks_path() -> Path:
    from mackes.state import CONFIG_DIR
    return CONFIG_DIR / "tweaks.json"


def _load_tweaks() -> dict:
    import json
    p = _tweaks_path()
    if not p.exists():
        return {}
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def _save_tweak(key: str, value) -> None:
    import json
    t = _load_tweaks()
    t[key] = value
    p = _tweaks_path()
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(t, indent=2, sort_keys=True), encoding="utf-8")


# ---- The panel ------------------------------------------------------------


class TweaksPanel(Gtk.Box):
    """System → Tweaks full-page panel."""

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        outer.set_margin_top(32); outer.set_margin_bottom(32)
        outer.set_margin_start(40); outer.set_margin_end(40)

        outer.pack_start(_breadcrumb(["Mackes Shell", "System", "Tweaks"]),
                         False, False, 0)

        title = Gtk.Label(label="Tweaks")
        title.set_xalign(0); title.get_style_context().add_class("mackes-page-title")
        outer.pack_start(title, False, False, 0)
        outer.pack_start(_page_subtitle(
            "Turn Mackes' birthright features on or off. Each toggle "
            "writes a setting and starts or stops the related service "
            "right away — no restart needed."
        ), False, False, 0)

        outer.pack_start(self._build_window_section(), False, False, 0)
        outer.pack_start(self._build_clipboard_section(), False, False, 0)
        outer.pack_start(self._build_thunar_section(), False, False, 0)
        outer.pack_start(self._build_remmina_section(), False, False, 0)
        outer.pack_start(self._build_hud_section(), False, False, 0)
        outer.pack_start(self._build_shortcut_section(), False, False, 0)

        self.pack_start(outer, True, True, 0)

    # ---- Sections --------------------------------------------------------

    def _build_window_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("Window behavior"), False, False, 0)
        box.pack_start(section_description(
            "Make every new window open at full size automatically. "
            "Helpful on small laptop screens."
        ), False, False, 0)
        row = self._switch_row(
            label="Always-maximize new windows",
            initial=_unit_active("mackes-maximizer.service"),
            on_change=lambda v: _set_unit_enabled("mackes-maximizer.service", v),
        )
        box.pack_start(row, False, False, 0)
        return box

    def _build_clipboard_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("Mesh clipboard"), False, False, 0)
        box.pack_start(section_description(
            "When you copy text on any Mackes machine, paste it on any "
            "other one. Works through the encrypted Mesh — your "
            "clipboard never leaves your fleet."
        ), False, False, 0)
        row = self._switch_row(
            label="Share clipboard across the mesh",
            initial=_unit_active("mackes-clipboard-daemon.service"),
            on_change=lambda v: _set_unit_enabled(
                "mackes-clipboard-daemon.service", v),
        )
        box.pack_start(row, False, False, 0)
        return box

    def _build_thunar_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("File manager (Thunar)"), False, False, 0)
        box.pack_start(section_description(
            "Open Thunar automatically when you sign in. Quick if you "
            "use the file manager often; turn it off for a cleaner "
            "first-paint."
        ), False, False, 0)
        autostart_file = Path.home() / ".config/autostart/thunar-autostart.desktop"
        row = self._switch_row(
            label="Start Thunar on login",
            initial=autostart_file.exists(),
            on_change=lambda v: self._toggle_thunar_autostart(v, autostart_file),
        )
        box.pack_start(row, False, False, 0)
        return box

    def _build_remmina_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("Remote desktop (Remmina)"), False, False, 0)
        box.pack_start(section_description(
            "Add every detected SSH, RDP, and VNC service on your mesh "
            "to Remmina automatically. Auto-managed entries live in a "
            "'Mesh Peers' group; your other Remmina connections are "
            "never touched."
        ), False, False, 0)

        from mackes import remmina_sync as rs

        # Status line shows current enabled state + entry count
        self._remmina_status = Gtk.Label(label="(checking…)")
        self._remmina_status.set_xalign(0)
        box.pack_start(self._remmina_status, False, False, 0)

        # Toggle row
        row = self._switch_row(
            label="Sync automatically (every 5 minutes)",
            initial=rs.is_enabled(),
            on_change=self._toggle_remmina_sync,
        )
        box.pack_start(row, False, False, 0)

        # "Sync now" button
        sync_now = Gtk.Button(label="Sync now")
        sync_now.connect("clicked", lambda *_: self._run_remmina_sync_now())
        sync_now.set_halign(Gtk.Align.START)
        sync_now.set_margin_top(4)
        box.pack_start(sync_now, False, False, 0)

        # Populate status on a thread so panel-construct stays fast
        import threading
        threading.Thread(target=self._refresh_remmina_status,
                         daemon=True).start()
        return box

    def _refresh_remmina_status(self) -> None:
        from mackes import remmina_sync as rs
        managed = rs._existing_managed_files()
        text = (f"✓ Auto-sync on · {len(managed)} managed entr"
                + ("ies" if len(managed) != 1 else "y")
                if rs.is_enabled() else
                f"Auto-sync off · {len(managed)} entr"
                + ("ies" if len(managed) != 1 else "y") + " present")
        GLib.idle_add(self._remmina_status.set_text, text)

    def _toggle_remmina_sync(self, enabled: bool) -> None:
        import threading
        from mackes import remmina_sync as rs
        def worker():
            (rs.enable if enabled else rs.disable)()
            self._refresh_remmina_status()
        threading.Thread(target=worker, daemon=True).start()

    def _run_remmina_sync_now(self) -> None:
        import threading
        from mackes import remmina_sync as rs
        def worker():
            report = rs.sync()
            GLib.idle_add(self._remmina_status.set_text, str(report))
        threading.Thread(target=worker, daemon=True).start()

    def _build_hud_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("Desktop HUD"), False, False, 0)
        box.pack_start(section_description(
            "The right-edge Carbon HUD shows mesh, fleet, and drift "
            "state at a glance. Pick how much information it shows."
        ), False, False, 0)

        t = _load_tweaks()
        # HUD on/off
        hud_on = bool(t.get("show_conky", True))
        row = self._switch_row(
            label="Show HUD",
            initial=hud_on,
            on_change=self._toggle_conky,
        )
        box.pack_start(row, False, False, 0)

        # Density radio group
        density_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        density_row.set_margin_top(8); density_row.set_margin_bottom(8)
        self._density_buttons = {}
        current_density = t.get("conky_density") or "standard"
        for opt in ("compact", "standard", "full"):
            b = Gtk.ToggleButton(label=opt.title())
            b.set_active(current_density == opt)
            b.connect("toggled",
                      lambda btn, o=opt:
                          btn.get_active() and self._set_hud_density(o))
            self._density_buttons[opt] = b
            density_row.pack_start(b, True, True, 0)
        density_lab = Gtk.Label(label="Density")
        density_lab.set_xalign(0); density_lab.set_margin_top(8)
        box.pack_start(density_lab, False, False, 0)
        box.pack_start(density_row, False, False, 0)

        # Monitor combo
        mon_combo = Gtk.ComboBoxText()
        mon_combo.append("", "Primary (auto-detect)")
        try:
            from mackes.conky_hud import _xrandr_outputs
            for o in _xrandr_outputs():
                mon_combo.append(o["name"], f"{o['name']} ({o['w']}×{o['h']})")
        except Exception:  # noqa: BLE001
            pass
        mon_combo.set_active_id(t.get("conky_monitor") or "")
        mon_combo.connect("changed",
                          lambda c: self._set_hud_monitor(c.get_active_id() or None))
        mon_lab = Gtk.Label(label="Monitor")
        mon_lab.set_xalign(0); mon_lab.set_margin_top(12)
        box.pack_start(mon_lab, False, False, 0)
        box.pack_start(mon_combo, False, False, 0)

        return box

    def _build_shortcut_section(self) -> Gtk.Widget:
        box = panel_box()
        box.pack_start(section_header("Quick access"), False, False, 0)
        box.pack_start(info_label(
            "The Tweaks gear is also available as a floating drawer "
            "anywhere in Mackes — click the gear in the bottom-right "
            "of any panel."
        ), False, False, 0)
        return box

    # ---- Toggle handlers -------------------------------------------------

    def _toggle_thunar_autostart(self, enabled: bool, autostart_file: Path) -> None:
        try:
            if enabled:
                autostart_file.parent.mkdir(parents=True, exist_ok=True)
                autostart_file.write_text(
                    "[Desktop Entry]\n"
                    "Type=Application\n"
                    "Name=Thunar (Mackes autostart)\n"
                    "Exec=thunar --daemon\n"
                    "X-GNOME-Autostart-enabled=true\n"
                    "X-Mackes-Managed=1\n",
                    encoding="utf-8",
                )
            else:
                if autostart_file.exists():
                    autostart_file.unlink()
        except OSError:
            pass

    def _toggle_conky(self, enabled: bool) -> None:
        _save_tweak("show_conky", enabled)
        try:
            from mackes.conky_hud import apply_tweak
            apply_tweak(enabled)
        except Exception:  # noqa: BLE001
            pass

    def _set_hud_density(self, value: str) -> None:
        _save_tweak("conky_density", value)
        for opt, b in self._density_buttons.items():
            if b.get_active() != (opt == value):
                b.set_active(opt == value)
        try:
            from mackes.conky_hud import is_running, restart_with
            if is_running():
                restart_with(density=value)
        except Exception:  # noqa: BLE001
            pass

    def _set_hud_monitor(self, value: str | None) -> None:
        _save_tweak("conky_monitor", value)
        try:
            from mackes.conky_hud import is_running, restart_with
            if is_running():
                restart_with(monitor=value)
        except Exception:  # noqa: BLE001
            pass

    # ---- _switch_row helper ----------------------------------------------

    def _switch_row(self, *, label: str, initial: bool,
                    on_change) -> Gtk.Widget:
        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        row.set_margin_top(6); row.set_margin_bottom(6)
        text = Gtk.Label(label=label); text.set_xalign(0)
        row.pack_start(text, True, True, 0)
        sw = Gtk.Switch()
        sw.set_active(initial)
        sw.connect("notify::active",
                   lambda s, _gp: on_change(s.get_active()))
        row.pack_start(sw, False, False, 0)
        return row


__all__ = ["TweaksPanel"]
