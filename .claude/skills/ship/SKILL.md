---
name: ship
description: >-
  Autonomously drain the MackesWorkstation worklist: a rescue pass to catch
  dead/mock code, then implement open tasks fully (no stubs), building +
  verifying each, committing as you go. TRIGGER when the user says "ship it",
  "execute", "continue", "drain the worklist", or "work through the backlog" for
  this Rust monorepo shell. Do NOT use for a single scoped edit (just do it) or
  anything needing a release cut (use /release).
---

# ship — autonomous worklist drain (MackesWorkstation)

Implements `docs/PROJECT_WORKLIST.md` to empty, under the standing autonomy in the
root `CLAUDE.md` §6. Heads-down: the commit body is the record, one short note per
phase boundary, no marketing copy.

> **Worklist may not exist yet.** `docs/PROJECT_WORKLIST.md` is the intended single
> tracker but is created when execution past E0 begins (CLAUDE.md §5). E0 (the
> merge) is complete; the executable plan is the E0–E8 epic sequence
> (`docs/MACKES-WORKSTATION-PLAN.md` §11). If the worklist file is absent, pull the
> next actionable items from the E1+ epics and create the worklist as the durable
> record before draining.

## Phase 0 — Rescue pass (always first)

Before new work, catch the project's recurring failure mode (shipped-but-dead /
mockup-only code). This is the single highest-value step.

1. **Dead-module grep** (`crates/**/src`): for each `pub mod`/`mod`, confirm an
   external `<mod>::` reference exists. A module with helpers + tests but no caller
   is **not done** — it's unreachable. List offenders.
2. **Stub/mock grep:** `rg 'todo!\(|unimplemented!\(|panic!\("not |coming soon|placeholder|demo_data'`
   across `crates/**/src`. Each hit is either real work or a mislabelled task.
3. **Reachability:** every shell feature must be reachable from an `mde <subcommand>`
   path and *do something* when launched (`timeout 3 ./target/debug/mde <sub>`).
4. **Re-cue misleading `[✓]`:** any worklist item marked done but failing 1–3 flips
   back to `[>]` with a one-line note. If ≥3 rescues, write a short audit note.

## Phase 1–N — Drain loop

For each open `[ ]` task, highest priority first:

1. Mark `[>]` in the worklist (restart-safe claim).
2. Implement **fully** per CLAUDE.md §3 (Definition of Done) — no stubs,
   runtime-reachable, no raw hex outside `crates/shell/mde-ui/src/palette.rs` (§2.1),
   metrics single-source via the metrics module (§2.3).
3. **Gate before commit** (auto-fix in scope; SOFT-ESCAPE if the same fix fails 3×).
   Run from the repo root:
   - `cargo check --workspace` · `cargo build --workspace` (or
     `cargo build --release` for packaging tasks)
   - `cargo test` (and `cargo test -p mde-ui` for palette/metric changes —
     `mde-ui` = `crates/shell/mde-ui`)
   - `cargo clippy --all-targets` · `cargo fmt --all`
   - **Visual tasks:** confirm the render, don't trust a green `cargo test`. Run
     `./preview.sh gallery` (the accuracy harness, ported to the repo root in E0.8;
     needs sway + grim) and **Read** the PNGs in `tests/accuracy/captures/gallery/`.
     For a quick single-surface check, `timeout 3 ./target/debug/mde <sub>`.
   - Note: a full build needs the system dev libs (`sudo dnf install -y gtk3-devel
     alsa-lib-devel`) — the audio chain links ALSA. Since E0.2 no crates are excluded;
     `.cargo/config.toml` sets `CMAKE_POLICY_VERSION_MINIMUM=3.5` for the vendored Opus.
4. Commit named pathspecs with a why-not-what message + the `Co-Authored-By`
   trailer:
   `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
   Flip the task `[✓]`. **Do not push** (§0) — that stays gated.
5. Run independent tasks in parallel where they don't touch the same files.

**Commit cadence — per-epic squash (50-Q survey lock R12, 2026-06-03).** Work an
epic's tasks in small local commits as above, then **squash to one commit per epic at
epic close** (when every task in that `E<n>` is `[✓]`). One squashed commit per master
epic lands the epic's whole diff with a summary body that enumerates the tasks. This
**supersedes** the older "small commits direct to `main`, every task = 1+ commits"
default (governance §8 / Q61) — newest wins. Per-task DoD (§3) is unchanged: each task
must be individually runtime-reachable and stub-free before it folds into the squash.

## Stop conditions

Worklist empty (only gated items remain) · a push/release/cutover moment · a
destructive op · a product-direction change · two consecutive unexplained gate
failures · ≥10 rescues at once. On stop: a short factual summary + what's left.

Pushing is `git push origin main` only — single `origin` remote, no dual-remote.
The RPM (epic E8) is held until every feature is §3-complete and is always
operator-gated.

## NOT this skill

Single obvious edit → just do it. Release cut → `/release`. Deep integrity sweep
with a written report → `/audit`. (The old MDE `autonomous-worker`,
`complete-remaining-work`, `iteration`, and `mackes-worklist-management` skills are
retired ancestors — their content folds into this skill and `/plan`.)
