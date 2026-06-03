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

## Next — build-green (the rest of E0)

The three imports are still **three separate cargo workspaces** (nested workspaces don't
build), so `cargo build` at the root does not yet work. Bringing it to one green build:

1. **Unify the workspace.** One root `Cargo.toml` `[workspace]` listing every member;
   remove the three inner `[workspace]` tables; add `[workspace.package]` so the crates
   that inherit (`version.workspace = true`, etc.) resolve against one source.
2. **Reconcile dependency versions** across the three (iced, rustls, tokio, zbus, serde,
   …) to a single set; fix the path-dep edges now that everything is co-located
   (MDE-Retro's `mde-kdc-host` path dep becomes an in-workspace member).
3. **Reorganize to the §12 layout** (`crates/platform`, `crates/shell`, `crates/kdc`,
   `crates/workbench`, `crates/services`, `crates/shared`, `crates/applets`, `crates/mesh`)
   with `git mv` (history follows), retiring the `import/` staging dirs.
4. **One version line** `10.0.0`, GPL-3.0; wire `mde-bus`; confirm the `mde <subcommand>`
   dispatch binary builds; `mackesd` as a supervised service entry point.
5. `cargo build` + `cargo test` green across the union → **E0 complete**.

## After E0

Per the plan: E1 deployment-role install, E2 KDE Connect convergence (incl. the inbound
listener already landed in `import/mde-kdc`), E3 LizardFS mesh-storage, E4 Win10 shell
replaces mde-portal, E5 apps, E6 Workbench re-skin, E7 merged OOBE, E8 polish + held RPM.

The old repos (MDE, mde-retro-workstation, MDE-KDECnt-Rust) are **archived read-only**
once this repo builds and is pushed — they live on as the history merged here.
