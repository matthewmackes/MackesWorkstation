# SPEC — Carbon Design re-theme of the MDE shell

Replaces the Windows 2000 look-and-feel with an IBM Carbon Design System
interface, as a switchable theme (Carbon = default, Win2000 kept). Locked via a
15-question survey on 2026-06-01.

## Decisions

1. **Strategy** — Theme system with `Carbon` (default) and `Win2000` (kept,
   switchable in Appearance). Existing Win2k palette/bevels remain reachable.
2. **Base themes** — Ship both Carbon **Gray 10** (light) and **Gray 90** (dark)
   with a light/dark toggle.
3. **Accent** — Carbon **Blue 60**: `#0f62fe` (light) / `#4589ff` (dark). Drives
   buttons, links, selection, focus.
4. **Icon colors** — Picker with **4** choices, each auto-shading per light/dark:
   - Neutral — light `#161616`, dark `#f4f4f4`
   - Blue    — light `#0f62fe`, dark `#78a9ff`
   - Orange  — light `#ba4e00`, dark `#ff832b`
   - Red     — light `#da1e28`, dark `#fa4d56`
   Recolor the embedded single-color Carbon SVGs at render via iced svg tint.
5. **Default mode** — **Dark (Gray 90)** on first run.
6. **Title bars** — **Carbon UI Shell header**: taller flat header, hamburger +
   breadcrumb/title at left, controls at right. No gradient, no bevel.
7. **Window controls** — **Accent-tinted**: min/max hover = accent (blue),
   close hover = Red 60 `#da1e28`. Thin square glyphs.
8. **Buttons** — **Full Carbon set**: primary (filled accent), secondary
   (outline), ghost (text). Flat, left-aligned labels.
9. **Corners** — **2px** radius on windows/buttons/fields/menus.
10. **Depth** — **Flat layers + 1px borders**. Use Carbon layer grays for
    stacked surfaces; 1px dividers (`#393939` dark / `#e0e0e0` light). Drop
    shadows ONLY on overlays (menus, dialogs). Remove all 3D bevels.
11. **Typography** — **IBM Plex Sans** for all text (UI + monospace contexts).
12. **Taskbar** — Replace bottom taskbar with a **top Carbon UI Shell bar**
    (global): ≡ menu + breadcrumb at left, running apps, tray + clock at right.
    Nothing at the bottom.
13. **Start menu** — **Carbon product-switcher grid** of app tiles (9-dot style),
    not a vertical list.
14. **States** — **Full Carbon**: 2px accent focus ring (replaces dotted focus),
    accent-tinted selection rows, thin flat scrollbars (no arrow buttons/bevels).
15. **Rollout** — Implement **all at once**, one pass, then gallery + iterate.

## Carbon token reference (Gray 10 / Gray 90)

| Token | Light (Gray 10) | Dark (Gray 90) |
|---|---|---|
| background        | `#f4f4f4` | `#262626` |
| layer-01          | `#ffffff` | `#393939` |
| layer-02          | `#f4f4f4` | `#525252` |
| field             | `#ffffff` | `#393939` |
| border-subtle     | `#e0e0e0` | `#525252` |
| border-strong     | `#8d8d8d` | `#6f6f6f` |
| text-primary      | `#161616` | `#f4f4f4` |
| text-secondary    | `#525252` | `#c6c6c6` |
| text-on-color     | `#ffffff` | `#ffffff` |
| interactive/accent| `#0f62fe` | `#4589ff` |
| focus             | `#0f62fe` | `#ffffff` |
| danger (close)    | `#da1e28` | `#da1e28` |
| field-hover       | `#e8e8e8` | `#474747` |

Header bar uses Gray 100 (`#161616`) on dark / `#ffffff` on light per UI Shell.

## Implementation notes (to be refined after architecture map)

- New `mde-ui` theme module: an active-theme enum + token lookup, replacing the
  raw Win2000 `palette::*` constants with theme-routed tokens. Keep Win2000
  tokens as one theme's values.
- `state.rs`: add `theme` (`carbon`/`win2000`), `theme_mode` (`dark`/`light`),
  and `icon_color` (`neutral`/`blue`/`orange`/`red`). Defaults: carbon, dark,
  neutral. Surface toggles in Display ▸ Appearance.
- `icons.rs`: tint embedded SVGs using the resolved icon color for the active
  mode.
- Flatten `widget/bevel.rs`, `frame.rs`, `button.rs`, etc. to Carbon styles
  gated by active theme.
- `panel.rs`: top UI Shell bar. `menu.rs`: switcher grid.
- Verify with `./preview.sh gallery` after the pass.
