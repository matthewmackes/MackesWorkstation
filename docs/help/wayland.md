# Wayland support

Mackes Shell is **X11-only** by design. The Mackes XFCE Workstation
stack assumes XFCE + i3 + xfsettingsd, all of which require an X11
session. Wayland is a work-in-progress port (see
`docs/design/wayland-readiness.md` for the per-surface audit).

## Status line

If you are running on Wayland today:

- **GNOME on Wayland is not supported.** The dock's running-windows
  tasklist depends on `wmctrl -lp` + `xprop -id WM_CLASS` (X11 only)
  and the EWMH `_NET_CLIENT_LIST` model. GNOME-shell on Wayland
  exposes no equivalent protocol — `zwlr_foreign_toplevel_manager_v1`
  is the wlroots replacement but is not implemented by GNOME-shell.
  Picking GNOME-shell on Wayland will leave the dock with pinned
  launchers only and an empty tasklist.
- **sway, Hyprland, river, and other wlroots compositors** will work
  once the layer-shell port (worklist items W1–W5) lands. Until then,
  Mackes will not start the panel or wallpaper on these compositors.

## What does work on Wayland today

- `mackes-wm probe-wayland` reports `XDG_SESSION_TYPE`,
  `WAYLAND_DISPLAY`, `DISPLAY`, and `wayland-info` availability.
- `mackes-wm session-pick` lists every installed
  `/usr/share/wayland-sessions/*.desktop` so you can switch from the
  greeter.

## Switching to X11

If you landed here because the panel or dock looks broken on Wayland:

```bash
# List sessions
mackes-wm session-pick

# Then log out and pick "Xfce Session" (or any *xsession.desktop) from
# the greeter's session dropdown.
```

## See also

- [Keyboard shortcuts](keybindings.md) — every i3 binding is X11-only
- [Troubleshooting](troubleshooting.md) — panel / dock recovery
- `docs/design/wayland-readiness.md` — full per-surface compatibility
  audit (developer doc)
