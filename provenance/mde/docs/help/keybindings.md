# Keyboard shortcuts

**Mackes Desktop Environment (MDE)** ships its own keybindings under
sway (replacing the XFCE accelerators 1.x carried) + a small set of
MDE-specific accelerators when the workbench window is focused.

## MDE workbench window

| Shortcut | Action |
|---|---|
| `Ctrl+,` | Open Look & Feel → Appearance |
| `Ctrl+Shift+S` | Create a new snapshot (Maintain → Snapshots) |
| `Ctrl+Shift+R` | Reapply active preset (Maintain → Repair) |
| `Ctrl+Shift+H` | Open Help (this guide) |
| `Ctrl+Shift+L` | Open Logs (Maintain → Logs) |
| `Ctrl+W` | Close window |
| `F5` | Refresh current panel |
| `Esc` | Close popover / dialog |

## Mesh in mde-files

When viewing the mde-files panel:

| Shortcut | Action |
|---|---|
| `Ctrl+F` | Search inside the current view |
| `Ctrl+Shift+F` | Mesh-wide search across every peer |
| `Ctrl+D` | Send selected files to a peer (opens destination picker) |
| `Tab` / `Shift+Tab` | Cycle keyboard focus between toolbar / sidebar / list |
| `Space` | Toggle the focused row in the multi-select set |
| `Esc` | Clear selection / dismiss the floating context menu |
| `Delete` | Delete selected files (with confirm) |

## sway-managed shortcuts

Configured in **Devices → Keyboard → Keyboard Shortcuts** (writes
to the `keybinds.*` keys via `mde_settings_bridge`; the matching
Rust applier emits a fresh `~/.config/sway/config.d/mde-bindings
.conf` and reloads sway IPC). Common defaults port over from
the v1.x i3 stack with the same modifier mappings:

| Shortcut | Action |
|---|---|
| `Super` | Open MDE Start menu |
| `Super+E` | Open mde-files |
| `Super+T` | Open foot (terminal) |
| `Super+L` | Lock screen (swaylock) |
| `Super+D` | Show desktop (focus drawer) |
| `Super+1..9` | Switch to workspace N |
| `Super+Q` | Close focused window |
| `Super+F` | Toggle fullscreen on focused window |
| `Super+Space` | Toggle floating on focused window |
| `Super+J/K/L/;` | Focus window left / down / up / right |
| `Print` | Screenshot (full screen) via `grim` |
| `Shift+Print` | Screenshot (region) via `grim` + `slurp` |

## Mesh SSH

`mde ssh <peer>` from any terminal opens a session to the named
peer. Tab-completion of peer names if you've enabled the MDE shell
completions (`source /usr/share/bash-completion/completions/mde`).
