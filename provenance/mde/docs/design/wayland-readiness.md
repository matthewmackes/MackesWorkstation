# Wayland readiness — Mackes XFCE Workstation 1.0.x

**Status:** X11-only as of 1.0.7. This doc surveys every place the panel,
the workbench, and the helper binaries assume X11 so the port — when it
lands — has a starting map. Each row names the **surface**, the
**X11 thing we use today**, the **Wayland-side replacement**, and a
**rough effort estimate**.

The honest one-line read: under a typical wlroots compositor (Sway,
Hyprland, river) every panel-side need has a clean equivalent —
foreign-toplevel for the dock, layer-shell for the surfaces, idle-notify
for status. Under GNOME-shell-on-Wayland we'd need a shell extension
because GNOME ships no equivalent of `foreign-toplevel`. The realistic
target for a first Wayland release is **wlroots compositors only**.

## Inventory

| Surface | X11 today | Wayland-side replacement | Effort |
|---|---|---|---|
| **Top bar + dock surface positioning** | `WindowTypeHint::Dock` GDK hint | `wlr-layer-shell-v1` protocol — anchor TOP/BOTTOM, exclusive zone | ~300 LOC against `smithay-client-toolkit` |
| **Reserved screen space** | `_NET_WM_STRUT_PARTIAL` published via `xdotool search --name` + `xprop -id` (`crates/mackes-panel/src/strut.rs`) | Layer-shell `set_exclusive_zone()` — drops the strut module entirely | Replace strut.rs (-126 LOC) |
| **Open-window enumeration** (dock tasklist) | `wmctrl -lp` + `xprop -id WM_CLASS` (`crates/mackes-panel/src/windows.rs`) | `zwlr_foreign_toplevel_manager_v1` (sway/Hyprland/river) | ~250 LOC; loses windows on GNOME-shell |
| **Active window id** | `wmctrl` parse of `(active)` marker | `foreign-toplevel` `activated` state | Folded into above |
| **Window switching** (toggle, raise, minimize, close, maximize) | `wmctrl -i -a`, `xdotool windowminimize`, EWMH `_NET_CLOSE_WINDOW`/`_NET_WM_STATE_*` via `xprop -id` | `foreign-toplevel` `activate`, `close`, `set_minimized`, `set_maximized` requests | Folded into above |
| **Global hotkeys** (Super+M drawer, Super+Space apple menu, Super+L lock, …) | `XGrabKey` (not yet shipped — Phase 6.4 is open) | `org.freedesktop.portal.GlobalShortcuts` (xdg-desktop-portal) OR compositor-specific (`sway-ipc bindsym`, `hyprctl keyword bind`) | ~400 LOC; portal path also works on KDE/GNOME so this is the higher-value target |
| **Apple-menu popup positioning** | `gtk::Menu::popup_at_widget` (works on Wayland via `xdg_popup`) | Same call — no change required. ✓ already portable | 0 |
| **Status-cluster popovers** | `gtk::Popover` w/ `PopoverConstraint::None` | Same — `xdg_popup` honors the constraint hint | 0 |
| **Weather popover** | `gtk::Popover` anchored to clock button | Same | 0 |
| **Wallpaper rendering** | Desktop-hint window + scaled pixbuf | Layer-shell BACKGROUND layer, anchored to all edges | ~30 LOC delta |
| **System tray / notification daemon** | Not used; we run our own drawer | n/a | 0 |
| **Idle / sleep detection** | None today | `ext-idle-notify-v1` if we ever wire a screensaver | 0 today |
| **WM live-switch** (`bin/mackes-wm i3\|xfwm4`) | `i3 --replace`, `xfwm4 --replace` (X11-only managers) | Wayland compositor switch is a session-restart operation, not a live swap. Drop `mackes-wm` on Wayland; offer a logout-and-select flow instead. | New helper: detect session type, offer the right path |
| **`mackes-maximizer.service`** | Watches X11 windows via libwnck, maximizes new toplevels under xfwm4 | Wlroots maximizes via the same `foreign-toplevel.set_maximized` we already use in the dock port. Service becomes a no-op under Wayland; conditionalize on `XDG_SESSION_TYPE=x11` in the unit file. | 1-line drop-in |
| **LightDM greeter** | Native X11 greeter | LightDM has a `lightdm-gtk-greeter` build that runs under Mir/Wayland but adoption is thin. Realistic path on Wayland-first distros: switch to `tuigreet` (already works) or GDM. | Out of scope for the panel port |
| **Drawer Python (today) / Rust (4.3)** | GTK3 — already Wayland-aware via GDK backend | No code changes needed once 4.3 lands. The slide-in-from-right animation uses `Gtk.Window::move()` which is X11-only — replace with layer-shell anchor RIGHT + an animated `margin_right` tween. | ~50 LOC delta inside the drawer port |

## The hard parts

1. **Foreign-toplevel is wlroots-only.** GNOME-shell does not implement
   `zwlr_foreign_toplevel_manager_v1`. There is no portable Wayland
   protocol for "list every running window" outside of compositor
   extensions. **Decision:** target wlroots compositors for the dock
   tasklist; on GNOME the dock falls back to pinned-launchers-only.

2. **Global hotkeys need the portal.** The
   `org.freedesktop.portal.GlobalShortcuts` portal is the only
   cross-compositor way to bind shortcuts. It pops a permission prompt
   on first registration (good for security, costly for UX). This is
   actually the right design for 6.4 even on X11 — switch off
   `XGrabKey` once we have the portal path; X11 sessions get an
   `XGrabKey` fallback.

3. **`mackes-wm` doesn't translate.** A live WM swap under Wayland
   isn't a thing — the compositor *is* the display server. We'd offer
   "log out, pick Sway / Hyprland / xfwm4-on-X11 at the greeter" via
   shipping additional `wayland-session.desktop` files in
   `/usr/share/wayland-sessions/`.

4. **Plymouth + LightDM + the rest of the boot chain** keep working
   under Wayland once the compositor is. Continuity (Phase 8) is
   unaffected.

## Recommended sequencing

Once we're ready to start the Wayland port:

1. **Layer-shell + wallpaper** (small, isolated) — proves the surface
   plumbing under a wlroots compositor.
2. **Global shortcuts via portal** — replaces the open Phase 6.4 work
   with the portable version; lands on both X11 and Wayland.
3. **Foreign-toplevel dock** — biggest unit of work; gated on
   maturity of `wayland-protocols` 1.43 (released 2026-Q1) which
   stabilized v3 of foreign-toplevel.
4. **Drawer layer-shell anchor** — folds into 4.3 if we want.
5. **Drop `mackes-maximizer` under Wayland** — autostart guard +
   one-line systemd condition.
6. **Replace `bin/mackes-wm`** with a logout-and-pick flow.

Realistic estimate: 4–6 weeks of focused work after 4.3 lands, with
**wlroots compositors as the supported target**. GNOME-shell support
is a separate "shell extension or nothing" question and is out of
scope for the first Wayland release.

## Runtime probe

`mackes-panel` already runs under Wayland when `GDK_BACKEND=wayland`
forces it — the GTK3 chrome paints; the panel surface falls back to a
top-level window that the compositor places wherever it likes (no
strut, so windows overlap it). The dock segment is empty (wmctrl
returns nothing under a Wayland compositor). That's enough to confirm
no segfaults block the port — every X11-specific call already lives
behind a `Command::new("wmctrl")` / `xprop` boundary, so the
replacement points are localized.
