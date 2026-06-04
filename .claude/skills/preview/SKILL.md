---
name: preview
description: >-
  Render and visually verify the MackesWorkstation shell — the accuracy harness.
  TRIGGER when the user wants to "preview", "show the gallery", "screenshot the
  shell", "verify the render", or confirm a visual change actually looks right
  (Carbon dark default, or one of the four themes). Use this instead of trusting a
  green `cargo test` for any UI change. The harness is staged at the repo root:
  `./preview.sh gallery` (needs sway + grim for the isolated nested-compositor capture).
---

# preview — render & accuracy verification (MackesWorkstation)

A green `cargo test` does **not** verify the render. This skill drives the real
visual check via `./preview.sh` (ported to the repo root in E0.8), which captures
the shell in an **isolated nested sway** (`WLR_BACKENDS=headless` + grim) — no effect
on the live desktop.

## Commands (run from the repo root)

```sh
./preview.sh gallery     # regenerate ALL component screenshots in the isolated
                         # nested compositor → tests/accuracy/captures/gallery/*.png
                         #   (+ _contact-sheet.png + per-era carbon/win2000/windows10/).
./preview.sh verify      # run the accuracy harness the same isolated way
./preview.sh nav-sweep   # keyboard-nav parity: no-panic launch sweep of every surface
./preview.sh <component> # launch ONE live on the CURRENT session, click around, kill it.
                         #   Components: panel menu files control-panel system-properties
                         #   security phone run properties logoff shutdown setup
./preview.sh             # help
```

Requires `sway` (headless backend), `grim`, `swaymsg` for the capture paths;
`cargo build` first (preview.sh auto-builds `target/debug/mde` if missing).

## How to use

1. **Render + Read the result.** `./preview.sh gallery`, then **Read the PNGs**
   (`tests/accuracy/captures/gallery/`, e.g. `_contact-sheet.png`) and confirm the
   change against the intent. The captures are **gitignored** (generated); only the
   harness + `tests/accuracy/refs/` are committed.
2. **Quick fallback** for a single surface (no capture): `timeout 3 ./target/debug/mde
   <sub>` — confirm it draws + doesn't panic. The live compositor is **labwc**; each
   `mde <subcommand>` is its own process re-reading `~/.config/mde/menu.json`, so a
   theme change is not live across already-running surfaces — relaunch them.
3. **Four themes, one edge.** The shell ships **one theme engine, four switchable
   looks** flipped at the single `palette::color()` edge: **Win2000 Classic, IBM
   Carbon (DEFAULT dark), Windows 10, BeOS**. The gallery captures per-era folders
   (`carbon/`, `win2000/`, `windows10/`). To check a non-default look interactively,
   set `~/.config/mde/menu.json` (`theme` / `theme_mode` / `icon_color`) before
   rendering and **restore it after**. The daily-driver target is the Windows 10 shell.
4. **Static-only check** (palette + metric ground truth, always headless-safe):
   `cargo test -p mde-ui` (the crate at `crates/shell/mde-ui`).

## Notes

- No raw hex lives anywhere but `crates/shell/mde-ui/src/palette.rs` (CLAUDE.md §2.1);
  ground truth is pinned in `crates/shell/mde-ui/tests/checklist.rs`; UI sizes are
  single-sourced via the metrics module (`crates/shell/mde-ui/src/metrics.rs`, §2.3).
  If a render looks off, suspect a palette/metric edit before the surface code.
- Captures assume the component is unoccluded and the screen is awake; a blanked screen
  or an overlapping window fails by design.
- `tests/accuracy/refs/*.png` are foreign-DPI real Win2000 shots for **eyeballing
  only** — never SSIM-diffed.
- Visual verification is the §3 Definition-of-Done gate for any UI change — do not mark
  a task `[✓]` in `docs/PROJECT_WORKLIST.md` on a green `cargo test` alone.

See also: `/audit` (find dead/mock/stub UI), `/ship` (drain the worklist, accuracy-
verifying each change), `/release` (operator-gated RPM cut).
