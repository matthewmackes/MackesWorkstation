# Mackes Shell

A control panel for XFCE on Fedora. Replaces `xfce4-settings-manager` as
the daily interface and adds the surfaces XFCE doesn't ship: a polybar
editor, drift detection, four opinionated presets, a snapshot/recovery
system, and a MaintenanceKit of small purpose-built tools.

Benchmarked against [CrunchBang Linux](https://en.wikipedia.org/wiki/CrunchBang_Linux)
— restraint with a ritual.

## Install

On Fedora (XFCE session), one curl-pipe-bash bootstrap:

```sh
curl -L https://github.com/matthewmackes/MAP2-RELEASES/releases/latest/download/install.sh | bash
```

The first launch opens the wizard (three acts: hello → pick a preset →
narrated apply). Subsequent launches open the workbench.

Force the wizard later with `mackes --wizard`. Re-apply a preset headlessly
with `python3 -m mackes.cli_apply --preset hashbang`.

## Presets

| | Vibe | Default? |
|---|---|---|
| **`#!`** | CrunchBang reincarnation — black, monospace, sparse. Modern stack (alacritty / neovim / firefox / mpv). | yes |
| **Mackes** | Warm-dark, the house style. Curated dev toolset (VS Code, Cursor, Claude Code, terminator). | |
| **Daylight** | Cool-light productivity (LibreOffice, Thunderbird, GIMP, Inkscape). | |
| **Vanilla** | Fedora XFCE defaults; Mackes manages snapshots only, doesn't touch theme/apps/shell. | |

Switch with **Maintain → Reset to Preset** or re-run the wizard.

## The workbench, in eleven sections

- **Look & Feel** — Appearance (theme/icon/cursor/font/wallpaper)
- **Shell** — Polybar Editor (theme picker + 3-zone DnD modules + live
  apply), Plank dock, Rofi launcher, XFCE Panel
- **Devices** — Display, Keyboard, Mouse, Sound, Power
- **Network** — Wi-Fi/Ethernet, VPN (.ovpn / .conf importer), QNM, Firewall
- **System** — Window Manager, Workspaces, Session/Startup, Notifications,
  Default Apps, Removable Media, Date & Time
- **Apps** — Install, Remove, Installed
- **Maintain** — Snapshots, **Drift** (per-key revert/adopt), System
  Update, Fonts, Power, Resources, Health Check, Dependencies, Logs,
  Repair, Reset to Preset, Uninstall

## Polybar Editor

The polybar surface is the deepest customization point. The editor:

- Picks one of 21 vendored adi1090x families (`simple` + `bitmap` variants)
- Lets you adjust position / height / corner radius live
- Lets you drag modules between left / center / right zones with full
  cross-zone DnD
- Add module → popover lists every module the active family defines
- Save the current config as a named profile under
  `~/.config/mackes-shell/shell-profiles/polybar/`
- Live debounced apply — polybar relaunches ~300 ms after the last edit

Generated configs are self-contained (no `include-file` references) so
they're portable.

## Recovery

Mackes ships a TTY-driven recovery shell for when the GUI won't come up.

```sh
sudo /usr/share/mackes-shell/install-helpers/install-recovery.sh
```

Installs three things:

1. `mackes-recovery.target` (systemd) — multi-user + network, no graphical
2. `40_mackes_recovery` (grub.d) — adds a "Mackes Recovery" GRUB submenu
3. `/usr/local/bin/mackes-recover` — TTY snapshot picker

Boot the GRUB entry, log in to the tty, run `mackes-recover` to restore a
previous snapshot.

## Build an ISO

```sh
sudo dnf install lorax pykickstart
make iso
```

Builds a Fedora-derivative live ISO with mackes-shell baked in. See
`packaging/iso/README.md` for the kickstart details.

## Layout

```
mackes/                 — Python package (workbench panels, wizard pages,
                          state, presets, polybar generator + catalog,
                          recover CLI)
data/
  css/                  — design system (base + per-preset accents)
  presets/              — 4 preset YAMLs
  shell-profiles/       — polybar / plank / rofi configurations,
                          + vendored adi1090x upstream tree
  wallpapers/           — per-preset wallpapers
  dnf/                  — mackes-shell.repo (gh-pages-served)
  systemd/, grub/       — recovery target + GRUB submenu source
install-helpers/        — root-needed scripts (recovery, dnf-repo, menus)
packaging/
  fedora/               — RPM spec
  iso/                  — kickstart + build docs
tests/                  — pytest suite (also runnable without pytest:
                          python3 tests/_run_without_pytest.py)
```

## Develop

```sh
git clone https://github.com/matthewmackes/MAP2-RELEASES.git
cd MAP2-RELEASES
make install-deps             # one-time
python3 -m mackes --wizard    # run from source
make smoke                    # import-walk
make test-nodeps              # tests without pytest
make test                     # tests with pytest
make rpm                      # build a .rpm
make iso                      # build a live ISO (Fedora host)
```

## License

GPL-3.0 (matches the vendored adi1090x/polybar-themes). See `LICENSE`.
