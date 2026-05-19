# Mesh admin guide

User-facing reference surfaced from the Workbench Help tab. Covers
the three things operators most often need to understand:
**how to configure a site-to-site mesh**, **how to set up a
failover route**, **what a drift warning means**.

## How to configure a site-to-site mesh

A site-to-site mesh links two LAN segments (e.g. your office and
your home) so peers on either side reach each other as if they
were on the same network. Set this up once on each side; the
mackesd reconciler handles the rest.

### What you need

- One **Host peer** per site that the other peers route through.
  Usually a desktop or NAS that's always on.
- The shared **16-character passcode** (Host operator can read it
  via `mackesd show-passcode`).
- Both sites need internet egress on UDP 41641 (Tailscale's
  default port) OR you fall back to the TCP/443 path automatically.

### Step-by-step

1. **On Site A's Host peer:** open Workbench → Network → Mesh
   and click **"This peer is a Host."** Generate the passcode if
   one doesn't exist yet.
2. **On every other Site A peer:** open Workbench → Network →
   Mesh → **Pair with Host** and enter the passcode.
3. **On Site B's Host peer:** click **"Pair with another Host"**
   and enter Site A's Host URL + the same passcode.
4. **On every other Site B peer:** pair with Site B's Host peer
   the same way you did at Site A.

The reconciler takes ~30 seconds to propagate the new topology
to every peer. After that, every peer on Site A can reach every
peer on Site B by hostname (e.g. `ssh laptop-b1.mesh`).

### How the routing works

Each Host advertises its site's subnet as a route. The other
site's Host accepts the advertisement and installs it. Mesh
traffic between sites flows Host-to-Host; intra-site traffic
stays on the LAN.

When Q23 throughput-aware routing is enabled (Phase 12.22), the
reconciler periodically measures both paths and prefers the
higher-bandwidth one — usually the Host-to-Host tunnel for
cross-site, but it'll fall back to the public DERP path if your
internet uplink is saturated.

## How to set up a failover route

Useful when the primary Host is occasionally offline (e.g. a
home NAS that reboots for backups). A failover Host takes over
when the primary's heartbeat goes stale.

### Failover prerequisites

- At least **two peers per site** that can serve as Host (must
  have the disk space for the SQLite store, plus internet egress).
- Both must be enrolled and reach `healthy` state.

### Promoting a failover Host

1. Open Workbench → Network → Mesh → **Hosts** tab.
2. Click **Add failover** and pick the candidate peer.
3. Set the **lease threshold** — how many missed heartbeats from
   the primary before the failover takes over. Default 3 (≈ 30 s).

The leader election described in `mesh-ops.md § Recovering from
split-brain` handles the actual takeover. You don't need to do
anything manually when the primary goes offline — the failover
acquires the QNM-Shared lockfile, replays the audit log, and
serves reads within one lease cycle.

### What changes when failover fires

- The Workbench → Network → Mesh banner switches to **"Failover
  active — primary unreachable for N seconds."** No popups, no
  modals.
- New revisions queued during the outage land in the failover's
  `desired_config`; they replay back to the primary on its next
  recovery.
- Audit log entries record the takeover with `kind = lifecycle,
  detail = failover_promoted`.

## What a drift warning means

A drift warning means the **runtime state** of the mesh
disagrees with the **desired state** in the configuration.
mackesd's reconciler detects drift on its 30-second tick and
surfaces it in the panel.

### Two severities

- **Auto-repairable** — the reconciler can re-push the desired
  state on its own. Examples: a peer's WireGuard routes drifted
  because Tailscale restarted; a service is supposed to be
  enabled and isn't. These resolve themselves within one or two
  reconcile cycles.

- **Manual review** — the reconciler can't fix this without
  operator approval. Examples: a peer is reporting a hardware
  fingerprint that doesn't match its enrollment record (possible
  identity drift); a policy that was enabled is now missing from
  the store (possible tampering). These land in the **Pending
  Changes** inbox and require an explicit "Approve" or "Reject"
  click.

### How to read a drift row

| Field          | What it means                                   |
|----------------|-------------------------------------------------|
| `severity`     | auto-repairable / manual-review                 |
| `detector`     | which subsystem noticed (topology / telemetry / policy / identity) |
| `since`        | how long the drift has persisted                |
| `reason_chain` | the comparison that produced the row (e.g. "desired peer adjacency [a, b] missing in observed_telemetry") |
| `next_action`  | what the reconciler intends — "retry in 30 s",
  "awaiting approval", "rolled back at r-2026-05-19-0042"        |

### When a drift warning is normal

- During a deploy. The reconciler shows brief drift while the
  state transitions; it clears as `Verified`.
- During a network blip. A peer's heartbeat may miss for a few
  seconds; the drift resolves on the next successful heartbeat.

### When a drift warning is concerning

- It persists for > 5 minutes despite multiple reconcile cycles
  with `severity = auto-repairable`. Means the reconciler is
  retrying but failing. Check the failure reason chain.
- It carries `severity = manual-review` and you don't recognize
  the change. Don't approve until you understand why it surfaced
  — could be tampering.
- `mackesd audit verify` reports a hash-chain break around the
  same time as the drift. This is a serious finding; do not
  approve any new changes until the audit chain is reconciled.

See the operator runbook (`mesh-ops.md`) for the
command-line diagnostics that pair with each drift row.
