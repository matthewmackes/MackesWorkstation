# Migration: xfce11-unified v2.2 → Mackes Shell 0.1

Concrete mapping of every v2.2 artifact to its destination in Mackes Shell.

---

## 1. Files

### 1.1 Deleted entirely

| Path | Why |
|---|---|
| `install-unified.sh` | Replaced by `install.sh` (curl-bootstrap target, RPM-based) |
| `scripts/install.sh` | Same — single bootstrap entry |
| `scripts/install-unified.sh` | Same — duplicate |
| `scripts/install-gui.sh` | Same — duplicate |
| `scripts/install-qnm.sh` | QNM install handled by the RPM `Requires:` line + Mackes Network panel |
| `scripts/xfce11-unified.sh` | Replaced by the `/usr/bin/mackes` entry point |
| `scripts/debug-install.sh` | Folded into Maintain → Logs panel |
| `scripts/show-last-log.sh` | Folded into Maintain → Logs panel |
| `scripts/uninstall.sh` | RPM `dnf remove mackes-shell` handles this |
| `scripts/verify-install.sh` | Folded into Maintain → Health Check panel |
| `scripts/install-gui.sh` | Same |
| `START-HERE-XFCE11-UNIFIED.desktop` | Replaced by `mackes-shell.desktop` (the canonical menu entry) |
| `desktop/quick-network-mesh.desktop` | Stays with QNM; Mackes Network panel links to it |
| `web-workbench/` (entire directory) | Q15 — deleted entirely |
| `native-workbench/xfce11_workbench.py` | Replaced by `mackes/workbench/` package |
| `launcher/xfce11-launcher` | Becomes one of the Rofi profiles in `data/shell-profiles/rofi/` |
| `docs/INSTALLER_WORKBENCH_IMPROVEMENTS.md` | Superseded by this doc + the master spec |
| `docs/MENU_CHROME_V2_2.md` | Folded into the Theme preset docs |
| `docs/POLYBAR_ICON_ONLY.md` | Folded into Polybar profile docs |
| `docs/POLYBAR_PLANK_WORKBENCH.md` | Folded into shell-profiles docs |
| `docs/POLYBAR_THIN_NUMERIC_V2_2.md` | Folded into Polybar profile docs |
| `docs/XDG_MENU_FOLDER.md` | Folded into menu_integration docs |
| `docs/NO_STUBS_IMPLEMENTATION.md` | Obsolete with redesign |
| `RELEASE_NOTES_V2.md` | Replaced by per-release notes on GitHub Releases |
| `MIGRATION_I3_TO_POLYBAR.md` | Historical; archive in `docs/history/` if desired |
| `requirements/XFCE11_UNIFIED_V2_*.{md,json}` | Historical; archive |
| `requirements/MIGRATION_I3_TO_POLYBAR.md` | Historical; archive |
| `requirements/FEATURE_INVENTORY.md` | Replaced by the spec |
| `FEATURE_INVENTORY.md` (root) | Same |
| `QNM_BUILD_MANIFEST.md` | Stays with QNM source tree |
| `QUICK_NETWORK_MESH.md` | Stays with QNM source tree |
| `TERMINAL_FONT_OPTIONS.md` | Folded into Appearance → Fonts docs |
| `rollback/README.md` | Folded into Maintain → Snapshots docs |
| `systemd/user/*` | Reviewed individually; QNM units stay, anything else folded into mackes-shell.spec |

### 1.2 Refactored and kept

| Old path | New path | Notes |
|---|---|---|
| `scripts/xfce11v2.py` | Split across `mackes/presets.py`, `mackes/snapshots.py`, `mackes/shell_profiles.py`, `mackes/menu_integration.py`, `mackes/xfconf_bridge.py` | The big monolithic backend is decomposed into focused modules. The CLI surface is dropped (Q14). |
| `polybar/scripts/*` | `data/shell-profiles/polybar/scripts/` | Polybar helper scripts stay alongside their profile configs |
| `quick-network-mesh/qnm` | `quick-network-mesh/qnm` (unchanged) | QNM source code stays untouched; its own subtree |
| `quick-network-mesh/qnmctl` | unchanged | Same |
| `quick-network-mesh/qnm-daemon` | unchanged | Same |
| `quick-network-mesh/qnm-gui.sh` | unchanged | Same — Mackes Network panel calls it |
| `packaging/fedora/*.spec` | `packaging/fedora/mackes-shell.spec` | Rewritten for the new layout. xfce4-settings becomes a `Requires:` (Q19) |
| `scripts/build-rpm.sh` | `packaging/fedora/build-rpm.sh` | Mostly unchanged, repathed |
| `vendor/xfce-control-center-fedora44/` | `vendor/xfce-control-center-fedora44/` | Vendored XFCE references stay where they are |

---

## 2. Backend actions

The v2.2 Workbench exposed ~30 actions via `xfce11v2.py`. Each one has a destination in Mackes:

| v2.2 action | Destination in Mackes |
|---|---|
| `install-all` | First-run wizard final step |
| `preflight` | Maintain → Health Check (button) |
| `install-deps` | Maintain → Dependencies (button) + auto-suggested on Dashboard when missing |
| `restore-point` | Maintain → Snapshots ("Create" button) + Dashboard quick action |
| `validate` | Maintain → Health Check (button) |
| `remove-legacy-menu` | Auto-run on Mackes install; never user-facing |
| `menu-folder` | Auto-run on Mackes install (menu_integration.py) |
| `polybar` | Shell → Polybar (preset picker) |
| `plank` | Shell → Plank (preset picker) |
| `launcher` | Shell → Rofi Launcher (preset picker) |
| `menu-chrome` | Becomes one of the Theme presets in Look & Feel → Appearance |
| `disable-xfce-panel` | Shell → Panel Visibility (toggle) |
| `restore-xfce-panel` | Shell → Panel Visibility (toggle) |
| `install-fonts` | Maintain → Dependencies (auto-detected as missing); Appearance → Fonts also has a "Install missing fonts" button |
| `install-themes` | Same pattern — Maintain → Dependencies + Appearance → Theme has install button |
| `themes-fonts` | Folded into both Maintain → Dependencies and the wizard |
| `theme --theme mac-dark` etc. | Look & Feel → Appearance → Theme picker dropdown |
| `terminal-font --profile *` | Look & Feel → Appearance → Fonts (monospace picker) |
| `qnm` | Auto-run on install if profile enables QNM; Network → Quick Network Mesh has manual install button |
| `qnm-gui` | Network → Quick Network Mesh → "Open QNM GUI" button |
| `firewall-qnm` | Network → Firewall (auto-applied when QNM enabled) |
| `list-rollback` | Maintain → Snapshots (the panel IS the list) |
| `web` | Deleted (Q15) |
| `open-log` | Maintain → Logs (panel) + Dashboard quick action |
| `status` | Dashboard (status strip) — no separate panel needed |

Every action either becomes a panel control, a quick action on the Dashboard, or runs automatically during install/provision.

---

## 3. UX moves

### 3.1 Entry points

```
v2.2 had 5+:
  install-unified.sh          ──┐
  scripts/install.sh           ──┤
  scripts/install-unified.sh   ──┤
  scripts/install-gui.sh       ──┤    →  All five replaced by `mackes`
  START-HERE-*.desktop         ──┤        which routes based on state.
  native-workbench/...         ──┘
  web-workbench/...            ────→  Deleted.
```

### 3.2 The "Apply Black Droid Menu Chrome" duplication

In v2.2 this action appeared in TWO sections (Desktop Shell + Fonts and Themes). In Mackes it's a Theme preset called "Black Droid Chrome" inside Look & Feel → Appearance → Theme. The duplication is gone because there's only one place themes live.

### 3.3 The five terminal-font buttons

v2.2 has five separate buttons: Terminus, JetBrains Mono, Iosevka, DejaVu Sans Mono, Source Code Pro. Mackes has **one** monospace font picker in Appearance → Fonts that lists all installed monospace fonts (including those five) and applies via xfconf. Adding new fonts = install the font package; the picker discovers them automatically.

### 3.4 The Dashboard

v2.2 had no dashboard. The "Dashboard" sidebar entry showed five suggested actions. Mackes' Dashboard is a real live status surface with drift detection, hardware summary, last snapshot, recent actions, and quick actions (§4 of the spec).

### 3.5 First-run

v2.2 first-run = open Workbench, find "Install Everything" button, click it. Mackes first-run = 10-screen wizard with live preview, environment scan, preset selection, hardware/network/snapshot setup, and a review diff before apply (§5 of the spec).

---

## 4. Behavioral changes worth flagging

1. **Immediate apply** (Q9). Toggling anything in Mackes writes through instantly. v2.2's "click button → run action" model is gone. Snapshot before risky changes.

2. **Snapshots are manual** (Q10). Mackes does **not** auto-snapshot before changes. The Dashboard reminds the user to snapshot when there's been no snapshot recently and significant changes are pending. This is a deliberate "trust the user" choice — set it differently if it bites.

3. **Curated presets only** (Q7). Daily Workbench changes do **not** modify the active preset file. Drift is informational. If you want a customization to persist across reinstalls, edit the preset YAML or accept that re-running the wizard resets it.

4. **xfce4-settings stays installed** (Q19). It's hidden from menus but still functional. `xfce4-settings-manager` from a terminal still works. This is a safety net while Mackes proves itself; not a permanent design.

5. **No CLI** (Q14). Scripting/headless provisioning is out of scope for v0.1. If demand emerges, a `mackesctl` can be added later — but the GUI is the only first-class interface.

6. **No web admin** (Q15). Headless and remote use are out of scope for v0.1. Use SSH + X-forwarding or a remote VNC session if you must.

---

## 5. Practical migration path for an existing v2.2 install

For a machine running xfce11-unified v2.2 today, when Mackes 0.1 ships:

1. `dnf remove xfce11-unified` (if installed as RPM) or run the v2.2 `scripts/uninstall.sh`. This removes v2.2's menu entries, launcher, web admin, autostart entries. Configs in `~/.config/xfce4/`, `~/.config/polybar/`, `~/.config/plank/`, `~/.config/rofi/` stay intact.
2. `curl https://github.com/mattmacke/mackes-shell/releases/latest/download/install.sh | bash`
3. Wizard opens. The Environment Scan screen detects the existing Polybar/Plank/Rofi configs and offers to:
   - Adopt the closest matching shipped preset (and stash your custom configs to `~/.local/share/mackes-shell/legacy-v2.2/`)
   - Or skip the wizard and keep your existing configs (Mackes drops you into the Dashboard; drift will show against whatever preset you eventually pick)
4. Done.

QNM survives the migration untouched (it's the same daemon and CLI; only the GUI launcher moves into the Mackes Network tab).
