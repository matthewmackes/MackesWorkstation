# Mackes Workstation — Mesh Features Map & Audit

*Audit + reference of the mesh networking, mesh file-sharing, services, notification,
and Bus-alert surfaces — where each lives, how it's reached, and whether it's live or
pending. Compiled 2026-06-07 from the source (the prose lags the code; this was
traced against `crates/`). Status legend: **live** (runtime-reachable + functional) ·
**partial** (reachable, honest-empty until a backend lands) · **pending** (an open
epic) · **bench** (needs hardware / a multi-node bench to fully exercise).*

---

## 1. Network mesh controls — *where*

The mesh control plane is **`mackesd`** (the supervised daemon owning Nebula + the
CA + mesh state). The operator-facing controls live in three places:

### a) The Workbench (`mde-workbench`) — the primary mesh console
A standalone binary (`crates/workbench/mde-workbench`), launched as `mde-workbench`
(NOT `mde workbench`). Deep-link a specific panel with `--focus <slug>` or a role
landing with `--page <role>` (a running instance is re-focused over the Bus). Mesh
panels (`crates/workbench/mde-workbench/src/panels/`):

| Panel | Slug (`--focus network.<slug>`) | What |
|---|---|---|
| `mesh_control` | `mesh_control` | core mesh on/off, status, this-node identity |
| `mesh_join` | `mesh_join` | enroll this node into a mesh / accept an invite |
| `mesh_pending` | `mesh_pending` | pending enrollment requests (CA approve/deny) |
| `mesh_topology` | `mesh_topology` | peer graph / reachability |
| `mesh_federation` | `mesh_federation` | cross-mesh federation |
| `mesh_history` | `mesh_history` | mesh event history |
| `mesh_services` | `mesh_services` | services advertised across the mesh (see §3) |
| `mesh_storage` | `mesh_storage` | LizardFS mesh-storage controls (see §2) |
| `mesh_bus` | — | live Bus traffic inspector (topics, subs, mutes) |
| `vpn` | `network.vpn` | Nebula/VPN status |
| `fleet_settings`, `fleet_revisions` | — | fleet config + revision history |

### b) Settings ▸ Network & Internet (`mde settings`)
The modern Settings app surfaces the mesh panels as deep-links (no duplication, §2.7):
Settings ▸ Network ▸ {Mesh control, Topology, Federation, Join, Pending, History,
Services, SSH} each open the matching `mde-workbench --focus network.mesh_*` panel.
Plus the standard Win10 Network pages (Status / Wi-Fi / Ethernet / VPN / Hotspot /
Proxy / Airplane / Data usage / Cellular).

### c) The panel mesh-status chip (always-visible)
`crates/shell/mde/src/panel.rs` `mesh_chip()` — a tray chip showing the live peer
count + online glyph; **click → opens the Workbench**. Hidden until the first
`mackesd` poll lands. *(Audit fix 2026-06-07: the chip launched `mde workbench` — an
unknown subcommand → silent no-op; corrected to the real `mde-workbench` binary.)*

### d) KDE Connect (distinct from the Nebula mesh)
`mde connect` (roster) + `mde phone` (the Your-Phone window) — the `crates/kdc`
host runs inside `mackesd`. This is device pairing, not the Nebula overlay.

**Reachability:** ✓ `mde-workbench` binary present; `mde settings`/`mde connect`/
`mde phone` dispatch verified; the mesh-status chip now opens the Workbench.

---

## 2. Mesh shared files — *where*

### a) `mde files` — the Explorer (live, with mesh panes)
`crates/shell/mde/src/files.rs`. Left-nav panes (`Pane`): **Quick access**, **This
PC**, **Network** (SMB browse via gio/smbclient), **Cloud device** (paired KDE
Connect peers — the roster comes from the `kdc_host` worker over the Bus via
`connect::devices()`). **live** for SMB + Cloud-device browse.

### b) The Bus file-transfer surfaces (`mackesd/src/ipc/files.rs`)
Migrated off the `dev.mackes.MDE.Shell.*` D-Bus onto the mesh **Bus** (E0.3.2):
- **Fleet.Files** — **live**: the peer roster (`nodes` SQLite table) that `mde files`'
  mesh-browse reads.
- **Inbox / Outbox / Downloads / FileOperations** — **partial**: Bus responders on
  `action/files-{inbox,outbox,downloads}/<verb>` + `action/file-ops/<verb>` that
  return honest empty / "transport not configured" states. The real Send-To transfer
  engine (the `mackesd::orchestrator` state machine) is a future feature epic.

### c) LizardFS mesh storage
`mde-workbench` ▸ `mesh_storage` panel + the `mackesd` `meshfs_worker` (Server/
Workstation roles). The FUSE mount is **bench**-proven (E3.1); the master/chunk
daemons + auto-mount of the mesh XDG dirs land with the RPM (E3.2, **pending**).

### d) Unified mesh-first file manager — **pending (E10)**
The flagship "Artifact Manager" (mesh sidebar, send-to-peer, first-class LizardFS
browse, one `mde files` engine) is E10 (libcosmic→iced port; currently `mde-files`'s
backend still serves `demo_data` until E10.2). Today's mesh file access is via the
panes in (a) + the Workbench storage panel — not yet the unified sidebar.

**Reachability:** ✓ `mde files` dispatched; Network/Cloud panes live; transfer
engine + LizardFS daemons + the unified manager are tracked (E3/E10).

---

## 3. Services — *where*

- **`mde-workbench` ▸ `mesh_services`** — services advertised across the mesh (browse
  / status). The Workbench role landing (`--page dashboard`, the "Manage Workstation"
  app) is the operator console.
- **`mackesd` role-gated workers** — the actual services, supervised by `mackesd` and
  gated by the install-time role (Lighthouse ⊂ Server ⊂ Workstation): enrollment(CA)/
  leader/health on every role; +fleet/meshfs/metrics on Server; +voice/media on
  Workstation. Inspect with `mackesd role-workers` (static census) / `mackesd healthz`.
- **Cross-segment mDNS service relay** (`MESH-MDNS-RELAY`) — **live** code (republish a
  peer's Jellyfin/Chromecast/AirPlay onto other segments over the overlay); the 2-
  segment round-trip is a post-release **bench**.

**Reachability:** ✓ workbench panel + `mackesd` CLI; live-over-Bus worker listing
needs a running `mackesd` (bench).

---

## 4. Notifications — *where*

| Surface | Command | What |
|---|---|---|
| **Action Center** | `mde action-center` (Win+A) | the notification pane + quick-action tiles; reads the notifyd mirror, stamps last-read |
| **Toast** | `mde toast <id>` | a transient bottom-right toast for one notification |
| **notifyd** (daemon) | hosted in the long-lived `mde panel` process | the freedesktop notification server (`org.freedesktop.Notifications`) — serves D-Bus + mirrors to `notifications.json`. **Universal** since the Carbon collapse (E9.7) — runs on every theme, not just Win10. |
| **mde-popover** | `mde-popover notifications` / `mde-popover toast` | the notifications-list popover (bell click) + the long-running toast render surface (tails `~/.cache/mde/toasts.jsonl`, stacks ≤3) |

FDO interop only (`org.freedesktop.Notifications`) — no MDE-private notification
D-Bus. **Reachability:** ✓ `mde action-center` + `mde toast` dispatched; notifyd
starts with the panel universally.

---

## 5. Bus messages — alerts & activities (what happens)

The internal backbone is **`mde-bus`** (pub/sub; `crates/platform/mde-bus`). Every
stored message carries a `priority`, and **`surface::dispatch(msg, surfaces)`**
(`crates/platform/mde-bus/src/surface.rs`, BUS-2.1) routes it to one of four
escalating on-screen treatments:

| `priority` | Surfaces lit up (`Surfaces` trait method) |
|---|---|
| `min` | **silent log only** — history kept, no UI (`log_silent`) |
| `default` | **tray icon + Dock breadcrumb badge** (`tray_and_badge`) |
| `high` | **status-zone slide-up strip + sound + persistent-until-ack** (`status_strip_and_sound`) |
| `urgent` | **Theater takeover (full-screen) + wallpaper stripe + phone push** (KDC2 + ntfy) (`theater_wallpaper_phone`) |

- **Emit path:** `mde-alert-emit` maps a human priority string to the Bus priority
  (`CRITICAL`/`CRIT`/`EMERGENCY` → `urgent`; `high`/`default`/`min`), publishes on the
  Bus; the subscriber dispatches via the table above.
- **DND/quiet hours:** `surface::dispatch_with_suppression` (BUS-2.8) suppresses
  per the focus-assist / quiet-hour rules before lighting surfaces.
- **Urgent theater:** the `mde panel` subscribes to urgent alerts and raises the
  full-screen `mde-popover urgent` takeover (BUS-2.5) for `urgent` fleet/announce
  segments.
- **Inspect live traffic:** `mde-workbench` ▸ `mesh_bus` panel (topics, subscriptions,
  mutes).

So an "activity" (a peer event, a service alert, a fleet announce) becomes a Bus
message with a priority, and the operator's UI lights up proportionally — from a
silent history entry up to a full-screen takeover + phone push.

---

## 6. Audit summary

**Verified live + reachable:** the Workbench mesh panels (via `mde-workbench
--focus/--page`), the Settings▸Network deep-links, the panel mesh-status chip
(after the fix below), `mde files` Network/Cloud panes, `mde connect`/`mde phone`,
the notification surfaces (`mde action-center`/`mde toast` + universal notifyd), and
the Bus priority→surface dispatch.

**Fixed during this audit (2026-06-07):** the panel mesh-status chip launched
`mde workbench` (unknown subcommand → silent no-op) — corrected to `mde-workbench`,
restoring the primary always-visible mesh entry point.

**Known gaps (tracked):** the Send-To file-transfer engine (Bus responders are
honest-empty today); LizardFS master/chunk daemons + auto-mount (E3.2, lands with
the RPM); the unified mesh-first file manager + `demo_data` removal (E10); the
cross-segment mDNS relay's multi-node round-trip + worker-over-Bus listing
(post-release bench).
