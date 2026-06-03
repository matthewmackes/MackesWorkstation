# MDE-Retro

> ## ⚠️ Warning, Disclaimer & Mission
>
> **Mackes Workstation** is an educational, experimental, open-source workstation platform designed to explore encrypted mesh networking, private service streaming, automated replication, distributed services, and resilient private-network infrastructure. Its mission is to provide a learning and research environment for understanding how secure workstation nodes, peer-to-peer connectivity, service availability, and controlled replication can be built using open-source technologies.
>
> It is provided for **education, experimentation, demonstration, and personal research only** — not a commercial product, managed service, enterprise support platform, security appliance, backup product, or guaranteed high-availability system. It may modify network behavior, establish encrypted mesh connections, replicate data, expose private services, change service routing, and alter how systems communicate inside a private network. Improper installation, configuration, or use may result in data loss, service interruption, unauthorized access, network instability, privacy exposure, security weaknesses, compliance issues, or unintended replication of sensitive information.
>
> Provided **“as is” and “as available,” without warranties of any kind**, express or implied — no guarantee of security, reliability, availability, performance, legal compliance, fitness for a particular purpose, compatibility, data integrity, or suitability for production use; and no support, SLA, recovery assistance, incident response, or warranty service unless separately agreed in writing. By installing, using, modifying, distributing, or operating it you **accept full responsibility for all risks and outcomes**, and you are responsible for backups, securing your systems, and complying with all applicable laws. **Do not** use it on systems/networks/data you do not own or have explicit permission to manage, or in production / regulated / safety-critical / medical / financial / emergency / public-infrastructure / high-availability environments without independent security, legal, operational, and backup review. Third-party open-source components retain their original licenses and terms. To the maximum extent permitted by law, the authors, maintainers, contributors, and distributors are **not liable** for any damages arising from its use.
>
> **Use at your own risk. If you do not understand the risks, do not install or use it.** The full statement is in [`DISCLAIMER.md`](DISCLAIMER.md).

> **⚠️ This page documents the LEGACY script-based sway desktop.** The live
> product is the native **Rust shell** in [`rust/`](rust/), which runs on
> **labwc** (not sway) and defaults to an **IBM Carbon** theme with Windows 2000
> selectable. Start at [`rust/README.md`](rust/README.md); see
> [`.claude/CLAUDE.md`](.claude/CLAUDE.md) §1 and [`docs/COMPLIANCE.md`](docs/COMPLIANCE.md)
> for the current architecture. The sway/Waybar/wofi setup below is kept for
> history and the `install.sh` legacy path only.

A **Windows 2000 / 95-style desktop for [Sway](https://swaywm.org/)** (Wayland) on
Fedora (legacy path). It turns a tiling compositor into a familiar classic-Windows
environment: silver 3D window frames, navy title bars, a gray taskbar with a Start
button and tray clock, floating overlapping windows, click-to-focus, and the
keyboard muscle memory you already have (`Alt+F4`, `Alt+Tab`, `Ctrl+Esc`, `Win+R`, `Win+E`).

It is the *retro theme layer* of a larger personal desktop ("MDE"); everything
here is self-contained and driven by plain Sway + Waybar + wofi config plus a few
small scripts — no patched compositor, no exotic dependencies.

![theme: Windows 2000](https://img.shields.io/badge/theme-Windows%202000-0a246a)
![wm: sway](https://img.shields.io/badge/wm-sway-3a6ea5)
![distro: Fedora%2044](https://img.shields.io/badge/distro-Fedora%2044-d4d0c8)

---

## What you get

| Piece            | How it's done                                                              |
| ---------------- | -------------------------------------------------------------------------- |
| **Desktop**      | Solid Win2000 blue `#3a6ea5`                                               |
| **Window frames**| Silver `#d4d0c8` 3D borders, navy `#0a246a` title bars, white title text   |
| **Behavior**     | Windows float & overlap, click-to-focus (no focus-follows-mouse)           |
| **Taskbar**      | Waybar: ⊞ Start, window-button taskbar, tray, volume, sunken clock         |
| **Start menu**   | `wofi`-based, Win2000-styled (left-click = menu, right-click = System Tools)|
| **Run dialog**   | `Win+R` → wofi run mode                                                     |
| **Control Panel**| Maps installed Fedora tools to Win2000 Control Panel names                  |
| **Icons**        | `Win2k` GTK icon theme, inheriting Chicago95 for coverage                   |
| **Cursors**      | `Chicago95_Standard_Cursors` (arrow, animated hourglass, I-beam, …)         |
| **Sounds**       | Chicago95 sound theme; classic login chime on session start                |
| **Font**         | "Tahoma" everywhere, aliased to Droid Sans where Tahoma isn't installed    |

If Waybar isn't installed the config automatically falls back to a themed
built-in `swaybar`, so the desktop always has a working taskbar.

---

## Install

```sh
git clone https://github.com/<you>/MDE-Retro.git
cd MDE-Retro
./install.sh --assets        # deploy configs + download/install the visual assets
```

`install.sh` **symlinks** the trees in `home/.config/` into your real `~/.config`
(backing up anything in the way to `*.bak.<timestamp>`). Use
`MDE_RETRO_COPY=1 ./install.sh` to copy instead of symlink.

Runtime packages (install what you're missing):

```sh
sudo dnf install sway waybar wofi foot wmenu NetworkManager-applet grim
```

Then log into a Sway session and reload with **`Win+Shift+C`**.

### Optional: a "Windows 2000" login entry

```sh
sudo cp ~/.config/sway/resources/windows2000.desktop /usr/share/wayland-sessions/
```

(The stock "Sway" greeter entry already launches this same config.)

---

## Keyboard cheat sheet

| Action            | Key                  |
| ----------------- | -------------------- |
| Start menu        | `Ctrl+Esc` (or `Win+D`) |
| System Tools menu | `Ctrl+Shift+Esc`     |
| Run               | `Win+R`              |
| Close window      | `Alt+F4`             |
| Switch windows    | `Alt+Tab` / `Alt+Shift+Tab` |
| My Computer       | `Win+E`              |
| Terminal          | `Win+Enter`          |
| Maximize / restore| `Win+Up`             |
| Switch desktops   | `Ctrl+Alt+Left/Right`|
| Move window       | `Win+Shift+Arrows`   |
| Resize mode       | `Win+S`              |
| Log out           | `Ctrl+Alt+Delete`    |

---

## Layout

```
home/.config/
├── sway/
│   ├── config                 # the desktop: theme, behavior, keybindings, taskbar
│   ├── config.d/              # drop-in overrides (e.g. MDE Workbench output)
│   ├── scripts/               # Start menu, Control Panel, taskbar, power, brightness
│   └── resources/             # session .desktop + cached Win2k icon tarball
├── waybar/{config.jsonc,style.css}   # themed taskbar
├── wofi/{config,style.css}           # themed Start menu / Run dialog
├── gtk-3.0,gtk-4.0/settings.ini      # icon/cursor/GTK theme selection
└── fontconfig/fonts.conf             # Tahoma -> humanist sans alias
assets/
├── install-assets.sh          # orchestrator: runs the installers below
└── install-chicago95.sh       # icons/cursors/sounds/GTK theme from grassmunk/Chicago95
```

The large asset sets (Chicago95 is ~76 MB of icons) are **not** committed — the
installers fetch them from upstream so their licenses travel with them. See
[`LICENSE`](LICENSE) for the asset-licensing note.

---

## License

Configs and scripts: MIT. Bundled-by-download visual assets keep their upstream
licenses (Chicago95 GPL-3.0, Win2k icon theme per its store page). See
[`LICENSE`](LICENSE).
