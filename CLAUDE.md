# Mackes Workstation — Claude Workspace Instructions

- **Project:** the **successor monorepo** that fuses three now-archived repos into one
  native-Rust operating environment: **MDE** *(MackesDE for Workgroups, the mesh
  platform)*, **MDE-Retro** *(the Windows-10 / IBM-Carbon shell)*, and
  **MDE-KDECnt-Rust** *(the KDE Connect host)*. It **EOLs and absorbs** all three.
- **Repo:** `github.com/matthewmackes/MackesWorkstation` (single `origin`, private;
  flips public after review). Default branch `main`.
- **Governance compass:** [`docs/AI_GOVERNANCE.md`](docs/AI_GOVERNANCE.md) — the
  platform identity + architectural locks (mesh, Bus, storage, security, naming, the
  AI-collaboration model), imported from MDE and reconciled to the monorepo (read its
  top "⚑ MONOREPO RECONCILIATION" banner first).
- **Plan / source of truth:** [`docs/MACKES-WORKSTATION-PLAN.md`](docs/MACKES-WORKSTATION-PLAN.md)
  — decisions (§0), reuse table (§9), shell changes (§10), epics E0–E8 (§11),
  workspace layout + deployment roles (§12). Status: [`MIGRATION.md`](MIGRATION.md).
- **Skills + hooks:** live in [`.claude/`](.claude/) — skills `plan` · `ship` ·
  `release` · `audit` · `preview`; hooks wired via `.claude/settings.json`
  (see `.claude/hooks/README.md` for harness- vs git-wired).
- **Heritage rulebooks** (reference, NOT live): `provenance/mde/.claude/CLAUDE.md`
  (the heavyweight Python MDE rulebook) and `provenance/mde-retro/.claude/CLAUDE.md`
  (the Rust shell rulebook this file descends from). Useful for the *why* behind a
  rule; this file is the live one.

This is an **operational rulebook**, not an architecture tour — architecture facts
live in §1, the rest is how to *act* here. When rules conflict, the **newer lock wins
silently**; authority ranks **Memory > this file > `docs/AI_GOVERNANCE.md` >
`docs/MACKES-WORKSTATION-PLAN.md` + `docs/design/*.md` > `docs/PROJECT_WORKLIST.md`
body**.

---

## §0 — Commit & Push Rulebook

- **§0.1 Separate authorizations.** Committing and pushing are each their own explicit
  ask. Do **not** commit or push unsolicited — writing code, building, and running
  tests never licenses a commit. "Save it" / "ship it" / a `/ship` run authorizes
  commits; pushing still needs its own go-ahead. One approval is not a standing license.
- **§0.2 Branch policy.** Work on `main` (the working branch). For risky or
  outward-facing visual reworks, branch first (`ux/<topic>` or `<area>/<topic>`), then
  ask before merging. Never force-push `main`; never `--amend` / `--no-verify` a pushed
  commit — always prefer a new commit over amending.
- **§0.3 Explicit staging.** Stage named pathspecs — `git add -- <file>…` or
  `git commit <file>…`. **Never** `git add -A / . / -u`: it sweeps unrelated in-flight
  edits into the wrong commit.
- **§0.4 Commit messages.** Read `git log` first to match voice (the merge used `E0:`
  prefixes; per-epic work uses `E<n>:` / area prefixes). Explain **why, not what**.
  Verb taxonomy: `add` (feature) / `update` (enhancement) / `fix` (bug) / `refactor`
  (no behavior change) / `packaging` / `docs`. Use a HEREDOC body to preserve newlines.
  End every message with:
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- **§0.5 Destructive ops need confirmation.** `git reset --hard`, `checkout -- .`,
  `restore .`, `clean -f`, `rm -rf` outside `target/`, history rewrites,
  branch/remote deletion, and any GitHub state change (repo visibility, default
  branch, releases, archiving the old repos) are confirmed first unless the user just
  asked for exactly that.
- **§0.6 Outward-facing = confirm.** Pushing, publishing, opening PRs, flipping the
  repo public — anything third parties can see. Public is hard to reverse
  (indexing/caching). **Secrets-scan before the first public push.**
- **§0.7 Push-conflict recovery.** On a non-fast-forward rejection,
  `git fetch origin main` + rebase + re-push; don't stop the loop for a routine
  pull-rebase. Any genuine content conflict → stop and surface it; don't guess.

## §1 — Architecture (the load-bearing facts)

> **The prose lags the code.** READMEs and old design docs carry stale framing. Trust
> the manifests + source defaults (`Cargo.toml`, `crates/shell/mde/src/main.rs`,
> `crates/shell/mde-ui/src/palette.rs`/`state.rs`) over prose. Fixing drift is itself
> work.

- **Workspace:** ONE cargo workspace at the repo root, `version = "10.0.0"`,
  `edition 2021`, GPL-3.0-or-later, toolchain pinned by `rust-toolchain.toml` (1.94).
  Members live under `crates/{platform,mesh,shell,workbench,services,shared,applets,kdc,legacy}/`.
- **The shell is one multiplexed binary** (`crates/shell/mde`): `main.rs` dispatches
  subcommands from `argv[1]` (`mde panel`) or the `mde-<cmd>` symlink basename. GUI
  surfaces are layer-shell or xdg-toplevel iced apps; **each subcommand is its own
  process** (re-reads state + sets the palette at launch — theme changes are not live
  across already-running surfaces).
- **The platform sits underneath:** `mackesd` (supervised daemon, owns workers +
  mesh/CA state) · **`mde-bus`** internal pub/sub backbone · **Nebula** encrypted
  overlay · **LizardFS** mesh-storage (NOT Gluster — retired wholesale) · the
  KDE Connect host (`crates/kdc`, the canonical `MDE-KDECnt-Rust` lineage). Shell +
  Workbench consume state and send actions **over the Bus** — no in-process worker
  pool in the shell, no MDE-internal D-Bus (FDO interop like
  `org.freedesktop.Notifications` only).
- **Toolkit:** iced 0.13 (wgpu, image/svg/advanced/tokio) + iced_layershell 0.13;
  pure-Rust stack — rustls (no OpenSSL), cosmic-text (no FreeType).
- **Theme system — one edge.** `palette::color(rgb) -> iced::Color` remaps per the
  active `Theme` before producing a color, so call sites never change when the theme
  switches. **Four switchable looks share the engine: Windows 2000 Classic, IBM
  Carbon (default dark), Windows 10, BeOS.**
- **Compositor: labwc** (Wayland/wlroots; MDE's sway-specific bits adapt to it).
  Window control via wlr-foreign-toplevel. **labwc draws title bars, frames, z-order;
  mde draws only client areas + its own layer-shell surfaces** — never make mde a
  window manager.
- **Deployment: ONE RPM, install-time role chooser** — Lighthouse (relay) ⊂ Server
  (headless) ⊂ Workstation (full desktop), each a strict superset; role gates which
  `mackesd` workers + surfaces are enabled (§12 of the plan).

## §2 — Project conventions (the spine)

- **§2.1 No raw hex outside `palette.rs`.** Every color is a palette role constant
  through `palette::color()` (the one theme-remap edge). App-chrome colors live in
  `palette.rs` too.
- **§2.2 Ground truth is pinned in tests.** The `mde-ui` checklist tests encode the
  exact reference palette + metrics. Change a palette/metric value only with a
  reference to back it, and update the matching assertion in the same commit.
- **§2.3 Metrics are single-source** via the `metrics` module — never a scattered
  `.size(...)` literal.
- **§2.4 Icons via the resolver** (`icon_any` / `icons.rs`): embedded SVGs first, then
  the freedesktop chain; missing → empty space, never tofu. New shell icons go in the
  embedded icon module.
- **§2.5 State stays compatible.** `~/.config/mde/menu.json`: every field
  `#[serde(default)]`; the manual `Default` impl must agree with `parse("{}")`;
  `save()` is atomic; garbage → defaults.
- **§2.6 Workspace lints are load-bearing** (`Cargo.toml` `[workspace.lints]`):
  `unsafe_code = forbid`, `unused_must_use = deny`; clippy `pedantic`/`nursery` warn,
  and `unwrap_used` / `panic` / `todo` warn — treat these as work, not noise.
- **§2.7 Reuse is the spine.** New code is glue, not reimplementation — the whole
  project thesis is fusing existing crates. Before writing a subsystem, check the
  per-crate reuse table (plan §9) for what already exists.

## §3 — Definition of Done (no stubs, runtime-reachable)

Code existing is **never** "done". A change is done only when it is **reachable from a
runtime entry point and observably works**:

- Reachable from an `mde <subcommand>` path (or an iced `update`/`view`/subscription it
  feeds, or a `mackesd` worker / Bus subscription), and verified by launching it
  (`timeout 3 ./target/debug/mde <sub>` no-panic check, or the preview harness for
  visual work).
- **No stubs:** no `todo!()` / `unimplemented!()` / `panic!("not yet")`, no stub
  `match` arms, no `pub mod foo;` with zero external `foo::` refs, no commit body
  saying "wiring lands in a follow-up". If it can't ship complete in one commit,
  re-split at write time into tasks that each CAN.
- **No mockups passing as features:** no `demo_data`/placeholder constants or
  "coming soon" strings standing in for real behavior.
- Builds clean (`cargo check`/`build --workspace`), `cargo test` green,
  `cargo clippy` + `cargo fmt` clean, and — for any **visual** change — confirmed
  against the reference, not eyeballed once.
- **Refuse / surface** "just stub it", "scaffold the module", "phase 2 wires it".

## §4 — Build · test (run from repo root)

```sh
cargo check --workspace        # green; whole tree (no crates excluded since E0.2)
cargo build --workspace
cargo test                     # accuracy + unit tests
cargo clippy --all-targets     # lint (warnings are work)
cargo fmt --all
```

> **System dev libs required** (E0.2, 2026-06-03): the audio chain
> (`crates/services/mde-music{,d}`, `crates/workbench/mde-workbench`) links ALSA, so a
> full build needs `sudo dnf install -y gtk3-devel alsa-lib-devel`. There are no longer
> any excluded crates — the audio chain is back in `members` and the legacy gtk3
> `mackes-panel` is retired (deleted; the iced shell's `panel.rs` replaces it).
> `.cargo/config.toml` sets `CMAKE_POLICY_VERSION_MINIMUM=3.5` so the vendored Opus
> (`opus`→`audiopus_sys`) configures under CMake 4 — see that file's comment.

## §5 — Worklist & planning

- **Today the plan IS the tracker.** `docs/MACKES-WORKSTATION-PLAN.md` holds the
  decisions + the E0–E8 epic sequence; `MIGRATION.md` holds live E0 status. There is
  no `docs/PROJECT_WORKLIST.md` yet — when execution past E0 begins, create one as the
  single durable tracker (status legend `[ ]` Open · `[>]` In Progress · `[✓]` Done ·
  `[!]` Blocked; no `[~]` deferred, no silent deferrals).
- In-session Task tools are a scratchpad; a durable file wins on any divergence.
- Design docs (future `docs/design/*.md`) are **not** a parallel worklist — lift
  actionable items out of them.

## §6 — Autonomy

On "execute" / "continue" / "ship it": work the highest-priority open work first
(respect the E0–E8 dependency order in plan §11), run independent work in parallel,
mark `[>]` before substantive edits, implement **fully** (§3), add follow-up items for
any debt, and keep going until a real obstacle. **Standing authorization:** make
best-choice decisions on loose specs (record the choice in the commit body), move past
blocked work, improve "in the spirit", add tracker items. **Still gated (stop and
ask):** pushing, flipping the repo public, archiving the old repos, cutting the RPM
release, and any §0.5 destructive op. Clarifying questions go one at a time via
`AskUserQuestion`.

## §7 — Release (operator-triggered only)

The RPM (E8) is **held until every feature is §3-complete**; hardware bench is
post-release. One spec, conditional subpackages (`mde-core` / `mde-headless` /
`mde-desktop`), one git tag `MackesWorkstation-v10.0.0`. **Disclaimer pre-flight
gate:** `DISCLAIMER.md` must exist + be non-empty before any RPM build. Never cut a
release unless the operator explicitly asks; never auto-trigger from a `/ship` run.

## §8 — File index

| Path | What |
|---|---|
| `Cargo.toml` | workspace members, version, lints, excluded crates |
| `crates/shell/mde/src/main.rs` | subcommand dispatch + startup theme select |
| `crates/shell/mde-ui/` | the look library — `palette.rs` (the one hex/theme edge), metrics, widgets |
| `crates/shell/` | `mde` + surfaces (panel, popover, installer, wizard, session…); `mde-portal` retired E4.20, `mde-drawer` retiring |
| `crates/platform/mde-bus` | internal pub/sub backbone |
| `crates/mesh/` | `mackesd` (control plane), Nebula tunnel, mesh types/config/transport |
| `crates/kdc/` | `mde-kdc-proto` + `mde-kdc-host` (canonical KDE Connect host) |
| `crates/workbench/` | `mde-workbench`, `mde-virtual` (KVM/Podman compute) |
| `crates/{services,shared,applets}/` | daemons · theme/components · 17 status applets |
| `crates/legacy/` | retiring crates (kept for reference) |
| `docs/AI_GOVERNANCE.md` | platform identity + locks (mesh/Bus/storage/security/naming/AI-model), monorepo-reconciled |
| `docs/MACKES-WORKSTATION-PLAN.md` | decisions, reuse table, E0–E8 epics, layout + roles |
| `MIGRATION.md` | live E0 merge status + remaining-to-close list |
| `.claude/skills/` · `.claude/hooks/` | `plan`/`ship`/`release`/`audit`/`preview` skills · session-start + worklist hooks |
| `provenance/` | the three source repos' full trees (incl. their `.claude/` rulebooks) |
</content>
</invoke>
