---
name: audit
description: >-
  Integrity sweep of the MackesWorkstation Rust shell: find dead/unreachable
  code, stubs, mockups passing as features, convention violations (raw hex,
  scattered metrics), and stale docs — each finding gets a FINISH-or-REMOVE
  verdict. TRIGGER when the user asks to "audit", "evaluate compliance", "check
  for dead code/stubs", or "find what's not really done" in the workspace.
  Produces a findings table / report; it does NOT fix things unless asked.
---

# audit — compliance & integrity sweep (MackesWorkstation)

Catches the gap between "marked done" and "actually reachable + correct", and
checks compliance with the root `CLAUDE.md` (the operational rulebook at the
repo root, never `.claude/CLAUDE.md`). Output is a findings **table**
(`Location | Category | Evidence | Confidence | Verdict`) plus a short summary;
verdict is binary **FINISH** (wire it up / make it real) or **REMOVE** (delete
the dead surface). Don't fix unless asked — report first.

## Passes (run in parallel where possible)

1. **Unreachable code** — `pub mod`/`mod` with no external `<mod>::` ref; `pub fn`
   never called; dead `match` arms; a feature with no `mde <subcommand>` path to it.
2. **Stubs** — `todo!()`, `unimplemented!()`, `panic!("not …")`, stub arms,
   `pub mod foo;` with zero refs, "wiring in a follow-up" commit bodies.
   This is the §3 Definition of Done line: code existing is never "done".
3. **Mockups** — `demo_data`/placeholder constants, "coming soon"/"placeholder"
   strings, tabs/panels that render but do nothing.
4. **Convention violations** (root CLAUDE.md §2):
   - raw hex/RGB literal anywhere except `crates/shell/mde-ui/src/palette.rs` (§2.1)
     (`rg -n '#[0-9a-fA-F]{6}|from_rgb8?\(' crates/**/src` minus
     `crates/shell/mde-ui/src/palette.rs`);
   - `.size(` with a literal instead of the `metrics` module single-source (§2.3);
   - a palette/metric value changed without a matching `crates/shell/mde-ui/tests/checklist.rs`
     assertion (§2.2 — change a value only with a reference to back it);
   - mde drawing client-side title bars (labwc owns title bars/frames/z-order — §1).
5. **Doc drift** — prose claiming facts the code contradicts. Check prose against
   the *current* reality: **labwc** (Wayland/wlroots), the **IBM Carbon dark
   DEFAULT**, and the **four switchable looks** at the single `palette::color()`
   edge (Win2000 Classic · IBM Carbon · Windows 10 · BeOS — ChromeOS Classic and
   Material Symbols are dropped). Flag any prose still saying "sway"/"Hyprland",
   "Win2000 default", "Gluster", a fifth theme, or a `rust/`-rooted path. Each
   stale claim is a FINISH (fix the doc).
6. **Packaging reachability** — symbols/assets the RPM (`cargo generate-rpm`,
   ONE RPM with the deployment-role chooser Lighthouse ⊂ Server ⊂ Workstation)
   `%files`/`assets` list ships but nothing uses, or shell subcommands with no
   `mde-<cmd>` symlink in `%post`. Also confirm the DISCLAIMER.md pre-flight gate
   exists + is non-empty. (Note: the RPM, epic E8, is HELD until every feature is
   §3-complete; flag packaging gaps but don't treat a missing RPM as a defect.)

## Safeguards (avoid false positives)

Framework lifecycle callbacks (`iced` `update`/`view`/`subscription`, `Default`,
`Drop`, serde derives), `#[test]`/`#[cfg(test)]` helpers, and declaratively-wired
handlers are **reachable** even with no direct textual caller — don't flag them.
Confirm a "dead" symbol with `rg` across the whole workspace before the verdict.
Four crates are excluded from the default build (gtk3-devel/alsa-lib-devel):
`crates/legacy/mackes-panel`, `crates/services/mde-music`, `crates/services/mde-musicd`,
`crates/workbench/mde-workbench` — a symbol unreferenced only because its crate is
excluded is not dead.

## Output

A markdown findings table + counts by category, written to `docs/COMPLIANCE.md`
(or returned inline for a quick check). Lift every FINISH into
`docs/PROJECT_WORKLIST.md` (the single durable tracker — CLAUDE.md §5; it is
created when execution past E0 begins, so create it if absent) so the sweep
produces actionable work, not just a report.
