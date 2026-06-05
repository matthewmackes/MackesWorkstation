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

## Status — E1–E8 (updated 2026-06-05)

> Epic-level snapshot; `docs/PROJECT_WORKLIST.md` holds the per-task detail + the live
> `[ ]`/`[>]`/`[✓]`/`[!]` state. Per the operator's standing directive (2026-06-05):
> **all hardware/2-device verification is held until after release**, the peer-testing
> gates are open for single-node §3 implementation, and the platform is driven
> feature-complete (E0–E7) before the RPM cut (E8, the only held action).

- **E0 — merge + foundation: ✅ done** (bar operator-gated items). mde-bus migration
  (E0.3) largely complete; sway→labwc surface retirement (E0.16) done; the standalone
  applet/panel ecosystem retired (E0.17/E5.5). *Open:* E0.12 (archive old repos —
  operator-gated), E0.11 (no-supervised-Python audit — blocked on E3's `fs_sync`).
  **E0.3.7 ✅ (2026-06-05)** — closed by the E2.2 KDC convergence: the last
  MDE-internal D-Bus interface is gone, the lint-dbus-shape allowlist is empty,
  only FDO interop (`org.freedesktop`/`org.kde.StatusNotifier`/`org.mpris`) remains.
- **E1 — deployment-role install: ✅ done to floor.** Role chooser → `role.toml`,
  role-gated mackesd workers + systemd units (greetd role-gate drop-in), installer
  wired. Live cold-boot/per-role unit set = HW bench.
- **E2 — KDE Connect: E2.1 ✅ + E2.2 ✅ to floor.** E2.1 (inbound listener — built +
  loopback-tested; live phone-initiated = 2-device bench). E2.2 (converge the legacy
  mackesd KDC host onto the canonical `mde-kdc-host`) **done to single-node floor**
  (commit `3b3eb8b6`): the plugin-dispatch policy moved to `mde-kdc-proto::dispatch`,
  `KdcHostWorker` rewritten on the canonical `PairingStore` (`Arc<Mutex<…>>` + a
  worker-local outbound queue), mackesd's deps swapped to the canonical host+proto —
  `cargo tree` shows one host, the legacy `mde-kdc{,-proto}` are fully orphaned
  (#1/#2 met; discovery/pair/ping parity = 2-device bench). E2.3/2.4 downstream;
  the orphaned legacy `mde-kdc{,-proto}` are reference-only (they were mackesd path-deps,
  never `[workspace] members`, so removing that dep already took them out of the build
  graph — `cargo metadata` lists 35 packages, neither among them; kept under
  `crates/legacy/` per CLAUDE §8).
  **E2.3 ✅ to floor** (commits `4d6f9ed7`/`274757d1`/`2c49595e`): the canonical
  `PairingStore` is interior-mutable (one shared `Arc`); mackesd's `kdc_host` worker
  runs the single supervised host (UDP discovery + the E2.1 TLS listener) + serves the
  roster on `action/connect/devices`; the shell's `mde connect` is now a pure Bus
  client (the shell dropped its `mde-kdc-host`/`mde-kdc-proto` deps — `cargo tree -p
  mde` shows zero KDC-host refs). All 4 acceptance met single-node; live online/battery
  = 2-device bench.
  **E2.4 ✅ to floor** (commit `cdeb6597`): Explorer's Cloud Files pane was prebuilt
  (E8.7/8.8/8.10) but enumerated from a dead local `devices.json` nothing writes — wired
  it to the mackesd roster (`connect::devices()`, fetched off-thread) so it lists the
  actually-paired devices (#1); mount + browse a real phone over sftp = 2-device bench
  (#2/#3). **NET: the whole E2 epic (KDE Connect) is feature-complete to its single-node
  floor** — listener, one host, mackesd-owned store, Cloud Files enumeration — with the
  live 2-device round-trips as the post-release bench.
- **E3 — LizardFS mesh-storage: ❌ blocked** on the external LizardFS FUSE dependency
  (E3.1 `[!]`).
- **E4 — Win10 shell era: ✅ done** (the large E4.1–E4.23 epic — taskbar, tiled Start,
  Action Center, Settings, Search, Security, Storage, etc.; mde-portal retired).
- **E5 — apps: mixed.** E5.5/5.6 (applet-host retirement) ✅. E5.3 (Media Player Win10
  reskin) `[>]` — palette-integrated, app on the MDE dark theme; chrome polish +
  Airsonic/MPRIS = bench. E5.1 Explorer / E5.2 Phone depend on E3/E2; E5.4 VoIP is a
  greenfield PJSIP softphone.
- **E6 — Workbench reskin: ✅ all 8 role-landings done** (E6.1 foundation + the
  Manage-Your-Server console: Dashboard/Apps/Devices/Fleet/Look&Feel/Maintain/System/Help,
  each `mde-workbench --page <role>`, contract-tested). *Remaining:* E6.10 Compute
  (rebuild legacy `mde-virtual`, large + HW-bench) and E6.11 Preset/drift engine
  (blocked on the 5 undefined preset variants — product spec).
- **E7 — merged OOBE: E7.1 ✅ + E7.2 ✅ to floor.** E7.1 (Win10 OOBE, all stages +
  `mde oobe`). E7.2 — Role-picker (#1, pins `role.toml` + gates the flow), Nebula
  mesh-enrolment (#2, real `mackesd enroll`), KDC phone-pair stage (#3, honest KDE
  Connect guidance — unblocked by the E2 convergence, commit `40e250f4`), and
  "read before proceeding" Disclaimer (#4) all built + tested. Live mesh-enrolment
  cert-sign + live phone-pair persist = post-release bench.
- **E8 — release: held.** E8.2 (disclaimer single-source audit) partially verified
  (existing surfaces clean). The RPM cut (E8.5/8.6) is the one operator-gated action,
  held until E0–E7 are feature-complete.

**Net:** every cleanly-completable, in-this-environment task is drained; the remaining
work is large multi-iteration efforts (E2.2 convergence, E6.10 Compute), greenfield
(E5.4), product-spec-blocked (E6.11), external-dep-blocked (E3), or the held
HW-bench / RPM cut.
