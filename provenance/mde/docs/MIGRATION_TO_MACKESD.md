# Migration: legacy `mackes.mesh_*` probes → `mackesd` / `mackesd_core`

**Status:** in-progress migration window (Phase 12.13)
**Authority:** `docs/PROJECT_WORKLIST.md` § Phase 12.13.4
**Design source:** `docs/design/v12.0-enterprise-mesh.md`

The Phase 12 lock (Enterprise Mesh Backend, 2026-05-19) moved every
mesh-control-plane responsibility off the legacy Python probes and onto
the `mackesd` Rust daemon. The Python panel links the read API
(`mackesd_core`) directly into the process — no IPC, no networked API
per the 12.A.3 lock.

This document is the single source of truth for the migration:

1. The **module-by-module mapping** from `mackes.mesh_*` to its
   `mackesd_core::*` replacement.
2. The **two-release deprecation window** governing when each legacy
   module disappears.
3. The **call-site cutover order** so consumers move off the probes
   before they vanish.

---

## 1. Module mapping (Phase 12.13.4)

Each row names the live, public `mackesd_core` module(s) that own the
responsibility. Audit source: `crates/mackesd/src/lib.rs` (the only
modules exported by the crate as of this writing).

| Legacy `mackes.mesh_*.py` | Authoritative replacement in `mackesd_core` | Phase reference |
|---|---|---|
| `mesh_vpn`           | `mackesd_core::enrollment` + `mackesd_core::topology` + `mackesd_core::policy` | 12.3, 12.4, 12.7.2 |
| `mesh_discovery`     | `mackesd_core::enrollment` + `mackesd_core::passcode`                          | 12.3.1, 12.10.1   |
| `mesh_ssh`           | `mackesd_core::identity` + `mackesd_core::secrets`                             | 12.3.2, 12.10.4   |
| `mesh_services`      | `mackesd_core::telemetry` + `mackesd_core::health`                             | 12.6.1, 12.1.3    |
| `mesh_mdns`          | `mackesd_core::topology` + `mackesd_core::telemetry`                           | 12.4, 12.6.1      |
| `mesh_metrics`       | `mackesd_core::metrics` + `mackesd_core::telemetry`                            | 12.1.5, 12.6      |
| `mesh_perf`          | `mackesd_core::telemetry` + `mackesd_core::reconcile`                          | 12.6.2, 12.5      |
| `mesh_nats`          | `mackesd_core::store` + `mackesd_core::events`                                 | 12.2, 12.6.3      |
| `mesh_sync`          | `mackesd_core::store` + `mackesd_core::revisions`                              | 12.2, 12.2.2      |
| `mesh_notifications` | `mackesd_core::events`                                                         | 12.6.3, 12.6.4    |
| `mesh_derp`          | `mackesd_core::topology` + `mackesd_core::reconcile`                           | 12.4, 12.5        |
| `mesh_browser`       | `mackesd_core::topology`                                                       | 12.4              |
| `mesh_fs`            | `mackesd_core::reconcile`                                                      | 12.5              |
| `mesh_fs_fuse`       | `mackesd_core::reconcile`                                                      | 12.5              |
| `mesh_media`         | `mackesd_core::telemetry`                                                      | 12.6.1            |
| `mesh_thumbnailer`   | `mackesd_core::reconcile`                                                      | 12.5              |
| `mesh_wol`           | `mackesd_core::topology` + `mackesd_core::reconcile`                           | 12.4, 12.5        |

Every module above now emits a `DeprecationWarning` at import time
naming its replacement (Phase 12.13.4 lands in 1.0.8).

---

## 2. Two-release deprecation window

The deprecation policy mirrors Python's standard convention and the
Phase 12.13.3 lock (legacy probes stay for a two-release window with
`[deprecated]` log warnings).

| Release line | What happens to the legacy `mackes.mesh_*` probes |
|---|---|
| **1.0.8** (current — Phase 12.13.4) | Modules continue to function as today. Importing any legacy `mackes.mesh_*` module emits `DeprecationWarning` at the import site, naming the `mackesd_core` replacement and pointing at this document. The Workbench panels still call into the probes; the `mackesd` cutover in 12.13.3 moves panel reads to `mackesd_core::*` first. |
| **1.1.x** (next minor) | First **deprecation release** — warnings are wired up across every consumer; CI and release notes treat any new caller of a legacy probe as a regression. The probes still work. |
| **1.2.x** (second minor) | Second **deprecation release** — the warnings remain but no in-tree consumer is allowed to import a legacy probe. `tests/test_imports.py` continues to walk every `mackes.*` module so we know the import path stays clean. |
| **2.0.0** | **Removal.** The legacy `mackes.mesh_*` modules are deleted from the repo. Any out-of-tree caller is expected to have migrated to `mackesd_core::*` (Rust) or to call into the Python panel's thin reader bindings. |

The two-release window starts the moment Phase 12.13.4 ships
(release 1.0.8) and ends with the 2.0.0 cut. The actual deletion lands
behind the existing release rulebook in `.claude/CLAUDE.md` § 0.6 — no
ad-hoc removals.

### What "release line" means here

Mackes Shell ships off `main`; release lines are tags
(`vMAJOR.MINOR.PATCH`). Patches inside a minor (e.g. 1.1.0 → 1.1.1) do
not start a new deprecation clock; only the leading minor counts.

### What "no in-tree consumer is allowed" means

In 1.2.x the `tests/test_imports.py` smoke (which walks every
`mackes.*` submodule) is allowed to keep importing the deprecated
modules, because that's the point of the smoke. Other call sites must
have moved.

---

## 3. Call-site cutover order

The cutover order minimizes churn — modules whose callers are entirely
inside `mackes.workbench.*` and `mackes.wizard.*` move first; modules
that other legacy probes import (e.g. `mesh_fs`, `mesh_sync`) move
last so their fan-out clears.

1. **Leaf consumers** — single-purpose probes with no in-tree
   imports of other `mesh_*` modules: `mesh_thumbnailer`,
   `mesh_notifications`, `mesh_wol`, `mesh_media`, `mesh_mdns`,
   `mesh_discovery`, `mesh_perf`, `mesh_metrics`. Cut the Workbench
   panel that consumes each one over to `mackesd_core::*` reads.
2. **Topology / VPN core** — `mesh_vpn`, `mesh_derp`, `mesh_ssh`,
   `mesh_services`. Each has a dedicated Workbench panel; cut the
   panel to `mackesd_core::topology` / `policy` / `health` reads.
3. **Substrate** — `mesh_browser`, `mesh_fs`, `mesh_fs_fuse`,
   `mesh_sync`, `mesh_nats`. Cut these last because the substrate is
   what the other probes depend on; once they're free, the substrate
   can drop.

The Phase 12.13.3 worklist item ("Workbench Mesh panels switch to
library reads") drives the actual cutover edits. Phase 12.13.4 (this
work) is the warning + documentation precursor; Phase 12.13.4's
follow-up (item the worklist already names, when 2.0.0 ships) is the
deletion.

---

## 4. Verification gates (Phase 12.13.4)

Every legacy probe import is verified by the existing import smoke:

```bash
# Each module raises under simplefilter('error'):
python3 -c "import warnings; warnings.simplefilter('error'); import mackes.mesh_vpn"
# DeprecationWarning: mackes.mesh_vpn is deprecated. The mesh VPN control plane ...

# The default test session still passes — pytest's default warning
# filter prints but does not error on DeprecationWarning from third
# parties, and `tests/test_imports.py` keeps walking every module:
python3 -m pytest tests/ -q
```

If a release adds a new mesh subsystem, the file moves on
day one to `mackesd_core::*`. No new `mackes.mesh_*.py` modules ship
in 1.x.

---

## 5. Quick reference for downstream code

If you used to write:

```python
from mackes.mesh_vpn import compute_topology, apply_policy
```

…you now call into the Rust crate. From the Workbench (linked
directly):

```rust
use mackesd_core::topology;
use mackesd_core::policy;

let snapshot = topology::compute(&desired_state)?;
let resolved = policy::resolve(&snapshot, &policies)?;
```

From external Python (transitional only, until 2.0.0):

```python
import warnings
warnings.simplefilter("default")  # let the deprecation surface
from mackes.mesh_vpn import compute_topology  # emits DeprecationWarning
```

There is no Python binding to `mackesd_core` — the linkage is
internal to `mackes-panel`. External callers should either run
`mackesd` subcommands or wait for the 2.0 reader API.
