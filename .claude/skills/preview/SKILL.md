---
name: preview
description: >-
  Render and visually verify the MackesWorkstation shell — the accuracy harness.
  TRIGGER when the user wants to "preview", "show the gallery", "screenshot the
  shell", "verify the render", or confirm a visual change actually looks right
  (Carbon dark default, or one of the four themes). Use this instead of trusting a
  green `cargo test` for any UI change. NOTE: the `preview.sh`/`tests/accuracy/`
  harness is being PORTED from MDE-Retro and is not yet staged at the repo root —
  until then, fall back to `cargo build` + `timeout 3 ./target/debug/mde <sub>`.
---

# preview — render & accuracy verification (MackesWorkstation)

A green `cargo test` does **not** verify the render: the dynamic accuracy harness
**silently skips** when headless (it returns early with no `WAYLAND_DISPLAY` / no
captures), so unit tests never exercise the visual layer. This skill drives the real
visual check.

> **PORT-IN-PROGRESS CAVEAT (read first).** The `./preview.sh` launcher and the
> `tests/accuracy/` harness are being **ported from MDE-Retro** and are **not yet
> staged at the repo root** — they currently live under
> `provenance/mde-retro/rust/preview.sh`. Staging them is E-level roadmap work. Until
> the port lands, **use the fallback** (build + launch, below) for any visual check.
> Once ported, every command in this skill runs **from the repo root** (there is no
> `rust/` directory anymore — the workspace IS the repo root).

## Fallback (works today — use until the harness is ported)

```sh
cargo build                         # debug → target/debug/mde
timeout 3 ./target/debug/mde <sub>  # launch ONE surface live, auto-killed after 3s
                                    # — confirm it draws + doesn't panic, eyeball it
```

Subcommands worth launching: `panel menu files control-panel system-properties run
properties logoff shutdown setup`. The default theme is **IBM Carbon (dark)**.

For a **static-only** check (palette role colors + metric ground truth, always
headless-safe and available now):

```sh
cargo test -p mde-ui   # crates/shell/mde-ui — palette + metrics + checklist.rs
```

## Commands (POST-PORT — run from the repo root)

Once `preview.sh` + `tests/accuracy/` are staged at the repo root:

```sh
./preview.sh gallery     # regenerate ALL component screenshots in an isolated
                         # nested compositor → tests/accuracy/captures/gallery/*.png
                         #   (+ _contact-sheet.png). No effect on the live desktop.
./preview.sh verify      # run the accuracy harness the same isolated way
./preview.sh <component> # launch ONE live on the current session, click around,
                         # then kill it. Components: panel menu files control-panel
                         #   system-properties run properties logoff shutdown setup
```

## How to use

1. **Build first** — `cargo build` produces `target/debug/mde` (what `preview.sh`
   runs). The live compositor is **labwc** (Wayland/wlroots); each `mde <subcommand>`
   is its own process and re-reads `~/.config/mde/menu.json`, so a theme change is not
   live across already-running surfaces — relaunch them.
2. **Render + Read the result.** With the harness: `./preview.sh gallery`, then **Read
   the PNGs** (`tests/accuracy/captures/gallery/`) and confirm the change against the
   intent. Without it (today): `timeout 3 ./target/debug/mde <sub>` per surface and
   eyeball. The default theme is **Carbon dark**.
3. **Four themes, one edge.** The shell ships **one theme engine, four switchable
   looks** flipped at the single `palette::color()` edge: **Win2000 Classic, IBM
   Carbon (DEFAULT dark), Windows 10, BeOS**. To check a non-default look, set
   `~/.config/mde/menu.json` (`theme` / `theme_mode` / `icon_color`) before rendering
   and **restore it after**. The daily-driver target is the Windows 10 shell.
4. **Theme-aware capture:** panel/bar anchors differ by theme. The gallery crop takes
   the **top** strip (`0,0 1280x40`) for the Carbon top bar; use `0,920` for a
   Win2000-style bottom taskbar. Start menu / dialogs are captured full.
5. **Static-only check** (palette + metric ground truth, always headless-safe):
   `cargo test -p mde-ui` (the crate at `crates/shell/mde-ui`).

## Notes

- No raw hex lives anywhere but `crates/shell/mde-ui/src/palette.rs` (CLAUDE.md §2.1);
  ground truth is pinned in `crates/shell/mde-ui/tests/checklist.rs`; UI sizes are
  single-sourced via the metrics module (`crates/shell/mde-ui/src/metrics.rs`, §2.3).
  If a render looks off, suspect a palette/metric edit before the surface code.
- Captures assume the component is unoccluded and the screen is awake; a blanked screen
  or an overlapping window fails by design.
- `refs/*.png` (if/when staged) are foreign-DPI real Win2000 shots for **eyeballing
  only** — never SSIM-diffed.
- Visual verification is the §3 Definition-of-Done gate for any UI change — do not mark
  a task `[✓]` in `docs/PROJECT_WORKLIST.md` on a green `cargo test` alone.

See also: `/audit` (find dead/mock/stub UI), `/ship` (drain the worklist, accuracy-
verifying each change), `/release` (operator-gated RPM cut).
