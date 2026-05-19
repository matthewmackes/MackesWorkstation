# Mackes XFCE Workstation

**A unified XFCE shell for Fedora — single 40 px bottom taskbar,
focused-app hero, mesh-aware status cluster, plus a peer-to-peer
mesh fabric that connects every one of your machines.**

Replaces `xfce4-panel`, `xfdesktop`, `xfwm4`, `xfce4-settings-manager`,
and the legacy KDE Connect tray indicator with a single Rust-native
panel (`mackes-panel`) and a Python sidebar (`mackes`). Underneath
it's a standard XFCE session with **i3 as the only window manager**
(per the 1.0.8 lock), LightDM for login, Plymouth for boot, all
rebranded to a consistent PatternFly v6 visual language with Carbon
icons + Red Hat Display/Text/Mono fonts. Adds peer-to-peer filesystem
sharing, clipboard manager with mesh history, notifications,
media-service discovery, KDE Connect via DBus, and identity-based SSH
across up to 16 mesh peers, anywhere on the internet. Single binary,
headless mode for fileservers.

## Smoke test — fresh checkout

```bash
git clone https://github.com/matthewmackes/MAP2-RELEASES.git mackes-shell
cd mackes-shell

# Build the panel (Rust) + workspace tests
cargo build --release --workspace
cargo test --workspace

# Run the Python smoke harness
make test-nodeps        # quick — no pytest dep
python3 -m pytest tests/  # full — needs pytest

# Build the RPM
make rpm                # produces dist/ + rpmbuild/RPMS/x86_64/

# Verify the panel binary boots under Xvfb
PANEL_BIN=target/release/mackes-panel install-helpers/bench-panel.sh
```

The CI workflow at `.github/workflows/ci.yml` runs the same
sequence on every push.

## Panel CLI

```bash
# Open the Workbench focused on the named panel slug (1.0.8 lock)
mackes --focus mesh_join
mackes --focus dashboard

# Update Mackes via dnf (1.1.0 — same path the watermark left-click
# and the right-click admin menu use)
mackes update                  # interactive
mackes update --yes --refresh  # force-refresh metadata + auto-confirm
mackes update --check-only     # don't apply, just report

# Open the notification drawer
mackes --drawer

# Print version
mackes --version
```

## `mackesd` (mesh control plane) CLI

```bash
mackesd migrate              # apply SQLite migrations
mackesd status               # store + migration count
mackesd healthz              # JSON: leader/applied_revision/nodes/health
mackesd generate-passcode    # fresh 16-char URL-safe mesh passcode
mackesd rotate-passcode      # alias — emits a new passcode for rotation
mackesd audit-verify         # walk the event hash chain
mackesd peers-why <node-id>  # explain a peer's expected adjacencies
mackesd apply --dry-run      # validate desired state without mutating
```

## Window manager

i3 is the only WM. `bin/mackes-wm`:

```bash
mackes-wm status   # print live WM name (always "i3" on 1.0.8+)
mackes-wm reset    # rebuild ~/.config/i3/config from the shipped default
```

Default keybindings ship at
`/usr/share/mackes-shell/i3/config.d/mackes-defaults.conf` and
include Phase 6.4 hotkeys (Mod+Q kill, Mod+W close, Mod+L lock,
Mod+V clipboard, Mod+E Thunar) plus Mod+Space for the apple menu
and Super+M for the notification drawer. User overrides at
`~/.config/i3/config.d/mackes-overrides.conf` load lexicographically
after our defaults. See `docs/help/keyboard-shortcuts.md` for the
full catalog.

## Architecture at a glance

- **`mackes-panel`** — Rust binary, the bottom taskbar + wallpaper
  layer. Modules: `admin_menu`, `clipboard_manager`, `hero`,
  `i3_cluster`, `icon_mapper`, `logout_dialog`, `network_manager`,
  `start_menu`, `status_cluster`, `top_bar`, `watermark`,
  `windows`, `dock`, `mesh_module`, plus the wallpaper desktop layer.
- **`mackes`** — Python entry point. Workbench (`mackes/workbench/`)
  + drawer (`mackes/drawer.py`) + birthright (`mackes/birthright.py`)
  + CLI (`mackes/headless/cli.py`).
- **`mackesd`** — Rust mesh control-plane binary + library. Modules:
  `audit`, `health`, `identity`, `metrics`, `passcode`, `policy`,
  `revisions`, `secrets`, `store`, `telemetry`, `topology`,
  `validation`. Per-peer systemd unit; one is leader via the
  QNM-Shared lockfile.
- **`mackes-clipboard-daemon`** — XA_CLIPBOARD watcher; appends to
  `~/.cache/mackes/clipboard.json`; mesh-replicated whole-file via
  QNM-Shared. Auto-enabled via the shipped `90-mackes.preset`.

Detailed architecture lives at `docs/design/v12.0-enterprise-mesh.md`
(backend), `docs/design/v3.0.0-mackes-xfce-workstation.md` (UI lock),
`docs/design/v12-connectivity-scope.md` (mesh networking), and
`docs/design/wayland-readiness.md` (port roadmap).

---

## Install on Fedora

One command, copy-and-paste:

```sh
curl -fsSL https://raw.githubusercontent.com/matthewmackes/MAP2-RELEASES/main/install.sh | bash
```

That bootstrap:

1. Detects your Fedora release.
2. Queries GitHub Releases for the latest Mackes Shell tag.
3. Downloads `mackes-shell-<version>-1.fc<release>.noarch.rpm`.
4. Installs it via `sudo dnf install` (you'll be prompted for your sudo password — the script itself is **not** piped through `sudo`).
5. Launches `mackes`, which routes into the first-run wizard.

> **Headless / SSH session?** Same one-liner. Mackes auto-detects no
> display and runs `mackes init` instead — a stdin-prompts wizard that
> brings the fileserver onto the mesh.

### Alternative install paths

```sh
# Add the Mackes dnf repo and install via dnf (preferred for managed fleets)
sudo dnf config-manager --add-repo \
    https://matthewmackes.github.io/MAP2-RELEASES/data/dnf/mackes-shell.repo
sudo dnf install mackes-shell
```

```sh
# Download the RPM directly from the Releases page and install offline
wget https://github.com/matthewmackes/MAP2-RELEASES/releases/latest/download/mackes-xfce-workstation-1.0.6-1.fc44.x86_64.rpm
sudo dnf install ./mackes-xfce-workstation-1.0.6-1.fc44.x86_64.rpm
```

---

## Build from source

The repo is two co-resident projects: a **Python** workbench
(`mackes/`) and a **Rust** panel (`crates/`). One `make` command builds
both into a single RPM.

```sh
# Toolchain (Fedora 44+):
sudo dnf install python3 python3-build python3-gobject \
    gtk3 cairo-devel python3-pytest \
    rust cargo rust-toolchain \
    rpm-build rpmdevtools \
    appstream-util appstream

# One-shot build:
make rpm
# → rpmbuild/RPMS/x86_64/mackes-xfce-workstation-<version>-1.fc<rel>.x86_64.rpm

# Tighter dev loops:
make rust          # cargo build --release --workspace
make rust-check    # cargo fmt --check && cargo clippy -D warnings && cargo test
make test          # pytest tests/    (needs pytest installed)
make test-nodeps   # in-tree smoke harness, no pytest needed
make smoke         # walk mackes/ and import every module
```

The Rust panel runs standalone — useful for iterating on chrome
without rebuilding the RPM each time:

```sh
cargo run --release -p mackes-panel
# CTRL-C to stop. Reads ~/.config/mackes-panel/panel.toml on launch and
# hot-reloads it on save (gio FileMonitor / inotify).
```

The Python workbench:

```sh
python3 -P -m mackes              # GUI workbench
python3 -P -m mackes --drawer     # notification drawer
python3 -P -m mackes --about      # About Mackes window
python3 -P -m mackes status       # headless status (no DISPLAY needed)
```

> `python3 -P` (Python 3.11+) prevents the cwd from being prepended to
> `sys.path`. Without it, running `python3 -m mackes` from the repo
> root would silently import the in-tree copy instead of the
> installed package — a footgun we hit hard enough to enshrine in the
> RPM-installed `/usr/bin/mackes` wrapper.

---

## What's in 1.0.x — "Mackes XFCE Workstation"

1.0.0 pivoted from the 2.x-era polybar/plank/rofi shell-stack to a
standard XFCE base with a Rust-native panel and dock layered on top.
The Mackes UI runs on the **PatternFly v6** design tokens (adaptive
dark surfaces, Red Hat Display / Text / Mono typography, per-preset
accent). 1.0.6 fixed the first-boot visual issues; 1.0.7 brings the
dock to feature parity with Plank and adds optional i3 as a tiling
alternative to xfwm4.

On top of the standard XFCE base, Mackes adds the **mesh fabric**:

| Layer | What it does | Backed by |
|---|---|---|
| **Mesh VPN** | Routes packets between any two peers regardless of physical network | self-hosted Headscale + Tailscale-bootstrap rendezvous + WireGuard |
| **Mesh filesystem** | Live SSHFS mounts under `~/QNM-Mesh/<peer>/` — every peer sees every other peer's files | `qnmd mesh-fs` |
| **Mesh sync** | Distributed clipboard / notifications / Object Store (themes, snapshots, presets, file-drop) | `mesh_sync` substrate + NATS-equivalent API |
| **Mesh services** | Auto-discovered Jellyfin / Airsonic / Plex / Sonarr / Grafana / Home Assistant / 30+ more across every peer | port-prober + 5-layer surface (cheatsheet + Hub panel + Caddy proxy + native clients + mDNS bridge) |
| **Mesh SSH** | `ssh peer.mesh` works zero-config; identity-based ACLs via Headscale Tailscale-SSH | auto-distributed ed25519 keys + Tailscale-SSH |
| **Mesh in Thunar** | `mesh:///` URI scheme → real FUSE-backed GVFS surface for Peers, Clipboard, Notifications, Object Store | `gvfsd-mesh` + Tumbler thumbnailer |

Plus on every install:
- **Headless node mode** — `mackes init` runs as a pure stdin wizard on fileservers/NAS/VPS without a display; systemd-managed lifecycle
- **PadOS GTK theme** + **Carbon icon theme** (Apache-2.0) shipped at `/usr/share/themes/` and `/usr/share/icons/`
- **Standard wallpaper** at `branding/standard-wallpaper.png` applied to desktop and LightDM greeter
- **OpenSSH** enabled by default on first install
- **In-Mackes user guide** at Help tab (19 markdown topics; same content available headless via `mackes help [topic]`)

Full feature breakdown: [`CHANGELOG.md`](CHANGELOG.md). Design lock-in:
[`docs/MACKES_SHELL_SPEC.md`](docs/MACKES_SHELL_SPEC.md).

---

## Five presets

`mackes init` (headless) or the GUI wizard's preset picker offers:

| Preset | Vibe | Default? |
|---|---|---|
| **`#!`** | CrunchBang reincarnation — black, monospace, sparse. Modern stack: alacritty / neovim / firefox / mpv / conky / menulibre. | yes (default) |
| **`Mackes`** | Warm-dark house style. Curated dev toolset: VS Code, Cursor, Claude Code CLI, Terminator, FileZilla, Remmina, Edge. | |
| **`Daylight`** | Cool yellow accent on Carbon Gray 100. Productivity stack: LibreOffice, Thunderbird, GIMP, Inkscape, Evince. | |
| **`Vanilla`** | Fedora XFCE defaults preserved. Mackes manages snapshots + repair only — never touches your theme, panel, or app set. | |
| **`Node`** | Headless mesh-only. Empty appearance + apps; mesh-VPN + SSHFS + sync + SSH enabled. Auto-selected when no display is present. | (headless only) |

Switch later via **Maintain → Reset to Preset** or `mackes preset apply <name>`.

---

## Workbench tabs (GUI)

- **Dashboard** — status dots (xfce4-panel/xfdesktop/xfsettingsd/xfconf/NetworkManager/sshd) · drift card · hardware summary · quick actions · recent activity
- **Look & Feel** — Appearance: theme (PadOS locked), icons (Carbon locked), fonts (IBM Plex), wallpaper
- **Devices** — Display · Keyboard · Mouse · Sound · Power
- **Network** — Wi-Fi/Ethernet · VPN · QNM · **Mesh VPN** · **Mesh SSH** · **Mesh Services** · Firewall
- **System** — Window Manager · Workspaces · Session & Startup · Notifications · Default Apps · Removable Media · Date & Time
- **Apps** — Install (curated set per preset) · Remove (combined Bloat list: GNOME-on-XFCE + LibreOffice + asunder/parole/pragha/xfburn/transmission-gtk/claws-mail/pidgin) · Installed (rpm -qa browser)
- **Maintain** — Snapshots · Drift · System Update · Fonts · Power · Resources · Health Check · Dependencies · Logs · Repair · Reset to Preset · Uninstall
- **Help** — 19 in-window markdown topics covering every feature

---

## CLI

```sh
# Setup
mackes init                       # first-run setup (headless or GUI-fallback)
mackes init --tailscale-authkey=tskey-auth-…  --enable-on-boot --yes  # cloud-init
mackes join '<mesh-join://link>'  # join an existing mesh

# Day to day
mackes status                     # current node state
mackes peers                      # mesh peer list (DataTable equivalent)
mackes shares                     # SSHFS in/out
mackes ssh <peer>                 # open SSH to a mesh peer (TS-SSH preferred)
mackes notify <peer> "msg"        # send a notification across the mesh
mackes services list              # discovered HTTP services
mackes services launch <name>     # xdg-open the service URL

# Maintenance
mackes snapshot create [label]
mackes snapshot restore <name>
mackes maintain {repair|health|logs|reset}
mackes preset {list|apply <name>|show <name>|diff}
mackes apps {install|remove|list|catalog}

# Help
mackes help [topic]               # plain-text help; see `mackes help` for topic list

mackes uninstall                  # complete removal + final snapshot tarball
```

Headless detection is automatic (`$DISPLAY` + `$WAYLAND_DISPLAY` + logind
graphical session). Force either path with `--gui` or `--headless`.

---

## Recovery

Mackes ships a TTY-driven recovery shell for when the GUI won't come up.

```sh
sudo /usr/share/mackes-shell/install-helpers/install-recovery.sh
```

Installs `mackes-recovery.target` (systemd), a `Mackes Recovery` GRUB
submenu entry, and `/usr/local/bin/mackes-recover` (TTY snapshot picker).

---

## Develop

```sh
git clone git@github.com:matthewmackes/MAP2-RELEASES.git
cd MAP2-RELEASES
make install-deps             # one-time: pulls Fedora dev deps
python3 -m mackes --wizard    # run from source (GUI)
python3 -m mackes status      # run from source (headless)
make smoke                    # import-walk check
make test                     # pytest
make rpm                      # build the RPM
make iso                      # build a Fedora-derivative live ISO
```

---

## Source / Spec

- **Authoritative spec:** [`docs/MACKES_SHELL_SPEC.md`](docs/MACKES_SHELL_SPEC.md) — locked design decisions across 15 implementation sections
- **Help docs:** [`docs/help/`](docs/help/) — 19 user-facing topics rendered live by the Mackes Help tab
- **Changelog:** [`CHANGELOG.md`](CHANGELOG.md)
- **Issues:** <https://github.com/matthewmackes/MAP2-RELEASES/issues>

GPL-3.0. © 2026 Matthew Mackes.
