# MackesWorkstation — Migration Status (E0)

The successor monorepo. It **EOLs and absorbs** both upstream projects plus the KDE
Connect host; see [`docs/MACKES-WORKSTATION-PLAN.md`](docs/MACKES-WORKSTATION-PLAN.md)
for the full plan (decisions §0, reuse table §9, shell changes §10, epics §11, layout §12).

## ✅ E0 MERGE COMPLETE (2026-06-03)

The three repos are fused into **one monorepo, builds green, pushed to GitHub** —
development can begin:

- **`github.com/matthewmackes/MackesWorkstation`** (private; flip public after review).
- **History preserved** — 1,635 commits, all three repos' full history reachable; the
  `target/` bloat was stripped with `git-filter-repo` (`.git` 681M → 51M).
- **§12 layout** — `crates/{platform,mesh,shell,workbench,services,shared,applets,kdc,legacy}/`;
  shared embeds at `assets/`, `data/`, `crates/shell/assets`; `skel/`, `packaging/`, `docs/`;
  source-repo reference under `provenance/`.
- **`cargo check --workspace` green** (rustc 1.94, pinned by `rust-toolchain.toml`), minus
  4 system-lib crates excluded in `Cargo.toml` (see below) for libs absent in the build
  sandbox — they build on a box with `gtk3-devel` / `alsa-lib-devel`.

**Remaining to fully close E0:** archive the 3 old repos (read-only); wire `mde-bus`;
the E0.1/E0.3–E0.10 worklist tasks; resolve the lint warnings. Then E1–E8 per the plan.

> **Update 2026-06-03 (E0.2 done):** the 4 "excluded crates" are resolved — the audio
> chain (`mde-music`/`mde-musicd`/`mde-workbench`) is back in `[workspace] members`
> (`alsa-lib-devel` installed on the dev box) and the legacy gtk3 `mackes-panel` is
> **deleted** (the iced shell's `panel.rs` replaces it). `cargo check --workspace` is
> green over the **whole** tree; `.cargo/config.toml` carries the CMake-4 Opus fix. The
> "excluded crates" tables below are historical.

## Done — structural import (history preserved)

All three repos are merged in via the built-in subtree-merge (`merge -s ours` +
`read-tree --prefix`), so each repo's **full history is reachable** from this repo:

| Source repo | Imported at | From |
|---|---|---|
| MDE (MackesDE for Workgroups) | `import/mde/` | `github.com/matthewmackes/MDE@main` (`dfa76d1`) |
| MDE-Retro (the Win10/Carbon shell) | `import/mde-retro/` | `github.com/matthewmackes/mde-retro-workstation@main` (`ec8f058`) |
| MDE-KDECnt-Rust (KDE Connect host) | `import/mde-kdc/` | `github.com/matthewmackes/MDE-KDECnt-Rust@main` (`bf25291`) |

58 `Cargo.toml` manifests total; **3 workspace-root manifests** to reconcile:
`import/mde/Cargo.toml`, `import/mde-retro/rust/Cargo.toml`, `import/mde-kdc/Cargo.toml`.

## Done — build-green (in-sandbox)

The three imports are unified into **one root cargo workspace** and
**`cargo check --workspace` passes green** (0 errors). What this proves: the fusion is
structurally sound — the workspace resolves, ~600 dependency crates + 51 workspace crates
type-check as one, and the path-dep edges (incl. the shell→KDC host) all resolve.

**4 crates are excluded** in `Cargo.toml`'s `[workspace] exclude` because they need
**system dev libraries absent in this sandbox** (no root to install them). They build on
any box with the libs; this is a sandbox limit, not a fusion problem:

| Excluded crate | Needs | Note |
|---|---|---|
| `mackes-panel` | `gtk3-devel` (glib/gdk/atk/cairo) | legacy GTK panel — retire (iced shell replaces it) |
| `mde-musicd` | `alsa-lib-devel` (cpal audio out) | re-include with the lib on CI |
| `mde-music` | (via `mde-musicd`) | ↑ |
| `mde-workbench` | (via `mde-musicd`) | ↑ — major surface; build on a dev box |

The `mde-kdc-proto` clash was resolved by renaming MDE's retiring in-tree copy to
`mde-kdc-proto-legacy` (dependents alias it via `package =`).

## Next — finish E0

1. **Verify the excluded set on a real box** (`gtk3-devel`, `alsa-lib-devel`) — or retire
   `mackes-panel` and re-include the audio chain; full `cargo build` + `cargo test`.
2. **Reorganize to the §12 layout** (`crates/platform`, `crates/shell`, `crates/kdc`,
   `crates/workbench`, `crates/services`, `crates/shared`, `crates/applets`, `crates/mesh`)
   with `git mv` (history follows), retiring the `import/` staging dirs.
3. **Wire `mde-bus`**; confirm the `mde <subcommand>` dispatch binary builds; `mackesd`
   as a supervised service entry point. Resolve the 8 lint warnings.
4. Create the GitHub repo + push; **archive** the three old repos (both gated).

## After E0

Per the plan: E1 deployment-role install, E2 KDE Connect convergence (incl. the inbound
listener already landed in `import/mde-kdc`), E3 LizardFS mesh-storage, E4 Win10 shell
replaces mde-portal, E5 apps, E6 Workbench re-skin, E7 merged OOBE, E8 polish + held RPM.

The old repos (MDE, mde-retro-workstation, MDE-KDECnt-Rust) are **archived read-only**
once this repo builds and is pushed — they live on as the history merged here.
