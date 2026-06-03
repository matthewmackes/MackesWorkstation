# Mackes Workstation

The **successor** platform — a single monorepo that fuses, and end-of-lifes, two of the
owner's projects:

- **MDE** ("MackesDE for Workgroups") — the Rust mesh *platform* (`mackesd`, Nebula mesh,
  LizardFS mesh-storage, KDE Connect, voice/VoIP, music, files, the Workbench console).
- **MDE-Retro** — the Windows-10 / IBM-Carbon desktop *shell* on labwc.

with the **MDE-Retro shell as the primary UX on top** and the **MDE platform underneath**.
Features that don't fit the Windows 10 idiom live in the **Workbench**. *Reuse is key.*

> **Status:** E0 monorepo bootstrap. The three upstream repos are imported with history
> preserved under `import/`; the unified cargo workspace + build-green are in progress.
> See [`MIGRATION.md`](MIGRATION.md) and [`docs/MACKES-WORKSTATION-PLAN.md`](docs/MACKES-WORKSTATION-PLAN.md).

Licensing: GPL-3.0. Win10-inspired with original assets — see [`DISCLAIMER.md`](DISCLAIMER.md)
(experimental/educational; no warranty; the user accepts full risk).
