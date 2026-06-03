# MDE Files — cosmic-files upstream pin

**Phase 0.2 + Phase 4.1 lock.** This document records the upstream
[cosmic-files](https://github.com/pop-os/cosmic-files) commit MDE
Files vendors from, along with its tarball SHA-256, so vendor
updates are auditable + reproducible.

## Current pin

| Field | Value |
|---|---|
| Upstream | https://github.com/pop-os/cosmic-files |
| Pinned commit SHA | `e2c4f8a9b1d6c3e5f7a8b9c0d1e2f3a4b5c6d7e8` *(placeholder — set at vendor-bump time)* |
| Pinned commit date | TBD at vendor time |
| Tarball SHA-256 | TBD at vendor time |
| License | GPL-3.0-or-later (matches MDE's workspace license) |
| Vendor target | `crates/mde-files/src/upstream/` |
| Bump cadence | Manual, audited per Phase 4.x acceptance |

The placeholder values land when Phase 4.2 ("Vendor relevant
modules") actually pulls the tarball into the tree. Until then,
this file holds the lock — every Phase 4 substep references it as
the source-of-truth for the pinned upstream.

## How to bump

1. Choose a new upstream SHA on `master` (or the matching release
   branch). Verify it builds locally against the workspace's
   pinned libcosmic + Iced versions.
2. Download the tarball:
   `curl -fsSLO https://github.com/pop-os/cosmic-files/archive/<sha>.tar.gz`
3. Record the SHA-256:
   `sha256sum cosmic-files-<sha>.tar.gz`
4. Update the table above with the new SHA + date + tarball hash.
5. Re-vendor the relevant modules per Phase 4.2; re-run the
   Phase 4.3 data-model bridge tests; verify the Phase 4.4
   sidebar swap still applies cleanly.
6. Open a PR titled `chore(mde-files): bump cosmic-files pin to <short-sha>`.

## Why pin at all

cosmic-files is upstream's actively-developed file manager. We
pin a SHA + tarball hash so:

- Vendor builds reproduce exactly (no "works on my machine" drift).
- License obligations stay auditable — the SHA points at the
  exact GPL-3.0 source we're redistributing.
- Schema breaks (upstream renaming `Item` → `FileEntry`, e.g.)
  surface as a deliberate bump, not a surprise on the next
  vendor refresh.

## Attribution

cosmic-files is Copyright © 2023–2026 System76, Inc. + contributors,
licensed under GPL-3.0-or-later. Full license text + per-file
attribution lives at `LICENSES/COSMIC-FILES.md` (lands with Phase
4.1 vendor pin).

## See also

- `docs/design/v2.0.0-mde-files/design-spec.md` — MDE Files
  contract; upstream `Item` ↔ `FileRow` mapping documented there.
- `docs/design/v2.0.0-mde-files/upstream-bundle/` — original
  cosmic-files prototype handoff bundle (HTML + chat transcripts).
- `docs/PROJECT_WORKLIST.md` Phase 4.x — full vendor-merge plan.
