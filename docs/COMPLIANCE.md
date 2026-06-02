# MDE-Retro — Compliance Evaluation

First sweep against `.claude/CLAUDE.md`, run via the `/audit` skill (6 parallel
passes). Verdicts: **FINISH** = wire up / make real / fix; **REMOVE** = delete dead
surface; **OK** = false positive (reachable / exempt). Generated 2026-06-01.

**Score: 8 REMOVE · 29 FINISH · 7 OK.** The platform is structurally sound — no
real stubs, the compositor boundary is clean, asset staging is consistent — but
**the prose docs badly lag the code** (15 findings) and the **single-color-edge
rule (§2.1) has leaked** in a handful of spots. Resolution status is tracked in
`docs/PROJECT_WORKLIST.md`; ✅ = fixed in the reorg pass, ☐ = open.

---

## 1. Documentation drift (§1) — 15 × FINISH  ✅ fixed in reorg

The headline failure. README/PREVIEW/ACCURACY/DEV-SETUP/LABWC-MIGRATION still
describe a **sway-based, Windows-2000-default, pre-migration** project. Reality
(confirmed in code): compositor is **labwc** (RPM `requires` labwc; `Exec=labwc`
session; `wlr.rs` foreign-toplevel; zero `swaymsg` in `src`); the **default theme
is Carbon dark** (`state.rs` `def_theme="carbon"`, `palette.rs` `DARK=1`, `main.rs`
falls through to `Theme::Carbon`); and the **labwc migration is DONE** (every
phase has a code landing) despite `LABWC-MIGRATION.md` saying "PLAN ONLY".

| Location | Stale claim |
|---|---|
| `README.md` (intro, badges, install) | "desktop for Sway", `wm-sway` badge, `dnf install sway`, sway session |
| `rust/README.md` (intro, table, Cutover §) | "Runs on top of sway", "Win2000 Classic" as the look, "cutover pending / main is script desktop" |
| `rust/PREVIEW.md` (intro, "Not in this preview", test counts) | "riding on top of sway", "not yet cut over / sway config flip", stale `cargo test` counts |
| `rust/ACCURACY.md` (§0 boundary, spot-check) | "mde ↔ **sway** boundary", "the Win2000 look is verified" as the only theme |
| `rust/DEV-SETUP.md` (caveat, deps) | "authored without a live toolchain", missing labwc runtime deps |
| `rust/LABWC-MIGRATION.md` (status, framing) | "**PLAN ONLY**", "Today (sway) → labwc", "drives swaymsg in six files" (now zero) |

## 2. Unreachable / dead code (§3) — 8 × REMOVE

| Location | What | Status |
|---|---|---|
| `mde-ui/src/widget/flag.rs` (+ mod.rs, lib.rs export) | whole Win2000 flag widget — superseded by the raster PNG Start icon; orphans `LOGO_*` | ✅ removed |
| `mde-ui/src/widget/mod.rs` `progress_style` | style fn with no `progress_bar` consumer | ✅ removed |
| `mde-ui/src/palette.rs` `use_beos` | dead back-compat shim (BeOS now via `set_theme`) | ✅ removed |
| `mde-ui/src/widget/frame.rs` `frame::pressed()` | unused constructor (≠ live `Bevel::pressed`) | ✅ removed |
| `mde/src/tui_setup.rs` `_gauge` | dead TUI helper, already `#[allow(dead_code)]` | ✅ removed |
| `mde/src/wlr.rs` `Wm::close` / `Wm::set_maximized` | architecturally unreachable (labwc owns titlebar) — *keep as protocol API or remove?* | ☐ decision |
| `mde/src/outputs.rs` `Output::make` | write-only EDID field | ☐ open |

## 3. Convention violations (§2) — 10 × FINISH

**§2.1 raw hex outside `palette.rs`:**
| Location | Leak | Status |
|---|---|---|
| `panel.rs:178` | Carbon header Gray 100 / white `from_rgb8` | ✅ → `palette::SHELL_HEADER` role |
| `control_panel.rs:362` | disabled gray `0x70` | ✅ → `palette::GRAY_TEXT` |
| `display.rs:795` | desktop teal `0x3a6e6e` | ✅ → `palette::BACKGROUND` |
| `icons.rs:154` | 8-entry Carbon icon-accent table inline | ☐ open (move to palette) |

**§2.2 ground-truth not pinned in `checklist.rs`:**
| Constant | Status |
|---|---|
| `WINDOW_FRAME` sentinel `(0,0,1)` | ✅ pinned |
| `TITLE_TEXT` sentinel `(0xff,0xff,0xfe)` | ✅ pinned |
| `INFO_TITLE_PX = 16` | ✅ pinned |
| `TASKBAR_BUTTON_MIN = 160` | ☐ open (low) |

**§2.3 raw `.size()` literals:** `display.rs:756` (`48.0`), `installer.rs:196,240`
(`10.0`/`15.0`) → ☐ open (add named `metrics` constants).

## 4. Mockups passing as features (§3) — 2 × FINISH  ☐ open

- `display.rs` Effects tab: 3 enabled checkboxes (`ToggleFx*`) write state that is
  never read/persisted. → grey out (`cbox_disabled`) or persist via `state.rs`.
- `taskbar_properties.rs`: "Show clock" + "Use Personalized Menus" enabled but
  discarded. → grey out (matches the file's own honest pattern) or wire.

## 5. Packaging (§4 / release skill) — 2 × FINISH

- ☐ **License gap:** `font.rs` embeds **DroidSans (Apache-2.0)** into the shipped
  binary, but `assets/licenses/` + `NOTICE.md` cover everything *except* Droid.
  → add `DroidSans-Apache-2.0.txt` + a NOTICE entry (IBM Plex, embedded the same
  way, *is* covered).
- ✅ **CLI parity:** `%post`/`%postun` symlinks now include `mde-display`,
  `mde-filedialog`, `mde-taskbar-properties`, matching the documented subcommand set.

## 6. Confirmed OK (no action)

Compositor boundary clean (the only "Active Window" caption is the Display-Props
mock preview); themerc/openbox hex are compositor config (labwc owns title bars);
the installer translucent-black scrim and the menu banner SVG art are documented
§2.1 carve-outs (the menu banner SVG was **reclassified as a real §2.1 leak in the
second sweep below** — it renders in-process, not via an external tool — and is now
fixed); `__wlr-list` correctly has no symlink; asset staging matches the
asset list exactly; `Workgroup="WORKGROUP"` is authentic read-only display.

---

# Second sweep — 2026-06-02 (`/audit` as a workflow, adversarially verified)

Re-run after the 2.0 "Windows 10 era" work landed, this time as a multi-agent
workflow: 6 dimension-finders → per-finding adversarial verifiers (default
`real=false`, with a safeguards list to kill false positives). **29 raw findings →
14 confirmed.** No new stubs, no mockups, compositor boundary still clean. The
residue is (a) one §2.1 hex leak the *first* sweep wrongly waved through as a
"carve-out", (b) two genuinely unreachable surfaces, and (c) a fresh layer of
sway→labwc / Win2000→Carbon drift in the *module* docs the first pass (top-level
prose only) never reached. **All 14 resolved in this commit set.**

| # | Location | Cat | Verdict | Resolution |
|---|---|---|---|---|
| 1 | `mde-ui/palette.rs:53` `URGENT` | unreachable | REMOVE | zero refs; `SPEC-windows10` promised a critical-toast tint never wired → const removed, SPEC corrected, tint → worklist `E3-urgent-tint` |
| 2 | `mde/menu.rs:795` Start-banner SVG | hex (§2.1) | FINISH | 5 raw hex (`#3a6ad0/#0a1a40/#000000/#ffffff/#6f9fe0`) → `palette::LOGO_*` roles, emitted via new `palette::hex_fixed` (fixed brand art, no remap); pinned in `checklist.rs`. Byte-identical render. |
| 3 | `mde/popup.rs:132` `items_for("desktop")` | unreachable | REMOVE | no caller — labwc serves the desktop right-click from its own static `menu.xml`; arm removed, era-aware desktop menu → worklist `E7.10a` |
| 4 | `mde/action_center.rs:8` | docdrift | FINISH | tile grid (E3.5) shipped here; "layers on later" caveat narrowed to E3.6 backends + inline actions |
| 5 | `mde/search.rs:311` | docdrift | FINISH | "Settings is a later epic" false — `mde settings` ships; comment corrected |
| 6 | `mde/main.rs:49` `USAGE` | docdrift | FINISH | "Windows 2000 … for Sway" → "Carbon/Win2000 … for labwc" (user-visible `--help`) |
| 7 | `mde/panel.rs:1,3` | docdrift | FINISH | "anchored to the bottom edge" / "fed by sway IPC" → per-era anchor (Carbon top default) + wlr-foreign-toplevel |
| 8 | `mde/files.rs:1` | docdrift | FINISH | "sway draws the navy title bar" → labwc draws title bar + frame via themerc |
| 9 | `mde-ui/palette.rs:14` | docdrift | FINISH | ACTIVE_TITLE comment: sway/`client.focused` → labwc/`window.active.title.bg` |
| 10 | `mde-ui/metrics.rs:10` | docdrift | FINISH | SIZE_FRAME/FIXED_FRAME "Sway-owned" → "labwc-owned" |
| 11 | `mde/display.rs:2` | docdrift | FINISH | "wired to Wayland/sway" + "sway `config.d` fragment" → labwc + generated `~/.config/mde/display.sh` |
| 12 | `mde/install.rs:12` | docdrift | FINISH | aliases "under `~/.config/sway`" → `~/.config/labwc` |

(The raw set counted the banner hex leak twice — two finders, two rows — and split
the panel doc into `:1` and `:3`; collapsed here. 14 confirmed = these 12 distinct
fixes, with the two banner rows merged into #2.)

**Carried forward (tracked, not in this commit):** `assets/install-assets.sh` still
hardcodes `SWAY_SCRIPTS=$HOME/.config/sway/scripts` + "swaymsg reload" — a shell
script, not Rust; lifted to worklist `install-assets-sway-drift` rather than edited
blind (the root `install.sh` is the OLD sway installer per §7, so the script's live
status needs confirming first).

---

# Third sweep — E8 focused (2026-06-02, workflow, adversarially verified)

After E8.2–E8.6 (Win10 File Explorer: Quick access, pins, This PC, navigation
pane, `mde mount`), a focused adversarial workflow audited ONLY that diff (the
rest of the repo was the second sweep's scope). **9 raw → 5 confirmed**, all
low-severity **doc-drift** — no unreachable/stub/mockup code, no §2.1 hex or §2.3
metric leaks, no §2.6 state gaps, no reachability holes in the new Explorer code.
The 5 stale comments the E8 work introduced, all fixed in the same commit:

| Location | Stale claim → fix |
|---|---|
| `state.rs` explorer_pins doc | "`explorer_landing` lands with E8.4" (it has) → dropped the forward-reference |
| `files.rs` Pane enum doc | omitted `ThisPc`; "QuickAccess is *the* Win10 landing" → documents both, "the default" |
| `files.rs` module doc | "Client area" listed only classic chrome → added the Win10 command-bar + nav-pane layout |
| `files.rs` `folder_body` doc | "Every non-QuickAccess view uses this" (false under Win10) → "Non-Win10 eras use this" |
| `files.rs` `nav_node` doc | active node "navy" (Win10 remaps HIGHLIGHT→accent) → "accent-filled" |
