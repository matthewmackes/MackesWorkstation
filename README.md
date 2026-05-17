# Mackes Shell

**A single control panel for XFCE on Fedora — plus a mesh fabric that
connects every one of your machines.**

Replaces `xfce4-settings-manager` for daily work. Adds peer-to-peer
filesystem sharing, clipboard, notifications, media-service discovery,
and identity-based SSH across up to 16 mesh peers, anywhere on the
internet. Carbon Design System UI, single binary, headless mode for
fileservers.

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
wget https://github.com/matthewmackes/MAP2-RELEASES/releases/latest/download/mackes-shell-1.0.0-1.fc44.noarch.rpm
sudo dnf install ./mackes-shell-1.0.0-1.fc44.noarch.rpm
```

---

## What's in 1.0.0 — "XFCE Provisioner"

Mackes Shell 1.0.0 ships a complete pivot from the v0.2 polybar/plank/rofi
shell-stack to a standard XFCE shell (Whisker Menu + Docklike Taskbar +
xfce4-panel + xfdesktop), with the Mackes window itself running on the
**IBM Carbon Design System** (Gray 100 palette, IBM Plex typography,
per-preset accent).

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
