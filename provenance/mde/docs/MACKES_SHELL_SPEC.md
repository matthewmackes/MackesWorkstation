# Mackes Shell — Master Specification

**Version:** 0.1 (Design Spec)
**Successor to:** xfce11-unified v2.2
**Status:** Design locked via 20-question survey; implementation pending

---

## 0. What Mackes Shell Is

**Mackes Shell is the persistent control panel that replaces `xfce4-settings` as the daily interface for managing an XFCE-based Fedora workstation.**

It also serves two secondary roles:
- A first-run provisioner that brings a fresh machine to a known-good state via curated preset
- A drift monitor that warns when current configuration deviates from the active preset

It is **not** a configuration framework, not a fleet manager in the Ansible sense, and not a kitchen-sink desktop environment. It is a focused, opinionated, single-binary GTK app that swallows `xfce4-settings` plus the Xfce11 shell stack (Polybar / Plank / Rofi) and presents them as one coherent control surface.

---

## 1. Locked Design Decisions

The full design was settled across a 20-question survey. The complete table is included for posterity and as a reference when implementation drifts.

| # | Decision Area | Locked Answer |
|---|---|---|
| 1 | Tool identity | Persistent control panel replacing xfce4-settings as everyday interface |
| 2 | Replacement scope | Full xfce4-settings parity — Appearance + Shell + Hardware + Window Manager + Workspaces + Session/Startup + Notifications + Removable Media + Date/Time + Default Apps |
| 3 | Navigation model | Two-level hybrid — task-oriented top tabs, object panels inside each |
| 4 | Entry point | Single binary `mackes`; detects state and routes to wizard or workbench |
| 5 | Daily landing | Live status dashboard — service health, active profile, hardware summary, last snapshot, drift warnings, quick links |
| 6 | First-run flow | Full guided wizard (7+ screens) with live preview at each step |
| 7 | Profile model | Curated presets only — Workstation / Laptop / Audio Rig / Server Console |
| 8 | Bootstrap | One-line `curl …/install.sh \| bash` |
| 9 | Settings application | Immediate apply — every change writes through instantly (xfce4-settings behavior) |
| 10 | Snapshot / rollback | Manual only — user clicks "Create Restore Point" |
| 11 | Visual direction | Native GTK look — follow active GTK theme |
| 12 | Shell config model | Preset picker only — dropdown of named profiles for Polybar / Plank / Rofi |
| 13 | Appearance layout | One unified Appearance panel with internal sections (Theme / Icons / Cursor / Fonts / Wallpaper) |
| 14 | CLI surface | GUI only — no separate CLI tool |
| 15 | Web Workbench | Deleted entirely |
| 16 | Implementation strategy | Embed xfconf properties directly — Mackes panels bind to xfconf keys; `xfsettingsd` applies live |
| 17 | Operations location | Dedicated **Maintain** tab — Snapshots, Health Check, Dependencies, Logs, Repair, Reset to Preset |
| 18 | Network / QNM placement | Sixth top tab **Network** — Wi-Fi/Ethernet, VPN, QNM, Firewall |
| 19 | xfce4-settings coexistence | Hide menu entries + redirect deep links; xfce4-settings + xfsettingsd remain installed under the hood |
| 20 | Packaging / delivery | Direct RPM download from GitHub Releases — no COPR maintenance burden |

---

## 2. Architecture Overview

```
                ┌────────────────────────────────────────────────────┐
                │                  mackes  (GTK3 / PyGObject)        │
                │                                                    │
                │  ┌─────────────┐    ┌───────────────────────────┐  │
                │  │ State probe │───▶│  First-run?    Wizard     │  │
                │  └─────────────┘    │  Installed?    Workbench  │  │
                │                     └───────────────────────────┘  │
                │                                                    │
                │  ┌──────────────────────────────────────────────┐  │
                │  │  Panel registry — Look & Feel / Shell /      │  │
                │  │  Devices / Network / System / Maintain       │  │
                │  └──────────────────────────────────────────────┘  │
                │                          │                         │
                │                          ▼                         │
                │             ┌──────────────────────┐               │
                │             │   xfconf wrapper     │               │
                │             │ (python-xfconf or    │               │
                │             │  xfconf-query shell) │               │
                │             └──────────────────────┘               │
                └──────────────────────────┼─────────────────────────┘
                                           │
                                           ▼
                                  ┌────────────────┐
                                  │     xfconf     │ ◀── persists config in XDG
                                  └────────┬───────┘
                                           │
                                           ▼
                                  ┌────────────────┐
                                  │  xfsettingsd   │ ◀── unchanged, kept running
                                  └────────┬───────┘
                                           │
                          ┌────────────────┼────────────────┐
                          ▼                ▼                ▼
                       xfwm4          (panel/dock)     fonts/cursor/theme
```

**Why this works:**
- `xfconf` is XFCE's configuration database. Every xfce4-settings dialog is a GTK form over xfconf.
- `xfsettingsd` watches xfconf and pushes changes live to xfwm4, the cursor, fonts, etc. No restart required.
- Mackes panels are GTK forms over the same xfconf keys. When a user toggles something, Mackes writes xfconf, xfsettingsd notices, the desktop updates.
- xfce4-settings stays installed but its menu entries are hidden — its dialogs still function if launched manually, and any third-party tool that reads xfconf still works.

**What Mackes adds on top of xfconf:**
- The Xfce11 shell layer (Polybar / Plank / Rofi) — managed via file-based configs, since xfconf doesn't cover these
- Preset definitions and apply logic (`mackes/presets_engine.py`)
- Snapshot/restore (file-tree backup of `~/.config/xfce4/`, `~/.config/polybar/`, `~/.config/plank/`, `~/.config/rofi/`, plus xfconf channel dumps)
- Drift detection (compare current state to active preset)
- Maintenance operations (health checks, dependency install, repair)
- QNM integration (existing QNM daemon, exposed in the Network tab)

---

## 3. Navigation Structure

```
mackes
├── Dashboard                      (landing — not a tab, the home view)
│
├── Look & Feel
│   └── Appearance                 (one unified panel)
│       ├── Theme                  (GTK theme picker, dark/light toggle)
│       ├── Icons                  (icon theme picker)
│       ├── Cursor                 (cursor theme + size)
│       ├── Fonts                  (UI font / monospace / TTY console font + sizes)
│       └── Wallpaper              (per-monitor wallpaper + style)
│
├── Shell
│   ├── Polybar                    (preset picker only)
│   ├── Plank                      (preset picker only)
│   ├── Rofi Launcher              (preset picker only)
│   └── Panel Visibility           (toggle xfce4-panel autostart on/off + rollback)
│
├── Devices
│   ├── Display                    (xfconf: /Display)
│   ├── Keyboard                   (xfconf: /Keyboard + shortcuts)
│   ├── Mouse & Touchpad           (xfconf: /Pointers)
│   ├── Sound                      (PulseAudio default sink/source picker)
│   └── Power                      (xfconf: xfce4-power-manager channel)
│
├── Network
│   ├── Wi-Fi / Ethernet           (nmcli-backed status + connect/disconnect)
│   ├── VPN                        (NetworkManager VPN list + import .ovpn/.conf)
│   ├── Quick Network Mesh         (QNM enable/disable, GUI launcher, status)
│   └── Firewall                   (firewalld zone + service list)
│
├── System
│   ├── Window Manager             (xfconf: /xfwm4)
│   ├── Workspaces                 (xfconf: /xfwm4 workspace count + names)
│   ├── Session & Startup          (xfconf: /xfce4-session + autostart .desktop list)
│   ├── Notifications              (xfconf: /xfce4-notifyd)
│   ├── Default Apps               (mimeapps.list)
│   ├── Removable Media            (xfconf: /thunar-volman)
│   └── Date & Time                (timedatectl wrapper)
│
└── Maintain
    ├── Snapshots                  (list / create / restore / delete)
    ├── Health Check               (preflight + validate, unified)
    ├── Dependencies               (missing/optional package list + install button)
    ├── Logs                       (tail mackes.log + xfsettingsd journal)
    ├── Repair                     (re-apply current preset; rebuild menu folder; restore xfce4-settings menu entries)
    └── Reset to Preset            (revert all local changes to the active preset's defaults)
```

Six top tabs, ~25 second-level panels. Every panel has one job.

---

## 4. The Dashboard (daily landing)

A single scrolled view, top-to-bottom:

### 4.1 Status strip (always visible at top)

A horizontal row of compact status badges:

```
[●] Polybar running    [●] Plank running    [●] Rofi installed    [●] xfsettingsd alive    [●] xfconf reachable
Preset: Audio Rig                Last snapshot: 2h ago "before-display-rearrange"
```

Green dot = ok. Amber = warning. Red = action needed.

### 4.2 Drift card (only shown when drift exists)

```
┌────────────────────────────────────────────────────────────────────────┐
│  ⚠  Configuration drift from "Audio Rig" preset                        │
│                                                                        │
│   • GTK theme is "Arc-Dark", preset specifies "Mac-Dark"               │
│   • Polybar profile is "Custom-1", preset specifies "Icon-Only"        │
│   • 2 autostart entries not in preset                                  │
│                                                                        │
│   [Review drift]   [Reset to preset]   [Save current as new preset]    │
└────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Hardware summary card

Hostname, kernel, CPU, RAM, audio device, monitors, network interfaces. Read once on launch, refresh on demand.

### 4.4 Quick actions row

Six big buttons for the most-used operations:

```
[ Open Appearance ]  [ Switch Theme ]  [ Switch Polybar Profile ]
[ Create Snapshot ]  [ Health Check ]  [ Open Log ]
```

### 4.5 Recent actions

Last 5 changes Mackes made, with timestamps. Each clickable to open its panel.

---

## 5. First-Run Wizard

When `mackes` launches and detects the system has never been provisioned (no `~/.config/mackes-shell/state.json` or no installed-marker), the wizard runs. Live preview at every step means changes apply as you toggle them, and Back un-applies cleanly.

```
Screen 1  ┃  Welcome           — what Mackes is, what it will do, key bindings
Screen 2  ┃  Environment Scan  — detected hardware, Fedora version, XFCE version,
          ┃                      missing/optional packages, current XFCE panel state
Screen 3  ┃  Preset Selection  — Workstation / Laptop / Audio Rig / Server Console
          ┃                      cards with description and preview thumbnail
Screen 4  ┃  Appearance        — theme/icons/font choices for this preset; live preview
Screen 5  ┃  Shell Layout      — Polybar profile, Plank dock, Rofi launcher; live preview
Screen 6  ┃  Hardware          — display arrangement, default audio output, power profile
Screen 7  ┃  Network           — enable QNM Y/N, firewall stance, VPN import (optional)
Screen 8  ┃  Snapshot Policy   — create initial restore point Y/N, name it
Screen 9  ┃  Review            — full diff of what will be applied
Screen 10 ┃  Apply             — progress bar streaming actions; on success drop into
          ┃                      the Dashboard with a "Welcome to Mackes" overlay
```

The wizard is exit-able at any point (with a confirm prompt). On exit, whatever was already applied stays applied.

---

## 6. Profile Presets

Each preset is a YAML file in `data/presets/`. Presets are read-only at runtime; they define the *target* state. Daily changes via the Workbench update the live system but **never** modify the preset file — that's why drift exists.

### 6.1 Preset schema

```yaml
name: audio-rig
display_name: "Audio Rig"
description: "Performance-tuned for low-latency audio work. JACK-friendly. Minimal background services."

appearance:
  gtk_theme: "Mac-Dark"
  icon_theme: "Papirus-Dark"
  cursor_theme: "Adwaita"
  cursor_size: 24
  font_ui: "Droid Sans 10"
  font_monospace: "JetBrains Mono 10"
  wallpaper: "/usr/share/backgrounds/mackes/audio-rig.jpg"

shell:
  polybar_profile: "icon-only"
  plank_profile: "minimal-bottom"
  rofi_profile: "black-droid"
  xfce_panel_enabled: false

devices:
  display_scaling: "auto"
  power_profile: "performance"
  audio_default_sink: "pipewire"

network:
  qnm_enabled: true
  firewall_default_zone: "FedoraWorkstation"

system:
  workspace_count: 2
  window_manager_theme: "Default-xhdpi"
  autostart_extras:
    - jack-control.desktop

snapshot:
  initial_snapshot_name: "audio-rig-baseline"
```

### 6.2 Shipped presets

| Preset | Targeted at | Distinguishing choices |
|---|---|---|
| **Workstation** | Desktop dev box, multi-monitor | Mac-Dark theme, Polybar "Power User" profile (CPU/RAM/network/temp/clock), Plank standard dock, 4 workspaces, performance power profile, QNM on |
| **Laptop** | Portable, battery-aware | Mac-Light theme, Polybar "Mac-Style" profile (battery prominent), Plank intellihide, 2 workspaces, balanced power profile, Wi-Fi/VPN prominent, QNM optional |
| **Audio Rig** | Low-latency audio production | Mac-Dark theme, Polybar "Icon-Only" minimal profile, Plank minimal, 2 workspaces, performance profile, JACK autostart, no notification daemon noise, QNM on |
| **Server Console** | Headless-ish admin box | High-contrast theme, Polybar "Minimal" (no tray, no clock noise), no Plank, 1 workspace, power-save off (always-on), QNM on, firewall locked down |

Custom user presets are **out of scope** per Q7. Power users who want custom presets can drop a YAML file in `~/.config/mackes-shell/presets/` and it will appear in the picker — undocumented but supported.

---

## 7. File-System Layout

### 7.1 Source tree (the repo)

```
mackes-shell/
├── README.md
├── LICENSE
├── install.sh                     # the curl-bootstrap target
├── packaging/
│   └── fedora/
│       └── mackes-shell.spec      # RPM spec
├── data/
│   ├── presets/                   # shipped preset YAML files
│   │   ├── workstation.yaml
│   │   ├── laptop.yaml
│   │   ├── audio-rig.yaml
│   │   └── server-console.yaml
│   ├── shell-profiles/            # Polybar/Plank/Rofi config templates
│   │   ├── polybar/
│   │   │   ├── icon-only.ini
│   │   │   ├── power-user.ini
│   │   │   ├── mac-style.ini
│   │   │   └── minimal.ini
│   │   ├── plank/
│   │   │   ├── standard.dock
│   │   │   ├── minimal.dock
│   │   │   └── intellihide.dock
│   │   └── rofi/
│   │       ├── black-droid.rasi
│   │       └── default.rasi
│   ├── wallpapers/
│   ├── icons/
│   │   └── mackes-shell.svg
│   ├── applications/
│   │   └── mackes-shell.desktop   # the only menu entry Mackes installs
│   └── overrides/
│       └── xfce-settings-hidden/  # .desktop overrides to hide xfce4-settings menu entries
├── mackes/                        # the Python package
│   ├── __init__.py
│   ├── __main__.py                # `python -m mackes`
│   ├── app.py                     # Gtk.Application, state routing
│   ├── state.py                   # install state, active preset, drift detection
│   ├── xfconf_bridge.py           # xfconf read/write wrapper
│   ├── presets.py                 # preset load + apply engine
│   ├── snapshots.py               # snapshot create/list/restore
│   ├── shell_profiles.py          # Polybar/Plank/Rofi profile apply
│   ├── menu_integration.py        # hide xfce4-settings entries, install mackes menu
│   ├── qnm_bridge.py              # talk to the existing QNM daemon
│   ├── wizard/
│   │   ├── __init__.py
│   │   ├── window.py              # wizard Assistant
│   │   └── pages/
│   │       ├── welcome.py
│   │       ├── env_scan.py
│   │       ├── preset_pick.py
│   │       ├── appearance.py
│   │       ├── shell.py
│   │       ├── hardware.py
│   │       ├── network.py
│   │       ├── snapshot.py
│   │       ├── review.py
│   │       └── apply.py
│   └── workbench/
│       ├── __init__.py
│       ├── window.py              # Workbench main window
│       ├── dashboard.py
│       ├── look_and_feel/appearance.py
│       ├── shell/
│       │   ├── polybar.py
│       │   ├── plank.py
│       │   ├── rofi.py
│       │   └── panel_visibility.py
│       ├── devices/
│       │   ├── display.py
│       │   ├── keyboard.py
│       │   ├── mouse.py
│       │   ├── sound.py
│       │   └── power.py
│       ├── network/
│       │   ├── wifi.py
│       │   ├── vpn.py
│       │   ├── qnm.py
│       │   └── firewall.py
│       ├── system/
│       │   ├── window_manager.py
│       │   ├── workspaces.py
│       │   ├── session.py
│       │   ├── notifications.py
│       │   ├── default_apps.py
│       │   ├── removable.py
│       │   └── datetime.py
│       └── maintain/
│           ├── snapshots.py
│           ├── health_check.py
│           ├── dependencies.py
│           ├── logs.py
│           ├── repair.py
│           └── reset_to_preset.py
└── tests/
    └── …
```

### 7.2 Runtime paths (where Mackes writes on the user's machine)

```
~/.config/mackes-shell/
├── state.json                     # install state, active preset, last apply timestamp
├── presets/                       # user-added presets (optional, undocumented)
└── overrides/                     # backup of any xfce4-settings .desktop entries Mackes hid

~/.local/share/mackes-shell/
├── logs/
│   └── mackes.log                 # unified log
├── snapshots/
│   └── 2026-05-15T142300_pre-display/
│       ├── manifest.json
│       ├── xfconf/                # xfconf-query --channel xx --dump
│       ├── polybar/               # copy of ~/.config/polybar
│       ├── plank/                 # copy of ~/.config/plank
│       └── rofi/                  # copy of ~/.config/rofi
└── data/                          # cached/derived runtime data
```

System install paths (from RPM):

```
/usr/bin/mackes
/usr/lib/python3.X/site-packages/mackes/...
/usr/share/mackes-shell/data/...   (presets, shell-profiles, wallpapers)
/usr/share/applications/mackes-shell.desktop
/usr/share/icons/hicolor/scalable/apps/mackes-shell.svg
```

---

## 8. Implementation Notes (per-area)

### 8.1 xfconf binding

Each panel inherits a base `XfconfPanel` class:

```python
class XfconfPanel(Gtk.Box):
    channel: str        # e.g. "xsettings"

    def bind(self, widget, key: str, default=None):
        """Two-way bind a GTK widget to an xfconf key. Immediate apply."""
```

The bridge uses `python-xfconf` if available (preferred — it gets DBus change notifications for free) or falls back to `xfconf-query` subprocess. All writes are immediate (Q9). Mackes never batches.

### 8.2 Shell profiles (Polybar / Plank / Rofi)

Profiles are config-file blobs in `data/shell-profiles/`. Applying = copy file to `~/.config/<tool>/`, then signal the running daemon (Polybar: `killall -USR1 polybar` or full restart via launcher script; Plank: dconf overwrite; Rofi: no daemon to signal).

Polybar launcher script (`~/.local/bin/mackes-polybar-launch.sh`) is owned by Mackes and re-generated on profile switch.

### 8.3 Snapshots

A snapshot is a timestamped directory under `~/.local/share/mackes-shell/snapshots/`. Contents:
- xfconf dumps of all known channels (`xfconf-query --channel X --dump`)
- Copies of `~/.config/polybar/`, `~/.config/plank/`, `~/.config/rofi/`
- Copy of `~/.config/xfce4/panel/` (for full XFCE panel restore)
- A `manifest.json` with timestamp, optional user-supplied name, source preset, hostname

Restore = wipe live config dirs, copy snapshot contents back, `xfconf-query --channel X --load` each dump file, signal xfsettingsd.

### 8.4 Drift detection

On dashboard load, compare the active preset's declared values to the current system state. Each preset YAML field has a corresponding "read current value" function. Mismatches go into the drift card (§4.2). Drift detection is read-only and informational — no automatic remediation.

### 8.5 Menu integration

`menu_integration.py` does on Mackes install:

1. Write `~/.local/share/applications/xfce-display-settings.desktop` etc. with `NoDisplay=true` (XDG override that hides without removing). One override per xfce4-settings .desktop.
2. Snapshot the originals to `~/.config/mackes-shell/overrides/` so `mackes maintain repair` can restore them.
3. Install `mackes-shell.desktop` as the new top-level Settings entry.
4. Register deep-link mime handlers so `xfce4-display-settings &` routed via `xdg-open` opens the Mackes Display panel. (Optional, nice-to-have.)

On Mackes uninstall, the RPM scriptlet restores the originals.

### 8.6 QNM bridge

The existing `qnmctl` / `qnmd` / `qnm-gui` from v2.2 stay in their own service unit and binary. Mackes' Network → QNM panel just calls `qnmctl status`, exposes start/stop/restart, and embeds the existing GUI as a launcher button. No QNM logic moves into Mackes itself.

---

## 9. Build & Distribution

### 9.1 install.sh (the curl bootstrap)

Roughly:

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO="mattmacke/mackes-shell"           # GitHub repo
TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep -oP '"tag_name":\s*"\K[^"]+')
URL="https://github.com/$REPO/releases/download/$TAG/mackes-shell-${TAG#v}.fc$(rpm -E %fedora).noarch.rpm"
TMP=$(mktemp -d)
curl -L -o "$TMP/mackes.rpm" "$URL"
sudo dnf install -y "$TMP/mackes.rpm"
rm -rf "$TMP"
exec mackes
```

The user's curl-pipe-bash one-liner installs Mackes and immediately launches it into the first-run wizard.

### 9.2 RPM spec sketch

```spec
Name:           mackes-shell
Version:        0.1.0
Release:        1%{?dist}
Summary:        Mackes Shell — XFCE control panel and shell manager
License:        GPL-3.0
URL:            https://github.com/mattmacke/mackes-shell
Source0:        %{name}-%{version}.tar.gz
BuildArch:      noarch

Requires:       python3, python3-gobject, gtk3, xfconf, xfce4-settings
Requires:       polybar, plank, rofi
Requires:       NetworkManager, firewalld
Recommends:     papirus-icon-theme, arc-theme, google-droid-sans-fonts, jetbrains-mono-fonts

%post
# Install hidden-overrides for xfce4-settings menu entries
/usr/share/mackes-shell/install-helpers/hide-xfce-settings.sh || :

%preun
# Restore hidden xfce4-settings menu entries
/usr/share/mackes-shell/install-helpers/restore-xfce-settings.sh || :
```

The RPM **Requires** `xfce4-settings` (per Q19 lock — we keep it installed and hide menu entries, not uninstall it).

### 9.3 Release pipeline

GitHub Actions on tag push:
1. Build source tarball
2. Run `rpmbuild --define '_topdir $PWD/rpmbuild' -ba packaging/fedora/mackes-shell.spec` inside a Fedora container
3. Upload the resulting `.noarch.rpm` to the GitHub release as a release asset

`install.sh` finds it via the GitHub Releases API. No COPR needed (Q20 lock).

---

## 10. Migration from xfce11-unified v2.2

A full mapping is in `docs/MIGRATION_FROM_V2.2.md`. Summary:

- **Kept**: backend logic in `scripts/xfce11v2.py` (refactored into the `mackes/` Python package — split into `presets.py`, `snapshots.py`, `shell_profiles.py`, `menu_integration.py`, `qnm_bridge.py`)
- **Kept**: Polybar/Plank/Rofi config templates (move to `data/shell-profiles/`)
- **Kept**: QNM (`quick-network-mesh/`) entirely as a standalone — Mackes just embeds its GUI as a launcher
- **Kept**: Fedora RPM spec (rewritten to install `/usr/bin/mackes` instead of the v2.2 layout)
- **Deleted**: `web-workbench/` (Q15)
- **Deleted**: `native-workbench/xfce11_workbench.py` (replaced by `mackes/workbench/`)
- **Deleted**: `scripts/install*.sh` (replaced by single `install.sh`)
- **Deleted**: `START-HERE-XFCE11-UNIFIED.desktop` (replaced by `mackes-shell.desktop`)
- **Deleted**: the flat 30+ action list in the Workbench — actions become either settings panels or live in the Maintain tab
- **Deleted**: the "Apply Black Droid Menu Chrome" duplicate — it becomes a Theme preset in the Appearance panel, not a top-level button
- **Deleted**: separate per-font buttons — become one Font picker in the Appearance panel

---

## 11. Open Items for Later

These were intentionally deferred during the survey and are tracked here so they don't get lost:

1. **xfsettingsd decoupling** (Q19 fallback). Long-term goal of full xfce4-settings uninstall. Requires shipping a Mackes equivalent daemon. Not v1.
2. **Custom user presets UI.** YAML drop-in works but is undocumented. A "save current as preset" button could land in v2 if real demand emerges.
3. **Wayland.** Mackes targets X11 + XFCE today. xfce4 Wayland support is still nascent.
4. **Search bar across panels.** xfce4-settings doesn't have one. GNOME Settings does. Worth considering once panel count is final.
5. **Multi-machine sync of customizations.** Out of scope per Q7 (curated presets only). Re-evaluate if usage shows people manually editing the same settings on every machine.
6. **Wallpaper slideshow / per-workspace wallpaper.** v2 if requested. xfdesktop supports it.
7. **Keyboard-shortcut conflict detector.** Would be a nice addition to the Devices → Keyboard panel.

---

## 12. Success Criteria

Mackes Shell is succeeding when:

- A new machine can be brought from fresh Fedora install to working Xfce11 environment with **one** curl one-liner plus the wizard, in under five minutes.
- A user never needs to open `xfce4-settings-manager` for daily settings work.
- Switching presets is one click and visibly reconfigures the entire desktop.
- Drift from an active preset is always visible on the dashboard.
- The "disjointed" feeling of v2.2 — five entry points, two workbenches, 30 flat actions — is gone. There is one binary, one window, six tabs.
