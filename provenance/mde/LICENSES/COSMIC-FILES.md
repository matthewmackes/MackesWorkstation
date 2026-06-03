# cosmic-files — Upstream license + attribution

MDE Files (`crates/mde-files/`) vendors portions of the upstream
[cosmic-files](https://github.com/pop-os/cosmic-files) project per
Phase 4.1 of `docs/PROJECT_WORKLIST.md`. The vendored sources live
under `crates/mde-files/src/upstream/` once the Phase 4.2 vendor
pull lands.

## Copyright

Copyright © 2023–2026 System76, Inc.  
Copyright © 2023–2026 cosmic-files contributors (full list in the
upstream git log)

## License

cosmic-files is distributed under the GNU General Public License,
version 3 or (at your option) any later version. MDE itself ships
under the same license (`license.workspace = "GPL-3.0-or-later"`),
so the vendor incorporation is license-compatible.

The full text of GPL-3.0-or-later is included in MDE's root
`LICENSE` file. The summary terms — preserve copyright,
distribute source, propagate the same license — apply to the
vendored cosmic-files modules and to every downstream binary that
links them.

## What is vendored

Per Phase 4.2's lock:

  - `cosmic-files/src/tab.rs`  — file-list rendering primitives
  - `cosmic-files/src/mod.rs`   — mime sniffing
  - `cosmic-files/src/trash.rs` — trash-spec adapter

Each vendored file lands at `crates/mde-files/src/upstream/<name>.rs`
with a top-of-file attribution comment naming the upstream commit
SHA + path. Modifications stay minimal (we adapt the data types
via the Phase 4.3 bridge layer, not by editing upstream sources).

## Pinned upstream

See `docs/upstream/cosmic-files.md` for the current pinned commit
SHA + tarball checksum.

## Attribution requirement

Every binary that links MDE Files must reproduce this attribution
in its --version / about output. The Workbench's About panel
covers this for the GTK + Iced surfaces; CLI binaries print a
`See LICENSES/` reference at startup when run with `--version`.
