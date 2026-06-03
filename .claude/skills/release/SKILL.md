---
name: release
description: >-
  Cut a MackesWorkstation RPM (one spec, role-chooser subpackages): pre-flight
  gates incl. the DISCLAIMER.md gate, version check, asset staging,
  cargo-generate-rpm build, then commit/tag. TRIGGER ONLY when the operator
  explicitly types "cut release" / "build the RPM" / "release it" for this repo.
  NEVER auto-trigger from a /ship run — releasing is always operator-gated, and
  the RPM (epic E8) is HELD until every feature is §3-complete.
---

# release — RPM cut (MackesWorkstation)

Operator-triggered only. Pushing tags and publishing are outward-facing (CLAUDE.md
§0) — confirm before anything leaves the machine. Push to a **single** remote
`origin`, branch `main` (`git push origin main` / `git push origin <tag>`); there is
no dual-remote.

> **The RPM is HELD (epic E8).** Per the E0–E8 plan (docs/MACKES-WORKSTATION-PLAN.md
> §11) and CLAUDE.md §3/§7, the package does not cut until **every feature is
> §3-complete** (runtime-reachable, no stubs). HW bench is post-release. If the
> operator asks for a cut before that gate, surface it and confirm a scoped
> "cut for testing" before proceeding.

## Pre-flight gates (all must hold)

1. **DISCLAIMER gate.** `DISCLAIMER.md` (repo root) **must exist and be non-empty**
   before any RPM build. No disclaimer → no RPM. (Hard pre-flight.)
2. Clean git tree on `main`; nothing un-committed that belongs in the cut.
3. `docs/PROJECT_WORKLIST.md` (the single durable tracker, CLAUDE.md §5) has no open
   `[ ]`/`[>]` blocking the release scope — or the operator explicitly scoped a
   partial "cut for testing". (The worklist is created when execution past E0 begins;
   if it doesn't exist yet, the E8 hold above already blocks a real cut.)
4. `cargo build --workspace --release` clean; `cargo test` green; `cargo clippy
   --all-targets` and `cargo fmt --all --check` clean (run from the repo root).
5. **Visual verification.** Once the accuracy harness is staged, `./preview.sh verify`
   must pass (real render check, not the silent-skip path). **Pending-port caveat:**
   `preview.sh` and `tests/stage-rpm-assets.sh` are being PORTED from MDE-Retro and
   currently live under `provenance/mde-retro/`; they are NOT yet staged at the repo
   root (E-level work). Until staged, fall back to building + launching
   `timeout 3 ./target/debug/mde <sub>` and inspecting.

## Steps

1. **Version — single-sourced, no per-crate bump.** The version is
   `[workspace.package] version = "10.0.0"` at the **repo-root `Cargo.toml`** (one
   version, all crates inherit via `version.workspace = true`). Do NOT edit a
   per-crate `version` in `crates/shell/mde/Cargo.toml` — it inherits. Bump the
   workspace version on shell changes; bump only the
   `[package.metadata.generate-rpm] release` (in `crates/shell/mde/Cargo.toml`) for
   packaging/asset-only changes so `dnf upgrade` sees a newer NEVRA.
2. **Update** release notes if present.
3. **Stage assets:** `tests/stage-rpm-assets.sh` (stages the bundled primary look —
   Win2k icons + Chicago95 cursors/sounds + Plex fonts — into `target/rpm-assets/`,
   which the RPM `assets` list references). The 76MB Chicago95 icon fallback is **not**
   bundled — `mde install --assets` fetches it at first run (locked decision #7: ship
   code-only, redistribute no third-party asset bytes beyond the primary set). Verify
   `assets/licenses/NOTICE.md` covers anything bundled before a public RPM.
   **Pending-port caveat:** this script still lives under `provenance/mde-retro/`; until
   it is staged at the repo root, asset staging is part of the E8 hold, not yet runnable.
4. **Build:** `cargo generate-rpm -p mde` (run from the repo root) →
   `target/generate-rpm/mde-*.rpm`. `cargo generate-rpm` is the **mechanism** —
   never raw `rpmbuild`. The layout is **ONE spec** with an install-time
   deployment-role chooser producing conditional subpackages
   **`mde-core` / `mde-headless` / `mde-desktop`** (Lighthouse ⊂ Server ⊂ Workstation).
5. **Smoke test:** install in a throwaway env or at least confirm `%post` would symlink
   every `mde-<subcommand>`; branding is applied out-of-band by the
   `mde-activate-branding.service` one-shot on next boot (NOT inline in the txn).
6. **Commit** the version/release bump (named pathspecs, `Co-Authored-By` trailer).
   **Tag + push only after explicit operator go-ahead** (CLAUDE.md §0 — committing and
   pushing are separate authorizations). Release tag: **`MackesWorkstation-v10.0.0`**.

## Failure modes

`cargo generate-rpm` missing → `cargo install cargo-generate-rpm`. Asset list points at
absent `target/rpm-assets/**` → re-run `stage-rpm-assets.sh` (once ported). Empty/missing
`DISCLAIMER.md` → the build is gated; do not proceed. Branding must never run inside the
rpm transaction (dnf lock / no network) — it's the `%posttrans` one-shot.

> Retired ancestors: the old MDE skills (autonomous-worker, complete-remaining-work,
> iteration, mackes-worklist-management, batch) are not carried forward. The live skill
> set is exactly five: plan, ship, release, audit, preview. `release` is operator-gated
> and is never auto-triggered from a `/ship` run.
