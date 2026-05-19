# Keyboard shortcuts

Every key binding that Mackes XFCE Workstation defines or respects, in
one place. Surfaced inside the app from **Apple menu → Help → Keyboard
shortcuts** (1.0.7+) and shipped to `/usr/share/mackes-shell/help/` so
the headless `mackes help keyboard-shortcuts` flow works too.

Bindings are grouped by the layer that handles them — the window
manager, the panel, or the workbench. Mackes never overrides a binding
silently: if a shortcut would conflict with an existing user binding,
the wizard offers a "Keep mine" / "Use Mackes' / "Backup mine and use
Mackes'" choice and writes the answer to `panel.toml:[keybindings]`.

## Window manager (xfwm4 or i3)

These keys are owned by the active WM and survive a Mackes uninstall.

| Binding             | Action                                            | xfwm4 | i3   |
|---------------------|---------------------------------------------------|-------|------|
| `Super`             | Open the Apple menu (Mackes button)               | ✓     | ✓    |
| `Super + Space`     | Open the Apple menu (same as `Super`)             | ✓     | ✓    |
| `Super + L`         | Lock screen (`loginctl lock-session`)             | ✓     | ✓    |
| `Super + M`         | Toggle the notification drawer                    | ✓     | ✓    |
| `Super + E`         | Open Thunar file manager                          | ✓     | ✓    |
| `Super + Return`    | Open `xfce4-terminal` (drop-in i3 default)        | —     | ✓    |
| `Super + Shift + R` | Reload i3 config (`i3-msg reload`)                | —     | ✓    |
| `Super + Shift + E` | Exit i3 (returns to LightDM)                      | —     | ✓    |
| `Alt + F4`          | Close focused window                              | ✓     | —    |
| `Alt + F10`         | Toggle maximize on focused window                 | ✓     | —    |
| `Alt + F9`          | Minimize focused window                           | ✓     | —    |
| `Alt + Tab`         | Cycle visible windows                             | ✓     | ✓    |
| `Super + Tab`       | App-aware window switcher (Phase 6.1 — pending)   | —     | —    |
| `F3`                | Exposé grid (Phase 6.2 — pending)                 | —     | —    |
| `Super + Q`         | Close focused window (Phase 6.4 — pending)        | —     | ✓    |
| `Super + W`         | Close focused window (xfwm4 alt — pending)        | —     | —    |

Layouts shipped with i3 (defaults — see `/usr/share/mackes-shell/i3/config`):

| Binding              | i3 action                                     |
|----------------------|-----------------------------------------------|
| `Super + 1` … `4`    | Switch to workspace 1–4                       |
| `Super + Shift + 1`…`4` | Move focused window to workspace 1–4       |
| `Super + H` / `V`    | Split horizontal / vertical                   |
| `Super + F`          | Toggle fullscreen on focused window           |
| `Super + R`          | Resize mode (Esc / Enter to exit)             |

Workspaces are usually collapsed to one (`workspace_count = 1`) under
xfwm4 per Phase 6.3 — workspaces 2-4 only appear when i3 is the active
WM.

## Panel (`mackes-panel`)

The Rust panel owns its own shortcuts via the portal-friendly path
under Phase 11.3; today they're driven by the WM bindings above. No
panel-local key handlers run yet — every panel interaction goes through
the menu, the dock, or the status cluster.

## Workbench (Python sidebar — `mackes`)

When the Workbench window has focus, these bindings work in addition
to the standard GTK ones (`Tab` / `Shift+Tab` to navigate focus,
`Space` to activate, `Escape` to cancel a dialog).

| Binding         | Action                                             |
|-----------------|----------------------------------------------------|
| `Ctrl + F`      | Focus the Workbench search box (1.1.0+)            |
| `Ctrl + L`      | Focus the sidebar (jump back to navigation)        |
| `Ctrl + W`      | Close the current Workbench window                 |
| `Ctrl + Q`      | Quit Mackes (closes every Mackes window)           |
| `F1`            | Open the Help tab for the currently visible panel  |
| `Escape`        | Close the active dialog / cancel the current edit  |

Within a settings panel:

| Binding                   | Action                                                  |
|---------------------------|---------------------------------------------------------|
| `Ctrl + Return` / `Enter` | Apply the panel's edits                                 |
| `Ctrl + Z`                | Revert the last unapplied edit (where supported)        |
| `Ctrl + .`                | Open the Drift card for this panel (Maintain → Drift)   |

## Drawer (`mackes --drawer`)

Drawer-specific bindings work while the drawer is the foreground
window. `Super + M` toggles the drawer from anywhere.

| Binding            | Action                                       |
|--------------------|----------------------------------------------|
| `Escape`           | Close the drawer                             |
| `Tab` / `Shift+Tab`| Cycle focus through the quick-toggle row     |
| `Ctrl + N`         | Jump to the Notifications section            |
| `Ctrl + M`         | Jump to the Mesh section                     |
| `Ctrl + F`         | Jump to the Fleet section                    |

## CLI flags that mirror keyboard actions

```sh
mackes                 # open the Workbench (no flags)
mackes --drawer        # open the drawer
mackes --drawer --drawer-focus mesh   # open + scroll to Mesh section
mackes --about         # open the About Mackes window
mackes --wizard        # force the first-run wizard
mackes-wm i3           # live-switch to i3
mackes-wm xfwm4        # live-switch back to xfwm4
mackes-wm status       # print the running WM
```

These are useful for binding from `xfce4-keyboard-settings` or any other
shortcut surface — every keyboard binding above can be reproduced by
spawning the matching CLI invocation.

## Customizing

`~/.config/mackes-panel/panel.toml` carries a `[keybindings]` section
(empty by default — every shortcut above comes from the system layer).
Adding a key here overrides the system binding for the active session:

```toml
[keybindings]
"drawer"      = "super+grave"     # was super+m
"apple_menu"  = "super+w"         # was super+space (and super)
"lock_screen" = "ctrl+alt+l"      # was super+l
```

Save the file — `mackes-panel` watches it via inotify and reapplies the
bindings within ~1 s. Backups of any system bindings Mackes overrides
live at `~/.config/mackes-panel/keybindings.backup.toml`.

## Accessibility

Every interactive panel widget exposes an AT-SPI name + description
(Phase 11.2 — landed in 1.0.7). Screen readers announce the status
cluster as "Mesh: 3 peers online" / "Battery: 87 percent" /
"Notifications: 1 unread" rather than just "button". Focus order
follows reading order (top-to-bottom, left-to-right) on every panel.

If a screen reader does NOT pick up a panel's content, please file an
issue: it's almost certainly a missing `set_accessible_name` call on
the offending widget.
