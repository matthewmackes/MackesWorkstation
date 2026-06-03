# MackesWorkstation — Migration Status (E0)

The successor monorepo. It **EOLs and absorbs** both upstream projects plus the KDE
Connect host; see [`docs/MACKES-WORKSTATION-PLAN.md`](docs/MACKES-WORKSTATION-PLAN.md)
for the full plan (decisions §0, reuse table §9, shell changes §10, epics §11, layout §12).

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
