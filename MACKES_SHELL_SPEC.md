# Mackes Shell — Master Specification

**Version:** 0.1.1 (Design Spec — MAP2 Sub Testing Release, PRIVATE WORK)
**Successor to:** xfce11-unified v2.2
**Status:** Implemented. Latest round of decisions locked via the 50-question
survey + 11 clarifications + Lean-XFCE follow-up (X1–X5) + Polybar fix (P1).

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
| 7 | Profile model | Single curated preset (`chupre`). Earlier shipped presets (Workstation / Laptop / Audio Rig / Server Console) removed in 0.1.1. User-preset overrides in `~/.config/mackes-shell/presets/` still respected. |
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
│   ├── Session & Startup          (xfconf: /xfce4-session + autostart + managed-process supervisor)
│   ├── Notifications              (xfconf: /xfce4-notifyd)
│   ├── Default Apps               (mimeapps.list)
│   ├── Removable Media            (xfconf: /thunar-volman)
│   └── Date & Time                (timedatectl wrapper)
│
├── Apps                            ← new 7th tab (C1 lock)
│   ├── Install                    (curated set per active preset's apps.install)
│   ├── Remove                     (Fedora bloat + 'XFCE components replaced by Mackes')
│   └── Installed                  (searchable rpm -qa with per-row remove)
│
└── Maintain
    ├── Snapshots                  (list / create / restore / delete)
    ├── Health Check               (preflight + validate, unified)
    ├── Dependencies               (missing/optional package list + install button)
    ├── Logs                       (tail mackes.log + xfsettingsd journal)
    ├── Repair                     (re-apply current preset; rebuild menu folder; restore xfce4-settings menu entries; rewrite Polybar launcher + autostart)
    ├── Reset to Preset            (revert all local changes to the active preset's defaults)
    └── Uninstall                  ← new sub-panel (Q8 lock) — see §8.7
```

Seven top tabs, ~28 second-level panels. Every panel has one job.

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

The single shipped preset is `data/presets/chupre.yaml` — reproduced here verbatim
as the schema reference (Q33 lock — the shipped YAML *is* the example):

```yaml
name: chupre
display_name: "Chupre (default)"
description: >
  Dark, modern, polished. WhiteSur-Dark theme, Inter Nerd Font UI, the chupre
  Polybar with workspaces / window title / weather / battery / volume / power
  modules, matching Rofi launcher. Adapted for XFCE.

appearance:
  gtk_theme: "WhiteSur-Dark"
  icon_theme: "WhiteSur-dark"
  cursor_theme: "XCursor-Pro-Dark"
  cursor_size: 24
  font_ui: "Inter Nerd Font 11"
  font_monospace: "SFMono Nerd Font 11"
  wallpaper: "/usr/share/mackes-shell/wallpapers/chupre.jpg"

shell:
  polybar_profile: "chupre-custom"
  plank_profile:   "chupre"
  rofi_profile:    "chupre"
  xfce_panel_enabled: false

devices:
  display_scaling:    "auto"
  power_profile:      "balanced"
  audio_default_sink: "pipewire"

network:
  qnm_enabled:           false
  firewall_default_zone: "FedoraWorkstation"

system:
  workspace_count:       9
  window_manager_theme:  "Default"
  notifications_enabled: true
  autostart_extras:
    - picom.desktop

apps:
  install:            [filezilla, terminator, vlc, microsoft-edge-stable,
                       code, cursor, claude-code, remmina, mc, neofetch, dunst]
  remove_bloat:       [gnome-software, gnome-tour, gnome-maps, gnome-weather,
                       gnome-contacts, gnome-clocks, gnome-calendar,
                       gnome-music, totem, "libreoffice-*"]
  lean_xfce_remove:
    - {package: xfce4-panel,     replaced_by: polybar}
    - {package: xfce4-appfinder, replaced_by: rofi}
    - {package: xfdesktop,       replaced_by: plank}
    - {package: xfce4-notifyd,   replaced_by: dunst}

snapshot:
  initial_snapshot_name: "chupre-baseline"
```

### 6.2 Shipped presets

| Preset | Targeted at | Distinguishing choices |
|---|---|---|
| **Chupre** | Default for every install | WhiteSur-Dark theme, chupre-custom Polybar, matching Rofi launcher, 9 workspaces, picom autostart, Inter / SFMono Nerd Fonts |

Earlier shipped presets (Workstation / Laptop / Audio Rig / Server Console)
were removed in 0.1.1 — the wizard's preset-pick screen is auto-skipped when
only one preset is shipped (Q2 lock). Custom user presets remain undocumented
but supported via `~/.config/mackes-shell/presets/` (Q3, Q31 locks). When a
user-local preset exists, the wizard reinstates the preset-pick screen so the
user can choose between chupre and their own.

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

The existing `qnmctl` / `qnmd` / `qnm-gui` from v2.2 stay in their own service unit and binary. Mackes' Network → QNM panel just calls `qnmctl status`, exposes start/stop/restart, and embeds the existing GUI as a launcher button. No QNM logic moves into Mackes itself. RPM relation is `Recommends: qnm` (C2/Q38) — soft, not hard.

### 8.7 Uninstall (Maintain → Uninstall, `mackes --uninstall`)

Locked across Q8–Q30 and X5. The uninstall path lives in `mackes/uninstall.py` and is exposed two ways: the **GUI panel** at Maintain → Uninstall (streaming log + progress bar + single-checkbox consent + post-uninstall logout countdown dialog) and the **CLI flag** `mackes --uninstall` (which accepts `--yes` to bypass the interactive `UNINSTALL` confirmation).

Sequence (best-effort — every step records its own success/failure and the run continues):

1. **Pre-uninstall snapshot.** `snapshots.create_snapshot("pre-uninstall-<ts>")`, then tarball to `~/Desktop/mackes-shell-final-snapshot-<ts>.tar.gz` — the only artifact that survives the uninstall (Q11, Q12, Q13).
2. **Stop managed daemons.** `pkill -x polybar`, `pkill -x plank`, `pkill -x dunst`, `pkill -x picom` (Q16).
3. **Reinstall lean-XFCE.** Any packages Mackes removed during provisioning (tracked in `~/.config/mackes-shell/removed-by-mackes.json`) are reinstalled via `dnf install` so the user gets stock XFCE back (X5).
4. **Reset xfconf.** `xfconf-query --reset --root -r` on every known channel (xsettings, xfwm4, xfce4-desktop, xfce4-panel, xfce4-notifyd, xfce4-power-manager, xfce4-session, thunar-volman, keyboards, pointers), then `pkill -HUP xfsettingsd` (Q14, Q40).
5. **Re-enable xfce4-panel.** Drop the disabling autostart override, then `exec xfce4-panel` (Q17).
6. **Run `restore-xfce-settings.sh`** explicitly via pkexec/sudo to remove the `X-Mackes-Hidden=1` overrides from `/etc/skel/.local/share/applications/` (Q18).
7. **Delete user files.** `~/.config/mackes-shell/`, `~/.local/share/mackes-shell/`, snapshots, `~/.config/{polybar,plank,rofi,alacritty,gtk-3.0,gtk-4.0}/`, Polybar autostart .desktop, launcher script (Q15).
8. **Remove v2.2 leftovers.** Known path list: `~/xfce11-unified`, `~/Desktop/xfce11-unified`, `/opt/xfce11-unified`, `/usr/local/share/xfce11-unified`, `START-HERE-XFCE11-UNIFIED.desktop` (under `~/Desktop`, `/usr/share/applications`, `/usr/local/share/applications`). `quick-network-mesh/` is intentionally preserved (Q19, Q20, Q21).
9. **Remove the Mackes package.** Install mode is auto-detected (Q29) — `rpm -q mackes-shell` → `dnf remove`; `pip show mackes-shell` → `pip uninstall`; git checkout → no package removal (manual `rm -rf` is the user's job).
10. **Write the uninstall log** to `~/Desktop/mackes-shell-uninstall-<ts>.log` (Q27). Every line emitted to the streaming GUI view also lands here.
11. **Post-uninstall.** Show a "Log out in 10s — [Stay logged in]" dialog (Q25); on OK or expiry, fire `xfce4-session-logout --logout --fast` so the next login lands in a clean stock XFCE session.

The Releases artifact `uninstall.sh` (Q46) curl-pipe-bashes to either `mackes --uninstall --yes` (when Mackes is installed) or runs a standalone bash routine that cleans the v2.2 path list (Q47) — that's the only way to clean v2.2 residue from a machine where Mackes never landed.

### 8.8 App Management (Apps tab)

C1–C4 + C9 + C10 locks. Three sub-panels:

* **Apps → Install.** Renders the active preset's `apps.install` list. Per-row backend badge (`Fedora repo` / `third-party repo` / `AppImage` / `npm global`). Curated set: Filezilla, Terminator, VLC, Microsoft Edge, VS Code, Cursor, Claude CLI, Remmina, mc, neofetch, dunst. Third-party repos (Microsoft, VS Code) are added transparently via `pkexec bash -lc <snippet>` — every addition is logged.
* **Apps → Remove.** Two grouped subsections (X2 lock):
  - *Fedora bloat* — `apps.remove_bloat` from the preset. Default set: GNOME-on-XFCE apps + `libreoffice-*` (C9). Glob expressions stay verbatim; reinstall uses the same expression.
  - *XFCE components replaced by Mackes* — `apps.lean_xfce_remove`. Each entry declares its `replaced_by` daemon. A row is only marked *eligible for removal* when the replacement is actually running (X4) — so a user who hasn't started Polybar yet cannot accidentally remove `xfce4-panel`.
* **Apps → Installed.** `rpm -qa` browser with a filter box and per-row destructive remove.

Removals are tracked in `~/.config/mackes-shell/removed-by-mackes.json` with two categories — `bloat` and `lean_xfce`. `mackes --uninstall` reads `lean_xfce` to reinstall those packages (X5).

The catalog (`mackes/app_mgmt.py:CATALOG`) maps each curated name to a backend. Adding a new app means adding a `CATALOG` entry; the preset's `apps.install` list can name anything in the catalog or fall back to plain `dnf install <name>` for unknown names.

### 8.9 Session-manager extension

C6.a / C6.b / C11 locks. `mackes/session_manager.py` owns three responsibilities:

1. **Apply the chupre dotfiles bundle.** The bundle is staged under `data/shell-profiles/chupre/<subdir>/` and copied to `~/.config/<subdir>/` at preset-apply time. Applied subdirs: `alacritty`, `gtk-3.0`, `gtk-4.0`. Skipped: `i3`, `picom`, `sxhkd`, `nvim`, `networkmanager-dmenu` (not part of an XFCE session). Polybar / Plank / Rofi configs are owned by `shell_profiles.py` and not duplicated here.
2. **Supervise managed processes.** Registry: polybar, plank, dunst, picom. For each, `process_status()` returns `installed / running / pid`. Cheap pgrep — no background daemon.
3. **Surface state in the UI.** The Dashboard's status strip carries a "Managed:" row of dots (per-process); System → Session has a "Managed processes" section with Start / Stop / Restart per row.

The Polybar fix (P1) is wired through here: the launcher script `~/.local/bin/mackes-polybar-launch.sh` parses `[bar/<name>]` headers from the active profile's `.ini` (no more hardcoded `polybar mackes`) and redirects stderr to `~/.local/share/mackes-shell/logs/polybar.log`. An autostart entry `~/.config/autostart/mackes-polybar.desktop` ensures Polybar comes up on every login — that was the missing piece in the previous version.

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
