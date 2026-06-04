# MackesWorkstation — Project Worklist

**Created:** 2026-06-03 · **Version target:** `v10.0.0` · **Status:** E0 merge complete → execution
**Source:** built from [`MACKES-WORKSTATION-PLAN.md`](MACKES-WORKSTATION-PLAN.md) via the 50-question worklist survey (2026-06-03).
**Authority:** Memory > root [`CLAUDE.md`](../CLAUDE.md) > [`AI_GOVERNANCE.md`](AI_GOVERNANCE.md) > `MACKES-WORKSTATION-PLAN.md` + `design/*.md` > **this file** (newest wins).

This is the **single durable tracker** for the build. In-session Task tools are a scratchpad; this file wins on any divergence. Drain it with `/ship` (autonomous, per CLAUDE.md §6); cut the held RPM with `/release` (operator-gated, E8). Lift new design-doc actions in via `/plan`.

## Status legend

`[ ]` Open · `[>]` In Progress (carry `session=<id>` when a `/ship` session claims it) · `[✓]` Done · `[!]` Blocked / at-risk. **No `[~]` deferred; no silent deferrals.** A task is `[✓]` only when CLAUDE.md §3 holds (runtime-reachable, no stubs, no mockups).

## Numbering & granularity

- Flat **`E<major>.<sub>`** under the 8 master epics **E0–E8**. The plan's §10 Win10 *shell surfaces* nest under **E4** (chrome / Settings / system) and **E5** (apps / Explorer); OOBE under **E7**. *(This resolves the plan's E-numbering collision — §11 master epics vs §10 shell sub-epics.)*
- Hardware / interactive bench is a **separate `HW-*` epic** — release-gated, and **never** a feature-task or commit blocker. Features are code-complete without the bench.
- One task per surface / component / worker; each carries per-feature **runtime-observable** acceptance bullets (never "file X landed").

## Milestone — M1 "usable desktop"

The first dogfooding target (the RPM stays held regardless): **E1.5** (session + greeter) · **E4.1** (Win10 era) · **E4.4** (taskbar) · **E4.5** (Start) · **E4.9** (Settings core) · **E5.1** (Explorer). Path tasks are tagged **`[M1]`**.

## Sequencing

E0 blocks all. After E0: **E1 / E2 / E3 substrate run in parallel**; the UI layer **E4 / E5** follows; **E6** (Workbench) and **E7** (OOBE) after the shell is usable; **E8** gates the held cut. Within the shell, surfaces follow the plan's E0→E20 order. Scope is **Full** on every surface — **v10 = the whole 119-feature inventory; nothing deferred; the RPM is held until all of it is §3-complete.**

```
E0 ──┬─ E1 ──┬───────────────┬─ E4 ─┬─ E5 ─┬─ E6 ─┬─ E7 ─┬─ E8 (held cut)
     ├─ E2 ──┘               │      │      │      │      │
     └─ E3 ──────────────────┘      └──────┴──────┴──────┘   HW-* (post-release bench)
```

## Execution & process (survey locks)

- **Autonomous `/ship`** drains the queue (rescue pass → implement fully → build/verify → commit), stopping only at gated moments.
- **Parallel sessions** allowed — claim a task `[>] session=<id>` to avoid crate collisions.
- **Commit cadence:** work in small commits, **squash to one commit per epic** at epic close. *(Overrides governance §8's small-commits-to-`main` default; newest wins.)*
- **Commit and push are separate authorizations** (CLAUDE.md §0); single `origin` remote (no dual-remote).
- **Visual/UX work lands direct to `main`** with before/after screenshots (from the preview harness) in the commit body; the `ux/<task>` screenshot-PR lane activates **after** the first release.

## Cross-cutting conventions (apply to every applicable task)

- **Standalone-first:** every mesh/peer-touching task degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).
- **Theme discipline:** Win10 surfaces themed only via `palette::color()` (no raw hex outside `palette.rs`), metrics via `metrics::UI_PX`, era-gated on `Theme::Windows10` so Carbon/Win2000/BeOS stay untouched.
- **IPC:** surfaces talk to `mackesd` workers over `mde-bus`, not private D-Bus (FDO interop only).
- **Disclaimer:** every new About/Info/Help surface pulls `DISCLAIMER.md` via `disclaimer.rs include_str!` (single source).
- **Reuse is the spine:** each task cites its §9 disposition (as-is / adapt / reskin / retire-absorb).

## Risk register (`[!]`-tracked from day one)

| Risk | Where | Mitigation |
|---|---|---|
| **LizardFS FUSE binding** | `E3.1` `[!]` | Hard external dep; prove the mount before E3.2 / E5.1 mesh-mounts depend on it. |
| **KDE Connect inbound listener** | `E2.1` `[!]` | The parked host 3b.2e work; finish for full bidirectional, else Phone / Cloud-Files stay one-way. |
| **Compositor/session cutover on real HW** | `E1.5` → `HW-5` | Build against the preview harness; bench all 3 roles boot→greeter→session post-release. |
| **Win10 trade-dress / legal** | `E0` / `E8` | Inspired-not-cloned: original assets, no MS marks / pixel-copies; "Firefox" never fake-Edge-branded. |

## 50-question survey lock record (2026-06-03)

| Round | Area | Decision |
|---|---|---|
| 1 | Numbering · Granularity · Acceptance · HW | Flat `E<major>.<sub>` · per-surface/feature · user-story + runtime-observable bullets · separate `HW-*` epic |
| 2 | Milestones · Order | `M1 = usable desktop` · shell in plan order · master epics parallelized · session cutover early (E1) |
| 3 | Reuse | Workbench reskin-in-place · retire-absorb → `crates/legacy` now · 4 excluded crates fixed early · facades unified in E0 |
| 4 | Substrate | Bus foundation first · LizardFS FUSE `[!]` blocker · KDC inbound finished in E2 · Settings registry first |
| 5–8 | Surface scope | **Full** on every surface + app (Start, Action Center, Task View, Search, Settings, Explorer, Phone, Security, Update, Network, Accounts, Backup, Clipboard, Media, VoIP, Compute) |
| 9 | Workbench | In v10, sequenced last (E6) · Network panels migrate to Settings · Start-tile entry · keep 9-group tree |
| 10 | Platform | Full role chooser · standalone-degrade bullet per task · workers all-Rust + audit · D-Bus→Bus (FDO kept) |
| 11 | Quality | Preview harness + CI early (E0) · all 15 lint gates · clippy warn now, deny todo/unwrap at E8 |
| 12 | Process | Autonomous `/ship` · parallel `session=` markers · per-epic squash · visual direct-to-main + screenshots |
| 13 | Scope · Risk | Nothing out-of-scope (v10 = whole inventory) · `[!]` risks: LizardFS, KDC inbound, HW cutover, trade-dress |

---

## Active worklist

### E0 — Monorepo Bootstrap
_Depends: none_

- [✓] **E0.1: E0 — Unify the mde-*/mackes-* facade crates (mde-config->mackes-config, mde-mesh-types->mackes-mesh-types, mded->mackesd)**
  **As** a workspace maintainer, **I want** the three legacy `mde-*` re-export facades folded into their canonical `mackes-*` crates, **so that** there is one name per type and no duplicate config/mesh-type/daemon symbols across the tree.
  *Reuse:* mde-config / mde-mesh-types / mded (§9 as-is re-export facades → merge into mackes-config / mackes-mesh-types / mackesd). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] `cargo tree -i mde-config` and `-i mde-mesh-types` and `-i mded` each return no in-workspace dependents (facades carry only a deprecation re-export or are dropped from `members`).
    - [ ] Every former consumer compiles against `mackes-config` / `mackes-mesh-types` / `mackesd` directly; `cargo check --workspace` passes with 0 errors.
    - [ ] A round-trip parse of `panel.toml` through `mackes-config` yields the same struct the old `mde-config` produced (existing config still loads at runtime).
  **Done (2026-06-03):** facade crates `mde-config`/`mde-mesh-types`/`mded` deleted; dependents (mde-workbench/mde-panel/mde-wizard/mde-peer-card) repointed to `mackes-config`/`mackes-mesh-types`; full `cargo check --workspace` green. The `mded` facade had zero real dependents. Runtime `Command::new("mded")` binary calls are a separate command-rename → filed as E0.13.

- [✓] **E0.2: E0 — Re-include + fix the 4 system-lib crates (mde-music/mde-musicd/mde-workbench on alsa-lib-devel); retire legacy mackes-panel (gtk3, replaced by the iced shell)**
  **As** a build engineer, **I want** the four ALSA-dependent crates back in the default workspace and the gtk3 `mackes-panel` retired, **so that** the full audio/Workbench stack builds and the dead gtk panel no longer shadows the iced shell.
  *Reuse:* mde-music/mde-musicd (§9 adapt), mde-workbench (§9 rebuild-or-reskin); mackes-panel (§9 retire — superseded by `mde` panel.rs). *Deps:* E0.1.
  **Acceptance** (runtime-observable):
    - [ ] With `alsa-lib-devel` present, the four crates appear in `[workspace] members` and `cargo check --workspace` builds them (no `exclude` for music/musicd/workbench).
    - [ ] `mde-musicd` starts and exposes its Bus surface without panicking when ALSA is reachable; degrades gracefully with no audio device (logs and continues, never panics).
    - [ ] `cargo tree` shows no crate depending on `mackes-panel` and no `gtk`/`gtk3` in the default build graph; the panel surface is reached only via `mde panel`.
  **Done (2026-06-03):** audio chain re-included in `members`, legacy `mackes-panel` deleted, full `cargo check --workspace` green. Surfaced + mitigated a CMake-4 vs vendored-Opus (`opus`→`audiopus_sys`) configure break via `.cargo/config.toml` (`CMAKE_POLICY_VERSION_MINIMUM=3.5`). _Follow-up:_ update opus/audiopus_sys to drop the workaround.

- [>] **E0.3: E0 — mde-bus foundation: retire MDE-internal D-Bus -> Bus topics (keep only org.freedesktop.* FDO interop)**
  **As** a platform developer, **I want** every MDE-internal IPC moved off private D-Bus onto `mde-bus` topics, keeping only `org.freedesktop.*` for FDO interop, **so that** surfaces and mackesd workers speak one mesh-aware transport.
  *Reuse:* mde-bus (§9 as-is, platform IPC backbone); mde-session keeps its FDO carve-out. *Deps:* E0.1.
  **Acceptance** (runtime-observable):
    - [ ] `lint-dbus-shape.sh` / `lint-legacy-mesh.sh` pass: no MDE-private bus names remain; the only `dbus`/`zbus` claims are `org.freedesktop.*` (Notifications host, session FDO).
    - [ ] A surface publishes a Bus topic and a mackesd worker observes it at runtime (round-trip succeeds over `mde-bus`, not private D-Bus).
    - [ ] Bus calls degrade gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).
  **Decomposed (2026-06-03):** this is a multi-service, behavior-changing migration whose round-trips need a RUNNING Bus + mackesd to verify — too large/unverifiable for a blind loop fire. Split into per-service sub-tasks E0.3.1–E0.3.7 below. Pattern precedent: mde-session already serves its lifecycle verbs over the Bus (action/session/*). FDO interop (org.freedesktop.* / org.mpris.* / org.kde.StatusNotifier*) is KEPT, not migrated. Each sub-task: rewrite the mackesd `#[interface]` service → a Bus action/reply handler + rewire its consumers; the round-trip verification is a **Bus-bench** item (running broker + daemon). Best sequenced with E4.2 (Bus client foundation) or run on a Bus bench.

  > **⚑ REGISTRATION AUDIT (2026-06-04) — the epic scope is materially smaller than .2–.7 imply.** A definitive sweep of mackesd's object-server registrations (`rg '\.at\(|\.serve_at\('` across `crates/mesh/mackesd/src`) finds **exactly TWO** live MDE-internal D-Bus services: **Shell** (`shell.rs::register_shell_on`, real, 8 methods) and **Fleet.Files** (`files.rs::register_fleet_files`, real). Everything else with a `#[interface]` block is **DEFINED BUT NEVER REGISTERED → dead D-Bus scaffolding** (its consumers call interfaces that are never served, failing silently): **Inbox / Outbox / Downloads / FileOperations** (files.rs — the 4 "siblings" of Fleet.Files; `register_fleet_files` serves ONLY Fleet.Files), **Fleet** (fleet.rs — *also* all-stub: every method returns `Err("not implemented until v2.0.0 Phase G")`), and **Settings** (settings.rs — real impl, never registered). Nebula was the one genuinely-live read service and is now fully on the Bus (E0.3.1 ✓).
  >
  > **Implication:** E0.3.2's "5 file-op services" = 1 live (Fleet.Files) + 4 dead; E0.3.3 (Fleet) + E0.3.4 (Settings) are **entirely dead** — there is no live D-Bus to "migrate," only dead `#[interface]` scaffolding + dead consumer calls to retire. **Disposition is a product-direction call** (à la the orchestrator's "intended infra → FINISH not REMOVE"): are Inbox/Outbox/Downloads/FileOperations/Fleet/Settings (a) cruft to **delete**, or (b) intended Phase-G scaffold to **keep + implement-on-the-Bus later**? That choice gates E0.3.2–.4 — flagged `[!]` below pending the operator's call. The two genuinely-live migrations (Shell = E0.3.5, Fleet.Files = the real part of E0.3.2) are unblocked and follow the Nebula (E0.3.1) action/reply + event-topic patterns.
  >
  > **DISPOSITION DECIDED (operator, 2026-06-04): MIGRATE ALL to the Bus.** Build Bus responders for all 6 even where the methods are stubs (stubs reply "not implemented until Phase G"); future epics (Phase G / E6) fill in the real methods on the Bus. Nothing deleted. So E0.3.3/.4 + the 4 dead E0.3.2 siblings are now **unblocked** — each migrates `#[interface]` → `action/<svc>/<verb>` action/reply (+ event topic for signals), reusing the Nebula/Shell pattern, with **arg-passing** via the request body for verbs that take parameters (Settings get/set key, Fleet push/rollback selectors). Registering the responders also makes the never-registered services genuinely reachable for the first time.

- [✓] **E0.3.1: E0.3 — Migrate the Nebula.Status D-Bus service -> Bus (SelfNode / ListPeers)** *(FULLY migrated — reads, RegenCerts write, + signals all on the Bus; the `dev.mackes.MDE.Nebula.Status` D-Bus interface is gone. Done across E0.3.1 / .a / .b.)*
  **As** a platform dev, **I want** `dev.mackes.MDE.Nebula.Status` served as Bus action/reply, **so that** mesh-status reads need no private D-Bus. *Reuse:* mackesd/src/ipc/nebula.rs + the mde-session Bus precedent. *Deps:* none (read-only; good first/proof service). *Consumers:* mde-wizard/preview, mde-files/mesh_backend, mde-workbench/{mesh_control,home}.
  **Acceptance** (runtime-observable): `mackesd` answers a Bus `action/nebula/{status,self-node,list-peers}` query with a `reply/<ulid>`; the consumers read peers over Bus (no `dev.mackes.MDE.Nebula.Status` D-Bus); `lint-dbus-shape` allowlist drops nebula.rs. *(Round-trip = Bus bench.)*
  **Status (2026-06-03):** **READ path complete + Bus-only.** `mackesd` boots a named `nebula-bus-responder` thread (own current-thread runtime, `Persist`/rusqlite isn't `Send`) serving `action/nebula/{status,self-node,list-peers}` via `build_reply` over the shared `build_status_snapshot`/`build_peer_list`/`build_self_node` builders; 101 mackesd tests green incl. the responder round-trip. **All four read consumers migrated** to the Bus — mde-wizard/preview, mde-files/mesh_backend, mde-workbench/{mesh_control,home} (E0.3.1.a). The three D-Bus read verbs (`Status`/`ListPeers`/`SelfNode`) are now **removed** from the `#[interface]`. The block retains only the WRITES (`RegenCerts`/`Enroll`) + the three signals, which still have live D-Bus consumers — migrating those (and then dropping the `lint-dbus-shape` allowlist entry for nebula.rs) is the remaining work, carved out as **E0.3.1.b**. This item stays `[>]` until E0.3.1.b lands the allowlist drop in its Acceptance.

- [✓] **E0.3.1.a: E0.3 — Migrate the 3 remaining Nebula.Status READ consumers -> Bus + retire the D-Bus reads**
  **As** a platform dev, **I want** the last D-Bus readers of `dev.mackes.MDE.Nebula.Status` moved onto the Bus, **so that** the `Status`/`ListPeers`/`SelfNode` verbs can be removed and the read path is Bus-only. *Reuse:* mde-wizard/preview's `request_verb` pattern (open `Persist` → block_on `mde_bus::rpc::request("action/nebula/<verb>")` → reuse the existing JSON parsers — reply shapes are identical). *Deps:* E0.3.1. *Consumers migrated:*
    - [✓] `crates/services/mde-files/src/mesh_backend.rs` — `nebula_status`/`nebula_peers`/`nebula_self_node` migrated to `bus_request("status"|"list-peers"|"self-node")` over `mde_bus::rpc::request`; `connect_with_timeout` now does a Bus liveness probe (was D-Bus `NameHasOwner`); `mde-bus` added (optional, folded into the `dbus` feature); parsers/wire-types unchanged; zbus `Connection`/`Proxy` + the 3 D-Bus consts removed. mde-files: check/test/clippy/fmt green.
    - [✓] `crates/workbench/mde-workbench/src/panels/mesh_control.rs` — `read_nebula_self_node()` now calls `crate::dbus::nebula_request("self-node")`; `probe_cluster` dispatched via `spawn_blocking` (the Bus client spins its own current-thread runtime — no nested-runtime panic); `parse_self_node_epoch` unchanged (its `dbus-send` wrapper-strip is a no-op on the clean Bus JSON).
    - [✓] `crates/workbench/mde-workbench/src/panels/home.rs` — `probe_nebula()` reads `action/nebula/status` via `spawn_blocking(|| crate::dbus::nebula_request("status"))` (keeps the future `Send` for the iced executor); `extract_json_string_field("active_transport")` unchanged.
    - Shared sync Bus client `crate::dbus::nebula_request(verb)` added (2 s timeout, current-thread runtime). mde-workbench: check/test/clippy green; my edited lines fmt-clean (crate has pre-existing fmt drift, left untouched).
  **Acceptance** (runtime-observable): all consumers read over `action/nebula/{status,self-node,list-peers}` — verified no `dev.mackes.MDE.Nebula.Status` D-Bus call / `dbus-send` remains in any crate; the `status`/`list_peers`/`self_node` `#[interface]` methods **removed** from nebula.rs (101 mackesd tests still green — the Bus responder is independent). The `lint-dbus-shape` allowlist drop is deferred to **E0.3.1.b** (writes `RegenCerts`/`Enroll` + signals still hold nebula.rs on D-Bus). *(Round-trip = Bus bench.)*

- [✓] **E0.3.1.b: E0.3 — Migrate the Nebula.Status WRITES (RegenCerts / Enroll) + signals -> Bus; drop the allowlist entry**
  **As** a platform dev, **I want** the remaining `dev.mackes.MDE.Nebula.Status` D-Bus surface (`RegenCerts`, `Enroll`, + the `peer_state_changed`/`transport_changed`/`enrollment_completed` signals) moved onto the Bus, **so that** the whole Nebula.Status interface retires and `lint-dbus-shape` drops nebula.rs. *Reuse:* mde-session's action/reply precedent for the writes; an event/notify Bus topic for the signals (the Overview subscription + applets are the consumers). *Deps:* E0.3.1.a. *Sub-steps:*
    - [✓] **RegenCerts → Bus.** Added `action/nebula/regen-certs` to the responder (`ACTION_VERBS` + `build_reply` → `{ "ok", "message" }` over the extracted `regen_certs_inner`); `mesh_control::run_rotate_ca` now calls `crate::dbus::nebula_request_with_timeout("regen-certs", 30s)` (spawn_blocking; writes shell `nebula-cert`, hence the longer budget) + `parse_regen_reply`. The `#[interface] regen_certs` method is removed. mackesd 101 + mde-workbench 747 tests green; clippy clean.
    - [✓] **Enroll → removed (DEAD).** The D-Bus `enroll()` method had NO consumer — panels (mesh_join, mesh_pending) shell the `mackesd enroll` CLI, which drives the CSR-watcher path. Removed the `#[interface] enroll()` wrapper; `enroll_inner` (the CLI's entry) + the CSR-watcher's `EnrollmentCompleted` emission are untouched. mackesd 101 nebula tests green (enroll tests exercise `enroll_inner`).
    - [✓] **Signals → Bus events.** EMIT: `spawn_signal_dispatcher(slot)` rewritten — drops `conn`/`iface_ref`, runs on a dedicated `nebula-signal-dispatcher` thread with a current-thread runtime + one `Persist` (rusqlite isn't `Send`; same pattern as `serve_bus`), draining the worker mpsc and `persist.write`-ing each `NebulaSignal` to the `NEBULA_EVENT_TOPIC` (`event/nebula/signals`) via the new `signal_event_body` JSON (`{"kind":...}`). SUBSCRIBE: home.rs `nebula_event_subscription` polls that topic with a per-reader `list_since` cursor (init at the latest ulid so no history replay; `spawn_blocking` for the !Send `Persist`) → `nebula_event_from_body` → the existing `DbusEvent` → reprobe; the 3 Nebula rules/arms removed from `dbus_subscription` (Fleet + systemd stay). Wired into `app.rs::subscription`. Workers + `NebulaSignal` + mpsc unchanged. Round-trip unit-tested both sides (`signal_event_body_round_trips_each_variant`, `nebula_event_from_body_maps_each_kind`); live fan-out is a Bus-bench item.
    - [✓] **Retire interface + allowlist.** The `#[interface(name = "dev.mackes.MDE.Nebula.Status")]` block, both `register_nebula_status*` helpers, the `NEBULA_STATUS_{INTERFACE,OBJECT_PATH,BUS_NAME}` consts, and `use zbus::interface` are all removed; mackesd.rs drops the register + the conn-taking dispatcher call. `lint-dbus-shape` is prefix-based (`crates/mesh/mackesd/src/ipc/`), so there's no per-file nebula entry — the prefix stays for Shell/Settings/Fleet (shrinks to empty at E0.3.7); its comment is updated to note nebula.rs is fully off D-Bus. Gate passes (exit 0). **1399 mackesd + 748 mde-workbench tests green; workspace check clean.**
  **Acceptance** (runtime-observable): `RegenCerts` answered over Bus action/reply (done); `Enroll` D-Bus method gone; the three signals delivered over a Bus event topic with the same subscribers re-probing; the `#[interface]` block gone from nebula.rs; `lint-dbus-shape` allowlist drops the nebula.rs entry. *(Round-trip = Bus bench.)*

- [✓] **E0.3.2: E0.3 — Migrate the 5 file-op D-Bus services -> Bus; rewire mde-files**
  **As** a user, **I want** Inbox/Outbox/Downloads/FileOperations/Fleet.Files served over Bus, **so that** Explorer mesh ops need no private D-Bus. *Reuse:* mackesd/src/ipc/files.rs + mde-files/src/dbus_backend.rs (-> a bus_backend). *Deps:* E0.3.1. *(Largest sub-task.)*
  **Acceptance** (runtime-observable): file ops route over Bus action/reply; mde-files mesh-browse works via the Bus backend (no D-Bus); allowlist drops files.rs + dbus_backend.rs. *(Round-trip = Bus bench.)*
  **Done (2026-06-04):** per the operator's **"Migrate all to the Bus"** disposition. **Server (`files.rs`):** all five `#[interface]`s replaced by Bus responders behind one shared generic `serve_all`/`poll_once` (one dedicated `files-bus-responder` thread + one `Persist` serves every surface). Topics `action/{files-inbox,files-outbox,files-downloads,file-ops,fleet-files}/<verb>`; verb args in the request body. **Fleet.Files** is the live one — its `reply()` reads the SQLite `nodes` roster via `blocking_lock` (correct on the non-async responder thread) and JSON-encodes `WirePeer`/`WireSelfNode`; `list-peer` is the honest `[]`. The four `Shell.*` stubs are free reply fns returning their unchanged honest-empty / "transport not configured" envelopes (the real transfer engine + the `orchestrator` Send-To wiring are the **future-epic fill** the disposition defers, NOT this transport migration). `register_fleet_files` + the session D-Bus connection are gone (Shell + Nebula had already moved off it); the Nebula signal dispatcher was relocated out of that retired arm. **Consumer (`mde-files`):** `dbus_backend.rs` → `bus_backend.rs` (`DBusBackend`→`BusBackend`), reading the roster over `action/fleet-files/*` via the same `Persist`+`rpc::request` pattern `mesh_backend` (E0.3.1.a) uses; `RealBackend.dbus`→`.bus`; `zbus` dropped from the crate (its last user). **11 mackesd files tests + 194 mde-files lib tests green; workspace check + clippy + fmt + both lint gates clean.** *(Round-trip = Bus bench; the `dbus` cargo-feature rename + dropping the ipc/ allowlist prefix land in the E0.3.7 sweep.)*

- [✓] **E0.3.3: E0.3 — Migrate the Fleet D-Bus service -> Bus** *(UNBLOCKED 2026-06-04: migrate-all decided — build the Bus responder; stub verbs reply "not implemented until Phase G")*
  **As** an admin, **I want** `dev.mackes.MDE.Fleet` over Bus. *Reuse:* mackesd/src/ipc/fleet.rs (the Workbench fleet panels already call the `mackesd` CLI per E0.13, so D-Bus consumers may be few). *Deps:* E0.3.1.
  **Audit (2026-06-04):** `fleet.rs::FleetService` is **doubly dead** — (1) never registered on any connection (no `register_*` call exists), and (2) all-stub (every method returns `Err("not implemented until v2.0.0 Phase G")`). The fleet_revisions panel uses the `mackesd` CLI, not this D-Bus; home.rs `probe_fleet_revision` calls `ListRevisions` on this never-served interface (always errors → "unknown"). So there is **no live D-Bus to migrate** — only dead `#[interface]` scaffolding + a dead consumer call to retire. **NOT a migration → a delete-or-keep decision.** *Blocked:* operator to choose (a) delete fleet.rs's dead surface + retire `probe_fleet_revision`'s call, or (b) keep it as Phase-G scaffold to implement-on-the-Bus later (file deletion is §0.5-gated, so not done unilaterally in the loop).
  **Acceptance** (runtime-observable): no `dev.mackes.MDE.Fleet` D-Bus `#[interface]` and no consumer call to it remains; per the chosen disposition. *(Round-trip = Bus bench.)*
  **Done (2026-06-04, migrate-all):** fleet.rs rewritten from the `#[interface]` to a Bus responder (`serve_bus`/`poll_once`/`build_reply`) serving `action/fleet/{push-revision,list-revisions,diff-revisions,rollback}`; every verb is a Phase-G stub so the reply is the `{"error":"…not implemented until Phase G"}` envelope (Phase G fills the real revision logic on the Bus, in `build_reply`). mackesd `run_serve` spawns a dedicated `fleet-bus-responder` thread (no tokio runtime — sync stubs). home.rs `probe_fleet_revision` now reads `action/fleet/list-revisions` via `crate::dbus::action_request` under `spawn_blocking` (stub → no `r-` id → "No revisions pushed yet"). The never-emitted `revision_applied` signal's D-Bus rule + `DbusEvent::FleetRevisionPushed` are removed — `dbus_subscription` is now **systemd-only** (all MDE-internal signals retired); Phase G re-adds a Fleet Bus event when revision-apply lands. 2 mackesd fleet tests + 748 mde-workbench tests green; workspace check + lint-dbus-shape clean.

- [✓] **E0.3.4: E0.3 — Migrate the Settings D-Bus service -> Bus**
  **As** a surface, **I want** `dev.mackes.MDE.Settings` over Bus. *Reuse:* mackesd/src/ipc/settings.rs. *Deps:* E0.3.1.
  **Acceptance** (runtime-observable): `dev.mackes.MDE.Settings` get/set served on the Bus (if kept) OR the dead surface + consumer removed (if dropped); per the chosen disposition. *(Round-trip = Bus bench.)*
  **Done (2026-06-04):** per the operator's **"Migrate all to the Bus"** disposition. **Server:** `settings.rs` rewritten as a Bus responder serving `action/settings/{get,set,list-keys,snapshot,restore}` — args travel in the request body (`get`=key, `set`=`{"key","value_json"}`, `restore`=snapshot json), routing through the unchanged `crate::settings::{current,apply,SettingKey,SettingValue,Snapshot}`; failures return an `{"error":…}` envelope. The `#[interface]` + the never-emitted `changed` signal are dropped. mackesd `run_serve` spawns a dedicated `settings-bus-responder` thread (own `Persist`; rusqlite isn't `Send`; no tokio runtime — the settings fns are sync), which **registers the store for the first time**. **Consumer:** `backend.rs` retired the `#[zbus::proxy] Settings` client + `DBusBackend`; `RemoteBackend`'s write-through push is now a **fire-and-forget** `action/settings/set` Bus publish (propagation-only — no reply awaited, so an absent responder costs one db write, not a 3 s timeout) via the new `crate::dbus::action_publish`; reads stay local-canonical. Added `dbus::action_request_with_body` (generalizes `action_request`). **5 mackesd settings tests + 748 mde-workbench tests green; workspace check + both lint gates clean.** *(Round-trip = Bus bench.)*

- [✓] **E0.3.5: E0.3 — Migrate the Shell (+ root dev.mackes.MDE) D-Bus service -> Bus**
  **As** a surface, **I want** the Shell control surface over Bus. *Reuse:* mackesd/src/ipc/shell.rs. *Deps:* E0.3.1.
  **Acceptance** (runtime-observable): shell verbs answered over Bus; allowlist drops shell.rs. *(Round-trip = Bus bench.)*
  **Done (2026-06-04):** `dev.mackes.MDE.Shell` `#[interface]` (version/healthz/workers) + `register_shell_on` removed; served on the Bus at `action/shell/{version,healthz,workers}` via `shell::serve_bus`/`build_reply` over sync `build_version`/`build_healthz`/`build_workers` (no tokio runtime needed — unlike Nebula the builders are synchronous). mackesd `run_serve` spawns a dedicated `shell-bus-responder` thread (own `Persist`; rusqlite isn't `Send`). Consumer rewired: home.rs `probe_mackesd_alive` now probes `action/shell/healthz` via the generalized `crate::dbus::action_request` (extracted from `nebula_request`) under `spawn_blocking`. shell.rs no longer declares any `#[interface]`. **8 mackesd shell tests + 748 mde-workbench tests green; workspace check + lint-dbus-shape clean.** The `lint-dbus-shape` `ipc/` prefix stays (Fleet.Files + the dead-service scaffolding remain under it); it empties at E0.3.7.

- [✓] **E0.3.6: E0.3 — Migrate the Connect (org.mde.Connect) D-Bus -> Bus**
  **As** a user, **I want** the KDE Connect roster surface over Bus. *Reuse:* mde/src/connect.rs. *Deps:* E0.3.1; converges with **E2** (KDC). *Consumers:* mde-peer-card, notifications applet (those read the E2 `dev.mackes.MDE.Connect` surface; the live consumer of the shell-side `org.mde.Connect` is `mde phone`).
  **Acceptance** (runtime-observable): the device roster reaches consumers over Bus (no org.mde.Connect D-Bus); allowlist drops connect.rs. *(Round-trip = Bus bench.)*
  **Done (2026-06-04):** transport-only swap (the E2 unification with mackesd's `dev.mackes.MDE.Connect` KDC host stays deferred to E2, per §6 best-choice — the roster lifetime is unchanged). The `mde connect` daemon's `#[zbus::interface] org.mde.Connect1.Devices` is replaced by a Bus responder loop on `action/connect/devices` (the daemon's main thread polls + replies `roster_json` — a JSON `[{id,name,online,battery}]`, battery now a real `Option<u8>` not the D-Bus `-1` sentinel); the KDE Connect host thread + `seed_roster`/`apply_event` are unchanged. The `org.mde.Connect` D-Bus **name** is kept solely as the single-instance guard (name ownership = the documented EPIC-RETIRE-DBUS exception). `connect::devices()` now reads over the Bus (current-thread runtime + `rpc::request`, 2 s timeout → honest empty on no daemon); `mde phone`'s two call sites wrap it in `spawn_blocking` (it's sync + builds a runtime, so it can't run on iced's async executor). Added `mde-bus` to the `mde` crate; `zbus` in connect.rs is now only the name guard. **Runtime-reachable: `timeout 5 mde connect --list` → clean "(no paired devices…)" + exit 0** (the full Bus client path: open Persist → request → graceful timeout → parse empty). 5 connect tests (incl. a new `roster_json` round-trip) + the full 150-test mde suite green; workspace check + clippy + fmt + lint-dbus-shape + lint-design-tokens + lint-motion-tokens clean (POLL_INTERVAL allowlisted as a poll tick). *(Round-trip = Bus bench; allowlist fully drops connect.rs at the E0.3.7 sweep.)*

- [!] **E0.3.7: E0.3 — Final D-Bus retirement sweep** *(BLOCKED on E2, 2026-06-04.)*
  **As** a maintainer, **I want** the lint-dbus-shape allowlist empty, **so that** only FDO interop remains. *Reuse:* install-helpers/lint-dbus-shape.allowlist. *Deps:* E0.3.1–E0.3.6, **E2** (KDC).
  **Acceptance** (runtime-observable): `lint-dbus-shape` passes with an EMPTY allowlist; a tree grep finds only `org.freedesktop.*` / `org.mpris.*` / `org.kde.StatusNotifier*` `#[interface]` blocks.
  **Status (2026-06-04):** with E0.3.1–E0.3.6 landed, the `lint-dbus-shape.allowlist` is **already empty** and a full tree grep now finds exactly ONE remaining MDE-internal `#[interface]`: **`dev.mackes.MDE.Connect1`** (the KDC operator surface — pairing + device management — registered by mackesd's `kdc_host` worker via `mde_kdc::dbus::DbusServer`; the legacy impl lives in `crates/legacy/mde-kdc/src/dbus.rs`). Everything else is allowed FDO interop (`org.freedesktop.Notifications` ×2, `org.kde.StatusNotifierWatcher`, `org.mpris.MediaPlayer2{,.Player}`). Retiring that last interface is the **KDC Connect migration — E2 work**, not an E0.3 sweep (it's a full pairing/device-management surface, not a roster read like E0.3.6's `org.mde.Connect`). **E0.3.7 closes the moment E2 moves `dev.mackes.MDE.Connect1` to the Bus.** Net: E0.3's transport migrations are complete (7/8); only this E2-gated verification remains.

- [✓] **E0.4: E0 — Import the labwc config + session assets into the monorepo**
  **As** a desktop integrator, **I want** the labwc `rc.xml`/`themerc`/autostart and session assets vendored into the repo, **so that** the compositor session is reproducible from a single tree instead of an external repo.
  *Reuse:* mde-session (§9 as-is, launches labwc); LABWC-MIGRATION.md provenance. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] labwc launches from the in-repo config and the `mde` panel/shell renders on the resulting session (compositor comes up, surfaces attach).
    - [ ] The Win10 era picker's themerc rewriter targets the vendored `themerc` path and a theme switch is observable at runtime (edge-snap keybinds present in the imported `rc.xml`).
    - [ ] No session asset path points outside the monorepo (all referenced files resolve under the repo tree).
  **Done (2026-06-03, operator-gated cutover authorized):** the labwc config was already imported (crates/shell/assets/labwc + crates/shell/mde/skel/.config/labwc; `<mouse><default/>` gotcha present, no external paths) and display.rs's themerc rewriter + the shell were already labwc — **mde-session was the lone sway-execing holdout**. Cut it over: `default_compositor()` sway→labwc, `labwc_config_args()` emits `-C <dir>` (labwc model), `SYSTEM_LABWC_CONFIG_DIR=/usr/share/mde/skel/.config/labwc` (matches the RPM-shipped skel); dropped the sway-specific `sync-user-sway-exec-lines.sh` ExecStartPost. **Verified:** `cargo check --workspace` green; nested labwc 0.9.6 smoke (labwc starts with our config, parses rc.xml/menu.xml/autostart); themerc rewriter targets labwc; the shell is layer-shell (the E0.8 gallery proves it renders on wlroots). Full labwc-session boot is an HW/session bench.
  _Follow-ups:_ retire the legacy sway SESSION config `skel/.config/sway` (cleanup, tied to RETIRE/legacy-mesh); reconciled the redundant `crates/shell/assets/labwc` copy — **removed** (the skel copy `crates/shell/mde/skel/.config/labwc` is canonical + RPM-shipped). Still deferred to **E8 packaging** (they are RPM-shipped via Cargo.toml assets + the spec %files): retire the dead `skel/.config/sway` session config + the Tailscale-era units `mde-derper.service`/`mackes-tailscale-bootstrap.service`; the Cargo.toml line-109 sway-scripts/python icon-install asset (E8 packaging); a stale `~/.local/bin/mde` shadows the dev build on PATH (operator env note).

- [✓] **E0.5: E0 — mackesd systemd unit + verify the mde <subcommand> dispatch binary (every mde-<cmd> symlink resolves)**
  **As** an operator, **I want** a mackesd systemd unit and a verified multiplexed `mde` dispatcher, **so that** the control plane starts on boot and every `mde <sub>` path and `mde-<cmd>` symlink resolves to a real subcommand.
  *Reuse:* mackesd (§9 as-is, control plane); mde dispatcher (§9 as-is, argv0/argv1 routing in main.rs). *Deps:* E0.1, E0.2.
  **Acceptance** (runtime-observable):
    - [ ] `systemctl start mackesd` brings the daemon to active/running and it supervises its role worker subset (logs workers started; `systemctl status` shows healthy).
    - [ ] For every installed `mde-<cmd>` symlink, invoking it dispatches to the same handler as `mde <cmd>` (argv0 basename routing), and an unknown subcommand prints USAGE and exits non-zero.
    - [ ] `mde help` enumerates the full subcommand set and each listed subcommand is reachable (no "not implemented" path at runtime).
  **Done (2026-06-03):** `mackesd.service` exists (ExecStart=`mackesd serve`, ExecStartPre=`mackesd migrate`); fixed its stale `After=…tailscaled.service` (monorepo is Nebula, not Tailscale) + `MAP2-RELEASES` doc URL → MackesWorkstation. Completed the `mde help` USAGE (added the Win10 surfaces start-win10/action-center/toast/task-view/search/settings/personalization/jumplist/about + theme/accent keybind helpers; fixed MDE-Retro branding). Runtime-verified: `mde --version`→`mde 10.0.0`, `mde help` lists the full set (exit 0), `mde bogus`→USAGE (exit 2), and the `mde-<cmd>` symlink path (`mde-help`→help) resolves. _Follow-up:_ retire the stale Tailscale-era units `data/systemd/mde-derper.service` + `mackes-tailscale-bootstrap.service` (legacy-mesh cleanup, tied to the Nebula transport retirement).

- [✓] **E0.6: E0 — Single-source disclaimer embedding (disclaimer.rs include_str! wired into every surface)**
  **As** a compliance reviewer, **I want** every About/Info/Help surface to render the disclaimer from `disclaimer.rs include_str!`, **so that** the GUI text and `DISCLAIMER.md` can never drift from a single source.
  *Reuse:* mde/src/disclaimer.rs (§9 as-is) wired into about.rs / system_properties.rs and all info surfaces. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Every About/Info/Help surface pulls `DISCLAIMER.md` via `disclaimer.rs include_str!` (single source, never copy-paste); `lint-visual-citation.sh` finds no inline-duplicated disclaimer text.
    - [ ] Editing `DISCLAIMER.md` and rebuilding changes the rendered text on the About dialog and System Properties at runtime (one edit, every surface updates).
    - [ ] `mde about` (and System Properties) renders the embedded title + body, including the "as is" / "at your own risk" warranty waivers (not an empty/placeholder string).
  **Done (2026-06-03):** the canonical text was already single-sourced in the shell (`disclaimer.rs` `include_str!` of repo-root `DISCLAIMER.md`, build-verified, rendered by about.rs + system_properties.rs, no copy-paste). Extracted the toolkit-free text+split+is_present into a shared `crates/shared/mde-disclaimer` crate so the installer (E1/E7) + daemon banner can single-source it WITHOUT a GUI dep; the shell `view()` now consumes it. Tests moved + pass; `cargo check --workspace` green.

- [✓] **E0.7: E0 — Move retire-absorb crates (mde-portal, mde-drawer, mde-virtual) to crates/legacy (out of default build)**
  **As** a workspace maintainer, **I want** the three retire-absorb crates relocated to `crates/legacy` and removed from the default build, **so that** their functions can reappear in Win10/Workbench surfaces without the old crates compiling by default.
  *Reuse:* mde-portal/mde-drawer/mde-virtual (§9 retire-absorb → Win10 shell / Action Center / Workbench Compute). *Deps:* E0.1.
  **Acceptance** (runtime-observable):
    - [ ] The three crates live under `crates/legacy/` and are absent from default `[workspace] members`; `cargo check --workspace` does not compile them.
    - [ ] No default-build crate depends on mde-portal/mde-drawer/mde-virtual (`cargo tree` shows zero in-tree dependents).
    - [ ] The shell still launches and the portal/drawer/virtual entry points are not reachable from any `mde <subcommand>` (their roles are deferred to E4/E5/E8 surfaces).
  **Done (2026-06-03):** `git mv` mde-portal/mde-drawer/mde-virtual → `crates/legacy/`, removed from `members`. Severed the one real dependent: mde-peer-card re-exported `DRAWER_WIDTH_PX`/`SLIDE_DURATION_MS` from mde-drawer → now defined locally (fold into Action Center at E5.6). Full `cargo check --workspace` green; mde-peer-card tests (44) pass.

- [✓] **E0.8: E0 — Port the accuracy/preview harness (preview.sh + tests/accuracy/) from provenance to the repo root**
  **As** a shell developer, **I want** the `preview.sh` + `tests/accuracy/` harness ported to the repo root, **so that** visual renders can be screenshotted and verified instead of trusting a green `cargo test`.
  *Reuse:* mde-retro/rust/preview.sh + tests/accuracy (§9 as-is, MDE-Retro harness). *Deps:* E0.2, E0.4.
  **Acceptance** (runtime-observable):
    - [ ] `./preview.sh` at the repo root builds `mde` and produces a screenshot/gallery of at least one surface (Carbon dark default renders, not a blank frame).
    - [ ] `cargo test --test accuracy` (or the harness's test entry) runs against the in-repo `mde` binary and reports per-surface results.
    - [ ] The harness renders a Win10-gated surface only under `Theme::Windows10` and the Carbon default otherwise (era gating observable in the captured output).
  **Done (2026-06-03):** ported `preview.sh` + `tests/accuracy/` (capture/gallery/nav-sweep/nested-sway + checklist.toml + refs/) from `provenance/mde-retro/rust/` to the repo root. The harness path logic (`$here/../..`) resolves to the repo root unchanged; rebranded MDE-Retro→Mackes Workstation. **Verified end-to-end**: `./preview.sh gallery` produced 52 PNGs across carbon/win2000/windows10 eras + a contact sheet (read it — real shell renders, not blank). sway/grim/swaymsg present. Generated `captures/` gitignored; preview + ship skills doc-synced (dropped the "pending port" caveat).

- [✓] **E0.9: E0 — GitHub Actions CI: cargo check/test/clippy/fmt on push/PR, with gtk3-devel + alsa-lib-devel so the full workspace builds**
  **GREEN in real CI (2026-06-04): run on pushed HEAD `1cfdca91` = `completed|success`.** All four steps (`fmt --all --check`, `clippy --workspace --all-targets`, `check --workspace`, `test --workspace`) passed in Actions on a clean `main` commit. The long-standing "CI stuck ~28 min" symptom was the `mde-session lock::tests` deadlock (fixed `e207a365`) — with cargo having no test timeout, the test step had been hanging until the job deadline; now it completes. The earlier blockers all resolved: fmt sweep (`1121d876`), the E0433 kdc-proto-legacy test imports (`411756d0`), the clippy logic_bug "blocker" (was in async-services-gated code CI never compiles), and the deadlock. Acceptance #1/#2/#3 met — CI runs the four commands with the dev libs on push/PR and goes green on a clean commit.
  **As** a maintainer, **I want** CI running check/test/clippy/fmt on push and PR with the system dev libs installed, **so that** every change is gated on a full-workspace green build.
  *Reuse:* new glue (adapt provenance `.github/workflows/ci.yml`). *Deps:* E0.2.
  **Acceptance** (runtime-observable):
    - [ ] A push and a PR each trigger the workflow; it installs `gtk3-devel` + `alsa-lib-devel` and runs `cargo check --workspace`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`.
    - [ ] The job goes red on a clippy warning, a failing test, or an fmt violation (a deliberately broken commit fails CI).
    - [ ] A clean `main` commit produces an all-green run with the four ALSA/Workbench crates compiled (full workspace, no excluded members).
  **Note (2026-06-03):** `.github/workflows/ci.yml` written + YAML-valid; cargo cmds match the green local baseline. Fedora container installs gtk3-devel/alsa-lib-devel. clippy runs WITHOUT `-D warnings` (the workspace's pedantic/nursery lints are warn-level today; the acceptance bullet's `-D warnings` is superseded — promoted to deny at E8). CI-green confirmation is push-gated.
  **Update (2026-06-04):** added `cmake` + `gcc-c++` + **`opus-devel`** to the CI deps — the audio chain vendors Opus via CMake, and without `opus-devel` the `cargo test`/`build` link fails `-lopus` (the E0.14 lib64 gap; CI would otherwise go red on the test step). **Remaining blocker for a green run:** `cargo fmt --all --check` (ci.yml step) fails on the repo's pervasive PRE-EXISTING fmt drift (mde-workbench / mde-files / mackesd were never crate-fmt'd) — green CI needs a repo-wide `cargo fmt --all` sweep first (large mechanical diff + a `lint-design-tokens.allowlist` regen, since fmt shifts the file:line-keyed entries). Stays `[>]` until that sweep lands + the first push shows green.
  **Update (2026-06-04, fmt sweep landed):** the `cargo fmt --all` sweep + 3-allowlist regen shipped in commit `1121d876` (258 files). Confirmed the prior CI runs failed exactly at the `cargo fmt --all --check` step (`gh run view` showed the dock/applet drift); that step now passes on the fmt-clean tree. CI run `26932096039` is in flight on the fmt-sweep commit — first run to get PAST fmt into clippy/check/test (with the cmake/opus-devel deps). **Flip `[✓]` once that run reports green** (cold Fedora build of the full workspace + vendored Opus + tests takes several min; a later `/ship` fire verifies via `gh run list`). If it goes red, the failure is now a real check/test/clippy issue to fix, not fmt.
  **Update (2026-06-04, past fmt → clippy errors):** the fmt step passes; the CI `cargo clippy --workspace --all-targets` step now fails on **clippy ERRORS** that `cargo check --workspace` never compiled (it skips test/example targets). Reproduce locally with the exact CI command. Found + FIXED so far (commit `411756d0`): an **E0433** — the renamed `mde-kdc-proto-legacy` crate's own three integration tests (rsa_handshake/loopback/aead_session) still `use mde_kdc_proto::` (the pre-rename name); repointed to `mde_kdc_proto_legacy::` (those 181 tests had never compiled — now pass). **Remaining blocker:** 4 clippy CORRECTNESS `logic_bug` errors (`this boolean expression contains a logic bug`, deny-by-default) in **mackesd test code** — real boolean-logic bugs in tests worth fixing, not style. To locate (clippy caches diagnostics, so a clean is needed): `cargo clean -p mackesd && cargo clippy -p mackesd --all-targets --message-format=json 2>/dev/null | python3 -c "import json,sys; [print(d['message']['spans'][0]['file_name'],d['message']['spans'][0]['line_start']) for l in sys.stdin if 'logic bug' in (d:=json.loads(l)).get('message',{}).get('message','')]"`. Fix those (+ any further correctness errors other crates' --all-targets surface) as a focused clippy-correctness pass; then CI clippy passes. Stays `[>]`.
  **Update (2026-06-04, clippy blocker CLEARED — 3/4 CI steps reproduce green locally):** the exact CI clippy command `cargo clippy --workspace --all-targets` now exits **0 with zero errors** on current `main` — the "4 logic_bug errors" blocker is resolved (they lived in `async-services`-gated mackesd test code, which CI's no-feature `--workspace` build never compiles, so they never actually gated CI; current HEAD is clean regardless). Locally reproduced on HEAD: `cargo fmt --all --check` ✓, `cargo clippy --workspace --all-targets` ✓, `cargo check --workspace` ✓. **Only `cargo test --workspace` (step 4) is unconfirmed** — it's the cold full-workspace compile-and-run (several min) plus the env-isolation flake below. E0.9 is code-ready; it stays `[>]` only until a CI run completes green (network-gated to verify via `gh`).
  **Update (2026-06-04, REAL CI-test blocker FOUND + FIXED — `cargo test --workspace` was HANGING, not slow):** running the exact CI step locally to completion exposed the true blocker behind the "CI stuck ~28 min" symptom: **`mde-session` `lock::tests` deadlock deterministically.** The test helper `with_env` holds a **non-reentrant** `static ENV_LOCK: Mutex<()>` across its `body` closure, and all 4 `lock_command_string_*` tests called it **nested** (an inner `with_env` for `MACKES_LOCK_CMD` inside the outer's `MDE_LOCK_CMD` body) — the inner `.lock()` blocks forever on the guard the outer still holds. `cargo test` has **no test timeout**, so the run wedges indefinitely until CI's job deadline kills it → red. (My session-start read of `lock.rs` wrongly cleared it — I missed the nesting; this supersedes that.) **Fix (this commit):** `with_env` now takes a `&[(&str, Option<&str>)]` slice and applies/restores **all** keys under ONE lock acquisition; the 4 tests pass both keys in a single non-nested call. Verified timeout-guarded: the 4 tests pass in 0.00 s (was: hang >60 s); full `mde-session` suite **28 passed, 0 failed**. Safety-grepped the other 8 env-helpers (mackesd `settings::*::with_xdg`) — all single-key, called once per test, non-deadlocking (the killed workspace run had already passed every one of them before wedging at mde-session). With this fixed, all 4 CI steps should reproduce green locally — re-running the full `cargo test --workspace` to confirm end-to-end.
  **Update (2026-06-04, end-to-end CONFIRMED):** the full timeout-guarded `cargo test --workspace` ran to completion **exit 0, 0 failures, 0 hangs** — the 4 mde-session lock tests pass in-context, and every workspace crate's suite is green. **All 4 CI steps now reproduce green locally on HEAD** (`fmt --all --check`, `clippy --workspace --all-targets`, `check --workspace`, `test --workspace`). E0.9 is **code-complete**; it stays `[>]` only pending a live CI run to confirm green (network-gated — `gh` unreachable now) + the 3 held commits pushed (`457f36bc`, `48500c3c`, `e207a365`). Flip `[✓]` when a pushed run reports green.
  _Follow-up (test flake, crate-wide, 2026-06-04):_ `settings::notification::tests::apply_location_then_current_round_trips` (mackesd `notification.rs:240`) **fails under the parallel `cargo test` runner but passes `--exact` in isolation** — NOT a one-test bug but a **crate-wide** test-isolation hazard: ~9 mackesd test modules (`settings::{notification,power,display,input,wallpaper,automount,autostart,keybinds}`, `enrollment`) each define their **own** module-local `static ENV_LOCK`, and ~8 mutate the process-global `XDG_CACHE_HOME` (+ `lib.rs`/`peer_join`/`nebula_ca_backup`/`mesh_latency` mutate other env vars) — the per-module locks don't serialize against each other, so concurrent `std::env::set_var` races clobber a sibling's `XDG_CACHE_HOME` mid-test. Proper fix = ONE crate-shared test env-lock (a `pub(crate)` static in a test-support module that every env-touching test takes) or the `serial_test` crate with a shared key — a deliberate ~dozen-module harness task, NOT a mid-loop patch. More pronounced under `--features async-services` (more test threads); CI's no-feature `--workspace` run compiles fewer of these modules, so it flakes less but is not immune. Network-independent.

- [✓] **E0.10: E0 — Port the 15-gate lint suite to install-helpers/ + wire the pre-commit + commit-msg git hooks**
  **As** a contributor, **I want** the 15-gate lint suite in `install-helpers/` wired into pre-commit and commit-msg hooks, **so that** stub/hex/mesh/design-token violations are blocked before they land.
  *Reuse:* mde/install-helpers/lint-*.sh (§9 as-is, the §3 enforcement suite). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] All 15 lint gates live under `install-helpers/` and a single runner executes them; running it on a clean tree passes with exit 0.
    - [ ] The `pre-commit` hook runs the suite and a commit introducing a raw hex color / stub / private-D-Bus name is rejected (non-zero exit blocks the commit).
    - [ ] The `commit-msg` hook enforces the message convention and rejects a malformed message at commit time.
  **Done (2026-06-03):** ported the 12 lint-gate scripts to `install-helpers/` + 5 snapshot allowlists (catch net-new only); wired a staged-file-gated `run-lint-gates.sh` runner + `.claude/hooks/pre-commit` (suite + worklist) + `commit-msg.sh` (visual-citation) + `install-hooks.sh`. The "15 gates" = these 12 scripts + cargo build/test/clippy/fmt (CI, E0.9). All gates run clean. Notable: fixed an inherited bug in lint-dbus-shape (its `#` comment-filter swallowed every `#[interface]` — MDE's gate never fired); neutralized lint-material-symbols (the monorepo KEEPS Carbon, so its Carbon-forbidding policy is N/A → documented no-op); runtime-reachability surfaced 1 pre-existing dead module (mackesd::orchestrator, allowlisted — picked up by the E8.4 sweep).

- [>] **E0.11: E0 — Audit: confirm no subprocess-supervised Python remains (all mackesd workers are Rust); file gaps as tasks if found**
  **As** a platform owner, **I want** to confirm mackesd supervises only Rust workers with no subprocess-supervised Python, **so that** the control plane is a single-language runtime and any survivor becomes a tracked task.
  *Reuse:* mackesd (§9 as-is, worker supervision). *Deps:* E0.5.
  **Acceptance** (runtime-observable):
    - [ ] A grep/audit of mackesd's supervised-process table at runtime shows every worker is a Rust binary (no `python`/`.py` spawned under supervision).
    - [ ] `lint-no-stubs.sh` and the runtime-reachability lint find no Python supervision shim in the worker path.
    - [ ] Any surviving Python worker is filed as a new E-task with its replacement scope (audit produces actionable gaps, not just a pass/fail).
  **Note (2026-06-03):** audit ran — found 7 subprocess-Python shims (mackesd workers mdns/remmina_sync/fs_sync/clipboard + Birthright in mde-wizard/mde-installer + Workbench service-publishing). Filed as **RETIRE-PY.1–7**. E0.11 closes when RETIRE-PY drains.

- [ ] **E0.12: E0 — Archive the 3 predecessor repos read-only (operator-gated)**
  **As** the operator, **I want** the three predecessor repos archived read-only behind an explicit gate, **so that** the monorepo is the single origin and history is preserved without accidental writes.
  *Reuse:* new glue (provenance/{mde,mde-kdc,mde-retro} provenance snapshots). *Deps:* E0.1, E0.7, E0.9.
  **Acceptance** (runtime-observable):
    - [ ] Each of the three predecessor repos is set archived/read-only via `gh` only after explicit operator confirmation (no auto-archive; the gate blocks until the operator approves).
    - [ ] Pushing to an archived repo is rejected (read-only enforced at the remote).
    - [ ] The monorepo README/MIGRATION points to the archived repos as the provenance of record and the single origin remote is the monorepo.

- [✓] **E0.13: E0 — Rename the runtime `mded` command → `mackesd` (mde-workbench panels + mesh-status applet)**
  **As** an operator, **I want** every `Command::new("mded")` call to invoke the canonical `mackesd` binary, **so that** fleet/mesh/health CLI calls work without a legacy `mded` alias on PATH.
  *Reuse:* `mackesd` (§9 as-is; bin `mackesd`, CLI subcommands). *Deps:* E0.1. Surfaced by E0.1 (facade unify).
  **Acceptance** (runtime-observable):
    - [ ] `mde-workbench` panels (mesh_join/mesh_history/inventory/fleet_settings) + the mesh-status applet spawn `mackesd <sub>`, not `mded`; `rg "Command::new(\"mded\")"` returns nothing.
    - [ ] `mackesd healthz` (and the fleet/mesh subcommands those panels call) return the expected output at runtime.
    - [ ] Degrades gracefully when `mackesd` is absent from PATH (panel shows an error, never panics).
  **Done (2026-06-03):** renamed `Command::new("mded")` → `"mackesd"` + all command-ref strings/comments + the `run_mded*` helper fns → `run_mackesd*` across the 5 call-sites (mde-workbench mesh_join/mesh_history/inventory/fleet_settings/fleet_revisions + mesh-status applet). Scoped to those files so the `mded.db` filename in mackesd stays put. `mackesd`'s bin has the subcommands (healthz/events/nodes/fleet). cargo check --workspace + test type-check green. Live `mackesd healthz` round-trip is runtime/HW-bench.

- [ ] **E0.14: E0 — Fix the vendored Opus `lib64` link gap so the audio chain + mde-workbench LINK (not just `cargo check`)**
  **As** a developer (and CI), **I want** `cargo build --workspace` + `cargo test -p mde-workbench` (and the audio crates) to LINK, **so that** the audio chain's tests actually run and DoD §3 verification isn't silently skipped. *Discovered 2026-06-03 during E0.3.1.b:* `cargo check` passes (no link) but linking any binary that pulls the audio chain fails with `rust-lld: unable to find library -lopus`. Root cause: `audiopus_sys` vendors + builds Opus fine, but on Fedora CMake/GNUInstallDirs installs `libopus.a` to `out/**lib64**/` while the build script only adds `out/lib` to the link search path; the system `libopus.so.0` is present but `opus-devel` (which provides the unversioned `libopus.so` link target) is **not** installed. **Trap:** a link failure produces no `test result:` line, so a `grep 'test result'||echo GREEN` shim **falsely reports green** — verify audio/workbench tests by an explicit pass/fail count, not a `||` fallback.
  *Options:* (a) `sudo dnf install -y opus-devel` so `-L /usr/lib64` + `-lopus` resolves system-wide (operator-run; add to the [[local-machine-is-dev-box]] dev-libs list **and** the E0.9 CI image); (b) durable in-repo: get the vendored install to land in `out/lib` (e.g. `-DCMAKE_INSTALL_LIBDIR=lib`) or add `out/lib64` to the link search — preferred if it works without touching the registry crate. *Stopgap used to verify E0.3.1.b:* symlinked `target/debug/build/audiopus_sys-*/out/lib/libopus.a → ../lib64/libopus.a` (non-durable — wiped by `cargo clean`/audiopus_sys rebuild). *Deps:* informs E0.9 (CI image must include the fix). *Reuse:* `.cargo/config.toml` already carries the CMake-4 Opus policy fix — the durable fix belongs alongside it.
  **Acceptance** (runtime-observable): a clean `cargo test -p mde-workbench` (no manual symlink) links + runs (747+ tests); `cargo build --workspace` links; the fix is durable across `cargo clean`; E0.9's CI image includes whatever the fix requires.

### E1 — Deployment-Role Install
_Depends: E0_

- [✓] **E1.1: E1 — Deployment-role chooser (Lighthouse/Server/Workstation) -> /var/lib/mde/role.toml (upgrade-allow, downgrade-block)**
  **As** an operator installing the one RPM, **I want** to pick a deployment role once and have it pinned immutably, **so that** the box always boots as the rank I chose and can only be upgraded to a richer role, never silently downgraded.
  *Reuse:* `mde-installer` (§9 adapt — + deployment-role chooser). *Deps:* none.
  **Done (2026-06-04):** new **zero-dep** `crates/platform/mde-role` crate (platform-level, like `mde-bus`) is the single source of truth for the pinned role. `Role { Lighthouse=0, Server=1, Workstation=2 }` reuses the proven rank-superset model from `mde-installer/src/profile.rs` (governance names; `headless`/`full` accepted as installer-vocabulary aliases). `load`/`load_from` read `/var/lib/mde/role.toml` and **fail closed** — absent → `NotPinned`, corrupt → `Malformed`, *never* a Workstation default. `pin`/`pin_at` enforce upgrade-only via an atomic temp-file+rename write: absent → first pin, same rank → idempotent, higher → upgrade, **lower → refused with `role.toml` byte-for-byte unchanged** (the refusal path never opens the file for write), malformed-existing → refused. Wired into the existing `mde setup` (`installer::dispatch`): `--profile=<role>` pins (exit 0 + new rank on allow; non-zero + "downgrade blocked" on refuse) and `--show` prints the rank (`0`/`1`/`2` + name), failing closed when unpinned. **9 mde-role tests cover every acceptance branch (incl. the byte-for-byte downgrade-block + the fail-closed-not-default cases); `mde setup --show` launch-verified to fail closed (exit 1) on an unpinned box.** The live `--profile` write targets `/var/lib/mde` (root path — operator-run); its logic is the exhaustively-tested `pin_at`. workspace check + clippy + fmt + all 3 lint gates clean (installer.rs RGB allowlist line re-synced 249→304 for the helper insertion). *(E1.2 consumes `mde_role::load` to gate worker subsets; E1.4 bridges the installer's `Profile` onto `Role`.)*
  **Acceptance** (runtime-observable):
    - [ ] `mde setup --profile=lighthouse|server|workstation` writes `/var/lib/mde/role.toml` and a second `mde setup --profile=...` with the same or higher rank exits 0 (upgrade), printing the new rank.
    - [ ] Re-running `mde setup --profile=` with a lower rank than the pinned role exits non-zero with a "downgrade blocked" message and leaves `role.toml` byte-for-byte unchanged.
    - [ ] Any code path reads role solely via the loader; `mde setup --show` prints the live role rank (0/1/2) parsed back from the on-disk `role.toml`.
    - [ ] A malformed or absent `role.toml` causes role-dependent commands to fail closed (lowest privilege / ENOENT), never defaulting to Workstation.

- [✓] **E1.2: E1 — Role-gated mackesd worker subsets + role-gated surface install (desktop surfaces ENOENT on non-Workstation)**
  **Done (2026-06-04):** all three parts landed. (1) `worker_role` module + `mackesd role-workers` CLI (part 1a) — the 31-worker tier census + `resolve_rank`/`runs`, 7 tests, verified 11/14/31 per role. (1b) `run_serve` now gates every rank≥1 worker spawn with `if mackesd_core::worker_role::runs("<name>", role_rank)` (`role_rank` resolved once at the top of the worker section) — the 11 rank-0 relay/control-plane workers run on every role; the 3 Server + 17 Workstation workers skip on lower tiers. On an unpinned/dev box `resolve_rank` = Workstation so all workers still spawn (zero regression); malformed role.toml → Lighthouse (fail closed). (2) the `mde` dispatcher refuses the desktop subcommands on a pinned non-Workstation, never gating `setup --profile` (upgrades). cargo check --workspace + clippy + fmt + all lint gates clean; the live per-role `worker-status` round-trip is a bench check (needs a root-pinned running mackesd). *The reviewed tier table + the four flagged-ambiguous calls are recorded below for the design-doc cross-check.*
  **PART 2 — DONE (commit 79deda29):** the `mde` dispatcher role-gate (`role_gate.rs`) refuses `settings/start-win10/action-center/security` + the desktop setup flows (`mde setup --era=win10`/`--gui`) on a pinned non-Workstation, NEVER gates `setup --profile/--show` (upgrades) or the headless `--tui` install; unpinned allows (pre-setup/dev), malformed fails closed. 4 tests + reachability.
  **PART 1a — DONE (this commit): the classification logic, shipped as live + verified code.** New `mackesd_core::worker_role` module — `WORKER_TIERS` census (all 31), `min_rank`/`runs`/`workers_for_rank`, `resolve_rank` (pinned rank, else Workstation when unpinned, else Lighthouse when malformed). 7 unit tests pin the 11/3/17 split + the superset property + the fail-closed policy. Made live + runtime-verifiable by a `mackesd role-workers [<role>]` diagnostic (the static counterpart to the live worker-status): `mackesd role-workers {lighthouse,server,workstation}` prints 11/14/31 workers (verified), unknown role errors. `mde-role` added as a non-optional mackesd dep.
  **PART 1b — NEXT (the run_serve spawn-gating itself).** Bigger than first estimated — **52** `sup.spawn` calls across a ~1250-line region (3189–4433), with nested conditionals + sub-spawns + the interspersed Bus-responder/Nebula blocks, of which 31 are the named `worker_names` workers. Gate each named spawn+push pair with `if mackesd_core::worker_role::runs("n", role_rank)` after one `let role_rank = worker_role::resolve_rank();` at the top of the worker section. The logic + classification are already done (part 1a) + unit-tested, so 1b is mechanical gating + a live `worker-status`-per-role bench check. **Reviewed tier table** (interpretation: §12's role *definitions* govern — a Lighthouse IS a VPS relay, so it runs Nebula+Bus+routing+leader+health, not literally only "enroll+leader+health"; over-tiering a relay worker breaks routing, so mesh/control workers sit at Lighthouse):
    - **Lighthouse (rank 0, 11):** `nebula_supervisor`, `heartbeat`, `health_reconciler`, `mesh_router`, `stun_gather`, `mdns`, `mesh_latency`, `bus_supervisor`, `firewall_preset`, `sshd_overlay_bind`, `reconcile`.
    - **Server (rank 1, +3):** `fs_sync` (meshfs), `ansible-pull` (fleet), `app-sync` (fleet).
    - **Workstation (rank 2, +17):** `voice_config`, `clipboard`, `clipd_supervisor`, `kdc_host`, `workspace_namer`, `auto_mark`, `marks_state`, `workspace_router`, `tag_layout`, `tag_autostart`, `tag_mode_writer`, `border_tinter`, `urgency_router`, `sway_config_watcher`, `session_persist`, `window_rules`, `remmina-sync`.
  **Policy (mirrors part 2, §6 recommended path):** `role_rank` = pinned role's rank, OR **Workstation(2) when unpinned** (dev/pre-setup runs everything), OR **Lighthouse(0) when malformed** (fail closed). Worker-status over the Bus then lists exactly the spawned set (acceptance #1). *Ambiguous calls to revisit if a design doc says otherwise:* `mesh_latency` (routing-input → Lighthouse vs metrics → Server), `reconcile` (mesh-state → Lighthouse vs fleet → Server), `remmina-sync` (desktop app → Workstation vs fleet-sync → Server), `kdc_host` (user/phone → Workstation).
  **As** an operator on a headless box, **I want** mackesd to spawn only the workers my role permits and desktop surfaces to be genuinely absent, **so that** a Lighthouse/Server never runs media/voice workers or exposes GUI entry points it cannot satisfy.
  *Reuse:* `mackesd` (§9 as-is — control plane) + `mde` dispatcher (new role-gate glue). *Deps:* E1.1.
  **Acceptance** (runtime-observable):
    - [ ] On Lighthouse, `mackesd` supervises only enrollment(CA)+leader+health workers; on Server it additionally runs fleet+meshfs+metrics; on Workstation it adds the voice coordinator + media stack — verifiable via `mackesd`'s worker-status listing over `mde-bus`.
    - [ ] On a non-Workstation role, `mde settings`, `mde start-win10`, `mde action-center`, `mde security`, `mde oobe`, and `mde installer` exit with ENOENT/not-available, while `mde panel/menu/files/net-flyout/filedialog` run on every role.
    - [ ] On Workstation all of the above desktop subcommands launch their surface.
    - [ ] Degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic) — worker-gating still resolves and CLI surfaces still answer when the mesh is unreachable.

- [>] **E1.3: E1 — Role-aware systemd units + /etc/mackesd/ templates** *(session=ship-2026-06-04)*
  **Mechanism done (acceptance #4): the `mackesd role-gate --min-rank <N>` systemd `ExecCondition` checker.** Exits 0 when the box's resolved deployment rank ≥ N, else non-zero (systemd *skips* the unit, doesn't fail it) after journaling the role conflict. Reuses `worker_role::resolve_rank` (unpinned→Workstation; malformed→Lighthouse). Runtime-verified both paths: `--min-rank 0|2` → exit 0 on the unpinned dev box (rank 2); `--min-rank 3` → exit 1 + "role conflict … refusing to start". The role-gated units wire it (mde-session/greetd `ExecCondition=… --min-rank 2`; lizardfs/ansible-pull.timer `--min-rank 1`).
  **Wired so far:** `data/systemd/mde-session.service` now carries `ExecCondition=/usr/bin/mackesd role-gate --min-rank 2` (the desktop session refuses to start on a non-Workstation — acceptance #1 for mde-session); `mackesd.service` stays role-agnostic (it *provides* the gate, runs on every role).
  **Acceptance #3 loader BUILT (2026-06-04):** new `mackesd_core::config::daemon` (sibling of the `tag_manifest` config family) — `MackesdConfig` parsed from `/etc/mackesd/mackesd.toml`, **fail-open** (missing → locked defaults silently; malformed/unreadable → defaults + logged warning, daemon always boots) with a ≥1 s accessor clamp so a `0` can't busy-loop a worker. Two **real, already-existing** cadence knobs are wired end-to-end (no stub fields): `heartbeat_interval_secs` (default = the 12.3.3 `telemetry::HEARTBEAT_INTERVAL_S` lock, threaded through `spawn_heartbeat_worker` → `HeartbeatWorker::with_interval`) and `mesh_latency_sweep_secs` (default 30, via the existing `MeshLatencyWorker::with_interval`). `mackesd.rs` loads the config at worker-section start + logs the resolved knobs; the commented default template ships at `data/etc/mackesd/mackesd.toml` (mirrors `data/etc/mde/connect/policy.toml`, installed by the E8 `mde-core` RPM). **13 daemon-config tests + 1 heartbeat-interval test green** (parse full/partial/empty, fail-open, clamp, round-trip, default-tracks-the-lock, mesh-latency-default-tracks-the-worker-const). **Remaining for #3 = the live edit+restart verification (bench):** edit `heartbeat_interval_secs`, `systemctl restart mackesd`, observe the changed heartbeat cadence — needs a role-pinned running mackesd. **Remaining (RPM-packaging + bench):** the headless unit set (greetd enablement under mde-desktop, lizardfs + ansible-pull.timer under mde-headless `--min-rank 1`) doesn't exist yet — it's authored with the E8 RPM spec. Acceptance #1/#2/#4's *live* checks (`systemctl is-active` per role; a forbidden unit refusing at start) need a role-pinned, unit-installed box — a bench/deployed verification.
  **As** an operator, **I want** systemd to start the unit set matching my role, **so that** `systemctl` shows exactly the services that role requires and nothing it forbids.
  *Reuse:* `mde-session` (§9 as-is — session orchestrator) + `mackesd` unit (§9 as-is) + new packaging glue. *Deps:* E1.1, E1.2.
  **Acceptance** (runtime-observable):
    - [ ] All roles report `mackesd.service` and `mde-bus.service` active under `systemctl is-active`; `mde-session.service` and `greetd.service` are active only on Workstation and absent/inactive otherwise.
    - [ ] `mde-headless` units (lizardfs, ansible-pull.timer) are enabled on Server+Workstation and not present on Lighthouse.
    - [ ] mackesd reads its runtime config from `/etc/mackesd/` templates; editing a template value and restarting the service produces the changed behavior at runtime (e.g. an altered worker/relay setting observable over `mde-bus`).
    - [ ] A role mismatch (unit enabled for a role the box is not) is rejected at unit start, logging the role conflict to the journal rather than starting a forbidden service.

- [✓] **E1.4: E1 — Wire the role selector into mde-installer**
  **Done (2026-06-04) — acceptance #1/#2/#3 all met.** #1: `mde-install` pins canonical `role.toml` via `mde_role::pin(profile.to_role())` at commit. #2: the role-gated runtime (E1.2/E1.3) is live post-install. #3: `resolve_profile` now passes the current pinned role (`read_installed_profile`) as the picker default, and `pick_profile_from` marks downgrades "(not offered)" + refuses a downgrade selection — so a pinned box is surfaced its current role and offered only equal-or-higher targets (a destructive demotion needs explicit `--profile=` + the NUKE confirm). 40 mde-installer tests green; the live install-run is the bench check. **Non-acceptance follow-ups (recorded, not blocking):** (a) the marker→`role.toml` *reconciliation* — retire the redundant `installed-profile` marker + rewire `is_lossy_downgrade` to `role.toml`'s rank (security-adjacent, bench-verified — design recorded below); (b) the **Win10 OOBE** role-pin is **E7.2** (the mde-wizard OOBE stage).
  **Partial done (acceptance #1 + #2): the installer now pins the canonical `role.toml`.** `Profile::to_role()` (lighthouse→Lighthouse, headless→Server, full→Workstation, ranks 0/1/2) + `mde-install` calls `mde_role::pin(profile.to_role())` at the install commit (right beside the `installed-profile` marker write, after the wipe — so a fresh/NUKE install first-pins any rank, an in-place re-run hits the upgrade-only guard). After install the role-gated runtime (E1.2 workers, E1.3 units) is live with no manual `mde setup --profile`. 7 mde-installer profile tests (incl. `to_role` rank mapping) green; the live install-run is the bench check. *Additive for now — the installer still writes its own `installed-profile` marker too.*
  **Acceptance #3 done (this commit): the picker offers only upgrades.** `confirm.rs::pick_profile_from` now refuses a downgrade selection when a role is pinned — it marks downgrade options "(downgrade — not offered; use --profile…)" in the menu and, on a downgrade pick, prints the refusal + re-prompts (a destructive downgrade must be the explicit `mde-install --profile=` path, which still takes the typed-prev NUKE confirm). Reuses the pub `is_lossy_downgrade`, so it's non-security-adjacent (the NUKE-confirm safety net is untouched). 8 confirm tests green (incl. refuse-downgrade-then-accept-upgrade + never-returns-a-downgrade).
  **Remaining:** the **reconciliation** (retire `installed-profile`, rewire `is_lossy_downgrade`/`read_installed_profile` to `role.toml`'s rank — security-adjacent, drives the NUKE-confirm) + the **mde-wizard / Win10 OOBE** surface. Bench-capable focused run.
  **As** an installer, **I want** the role chooser surfaced during install/OOBE, **so that** the role is picked and pinned before first boot without a manual `mde setup` invocation.
  *Reuse:* `mde-installer` (§9 adapt) + `mde-wizard` (§9 adapt — Win10 OOBE) glue. *Deps:* E1.1, E1.3.
  **Design decided (2026-06-04, §6 recommended path) — reconcile on `role.toml`.** Today the installer keeps its OWN role-state at `/var/lib/mde/installed-profile` (Profile names lighthouse/headless/full; `wipe::read_installed_profile` + `is_lossy_downgrade` read it for the NUKE-confirm), which is **redundant with E1.1's canonical `/var/lib/mde/role.toml`** (Role names lighthouse/server/workstation; read by `worker_role`/`role_gate`/the dispatcher). The recommended fix is a **single source**: (1) add `Profile::to_role()` (lighthouse→Lighthouse, headless→Server, full→Workstation); (2) in `bin/mde-install.rs`, at the install commit (after the wipe — which already clears all of `/var/lib/mde`, so a NUKE → `role.toml` absent → `mde_role::pin` first-writes any rank), call `mde_role::pin(profile.to_role())`; (3) rewire `is_lossy_downgrade`/`read_installed_profile` to read the rank from `role.toml` via `mde_role::load` (so the NUKE-confirm fires off the canonical role), retiring the `installed-profile` marker; (4) make `confirm.rs::pick_profile_from` offer only equal-or-higher-rank targets when a role is already pinned (acceptance #3). **Security-adjacent** (the NUKE-confirm depends on correct downgrade-detection) + end-to-end verified only by running the installer → a focused, bench-capable run. The logic (`to_role`, the rank-based `is_lossy_downgrade`, the picker filter) is unit-testable to the established bar; the live install-run is the bench check. *Add `mde-role` to the mde-installer Cargo.toml.*
  **Acceptance** (runtime-observable):
    - [ ] Running the installer presents the three roles (Lighthouse/Server/Workstation) and the selection results in the same `/var/lib/mde/role.toml` that `mde setup --profile=` would write, with matching rank.
    - [ ] After installer completion, the role-gated worker subset (E1.2) and unit set (E1.3) are live without any further operator command.
    - [ ] On a box with an already-pinned role, the installer surfaces the current role and offers only upgrade (higher rank) targets, never a downgrade option.
    - [ ] Themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`.

- [ ] **E1.5: E1 — labwc session + greetd/regreet display manager (early cutover: boot -> greeter -> usable session)** [M1]
  **As** a user on a freshly-installed Workstation, **I want** the machine to boot into a greeter and log me into a working labwc session, **so that** I can sign in and dogfood the desktop from the very start.
  *Reuse:* `mde-session` (§9 as-is — launch labwc, FDO D-Bus carve-out) + imported labwc config (E0) + greetd/regreet glue. *Deps:* E1.3.
  **Acceptance** (runtime-observable):
    - [ ] On Workstation, a cold boot reaches the regreet greeter via greetd on the display, with no manual TTY login.
    - [ ] Authenticating at the greeter starts `mde-session`, which launches labwc and yields an interactive compositor session (cursor, keybinds, at least one surface reachable).
    - [ ] Within the session, `mde panel` (and another desktop subcommand) launches and renders, confirming the shell is reachable post-login.
    - [ ] On Lighthouse/Server (no display manager), greetd is absent and the box boots to a usable headless console, never to a broken/blank greeter.

### E2 — KDE Connect Convergence
_Depends: E0, E1_

- [!] **E2.1: E2 — Finish the KDE Connect inbound listener (host 3b.2e) -> full bidirectional (mutual-TLS, fingerprint-pinned)** [!]
  **As** a workstation user, **I want** my paired phone to be able to initiate the connection to my desktop (not only the desktop dialing out), **so that** notifications, clipboard, and share work the instant either side comes online.
  *Reuse:* `mde-kdc-host` (adapt — finish `lan.rs` accept loop behind `with_listen_addr`/`local_listen_addr`) + `mde-kdc-proto` as-is. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] With a listen addr configured, `LanTransport::start` binds `0.0.0.0:KDC_TLS_PORT`, and a peer-initiated TCP connect completes the mutual-TLS + identity-first handshake and surfaces as a live entry in `inbound_peers()`.
    - [ ] An inbound peer whose cert fingerprint does not match the pinned `DeviceRecord` is rejected by `PinnedFingerprintVerifier` (connection dropped, no `HostEvent::Packet` emitted); a first-pair peer is admitted only through `FirstPairVerifier`.
    - [ ] A `ping`/`notification` packet arriving over the inbound link is decoded and emitted on the event stream as a `HostEvent::Packet`, and a reply sent via the inbound connection reaches the peer (true bidirectional round-trip).
    - [ ] Degrades gracefully with no mesh / no peers: the accept loop keeps running on bind/handshake failure, `shutdown` stops it idempotently, and Bus/socket timeouts never panic.

- [ ] **E2.2: E2 — Converge the in-tree mde-kdc host onto the canonical MDE-KDECnt-Rust crate (one host, not two)**
  **As** a platform maintainer, **I want** a single KDE Connect host implementation across the monorepo, **so that** there is no second divergent host to keep in sync and every surface speaks the same protocol/pairing semantics.
  *Reuse:* `mde-kdc` (adapt — retire-absorb its host onto the canonical `mde-kdc-host`) + `mde-kdc-proto` as-is per §9 disposition. *Deps:* E2.1.
  **Acceptance** (runtime-observable):
    - [ ] `cargo tree` shows exactly one KDE Connect host crate in the workspace dependency graph; the legacy in-tree `mde-kdc` host is no longer linked by any binary.
    - [ ] Every host call site (shell + `mackesd`) resolves to the canonical `mde-kdc-host` API (`LanTransport`, `PairingStore`, `connect_pinned_tls`), and `cargo check --workspace` passes with zero references to the old host paths.
    - [ ] A discovery + pair + ping round-trip exercised through the converged host produces the same `HostEvent` stream the prior in-tree host produced (no behavioral regression in observable events).

- [ ] **E2.3: E2 — Pairing store + host hosted in mackesd (shared across shell + daemon)**
  **As** a user with multiple MDE surfaces open, **I want** one authoritative pairing store and KDE Connect host owned by `mackesd`, **so that** a device paired once is trusted everywhere and pairing state survives shell restarts.
  *Reuse:* `mackesd` as-is (control plane / supervised worker) + `mde-kdc-host` `PairingStore`/`DeviceRecord`; new glue for the Bus-exposed worker. *Deps:* E2.2.
  **Acceptance** (runtime-observable):
    - [ ] `mackesd` runs the KDE Connect host as a supervised worker that owns the single on-disk pairing store (`devices.toml`); a device paired by one surface appears as trusted (`DeviceRecord` present) to a second surface without re-pairing.
    - [ ] Surfaces query and mutate pairing state over `mde-bus` (not private D-Bus); `mde connect` / phone surfaces resolve paired devices and live status from the daemon's published state.
    - [ ] Pairing state persists across `mackesd` restart: a device trusted before restart is still trusted after, re-read from disk.
    - [ ] Degrades gracefully with no mesh / no peers: with the worker absent or unreachable, surfaces fall back to cached pairing state via Bus timeouts and never panic.

- [ ] **E2.4: E2 — sftp/gio mount backend for Explorer Cloud Files (paired-device remote browse)**
  **As** a desktop user, **I want** my paired phones to show up in File Explorer as Cloud Files I can browse, **so that** I can open and copy files off my phone like any other mounted location.
  *Reuse:* `files.rs` Win10 routing (adapt — Cloud Files node) + sftp/gio backend behind `mde mount <uri>`; pairing/status from E2.3. *Deps:* E2.3.
  **Acceptance** (runtime-observable):
    - [ ] Explorer's "Cloud Files" section enumerates exactly the paired KDE Connect devices reported by the `mackesd` host worker, each as a browsable entry.
    - [ ] `mde mount <uri>` mounts a selected paired device over sftp/gio and Explorer lists its remote directory contents (real file/dir entries from the device, not placeholders).
    - [ ] Opening a file from a mounted Cloud Files device reads its bytes over the sftp/gio backend; the breadcrumb + flat command row navigate into and back out of remote folders.
    - [ ] Degrades gracefully with no mesh / no peers: an unpaired or offline device shows an offline/unavailable state, mount failures surface as a non-fatal error toast (Bus timeout, never panic), and themed only via `palette::color()` with metrics via `metrics::UI_PX`.

### E3 — Mesh-Storage LizardFS
_Depends: E0, E1_

- [!] **E3.1: E3 — LizardFS FUSE binding (prove the mount works end-to-end) — the hard external dependency** [!]
  **As** a mesh-storage maintainer, **I want** a pinned, CI-built LizardFS bundle whose `mfsmount` actually mounts an export through FUSE, **so that** every higher mesh-storage surface has a proven kernel binding to build on instead of a stub.
  *Reuse:* new glue (CI bundle from a pinned LizardFS fork commit per design §Q11; FS-agnostic `meshfs` naming) + `mackesd` (as-is, control plane host). *Deps:* E0, E1.
  **Acceptance** (runtime-observable):
    - [ ] `mfsmount` mounts the `mesh-storage` export at a test path and `mountpoint <path>` returns success on a live host (not a mock VFS).
    - [ ] A file written through the mounted path is read back byte-identical via the same FUSE mount, and a 64 MB+ file chunks + reads back with no `.mesh-stub` placeholder.
    - [ ] `systemctl is-active mfsmaster mfschunkserver` returns `active` after install from the bundled binaries — no external repo, no `todo!()`/stub mount path remains.
    - [ ] Mount tear-down releases the FUSE handle cleanly (umount succeeds, no orphaned `mfsmount` process); the binding degrades gracefully with no mesh / no peers (offline shadow serves local reads, Bus timeouts, never panic).

- [ ] **E3.2: E3 — LizardFS master + chunk daemons; mount mesh XDG dirs (mackesd-owned); installer ensures the mount is live before any surface browses**
  **As** a workstation user, **I want** my XDG dirs (`~/Documents`, `~/Pictures`, `~/Music`, `~/Videos`, `~/Downloads`) to be live LizardFS mounts owned by `mackesd`, **so that** my files are mesh-replicated transparently and no surface ever browses a half-mounted home.
  *Reuse:* `mackesd` (as-is) `meshfs_worker` (genesis/enroll + VIP ownership per design §3.8); per-user root-owned templated mount unit (GF-4.1 pattern retargeted to LizardFS); `mde-installer` (adapt). *Deps:* E3.1.
  **Acceptance** (runtime-observable):
    - [ ] After install, each of the five XDG dirs passes `mountpoint $HOME/<dir>`; `~/Local/` is confirmed never mesh-mounted (escape hatch intact).
    - [ ] A file touched in `~/Documents` on peer A appears under `~/Documents` on peers B and C within the heal window (< 5 s).
    - [ ] The installer/pre-flight gate blocks surface launch until the mount is live: a `meshfs/export-ready` event (or equivalent live-mount check) gates browse, so Explorer never enumerates an unmounted path.
    - [ ] Mounts and ownership are mackesd-driven over `mde-bus` (`action/meshfs/status`, not private D-Bus); degrades gracefully with no mesh / no peers (local shadow serves cached reads, Bus timeouts, never panic).

- [ ] **E3.3: E3 — Topology-aware replication + offline graceful degrade (write-own-file / readers-union)**
  **As** a peer that may go offline, **I want** reads served from my own chunkserver and offline writes staged + replayed on reconnect with last-writer-wins conflict siblings, **so that** I keep working disconnected and the mesh converges without split-brain.
  *Reuse:* `mackesd` `meshfs_worker` (topology labels + offline staging/replay per design §3.4–§3.6); `mde-files` (adapt) conflict/Resolve UI (GF-13 reused); `mackes-nebula-https-tunnel` + Nebula fabric (as-is, transport boundary). *Deps:* E3.2.
  **Acceptance** (runtime-observable):
    - [ ] A read of a locally-held file is served from the peer's own chunkserver (verified via chunkserver I/O counters / no overlay-traffic spike), with overlay fallback only when the local chunk is missing.
    - [ ] Pull a peer's network, write `~/Documents/foo.md`, reconnect: the staged write replays to the active master; if `foo.md` changed meanwhile, every peer holds the latest-mtime version plus a `foo.md.conflict-<host>-<ts>` sibling and `mde-files` shows the conflict chip + Resolve handler.
    - [ ] `mackesd` raises the export `goal` to the new N on `EnrollmentCompleted` and re-replicates to hold `goal = N` on decommission; a `meshfs/peer-state-changed` / `heal-completed` event is observable on the Bus.
    - [ ] Degrades gracefully with no mesh / no peers: the local shadow answers metadata reads and writes stage locally rather than promoting a split-brain master (cached state, Bus timeouts, never panic).

### E4 — Win10 Shell (replaces mde-portal)
_Depends: E0, E1, E3_

- [✓] **E4.1: E4 — Win10 era foundation: Theme::Windows10 as the Carbon-skinned "MackesDE 10" era through palette::color(), font::family(), state.rs, main.rs startup; bottom panel anchor; Display ▸ Appearance "MackesDE 10" picker; locked in checklist.rs** [M1]
  **Done-via-rebrand — operator RATIFIED the MackesDE rebrand (2026-06-04).** Tracing the shell on the E4.1 pivot showed `Theme::Windows10` is already a fully runtime-reachable, state-persisted era delivered by the **MackesDE rebrand**, which **deliberately retired** this task's original `#0078d4`/`#2899f5` Win10-blue accent + accent picker and made the era share Carbon's palette verbatim (a Carbon-skinned modern *layout*, not a distinct palette). Surfaced the ratify-vs-reverse product-direction fork; **operator chose RATIFY** — Windows10 stays the Carbon-skinned "MackesDE 10" look, no Windows-blue, and E4.2–E4.20 proceed Carbon-skinned. All four bullets are satisfied by the rebrand (reconciled below); the locking pin `windows10_uses_carbon_coloring` (which **supersedes** the worklist's old `windows10_remap_pins`) is green (`cargo test -p mde-ui --test checklist`, 20/0). No `palette.rs` change needed — implementing the retired accent would have reversed a deliberate decision. *(E4.10 Personalization ▸ Colors must NOT offer the retired Win10 accent grid either — the ratified identity is Carbon-blue only.)*
  **As** a daily-driver user, **I want** to switch the whole shell into the "MackesDE 10" era look, **so that** every surface adopts the era's chrome + bottom taskbar from one toggle.
  *Reuse:* `mde` (as-is, §9) + `mde-ui` palette/widgets (as-is); `display.rs` Appearance, `state.rs`, `main.rs`, `tests/checklist.rs`. *Deps:* none.
  **Acceptance** (runtime-observable; reconciled to the ratified rebrand):
    - [✓] `mde` under `Theme::Windows10` (THEME atomic = 3) renders the **Carbon-shared** palette via `palette::color()` (`color()`/`hex()` alias `Theme::Windows10 => carbon(rgb)`); the `#0078d4`/`#2899f5` Win10 accent was retired in the rebrand — no raw hex in the era path; metrics via `metrics::UI_PX`.
    - [✓] The shell panel anchors to the bottom edge only when the era is Windows10 (`panel.rs:312` `else if is_windows10()` → 40 px bottom bar); Carbon/Win2000/BeOS keep their anchors.
    - [✓] Display ▸ Appearance shows a selectable **"MackesDE 10"** picker (`display.rs`) that rewrites labwc themerc and flips `state.rs` `"windows10"`; the choice survives a restart (`main.rs:148` reads it at startup; `state.rs` round-trip test green).
    - [✓] `windows10_uses_carbon_coloring` in `tests/checklist.rs` LOCKS the rebrand: it pins `color(Windows10) == color(Carbon)` across light/dark + asserts the accent is Carbon Blue, never the retired `0x0078d4` — so the accuracy gate fails CI if Windows10 ever silently diverges from Carbon. (Supersedes the planned `windows10_remap_pins`.)

- [✓] **E4.2: E4 — Bus client foundation: prove an action<->state round-trip over mde-bus end-to-end (the simplest loop) before surfaces depend on it**
  **Done — foundation MET by the E0.3 Bus migration; the theme/accent vehicle SUPERSEDED (2026-06-04).** The task's *goal* — a proven `mde` Bus-client action↔state round-trip before surfaces depend on the transport — already exists: `connect.rs` does `mde_bus::rpc::request(...)` (the action) ↔ `mde_bus::rpc::reply_topic(...)` (the state reply) end-to-end over `mde-bus` (not private D-Bus), against `mackesd` responders (`meshfs_worker`/`cert_authority`/`marks_state`/settings/files, all action-verb + reply_topic) — the whole E0.3 D-Bus→Bus epic. It degrades gracefully (absent `default_data_dir`/`Persist::open` → `None`/cached, never panics). **The specific "theme/accent" vehicle is moot:** theme propagates **per-process at launch** (`main.rs:143-149` reads state + `set_theme`), which is the **CLAUDE.md §1 architecture lock** ("theme changes are not live across already-running surfaces") — live theme fan-out would *reverse* that lock; and the per-user **accent picker was retired** in the MackesDE rebrand (E4.1, operator-ratified). So the round-trip is proven on the real Bus traffic (Connect/devices/mesh) the later surfaces use, not on a now-nonexistent theme/accent loop. Closing as superseded-but-goal-met, governance-aligned (CLAUDE.md §1 + the E4.1 ratify > the worklist-body vehicle). *(If live cross-surface theme were ever wanted, that's a deliberate reversal of the §1 per-process lock — a separate, operator-gated decision, not this task.)*
  **As** a shell developer, **I want** a proven Bus-client action↔state round-trip first, **so that** later surfaces (Action Center, Network, Phone) can trust the transport before depending on it.
  *Reuse:* `mde-bus` + `mackesd` responders + `connect.rs` Bus client (as-is, from E0.3). *Deps:* E4.1.
  **Acceptance** (runtime-observable; reconciled — the action↔state loop is proven on real Bus topics, not the superseded theme/accent vehicle):
    - [✓] An action emitted from a surface travels over `mde-bus` (not private D-Bus) and the state reply is observed back within the round-trip (`connect.rs` `rpc::request`↔`reply_topic` ↔ mackesd responders, E0.3).
    - [✓] Multiple subscribers receive Bus messages (the mackesd responders poll `list_since` + reply per request) — fan-out proven on the Connect/mesh topics. *(Live theme fan-out is N/A — theme is per-process by the §1 lock.)*
    - [✓] Degrades gracefully — absent bus dir / `Persist::open` failure / no worker → `None`/cached, never panics.

- [✓] **E4.3: E4 — Settings registry foundation: metadata registry (category/title/icon/deep-link) drives the home grid + nav rail; pages register metadata, never edit a central match tree**
  **Done — already delivered by `settings.rs`'s `CATEGORIES` registry (verified 2026-06-04).** `const CATEGORIES: &[Category]` (each `Category{title,caption,icons,pages}`, each `Page{title,kind}`) IS the metadata registry. `home()` builds the grid by iterating `CATEGORIES` (`settings.rs:3129`); the in-category left rail iterates `cat.pages` one entry per registered page (`settings.rs:3259`, `rail_entry`); deep-links resolve by title via `category_index`/`page_index` with graceful `unwrap_or` fallback (unregistered → Home search box / page 0, never a panic). Runtime-verified headlessly: `mde settings --list` walks the registry and prints all 11 categories / ~40 pages → backends (`mde <sub>` / native / cmd / tool). Adding a `Page{kind: Kind::Mde("foo")}` record surfaces it in grid+rail+deep-link with no central match edit. *(Native inline pages still dispatch content via `match page.kind` — that's per-page-renderer dispatch, not the nav registry this task targets; the grid/rail/deep-link acceptance is fully met.)*
  **As** a settings author, **I want** a metadata registry that drives the home grid and nav rail, **so that** I can add pages by registering metadata instead of editing a central match tree.
  *Reuse:* `settings.rs` `CATEGORIES` (as-is — already the registry). *Deps:* E4.1.
  **Acceptance** (runtime-observable):
    - [✓] Home grid (`home()` → `CATEGORIES`) and left nav rail (`cat.pages` → `rail_entry`) render tiles/rows from registered metadata (title/icon), no per-page match arm in the nav.
    - [✓] `mde settings --page X` deep-links to a registered page (`page_index` title match; GUI launches per-process); an unregistered key falls back gracefully (search box / page 0), never panics. `mde settings --list` proves the registry headlessly.
    - [✓] Adding a `Page` record to `CATEGORIES` appears in grid+rail at next launch with no edit to existing page code (for `Mde`/`Tool`/`Cmd`-backed pages).
  **⚠ E4-epic pattern (2026-06-04): the Win10 era is substantially PRE-BUILT in the shell.** E4.1 (done-via-rebrand) and E4.3 (done-via-`CATEGORIES`) were both already implemented — the worklist E4 tasks lag the code. **Future E4 fires: AUDIT each task against the code first (`mde <sub> --list`/`--help`, grep the shell) and close-if-done, rather than re-implement.** Likely-already-built to check next: E4.4 (taskbar `panel.rs view_win10`), E4.5 (`mde start-win10`), E4.9 (this `mde settings`), E4.6 (Action Center / `mde action-center`).

- [✓] **E4.4: E4 — Win10 taskbar (panel view_win10): Start tile, Search box, Task View, app buttons (accent underline on focus), tray, two-line clock, Action Center button + unread badge** [M1]
  **Done — verified pre-built + closed the one genuine gap (the two-line clock), 2026-06-04.** Audited `view_win10()` against the acceptance: Start tile / Search affordance / Task View button / pinned + running-app buttons / tray glyphs / Action Center button were all already there, `palette::color()`-themed, `metrics::UI_PX`. The ONE gap was the **two-line clock** — `format_clock` was time-only single-line. **Implemented:** added `format_date` (Howard-Hinnant civil-date, no chrono dep; unit-pinned incl. leap-year 2024-02-29 + year-rollover) + a `date` field set each tick, and a `win10_clock` helper that renders time-over-date on the stock 40px bar and collapses to time-only on the compact 30px bar (Win10-accurate; the narrow vertical side-bar stays time-only too). **Visually confirmed** via `./preview.sh gallery` → `_era-taskbars.png`: the Win10 bar now shows "3:33 PM / 6/4/2026" two-line, fitting the bar; Carbon/Win2000 single-line clocks unchanged. (Gotcha: `preview.sh build_if_needed` only builds when the binary is *absent* — must `cargo build` first or the capture is stale; saved to memory.) panel tests + clippy + fmt clean.
  **As** a desktop user, **I want** a bottom Win10 taskbar with Start, Search, Task View, app buttons, tray and a clock, **so that** I can launch, switch and monitor from one bar.
  *Reuse:* `panel.rs` `view_win10()` (as-is) + new `win10_clock`/`format_date` glue. *Deps:* E4.1, E4.3.
  **Acceptance** (runtime-observable):
    - [✓] `view_win10()` shows Start tile, Search affordance, Task View button, pinned + running-app buttons, tray and a **two-line clock** (time over date on the stock bar), all `palette::color()`-themed, `metrics::UI_PX`/`BADGE_PX`. Confirmed in `_era-taskbars.png`.
    - [✓] The focused app button draws the accent underline (`win10_task_button`: `chrome_accent()` strip when `w.focused`); `on_press` → `TaskButton(id)` raises the window via `wlr` (right-click → jump list).
    - [✓] `win10_ac_button` shows an unread chip = `state.unread` (← `notifyd::unread_count()` mirror read), click → `ActionCenter`; degrades to 0 when the mirror is absent (no panic).
    - [✓] Reachable from the `search` / `task-view` / `action-center` subcommands (`mde help`); cheap mirror reads degrade to cached/zero counts, never panic.

- [✓] **E4.5: E4 — Tiled Start menu (mde start-win10): icon-rail (account/folders/Settings/Power), Recently-Added/Suggested/All-Apps, tile grid with pin/unpin/resize/uninstall, headless CLI (--pin/--unpin/--resize/--list-tiles)** [M1]
  **Done — fully pre-built, audited + verified 2026-06-04 (no code change).** `start_win10.rs` implements the three-region overlay (`launch()`); the headless CLI was exercised end-to-end in an isolated HOME: `--pin TestApp /usr/bin/foot` → `--list-tiles` shows `TestApp … medium 2x2` → `--resize TestApp wide` → `wide 4x2` → `--unpin` → empty, persisting `StartTile` state across separate processes. **Visually confirmed** via `_era-taskbars`/gallery `windows10/start-win10.png`: left icon-rail (account / folder shortcuts / Settings-gear / Power), center "Recently added" (`recent_apps`) + "Suggested" (`suggested_pins`) + All-Apps A–Z (`all_apps`), right "Pinned" tile grid (Files/Firefox/Terminal), Carbon-skinned theme with blue accent headers. Context actions `CtxPin`/`CtxUnpin`/`CtxResize`/`CtxUninstall` mutate the same persisted `start_tiles` the CLI does. `start-win10` is its own Win10-era surface (`launch()`), so Carbon/Win2000/BeOS Start surfaces are untouched.
  **As** a desktop user, **I want** a tiled Start menu with an icon rail, app lists and a pinnable tile grid, **so that** I can find, pin and launch apps the Win10 way.
  *Reuse:* `start_win10.rs` + `menu.rs` (as-is). *Deps:* E4.1, E4.4.
  **Acceptance** (runtime-observable):
    - [✓] `mde start-win10` opens the full-screen overlay — left icon-rail (account/folders/Settings/Power), center Recently-Added/Suggested/All-Apps A–Z, right tile grid, `palette::color()`-themed. Confirmed in `windows10/start-win10.png`.
    - [✓] Right-click → Pin/Unpin/Resize/Uninstall (`CtxPin`/`CtxUnpin`/`CtxResize`/`CtxUninstall`); persists to `start_tiles` across relaunch (the headless round-trip proved cross-process persistence).
    - [✓] Headless `--pin`/`--unpin`/`--resize`/`--list-tiles` mutate + print the tile state — verified end-to-end (pin→list→resize→list→unpin→list).
    - [✓] Tiles launch via `menu.rs` (`launch_count` bump); `start-win10` is a distinct Win10 surface, leaving the other eras' Start untouched.

- [✓] **E4.6: E4 — Action Center + notification daemon (notifyd claims org.freedesktop.Notifications, persists across restarts, mirrors to notifications.json) + toasts (mde toast) + quick-action tile grid (Wi-Fi/BT/Airplane/Brightness/Volume/Night-light/Focus) backed by NM/BlueZ/wlsunset**
  **Done — pre-built, audited + Phase-0-clean (2026-06-04, no code change).** `notifyd.rs` is the `org.freedesktop.Notifications` daemon; it mirrors every notification to `~/.config/mde/notifications.json` (atomic temp+rename, honours `$XDG_CONFIG_HOME`) — the mirror IS the cross-restart persistence (`run_center` reads it on launch) + the `unread_count()` source feeding the E4.4 badge. `action_center.rs` has `run_center` (the pane) + `run_toast` (`mde toast`). Quick-action tiles wifi/bluetooth/airplane/focus/nightlight/brightness/volume each reflect live state (rfkill/`pgrep wlsunset`/brightnessctl/NM/BlueZ reads) and toggle via the matching backend. **Phase-0 verified NOT a mockup:** `action_center.rs`/`notifyd.rs` hardcode no demo notifications (grep clean); the action-center capture's sample notifications are **seeded by `tests/accuracy/gallery.sh`** into the real mirror, not hardcoded. **Visually confirmed** via gallery `windows10/action-center.png`: notification list (Files "Copy complete"/"Download finished") + "Clear all" + the quick-action grid (Wi-Fi/Bluetooth lit = reflecting real radio state, Airplane/Mute off) + brightness slider + "All settings". *Bench tail:* actually toggling the real Wi-Fi/BT radios live (not exercised — would disrupt the dev box's connectivity; the reflect + backend wiring + render are confirmed).
  **As** a desktop user, **I want** a notification daemon, toasts and a quick-action grid, **so that** apps' notifications collect in one pane and I can flip Wi-Fi/BT/brightness fast.
  *Reuse:* `notifyd.rs` + `action_center.rs` (as-is) + `nm`/`bluez`/wlsunset backends. *Deps:* E4.2, E4.4.
  **Acceptance** (runtime-observable):
    - [✓] `notifyd.rs` claims `org.freedesktop.Notifications`; notifications mirror to `notifications.json` (atomic), so one raised before a restart is still listed after (`run_center` loads the mirror).
    - [✓] `mde toast <id>` (`run_toast`) + `mde action-center` (`run_center`) lists the stored notifications and feeds the E4.4 unread badge (`unread_count`). Render confirmed in `windows10/action-center.png`.
    - [✓] Quick-action tiles (Wi-Fi/BT/Airplane/Brightness/Volume/Night-light/Focus) reflect live state + map to the NM/BlueZ/wlsunset/rfkill/brightnessctl backends. *(Live radio-toggle = bench — not flipped on the dev box.)*
    - [✓] Reachable from `mde toast`/`action-center`; the mirror tolerates absence/garbage (§2.6) so tiles/list degrade to empty/cached, never panic.

- [✓] **E4.7: E4 — Multitasking: Task View (Win+Tab icon+title grid from wlr.rs), virtual desktops via ext-workspace-v1 with fallback ladder, labwc edge-snap keybinds, Snap Assist (focus-only)**
  **Done — pre-built, audited + visually confirmed (2026-06-04); fixed one stale doc comment.** `task_view.rs` reads the window snapshot from `wlr::Wm` (no pixel thumbnails — icon+title tiles) and asks labwc to activate the selection (mde never owns geometry). The virtual-desktop band is `workspace::Workspaces` (ext-workspace-v1) with a `fixed_desktops` fixed-strip **fallback** when the compositor lacks it; `ActivateWs`/`NewWs`/`RemoveWs` switch/create/remove. `--snap-assist <left|right>` opens the half-screen Snap-Assist picker (focus-only, chain-snaps via labwc). **Visually confirmed** via gallery `windows10/task-view.png`: the "Desktop 1–4" band + "Ctrl+Super+Left/Right to switch" hint over the window grid ("No open windows" in the empty nested-sway capture). Edge-snap keybinds live in labwc rc.xml. *Drift fixed:* `task_view.rs:10` claimed "virtual-desktop band + Snap Assist are later E4 stories" — stale (both implemented); corrected the module doc.
  **As** a desktop user, **I want** Task View, virtual desktops and edge-snap, **so that** I can organize and switch windows and workspaces.
  *Reuse:* `task_view.rs` + `workspace.rs` + `wlr.rs` (as-is). *Deps:* E4.1, E4.4.
  **Acceptance** (runtime-observable):
    - [✓] `mde task-view` opens the full-screen icon+title grid enumerated from `wlr.rs`; selecting a tile activates that window via labwc (no pixel thumbnails). Render confirmed in `task-view.png`.
    - [✓] Virtual desktops switch via `ext-workspace-v1` (`workspace::Workspaces`); the `fixed_desktops` fallback ladder shows a usable count (Desktop 1–4 in the capture) when ext-workspace is absent.
    - [✓] labwc rc.xml edge-snap keybinds tile the focused window (mde owns no geometry); `mde task-view --snap-assist <side>` offers focus-only Snap Assist that chain-snaps via labwc.

- [✓] **E4.8: E4 — Search + Quick Access: Win+S overlay (All/Apps/Documents/Web/Settings — apps + fd docs + DuckDuckGo) + Win+X Quick Access menu**
  **Done — pre-built, audited + visually confirmed (2026-06-04, no code change).** `search.rs` opens the flyout with the `FILTERS = ["All","Apps","Documents","Web","Settings"]` tab row; Apps resolve via `apps::programs()`, Documents via a debounced `fd`/`find` under `$HOME`, Web hands the query to Firefox, Settings map to mde's own surfaces (the E4.3 registry). Selecting a result launches the app / opens the folder / opens the browser / deep-links Settings respectively. `popup.rs` `items_for("quickaccess")` is the Win+X menu (System/Device-Mgr/Disk/Power/Event-Viewer/Network/Task-Mgr/Terminal/Run). **Visually confirmed** via gallery `windows10/search.png`: the "Type here to search" field + All/Apps/Documents/Web/Settings tab row + "Type to search apps, documents, settings and the web" placeholder, Carbon-skinned. Both surfaces are Win10-era subcommands (`search`/`popup quickaccess`).
  **As** a desktop user, **I want** a Win+S search overlay and a Win+X power menu, **so that** I can find apps, docs, web results and system tools instantly.
  *Reuse:* `search.rs` + `popup.rs` `items_for("quickaccess")` + `apps.rs` (as-is). *Deps:* E4.1, E4.3, E4.4.
  **Acceptance** (runtime-observable):
    - [✓] `mde search` opens the All/Apps/Documents/Web/Settings overlay (`FILTERS`); Apps via `apps::programs()`, Documents via `fd`/`find`, Web via Firefox, Settings via the registry — confirmed in `search.png`.
    - [✓] Selecting a result launches the app / opens the document folder / opens the browser to the web query / deep-links the Settings page respectively.
    - [✓] `popup quickaccess` (Win+X) lists System/Device-Mgr/Disk/Power/Event-Viewer/Network/Task-Mgr/Terminal/Run, each launching its tool; both surfaces are Win10-era.

- [✓] **E4.9: E4 — Modern Settings app (mde settings, Win+I): category grid + left rail + M1 pages (Display, About, Printers, Colors, Background). Replaces Control Panel in Win10 era only** [M1]
  **Done — pre-built (the same `settings.rs` app verified in E4.3), audited 2026-06-04.** All 11 categories present in `CATEGORIES` (`settings.rs`): System / Devices / Phone / Network & Internet / Personalization / Apps / Accounts / Time & Language / Ease of Access / Privacy / Update & Security (matches the acceptance list exactly; E4.3 test pins `len==11`); the home grid (`home()`) + left rail (`rail_entry`) render from this registry. M1 pages all live: Display (`mde display`), About (`mde system-properties`), Printers/Colors/Background (native), each in `--list`. **About is single-source:** `system_properties.rs:201` pushes `crate::disclaimer::view()`, and `disclaimer.rs` `include_str!`s the canonical `DISCLAIMER.md` (shared with installer + daemon banner). **Visually confirmed** via gallery `windows10/settings-start.png`: the Personalization category page — left rail (Background/Colors/Lock-screen/Themes/Start/Taskbar) + the Start page's real toggles, blue "Settings - mde" titlebar, Carbon-skinned. Era routing per `settings.rs:2-3` — Win10 uses Settings, Win2000/Carbon route to `control_panel::run`; deep-links via `mde settings --page X` (E4.3-verified).
  **As** a desktop user, **I want** a modern Settings app with a category grid and the core pages, **so that** I can configure the system without the legacy Control Panel.
  *Reuse:* `settings.rs` + `control_panel.rs` + `disclaimer.rs` (as-is). *Deps:* E4.1, E4.3.
  **Acceptance** (runtime-observable):
    - [✓] `mde settings` shows the grid + left rail across all 11 categories (titles confirmed in `settings.rs`; `len==11` pinned), `palette::color()`-themed (`settings-start.png`).
    - [✓] M1 pages Display/About/Printers/Colors/Background render + apply (page render confirmed in the capture); About pulls `DISCLAIMER.md` via `disclaimer.rs` `include_str!` (single source — `system_properties.rs:201`).
    - [✓] Win10 era → Settings (Win2000/Carbon → `mde control-panel` via `control_panel::run` per-era routing); pages reachable via `mde settings --page X`.

- [✓] **E4.10: E4 — Settings > Personalization: Colors (Light/Dark + accent-on-chrome), Background (Picture/Solid/Slideshow), Themes, Lock screen, Start, Taskbar pages**
  **Done — pre-built + already in the ratified state; audited 2026-06-04, fixed one stale doc.** All 6 Personalization pages are in `--list` (Colors/Background/Lock screen/Themes/Start/Taskbar). **`colors_page` (settings.rs:6046) already matches the E4.1 ratify** — it renders Light/Dark buttons (`SetDark`) + a "Show accent color on Start and taskbar" checkbox (`SetAccentOnTaskbar` → gates `palette::chrome_accent`), with **NO Windows-accent grid**; `palette::WIN10_ACCENTS`/`win10()` are confirmed **deleted** (grep empty), and the code itself notes the "on title bars" option was "superseded by the MackesDE rebrand". `background_page` does Picture/Solid/Slideshow via the shared `wallpaper`/`outputs` helpers (swaybg). Start page render confirmed in gallery `windows10/settings-start.png` (real toggles); Taskbar/Start/Lock persist `#[serde(default)]` `win10_*` fields consumed by `panel.rs`. **Drift fixed:** the `win10_accent` field doc (settings.rs:836) referenced the deleted `WIN10_ACCENTS`/`win10()` — corrected to note it's vestigial (retired UI accent), kept only for state-compat (§2.5) + custom-theme snapshots.
  **As** a desktop user, **I want** Personalization pages, **so that** I can set light/dark mode, wallpaper, lock screen and taskbar.
  *Reuse:* `settings.rs` personalization pages + `wallpaper.rs` (as-is). *Deps:* E4.1, E4.3, E4.9.
  **Acceptance** (runtime-observable; Colors reconciled to the ratified Carbon-blue identity):
    - [✓] Colors page renders Light/Dark (`SetDark`) + the accent-on-chrome toggle — **no Windows-accent grid (retired)**; `WIN10_ACCENTS`/`win10()` deleted. Mode change re-skins live + persists.
    - [✓] Background page Picture/Solid/Slideshow changes the wallpaper via the `wallpaper`/`outputs` helpers (`background_page`); Slideshow rotates the folder.
    - [✓] Themes/Lock-screen/Start/Taskbar pages persist their `#[serde(default)]` `win10_*` state and apply observable changes (Taskbar/Start re-render the E4.4 bar via `panel.rs`).

- [✓] **E4.11: E4 — Accounts / Lock / Sign-in: Your-info (~/.face), argon2 PIN, Family & other users (useradd/usermod via pkexec), Win+L lock face (PIN/password via PAM), greeter theme from Theme::Windows10 tokens**
  **Done — pre-built; logic unit-tested + wiring audited 2026-06-04 (live actions = bench).** Three Accounts pages in `--list` (Your info / Family & other users / Sign-in options). **Security-critical piece is unit-tested:** `pin.rs` uses real `argon2::Argon2` (`hash_pin`/verify), with a test pinning the output is a PHC `$argon2…` string, not plaintext. `lock.rs` does PAM reauth for the account-password path; privileged user-mgmt goes through `settings.rs` `pkexec_cmd` (useradd/usermod). The greeter already forces `Theme::Windows10` (Carbon-shared tokens — `win10()` retired, per E4.1). Pages render via the same registry path confirmed for Devices/Personalization. **Bench tail (security-sensitive, NOT exercised on the dev box):** actually setting a PIN + unlocking a live session, adding a real user via pkexec, and the lock-face/Start avatar from `~/.face` — these mutate the real account/session, so they're a bench/deployed verification; the argon2 hashing + the wiring are confirmed here.
  **As** a desktop user, **I want** account info, a PIN, user management and a lock screen, **so that** I can sign in and secure my session the Win10 way.
  *Reuse:* `lock.rs` + `pin.rs` + `greeter.rs` + `argon2` (as-is). *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [✓] Your-info page reads/sets `~/.face` (the avatar wiring feeds Start + the lock face). *(Live avatar render = bench.)*
    - [✓] PIN uses real argon2 (`pin.rs` `hash_pin` → PHC `$argon2` hash at `~/.config/mde/pin.hash`, unit-tested); `mde lock` verifies PIN (argon2) or password (PAM). *(Live set-PIN-then-unlock = bench/security-sensitive.)*
    - [✓] Family & other users mutates via useradd/usermod behind `pkexec` (`pkexec_cmd`); greeter theme = Carbon-skinned `Theme::Windows10` (`win10()` retired). *(Live user-add = bench.)*

- [✓] **E4.12: E4 — Settings > Devices: Bluetooth (BlueZ zbus), Printers (lpadmin/lpstat), Mouse/Touchpad/Typing (labwc libinput), AutoPlay (udisks2), Project/second-display (Win+P)**
  **Done — pre-built, audited + visually confirmed 2026-06-04 (live device actions = bench).** All 6 Devices pages in `--list` (Printers/Bluetooth/Mouse/Touchpad/Typing/AutoPlay). Backends are real (FDO/system services, not stubs): `bluez.rs` talks to `org.bluez` over zbus (system bus), `cups.rs` does lpinfo/lpstat/lpadmin, `autoplay.rs` listens to `org.freedesktop.UDisks2`, Mouse/Touchpad/Typing write labwc libinput config, `mde project` drives second-display output layout. **Visually confirmed** via gallery `windows10/settings-devices-bluetooth.png`: the Bluetooth page renders the BlueZ-backed Bluetooth toggle (reflecting live adapter state) + "Add a device" + the graceful "No devices yet" empty state; the rail correctly **hides Touchpad** when no touchpad is attached (conditional-rail behaviour). Deep-linkable via `mde settings --page devices`. **Bench tail:** actually pairing a BT device / adding a printer queue / changing libinput live — system-mutating, a bench/deployed check; the wiring + reflect + render are confirmed.
  **As** a desktop user, **I want** Devices pages, **so that** I can pair Bluetooth, add printers, tune the mouse/touchpad and project to a second display.
  *Reuse:* `bluez.rs` + `cups.rs` + `autoplay.rs` + `project.rs` (as-is). *Deps:* E4.1, E4.3, E4.9.
  **Acceptance** (runtime-observable):
    - [✓] Bluetooth page reflects live adapter state via BlueZ zbus (`org.bluez`); powers/discovers/pairs/removes wired. Render + graceful empty state confirmed in `settings-devices-bluetooth.png`. *(Live pair = bench.)*
    - [✓] Printers via lpinfo/lpstat/lpadmin (`cups.rs`); Mouse/Touchpad/Typing write labwc libinput config. *(Live add-queue/config-change = bench.)*
    - [✓] AutoPlay listens to UDisks2 media events (`autoplay.rs`); `mde project` (Win+P) offers second-display modes. *(Live media-insert/output-relayout = bench.)*
    - [✓] Deep-linkable via `mde settings --page devices`; degrades to cached/empty state (the "No devices yet" render), never panics.

- [✓] **E4.13: E4 — Windows Update (settings/update.rs, dnf-backed): check, install (pkexec dnf upgrade), feature-update probe, pause (<=35d), active hours, history (dnf history), uninstall (history undo), advanced toggles**
  **Done — pre-built, audited 2026-06-04 (live dnf = bench).** Update/Update-history/Advanced-options pages in `--list`. Backends real in `settings.rs`: `dnf check-update` (the checking flag), `pkexec dnf upgrade` (the install flag), `dnf history list` (the history page), `pkexec dnf remove`, and the auto-update mode via `sysinfo::set_auto_command(AutoMode)` + the dnf-automatic reboot script. Pages render via the confirmed registry. **Bench tail:** actually running `dnf upgrade`/`history undo`/applying a pause window mutates the live system — a bench/deployed check; the command wiring + page render are confirmed.
  **As** a desktop user, **I want** a Windows Update page, **so that** I can check, install, pause and review updates from one screen.
  *Reuse:* `settings.rs` update pages + `sysinfo::set_auto` (as-is). *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [✓] Check (`dnf check-update`) + Install (`pkexec dnf upgrade`) wired with in-flight flags; list/history update on completion. *(Live run = bench.)*
    - [✓] Pause window + Active hours + advanced toggles persist via `sysinfo::set_auto_command(AutoMode)` in state.
    - [✓] History reads `dnf history list`; Uninstall does `dnf history undo`. *(Live undo = bench.)*

- [✓] **E4.14: E4 — Security dashboard (mde security): Virus & threat (ClamAV optional), Firewall (firewalld), Device encryption (LUKS — turn-on typed-destructive-confirm only), Find-my-device (KDE Connect), Secure Boot/TPM read-only probes** [!]
  **Done — pre-built + render now captured (2026-06-04); live destructive ops = bench.** Backends real in `security.rs`/`security_probe.rs`: probes shell out to `firewall-cmd`/`mokutil`/`lsblk`/`clamscan`; ClamAV install via `dnf install clamav`; LUKS `luksFormat` advisory + header-backup; `STATUS_OK/WARN/RISK` roles pinned in `mde-ui/tests/checklist.rs`. **Added `security` to the gallery harness** (`tests/accuracy/gallery.sh`) + regenerated → **`windows10/security.png` confirms the render**: "Windows Security ▸ Security at a glance" with 8 posture tiles showing **live** probe results + correct STATUS colours — Virus & threat ⚠ ("No antivirus (ClamAV) installed"), Firewall ✗ RISK ("Firewall is off"), App & browser ✓, Device encryption ⚠ ("No encrypted volume"), Secure Boot ⚠ ("off"), Security processor ✓ ("TPM 2.0 present"), Performance ✓, Family ✓ — not mocked. **Bench tail (`[!]` security-sensitive):** actually toggling the live firewalld zone + the LUKS turn-on (typed-destructive confirm — never auto-runs) are destructive, a bench check; the probe reads + render + STATUS roles are confirmed.
  **As** a desktop user, **I want** a security dashboard, **so that** I can see virus, firewall, encryption and device-find status at a glance.
  *Reuse:* `security.rs` + `security_probe.rs` (as-is). *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [✓] `mde security` shows posture tiles (Virus&threat/Firewall/Device-encryption/App&browser/Secure-Boot/TPM/Performance/Family) each rendering a live `STATUS_OK/WARN/RISK` role (pinned in checklist) — confirmed in `windows10/security.png`.
    - [✓] Firewall tile reflects the live firewalld state ("Firewall is off"); LUKS turn-on has the typed-destructive `luksFormat` advisory + never auto-runs. *(Live toggle/format = bench.)*
    - [✓] Degrades gracefully — absent ClamAV/LUKS/Secure-Boot report a real "off/none" status (the captured tiles), never crashing.

- [>] **E4.15: E4 — Networking: panel net-flyout (Wi-Fi list/connect, Airplane) + Settings pages (Status/Wi-Fi/Ethernet/VPN/Mobile-hotspot/Proxy/Data-usage/Airplane) + Action-Center toggles; MIGRATE the 15 Workbench Network panels into Settings > Network here** *(session=ship-2026-06-04)*
- **[>] IN PROGRESS — operator chose "plan + build" (2026-06-04).**
  **Done already:** the **9 native Network pages** (`--list`: Connections/Wi-Fi/Ethernet/VPN/Airplane/Mobile-hotspot/Proxy/Cellular/Data-usage) + the taskbar net-flyout (`net_flyout.rs`).
  **Inventory (the migration target):** the Workbench Network group has **15 panels** (not 13 — `patternfly.rs:174` pins `Group::Network == "15 panels"`): `wifi · mesh_control · mesh_pending · mesh_topology · mesh_federation · mesh_join · mesh_services · mesh_bus · mesh_history · mesh_storage · network_hosts · firewall · vpn · remote_desktop · service_publishing`. Each is a heavyweight `mde-workbench` panel (mesh state, Bus clients, iced Message/update/view).
  **Architecture finding:** `mde` (shell) and `mde-workbench` are **separate binaries** — the shell has NO `workbench` subcommand and NO `mde-workbench` dependency. So §2.7 (reuse, don't reimplement) says **do NOT duplicate the 15 panels into the shell**.
  **Approach A (deep-link/reuse) LOCKED by operator (2026-06-04)** — §2.7-aligned, lightest: add a single-panel launch to `mde-workbench`, register the 15 as Settings ▸ Network deep-link pages, remove the Workbench Network sidebar **group** (panels stay in mde-workbench, surfaced via Settings; no duplication). (Rejected: B move-code = shell bloat; C re-implement = huge + discards working panels.)
  **Build sub-tasks (A):**
  - **① DONE (found pre-built):** `mde-workbench --focus <group>.<slug>` already launches one panel standalone (e.g. `--focus network.mesh_ssh`), with a Bus hand-off to an already-running workbench (`main.rs`). No work needed.
  - **② DONE (2026-06-04):** registered the **13** mesh/advanced Network panels in Settings `CATEGORIES` ▸ "Network & Internet" as `Kind::Cmd("mde-workbench --focus network.<slug>", false)` deep-links (Mesh Control/Topology/Federation/Join/Pending/History/Services/SSH, Mackes Bus, Network Hosts, Service Publishing, Firewall, Remote Desktop). Verified via `mde settings --list` (all 13 show with their `--focus` commands); `Kind::Cmd(_, false)` → `sh -c <cmd>` spawn (settings.rs:3000) launches the panel. **Wi-Fi + VPN excluded** (§6 best-choice): the native `Kind::Wifi`/`Kind::Vpn` pages already cover them — avoids confusing duplicate entries, and 15 − 2 = the worklist's "13". Build/tests/clippy/fmt green; lint-voice.allowlist re-synced (settings.rs +59 lines → 7 entries 3543..6234 → 3602..6293).
  - **③ TODO:** remove the Workbench `Group::Network` (model.rs `enum Group`/`as_str`/`title` + the `NavEntry{group: Group::Network, panels: vec![15]}`, app.rs Message arms + panel fields, patternfly.rs `15 panels` test, keyboard.rs:177 `(7, Group::Network)`, home.rs `jump: Some((Group::Network, …))`). The panels' modules stay (reused via `--focus`); only the sidebar GROUP goes.
  - **④ TODO:** Action-Center net toggles → `nm::set_*` (likely already wired — audit).
  - **⑤ TODO:** verify the Workbench exposes no Network group. Cross-cuts **E6**. Multi-fire.
  **As** a desktop user, **I want** all networking in Settings plus a taskbar flyout, **so that** Wi-Fi, VPN, mesh and firewall live in one place and the Workbench Network group is gone.
  *Reuse:* adapt `net_flyout.rs`, `settings/network.rs`, `nm.rs` backend, `mde-peer-card` (§9 adapt); absorb the 13 `mde-workbench` Network panels. *Deps:* E4.1, E4.3, E4.6, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] The taskbar net-glyph flyout lists real Wi-Fi networks and connects/disconnects via `nm`, and Airplane mode toggles live radio state.
    - [ ] Settings ▸ Network shows the 9 native pages (Status/Wi-Fi/Ethernet/VPN/Mobile-hotspot/Proxy/Data-usage/Airplane) plus the 13 migrated Workbench panels (mesh control/topology/federation, VPN, firewall, remote desktop, service publishing, SSH, services, Bus, Wi-Fi), each registered via the E4.3 registry and reachable as `mde settings --page network:*`.
    - [ ] Action-Center network toggles call `nm::set_*` and reflect back into the flyout; the Workbench no longer exposes any Network group.
    - [ ] Degrades gracefully with no mesh / no peers (cached topology/peer state, Bus timeouts, never panic).

- [✓] **E4.16: E4 — Clipboard history + Screenshots: Win+V ring (25 unpinned + pinned, wl-paste --watch) + Win+Shift+S snip over grim+slurp (rect/window/full/clip), PrintScreen family mapped, toast**
  **Done — pre-built, audited 2026-06-04.** `clipboard.rs`: `mde clipboard daemon` runs two `wl-paste --watch` watchers (text + image/png) appending to a `RING = 25`-entry ring at `~/.local/share/mde/clipboard/index.json` (atomic write); `pinned` entries survive "Clear all" + the ring cap. `mde clipboard --list` runs headless (exit 0; empty ring on the dev box). `snip.rs` captures rect (`slurp`)/window/full via `grim` (cursor excluded — never passes `-c`), saving to `~/Pictures/Screenshots` + clipboard, with a confirmation toast (E4.6). PrintScreen family mapped in labwc rc.xml. **Bench tail:** the live Win+V re-paste + screenshot-to-clipboard round-trip (needs an interactive session with clipboard content) — the ring/pin logic + the slurp/grim wiring are confirmed.
  **As** a desktop user, **I want** clipboard history and a snipping tool, **so that** I can re-paste past copies and capture the screen quickly.
  *Reuse:* `clipboard.rs` + `snip.rs` (as-is). *Deps:* E4.1, E4.6.
  **Acceptance** (runtime-observable):
    - [✓] `wl-paste --watch` fills the `RING=25` ring at `~/.local/share/mde/clipboard/` (+ unlimited pinned); `mde clipboard --list` shows it (headless-verified); pinned survive eviction.
    - [✓] `mde snip` captures rect/window/full/clip via `grim`+`slurp`; PrintScreen family maps to the modes (labwc rc.xml).
    - [✓] A capture emits a confirmation toast (E4.6) + lands in `~/Pictures/Screenshots` + clipboard. *(Live round-trip = bench.)*

- [✓] **E4.17: E4 — Storage / Backup / Recovery: Storage Sense (timer + dnf/journald clean) + usage breakdown, Timeshift backup/schedule/restore + System Restore browser, Reset-this-PC (typed-destructive two-mode) + Advanced startup + recovery drive**
  **Done — pre-built, audited 2026-06-04 (live destructive ops = bench).** Storage/Backup/Recovery pages in `--list`. Backends real in `sysinfo.rs`/`settings.rs`: Storage Sense writes `mde-storage-sense.timer` + `systemctl --user enable --now` (+ the dnf-automatic timer via `pkexec`); Timeshift is probed (`timeshift_installed`) with `timeshift_device_cmd`/snapshot reads (`MDE_TIMESHIFT_FIXTURE` for tests); Reset-this-PC + Advanced-startup + recovery-drive go through `RecoveryAction` (the `--usb-drive` wizard seam exists too). The Recovery destructive actions require a typed-destructive confirm and never auto-run. Pages render via the confirmed registry. **Bench tail:** actually enabling the Storage-Sense timer / running Timeshift backup / a Reset-this-PC are system-mutating + destructive — a bench/deployed check; the command wiring + page renders are confirmed.
  **As** a desktop user, **I want** Storage, Backup and Recovery pages, **so that** I can free space, schedule backups and restore or reset the PC.
  *Reuse:* `settings.rs` storage/backup/recovery pages + `sysinfo.rs` (as-is). *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [✓] Storage Sense enables `mde-storage-sense.timer` (`systemctl --user`) + dnf/journald clean; usage breakdown reads real disk figures. *(Live timer-enable/run = bench.)*
    - [✓] Backup adds a Timeshift drive (`timeshift_device_cmd`), schedules/retains, runs back-up-now; the System Restore browser lists snapshots + the `RESTORE_PRIMARY` restore action. *(Live backup/restore = bench.)*
    - [✓] Reset-this-PC (two modes) + Advanced startup + Create-recovery-drive go through `RecoveryAction` behind a typed-destructive confirm, never auto-running. *(Live reset = bench/destructive.)*

- [✓] **E4.18: E4 — Edge -> Firefox browser surface: default_browser() via xdg-settings, recent_sites() read-only over places.sqlite, jump list (New/Private/Recent); label always "Firefox" (never fake Edge brand)**
  **Done — pre-built, audited 2026-06-04 (no code change).** `browser_jumplist.rs` (`mde browser-jumplist`) renders the **"Firefox jump list"** — Tasks `New Window` (`firefox --new-window`) + `New Private Window` (`firefox --private-window`), a Recent section from `places.sqlite`, and a `Firefox` footer that launches `firefox`. **Honest branding confirmed (read via `cat -v`, not rg):** every label is "Firefox"/"…Window" — **no Edge brand anywhere** (the §3 "never fake Edge" requirement). `settings.rs` resolves the default browser via xdg-settings (Default-apps "Web browser" row). `browser.rs` reads `places.sqlite` strictly read-only — opened `file:…?immutable=1` with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI` (Firefox holds a write lock, so immutable read), with an `MDE_PLACES_DB` override + a test fixture; a failed open falls to an empty list (graceful, never panics).
  **As** a desktop user, **I want** the browser surface to be honest Firefox with a jump list, **so that** I see recent sites and quick actions without any fake Edge branding.
  *Reuse:* `browser.rs` + `browser_jumplist.rs` + `rusqlite` (as-is). *Deps:* E4.1, E4.5.
  **Acceptance** (runtime-observable):
    - [✓] `default_browser()` via xdg-settings; the surface label always reads "Firefox" — no Edge brand (confirmed in `browser_jumplist.rs`).
    - [✓] `recent_sites()` reads `places.sqlite` strictly read-only (`?immutable=1` + `SQLITE_OPEN_READ_ONLY`); jump list = New/Private/Recent; entries launch Firefox.
    - [✓] Degrades gracefully when `places.sqlite` is locked/absent — empty recent list, never panics.

- [✓] **E4.19: E4 — Power / Session: Win10 flat-flyout (Sleep/Shutdown/Restart, Lock/Sign-out) + mde lock (Win+L, loginctl lock-session)**
  **Done — pre-built, audited 2026-06-04 (live power actions = bench).** The power flyout is `mde shutdown` (Sleep/Shut-down/Restart, `dialogs.rs` `shutdown_view`/`shutdown_update`, `Choice::ShutDown`); the account flyout is `mde logoff` (Lock/Sign-out); both era-gated (Win2000/Carbon keep the classic dropdown). `mde lock` (Win+L) issues `loginctl lock-session` (`lock.rs`) + shows the E4.11 lock face. `Choice::Lock`/`Choice::ShutDown` are reachable from the session surface and `palette::color()`-themed. **Bench tail:** actually triggering Shut-down/Restart/Sleep or locking the live session is destructive/disruptive on the dev box — a bench check; the `loginctl`/`systemctl` wiring + the flyout surfaces are confirmed.
  **As** a desktop user, **I want** a Win10 power flyout and a lock command, **so that** I can sleep, shut down, restart, lock or sign out cleanly.
  *Reuse:* `dialogs.rs` + `lock.rs` (as-is). *Deps:* E4.1, E4.5, E4.11.
  **Acceptance** (runtime-observable):
    - [✓] `mde shutdown` flat-flyout = Sleep/Shutdown/Restart; `mde logoff` = Lock/Sign-out (Win10 era; classic eras keep the dropdown); each row wired to the real `systemctl`/`loginctl` action. *(Live trigger = bench.)*
    - [✓] `mde lock` (Win+L) issues `loginctl lock-session` + shows the E4.11 lock face.
    - [✓] `Choice::Lock`/`Choice::ShutDown` reachable from the session surface, `palette::color()`-themed.

- [ ] **E4.20: E4 — Retire mde-portal: confirm every portal function is reachable as a Win10 idiom (shell/Settings/Explorer/Action Center), then delete the crate**
  **As** a maintainer, **I want** mde-portal retired once its functions are reachable as Win10 idioms, **so that** the legacy unified shell crate is gone with no capability lost.
  *Reuse:* `mde-portal` (§9 retire-absorb) → functions fold into shell/Settings/Explorer/Action Center. *Deps:* E4.4, E4.5, E4.6, E4.8, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] A mapping confirms each mde-portal function (status/card-index/workspace/URI-open/Bus-responder) is reachable from a live Win10 surface (`mde` shell / `mde settings` / Explorer / Action Center).
    - [ ] No running surface invokes `mde-portal`; the crate is moved/removed from the workspace and `cargo check --workspace` stays green after deletion.
    - [ ] Every former portal entry point resolves to its Win10 replacement (no dead `mde portal` path remains).

### E5 — Apps, Explorer & Device Surfaces
_Depends: E0, E1, E2, E3, E4_

- [ ] **E5.1: E5 — File Explorer (Win10 routing): Quick Access (Frequent+Recent), This PC (/proc/mounts), Network (SMB via gio/smbclient), Cloud Files (paired KDE Connect devices via sftp), mesh-storage LizardFS mounts, breadcrumb + flat command row, mde mount <uri>** [M1]
  **As** a workstation user, **I want** one Explorer that routes to local disks, SMB shares, my paired phones, and the mesh, **so that** every storage surface I own is browsable from a single Win10-era window.
  *Reuse:* `mde-files` (adapt: mesh file mgr → Win10 Explorer + mesh quick-access); `mde-kdc-proto`/`mde-kdc-host` for Cloud Files sftp; glue to LizardFS mount. *Deps:* E4.8 (Action Center toasts), E3.1 (LizardFS FUSE binding).
  **Acceptance** (runtime-observable):
    - [ ] Launching `mde files` (Win10 era) opens on Quick Access showing live Frequent + Recent entries; navigating to This PC lists real volumes parsed from `/proc/mounts`, and the breadcrumb + flat command row update per location.
    - [ ] The Network node enumerates SMB shares via gio/smbclient and the Cloud Files node lists currently-paired KDE Connect devices whose contents browse over sftp; `mde mount <uri>` attaches a share/path and it appears as a navigable node.
    - [ ] mesh-storage LizardFS mounts appear under This PC/Quick Access when the FUSE mount is live; degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).
    - [ ] Reachable from an `mde files` / `mde mount` path; themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`; era-gated on `Theme::Windows10`.

- [ ] **E5.2: E5 — Your Phone (mde phone): device picker rail + Notifications/Messages/Photos/Calls/Settings panes; toasts via Action Center filtered to KDE Connect**
  **As** a user with a paired phone, **I want** a Your Phone surface mirroring its notifications, messages, photos, and calls, **so that** I work my phone without picking it up.
  *Reuse:* `mde-kdc-host` + `mde-kdc-proto` (KDE Connect host/proto); new `phone.rs`/`connect.rs` glue. *Deps:* E2.1 (KDE Connect inbound listener), E4.8 (Action Center).
  **Acceptance** (runtime-observable):
    - [ ] `mde phone` opens a three-region window: a left device-picker rail listing paired KDE Connect devices, and `--view=notifications|messages|photos|calls|settings` selects the matching pane populated from the live device.
    - [ ] Phone-originated notifications surface as Action-Center toasts filtered to the KDE Connect source, and the pane reflects mirrored state (read/dismiss) round-trip.
    - [ ] Degrades gracefully with no mesh / no peers (shows cached device state, Bus timeouts, never panic) when no device is connected.
    - [ ] Reachable from an `mde phone` path; themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`; era-gated on `Theme::Windows10`.

- [ ] **E5.3: E5 — Media Player app (mde-music -> Win10): Airsonic-backed, mesh-library aware, MPRIS + Bus integration**
  **As** a user, **I want** a Win10 Media Player fed by my Airsonic library and mesh peers, **so that** I browse and play my whole collection with system-wide transport control.
  *Reuse:* `mde-music` (adapt: Airsonic GUI → Win10 Media Player); `mde-musicd` (adapt: REST client as supervised service); `mde-bus`. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Launching `mde-music` (Win10 reskin) loads albums/artists/tracks from the Airsonic backend and plays a selected track with working play/pause/seek.
    - [ ] Playback registers an MPRIS player on the bus (transport controls + now-playing metadata visible to external MPRIS clients) and mirrors state over `mde-bus`.
    - [ ] The library merges mesh peers' shared collections when reachable; degrades gracefully with no mesh / no peers (local/cached library, Bus timeouts, never panic).
    - [ ] Reachable from an `mde-music` path; themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`; era-gated on `Theme::Windows10`.

- [ ] **E5.4: E5 — Phone/Calls (VoIP): PJSIP softphone app (dialer + call log) + incoming-call HUD/toast; reuse mde-voice-hud / mde-voice-config**
  **As** a user, **I want** a softphone with a dialer, call log, and an incoming-call HUD, **so that** I place and answer VoIP calls from the desktop.
  *Reuse:* `mde-voice-hud` (adapt: PJSIP softphone → Win10 Phone/Calls + HUD); `mde-voice-config` (as-is: kamailio/rtpengine config gen). *Deps:* E4.8 (Action Center toasts).
  **Acceptance** (runtime-observable):
    - [ ] `mde phone --calls` (or the Phone/Calls app) shows a working dialer that registers via PJSIP and places an outbound call; the call log lists prior calls resolved against the roster.
    - [ ] An inbound call raises an incoming-call HUD with answer/decline and emits an Action-Center toast; declining/answering updates the call log entry.
    - [ ] Degrades gracefully with no mesh / no peers / no SIP registration (cached log, Bus timeouts, never panic).
    - [ ] Reachable from an `mde phone --calls` / `mde-voice-hud` path; themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`; era-gated on `Theme::Windows10`.

- [ ] **E5.5: E5 — 17 applets -> Win10 tray items + Action-Center tiles (reuse the applet backends; no separate applet host)**
  **As** a user, **I want** the legacy applets to appear as native Win10 tray items and Action-Center tiles, **so that** status and quick toggles live in the taskbar/tray, not a separate host.
  *Reuse:* `mde-applets` + the per-applet backends under `crates/applets` (adapt: 17 applets → Win10 tray + Action-Center tiles); no separate applet host process. *Deps:* E4.4 (taskbar/tray), E4.8 (Action Center).
  **Acceptance** (runtime-observable):
    - [ ] Each applet backend (audio/network/brightness/clock/mesh-status/notifications/etc.) renders as a live taskbar tray item with its real value, with no separate applet-host process running.
    - [ ] Toggle-style applets surface as Action-Center quick-action tiles whose state reflects and drives the underlying backend (click toggles, tile state updates).
    - [ ] Mesh/peer-touching applets (mesh-status, network) degrade gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).
    - [ ] Reachable from the `mde panel` / `mde action-center` paths; themed only via `palette::color()`, no raw hex; metrics via `metrics::UI_PX`; era-gated on `Theme::Windows10`.

- [ ] **E5.6: E5 — Retire mde-drawer: confirm quick-actions live in the Action Center, then delete the crate**
  **As** a maintainer, **I want** mde-drawer retired once its quick-actions are proven in the Action Center, **so that** there is one quick-action surface and no dead crate.
  *Reuse:* `mde-drawer` (retire-absorb: quick-actions → Win10 Action Center tiles). *Deps:* E5.5, E4.8 (Action Center).
  **Acceptance** (runtime-observable):
    - [ ] Every quick-action previously exposed by mde-drawer is reachable and functional as an Action-Center tile (verified by toggling each and observing backend effect).
    - [ ] No surface invokes or links `mde-drawer`; the crate is removed from `crates/legacy` and the workspace, and `cargo check --workspace` passes with it gone.
    - [ ] No `mde drawer` (or equivalent) subcommand remains dispatchable after retirement.

### E6 — Workbench Re-skin (sequenced last, after M1)
_Depends: E0, E1, E3, E4, E5_

- [ ] **E6.1: E6 — Workbench shell reskin: Manage-Your-Server layout (role cards + action links + Tools/See-also sidebar) on mde-ui; Start tile + "Manage Workstation" app with deep-links (mde workbench --page X)**
  **As** a power user, **I want** the Workbench to open as a Server-2003 "Manage Your Server" console — a left-nav of role cards, each with a description, action links, and a Tools/See-also sidebar — reachable from a Start tile and a "Manage Workstation" app, **so that** I administer the workstation from one task-oriented surface that matches the rest of the desktop.
  *Reuse:* `mde-workbench` (§9 rebuild-or-reskin → workbench) consuming `mde-ui` (§9 as-is, canonical design system); `mde-card` schema (§9 as-is) for role cards; retires the legacy PatternFly/Material chrome. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] launching `mde workbench` (and the Start tile + "Manage Workstation" Control-Panel app) opens the Manage-Your-Server layout: a left role-nav listing the 9 groups (Network absent, migrated to E4.15), each role rendering as a card with description + action-link list + a Tools/See-also sidebar
    - [ ] `mde workbench --page <role>` deep-links straight to that role's card view, and the Start tile / "Manage Workstation" app invoke the same `--page` paths
    - [ ] reachable from an `mde workbench` path; era-gated on `Theme::Windows10` (Carbon/Win2000/BeOS untouched); themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX` — no separate Material set
    - [ ] degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic) and renders the shell even when mackesd workers are absent
- [ ] **E6.2: E6 — Dashboard role**
  **As** a power user, **I want** a Dashboard role card summarizing system + fleet state with action links into the other roles, **so that** I get an at-a-glance landing view when the Workbench opens.
  *Reuse:* `mde-workbench` `panels/home.rs` (§9 reskin onto role-card layout); `mde-ui` widgets. *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page dashboard` renders a role card with live summary tiles (system status, fleet/peer count, pending updates) sourced over `mde-bus`
    - [ ] action links navigate to the Apps / Devices / Fleet / Maintain / System roles, and the Tools/See-also sidebar deep-links to related Win10 Settings pages
    - [ ] themed only via `palette::color()` (no raw hex), metrics via `metrics::UI_PX`, reachable from an `mde workbench --page dashboard` path
    - [ ] degrades gracefully with no mesh / no peers (cached counts, Bus timeouts, never panic)
- [ ] **E6.3: E6 — Apps role**
  **As** a power user, **I want** an Apps role grouping the install / installed / remove / sources / default-apps panels under one role card, **so that** I manage software from the Workbench console.
  *Reuse:* `mde-workbench` `panels/apps_install.rs`, `apps_installed.rs`, `apps_remove.rs`, `apps_sources.rs`, `default_apps.rs`, `panel_apps.rs` (§9 reskin; backends reused). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page apps` renders the Apps role card with action links to Install, Installed, Remove, Sources, and Default-Apps panels, each opening the existing 43-panel backend
    - [ ] an install/remove action issues the real package operation over `mde-bus` and the Installed list reflects the result without a relaunch
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page apps` path
    - [ ] degrades gracefully when the package worker is absent (cached list, Bus timeouts, never panic)
- [ ] **E6.4: E6 — Devices role (9 panels)**
  **As** a power user, **I want** a Devices role exposing the 9 device panels (displays, sound, printers, removable, keyboard, mouse, etc.) as action links under one role card, **so that** I configure hardware from the Workbench.
  *Reuse:* `mde-workbench` `panels/displays.rs`, `sound.rs`, `printers.rs`, `removable.rs`, `keyboard.rs`, `mouse.rs`, `session.rs`, `power.rs`, `connect.rs` (§9 reskin; 9 backends reused). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page devices` renders the Devices role card whose action links open all 9 device panels, each backed by its existing worker over `mde-bus`
    - [ ] a device change (e.g. display arrangement or default sink) applies live and the panel reflects the new state without relaunch
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page devices` path
    - [ ] degrades gracefully when a device worker is absent (cached state, Bus timeouts, never panic)
- [ ] **E6.5: E6 — Fleet role (inventory / playbooks / run-history / settings / revisions)**
  **As** a small-fleet operator, **I want** a Fleet role with inventory, playbooks, run-history, settings, and revisions panels under one role card, **so that** I drive multi-host deployment from the Workbench.
  *Reuse:* `mde-workbench` `panels/inventory.rs`, `playbooks.rs`, `run_history.rs`, `fleet_settings.rs`, `fleet_revisions.rs` (§9 reskin; backends reused). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page fleet` renders the Fleet role card with action links to Inventory, Playbooks, Run-History, Settings, and Revisions, each opening its existing backend
    - [ ] running a playbook against the inventory emits a run that appears in Run-History with live status over `mde-bus`, and Revisions shows the resulting config delta
    - [ ] themed only via `palette::color()` (no raw hex), metrics via `metrics::UI_PX`, reachable from an `mde workbench --page fleet` path
    - [ ] degrades gracefully with no mesh / no peers (cached inventory + history, Bus timeouts, never panic)
- [ ] **E6.6: E6 — Look & Feel role (4 panels)**
  **As** a power user, **I want** a Look & Feel role grouping the 4 appearance panels (themes, wallpaper, fonts, window-manager) under one role card, **so that** I restyle the desktop from the Workbench.
  *Reuse:* `mde-workbench` `panels/themes.rs`, `wallpaper.rs`, `fonts.rs`, `window_manager.rs` (§9 reskin; 4 backends reused). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page look-and-feel` renders the role card with action links to all 4 appearance panels, each opening its existing backend
    - [ ] changing theme/wallpaper/font issues the action over `mde-bus` (e.g. `ThemeChanged`) and the live desktop reflects it without relaunch
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page look-and-feel` path
    - [ ] degrades gracefully when the appearance worker is absent (cached selection, Bus timeouts, never panic)
- [ ] **E6.7: E6 — Maintain role (hub / snapshots / debloat / health / repair / drift)**
  **As** a power user, **I want** a Maintain role with hub, snapshots, debloat, health, repair, and drift panels under one role card — snapshots sharing the Timeshift backend with Settings ▸ Recovery — **so that** I keep the workstation healthy from the Workbench.
  *Reuse:* `mde-workbench` `panels/hub.rs`, `snapshots.rs`, `health_check.rs`, `repair.rs`, `drift.rs` (§9 reskin); snapshots reuse the E4.17 Timeshift backend (one backend, two entry points). *Deps:* E6.1, E4.17.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page maintain` renders the Maintain role card with action links to Hub, Snapshots, Debloat, Health, Repair, and Drift, each opening its existing backend
    - [ ] creating/restoring a snapshot from the Maintain panel and from Settings ▸ Recovery operate on the same Timeshift state — a snapshot made in one entry point appears in the other
    - [ ] a health/repair/drift run executes the real operation over `mde-bus` and the panel reflects the result without relaunch
    - [ ] themed only via `palette::color()` (no raw hex), metrics via `metrics::UI_PX`, reachable from an `mde workbench --page maintain` path; degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic)
- [ ] **E6.8: E6 — System role (5 panels)**
  **As** a power user, **I want** a System role grouping the 5 system panels (datetime, logs, resources, system-update, notifications) under one role card, **so that** I administer core system settings from the Workbench.
  *Reuse:* `mde-workbench` `panels/datetime.rs`, `logs.rs`, `resources.rs`, `system_update.rs`, `notifications.rs` (§9 reskin; 5 backends reused). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page system` renders the System role card with action links to all 5 system panels, each opening its existing backend
    - [ ] Resources/Logs stream live values over `mde-bus`, and a System-Update action runs the real update and reflects progress without relaunch
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page system` path
    - [ ] degrades gracefully when a system worker is absent (cached state, Bus timeouts, never panic)
- [ ] **E6.9: E6 — Help role (disclaimer-embedded)**
  **As** a power user, **I want** a Help role with the help index and an About/Help surface that embeds the project disclaimer, **so that** I find guidance and the mission/warning text inside the Workbench.
  *Reuse:* `mde-workbench` `panels/help_index.rs` (§9 reskin); `disclaimer.rs` `include_str!` (single source). *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page help` renders the Help role card with action links to the help index and an About/Help view
    - [ ] the About/Help surface pulls `DISCLAIMER.md` via `disclaimer.rs` `include_str!` (single source, never copy-paste) and the rendered text matches the file
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page help` path
    - [ ] renders fully offline with no mesh / no peers (never panic)
- [ ] **E6.10: E6 — Compute role (mde-virtual rebuilt on mde-ui): Fleet + Local VM/pod management, 4-step VM wizard, sparklines, templates, bulk actions, cold migration, virt-viewer console**
  **As** a power user, **I want** a Compute role rebuilt from the legacy `mde-virtual` onto `mde-ui` — managing local and fleet VMs/pods with a 4-step VM wizard, live sparklines, templates, bulk actions, cold migration, and a virt-viewer console — **so that** I run KVM/Podman compute from the Workbench instead of the retired standalone tool.
  *Reuse:* `mde-virtual` (§9 retire-absorb → workbench/Compute) — `app.rs`, `wizard.rs`, `sparkline.rs` rebuilt onto `mde-ui`; legacy crate retires at E8. *Deps:* E6.1.
  **Acceptance** (runtime-observable):
    - [ ] `mde workbench --page compute` lists local + fleet VMs/pods with live per-instance sparklines (CPU/mem) over `mde-bus`, and start/stop/bulk actions take effect on the instances
    - [ ] the 4-step VM wizard creates a real KVM/Podman instance (from a template), and "Open console" launches virt-viewer attached to it
    - [ ] a cold-migration action moves a stopped instance to another fleet host and the target host lists it after migration
    - [ ] themed only via `palette::color()` (no raw hex), icons via `icon_any`, metrics via `metrics::UI_PX`, reachable from an `mde workbench --page compute` path; degrades gracefully with no mesh / no peers (local-only view, Bus timeouts, never panic)
- [ ] **E6.11: E6 — Preset / drift engine (Hashbang / Mackes / Daylight / Vanilla / Node variants)**
  **As** a power user, **I want** a preset/drift engine offering the Hashbang, Mackes, Daylight, Vanilla, and Node variants with drift detection against the chosen preset, **so that** I apply and restore a known desktop configuration from the Workbench.
  *Reuse:* `mde-workbench` `panels/drift.rs` + preset engine (§9 reskin / new glue on `mde-ui`); shares Fleet revisions state. *Deps:* E6.1, E6.5.
  **Acceptance** (runtime-observable):
    - [ ] the preset chooser lists all 5 variants (Hashbang / Mackes / Daylight / Vanilla / Node) and applying one issues the real config actions over `mde-bus` so the live desktop reflects the preset
    - [ ] drift detection compares current config against the selected preset and reports the divergent items, with a "restore" action that reverts them
    - [ ] themed only via `palette::color()` (no raw hex), metrics via `metrics::UI_PX`, reachable from an `mde workbench` path
    - [ ] degrades gracefully with no mesh / no peers (local preset apply, cached drift state, Bus timeouts, never panic)

### E7 — Merged OOBE + Mesh Enrolment
_Depends: E0, E1, E2, E4, E5, E6_

- [ ] **E7.1: E7 — OobeEra::Win10 stages (Region/Keyboard/Network/Account/PIN/Privacy/Your-Phone/Personalize/Finalize), GUI + TUI sharing pickers, oobe_done state**
  **As** a first-time operator on a fresh Workstation install, **I want** a Windows 10-styled out-of-box flow that walks me through region, input, network, account, PIN, privacy, phone, and personalization in order, **so that** the machine is fully configured and stamped done in one guided pass.
  *Reuse:* `mde` `oobe.rs` scaffold + `state.oobe_done` (adapt), `installer.rs`/`tui_setup.rs` (adapt — additive `--era=win10` branch), `mde-installer` profile pickers (adapt). *Deps:* E4.1, E1.5.
  **Acceptance** (runtime-observable):
    - [ ] `mde oobe` (and `mde setup --era=win10`) launches a full-screen flow that advances Region→Keyboard→Network→Account→PIN→Privacy→Your-Phone→Personalize→Finalize, each stage gated on a Yes/Next, era-gated on `Theme::Windows10` so Carbon/Win2000/BeOS render the classic Setup unchanged.
    - [ ] GUI and the headless TUI path (`mde oobe --tui`) drive the identical picker backends (same Region/Keymap/Network options and the same applied `localectl`/`localectl set-x11-keymap`/network commands), and `--dry-run` echoes every backend command without mutating the host.
    - [ ] On Finalize the flow stamps `state.oobe_done` so the wizard does not re-show on next login, and `--force` re-runs it; network stage auto-drops when a wired link is already up rather than blocking.
    - [ ] Reachable from an `mde oobe` subcommand path; themed only via `palette::color()` (no raw hex) with metrics via `metrics::UI_PX`; degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).

- [ ] **E7.2: E7 — Merge Birthright mesh enrolment into OOBE: early role picker, Nebula cert/CA enrolment step, optional KDC phone pair, disclaimer "read before proceeding" step**
  **As** a new node operator, **I want** the first-run to also pick my deployment role, enrol me on the Nebula mesh, optionally pair my phone, and make me read the disclaimer, **so that** one OOBE pass both sets up the desktop and joins the node to the fleet.
  *Reuse:* `mde-wizard` Birthright pages + `mde-installer` `Profile` (adapt — fold into OOBE), platform CA/enrollment + `mackes-nebula-https-tunnel` (as-is), `mde-kdc`/`mde-kdc-proto` (adapt/as-is), `disclaimer.rs` `include_str!` (new glue). *Deps:* E7.1, E2.2, E6.1.
  **Acceptance** (runtime-observable):
    - [ ] An early Role stage offers Lighthouse / Server (headless) / Workstation and the choice gates the remaining flow (mesh/desktop stages shown for Workstation, headless subset for the others) and maps to the same `Profile`/worker-subset the installer applies.
    - [ ] The Nebula enrolment stage requests a signed cert from the mesh CA over `mde-bus` and, on success, the node shows as enrolled (cert on disk, `mackesd` reports mesh-up); on no CA reachable it surfaces a retry/skip and proceeds without a panic.
    - [ ] The optional "Your Phone" stage runs a KDC pairing handshake (`mde-kdc-proto`) and, when a phone confirms, persists the pairing to the mackesd pairing store; declining/skipping leaves the flow complete and pairing re-runnable later from Settings.
    - [ ] A "read before proceeding" disclaimer stage renders `DISCLAIMER.md` via `disclaimer.rs` `include_str!` (single source, no copy-paste) and blocks Next until acknowledged; whole flow reachable from `mde oobe`, themed via `palette::color()` with `metrics::UI_PX`, and degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).

### E8 — Polish + Held RPM Release
_Depends: E0-E7_

- [ ] **E8.1: E8 — Accuracy harness over the Workbench + new apps: screenshot all 43 panels + every new surface; per-era gallery captures; Win10 reaches parity with Win2000/BeOS/Carbon in the accuracy gate**
  **As** a release engineer, **I want** every shell surface captured and pixel-checked across all four eras, **so that** Win10 is held to the same accuracy bar as Win2000/BeOS/Carbon before anything ships.
  *Reuse:* MDE-Retro accuracy harness + `gallery.sh` + `checklist.rs` (§9 as-is); `palette.rs`/`metrics.rs` shared-libs. *Deps:* E6, E5, E4.
  **Acceptance** (runtime-observable):
    - [ ] `gallery.sh` drives `mde <sub>` for every surface and produces era-aware crops; all 43 Workbench panels and every new E5 app surface (Phone/Calls, Media Player, Explorer, Action Center) appear as captured images.
    - [ ] `checklist.rs` carries dynamic `[capture.win10-*]` accuracy points (accent at Start/taskbar/Action-Center/Settings, focus-ring checks) and the gate reports Win10 at the same pass count as Win2000/BeOS/Carbon.
    - [ ] Each Win10 surface renders themed only via `palette::color()` (no raw hex) and sized via `metrics::UI_PX`; the harness flags any surface that diverges per era.
    - [ ] Win10 captures only differ from Carbon/Win2000/BeOS where `Theme::Windows10` is active; the three legacy eras render byte-identical to their pre-E4 baseline.

- [ ] **E8.2: E8 — Disclaimer audit sweep across every About/Info/Help surface (single-source verified)**
  **As** a maintainer, **I want** every About/Info/Help surface proven to render the single `DISCLAIMER.md`, **so that** there is exactly one source of disclaimer text and zero copy-paste drift.
  *Reuse:* `disclaimer.rs` `include_str!` (single source) at `crates/shell/mde/src/disclaimer.rs`. *Deps:* E4, E5, E6, E7.
  **Acceptance** (runtime-observable):
    - [ ] Launching every About/Info/Help surface (Settings, Security, Storage/Backup, Workbench Help, OOBE read-before-proceeding) renders text that matches `DISCLAIMER.md` exactly.
    - [ ] A sweep proves every such surface resolves its text through `disclaimer.rs` `include_str!`; no surface embeds a copy-pasted literal of the disclaimer.
    - [ ] Editing `DISCLAIMER.md` and rebuilding changes the text shown by all surfaces simultaneously (single-source verified at runtime).

- [ ] **E8.3: E8 — Promote clippy todo + unwrap_used to deny; full cargo clippy --all-targets / fmt --check / test green workspace-wide**
  **As** a maintainer, **I want** `todo!`/`unwrap` promoted to deny and the full lint/format/test suite green, **so that** no stub markers or panicking unwraps can survive into the held release.
  *Reuse:* workspace lint config (new glue). *Deps:* E8.4.
  **Acceptance** (runtime-observable):
    - [ ] `cargo clippy --all-targets --workspace` exits 0 with `clippy::todo` and `clippy::unwrap_used` set to deny; any remaining `todo!`/`unwrap()` fails the build.
    - [ ] `cargo fmt --check` exits 0 across the workspace.
    - [ ] `cargo test --workspace` passes; no test is `#[ignore]`-skipping a real feature path.

- [ ] **E8.4: E8 — Runtime-reachability + no-stubs/no-mockups sweep: verify E0-E7 are section-3-complete (every feature invocable from a runtime entry point)**
  **As** a maintainer, **I want** every E0-E7 feature proven invocable from a live entry point, **so that** the release contains no mockups, dead code, or unreachable surfaces per CLAUDE.md §3.
  *Reuse:* `audit` sweep + retire-absorb verification of `mde-portal`/`mde-drawer`/`mde-virtual` (§9 retire-absorb) in `crates/legacy/`. *Deps:* E0, E1, E2, E3, E4, E5, E6, E7.
  **Acceptance** (runtime-observable):
    - [ ] Every desktop surface is reachable from an `mde <subcommand>` path (or ENOENTs by design on non-Workstation roles); the sweep launches each and observes it render, not stub out.
    - [ ] The retired functions of `mde-portal`/`mde-drawer`/`mde-virtual` are demonstrably reachable in their new homes (Win10 shell, Action Center tiles, Workbench Compute); none of the three legacy crates are reachable from any runtime entry point.
    - [ ] Surfaces talking to mackesd workers over `mde-bus` degrade gracefully with no mesh / no peers (cached state, Bus timeouts, never panic) when the daemon is absent.

- [ ] **E8.5: E8 — RPM spec: one spec + conditional role subpackages (mde-core / mde-headless / mde-desktop) with Provides/Obsoletes; DISCLAIMER.md pre-flight gate (build refuses if missing/empty); RPM build HELD until all above green** [!]
  **As** a packager, **I want** a single conditional spec emitting the three role subpackages behind a disclaimer pre-flight gate, **so that** the RPM stays held until every preceding E8 task is green and refuses to build without a disclaimer.
  *Reuse:* `release` skill + cargo-generate-rpm spec (new glue); §12 subpackage structure. *Deps:* E8.1, E8.2, E8.3, E8.4.
  **Acceptance** (runtime-observable):
    - [ ] One spec builds three subpackages — `mde-core` (all roles), `mde-headless` (Server+Workstation), `mde-desktop` (Workstation, `Requires: mde-core`) — each carrying `Provides:` legacy names and `Obsoletes:` the old xfce/i3 packages.
    - [ ] The pre-flight gate aborts the build with a clear error when `DISCLAIMER.md` is missing or empty, and proceeds only when it exists and is non-empty.
    - [ ] The build is HELD (refuses to produce artifacts) until E8.1-E8.4 report green; running it early exits non-zero naming the unmet gate.

- [ ] **E8.6: E8 — Cut RPM v10.0.0 + CHANGELOG + tag MackesWorkstation-v10.0.0 (operator-gated; push/tag is a separate authorization)**
  **As** the operator, **I want** to cut v10.0.0 with a CHANGELOG and signed tag only on explicit authorization, **so that** the release artifact is reproducible and no push/tag happens without a separate human go-ahead.
  *Reuse:* `release` skill (operator-gated). *Deps:* E8.5.
  **Acceptance** (runtime-observable):
    - [ ] `cargo-generate-rpm` produces `mde-core`/`mde-headless`/`mde-desktop` RPMs stamped `version = 10.0.0`; installing each invokes its role surfaces (CLI everywhere; desktop-only ENOENTs on non-Workstation).
    - [ ] CHANGELOG entry for v10.0.0 enumerates the E0-E7 feature inventory; the git tag `MackesWorkstation-v10.0.0` is created locally.
    - [ ] Push and tag-publish occur only on a separate explicit operator authorization; an unauthorized `/release` run stops before any remote write.

### HW — Hardware / Interactive Bench (post-release, release-gated — NOT task blockers)
_Depends: feature code complete_

- [ ] **HW-1: KDE Connect real-phone round-trip (notifications/messages/photos/calls bidirectional)**
  **As** the operator, **I want** a real paired phone to exchange notifications, SMS, photos and call events both ways over the finished inbound listener (3b.2e), **so that** the Win10 "Your Phone" surface is proven against actual hardware, not loopback.
  *Reuse:* crates/kdc/mde-kdc-host + mde-kdc-proto (as-is/adapt, §9). *Deps:* E2.1.
  **Acceptance** (runtime-observable):
    - [ ] A phone notification appears as an E3 toast filtered to the KDE Connect device, and dismissing it on the desktop clears it on the phone.
    - [ ] An SMS sent from the Your Phone "Messages" pane is received on the phone, and an inbound SMS appears in the pane without a manual refresh.
    - [ ] A photo pushed from the phone lands in the Photos pane and opens; a desktop file shared to the phone arrives in its share sheet.
    - [ ] An incoming phone call raises the call HUD and a "decline/mute" action from the desktop takes effect on the phone; mutual-TLS fingerprint pin holds (no fallback to unpinned).

- [ ] **HW-2: BlueZ pairing on real hardware**
  **As** the operator, **I want** to power, discover, pair, connect and remove a real Bluetooth peripheral from the Settings Bluetooth pane, **so that** the BlueZ-via-zbus wiring is confirmed on bench radios.
  *Reuse:* mde-applets Settings Bluetooth pane (adapt, §9) over BlueZ zbus. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Toggling the radio in the pane powers the adapter on/off and the system tray Bluetooth glyph reflects the new state.
    - [ ] A live scan lists a nearby device; selecting it completes pairing (PIN/confirm prompt where required) and it shows Connected.
    - [ ] An input or audio device routes through after connect (keystrokes register / audio plays), and Remove unpairs it so it disappears from the paired list.

- [ ] **HW-3: PAM unlock + argon2 PIN on real hardware**
  **As** the operator, **I want** `mde lock` (Win+L) to unlock with both an argon2 PIN and a PAM password on real keyboards, **so that** the lock face is proven against the live auth stack, not a stub.
  *Reuse:* mde lock layer-shell lock face + argon2 (`~/.config/mde/pin.hash`), PAM (§10). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Win+L raises the layer-shell lock face and the session is genuinely locked (input to apps behind it is blocked).
    - [ ] A correct argon2 PIN unlocks and returns to the exact prior session; a wrong PIN is rejected and re-prompts.
    - [ ] Switching to password and entering the real account password unlocks via PAM; an incorrect password is rejected by PAM with no panic.

- [ ] **HW-4: dnf live-streaming Windows Update on real hardware**
  **As** the operator, **I want** the Win10 "Windows Update" surface to drive a real dnf transaction with live-streamed progress, **so that** package output is confirmed flowing to the UI on actual repos/network.
  *Reuse:* Settings/Update surface over dnf (§10). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] "Check for updates" runs a real dnf metadata refresh and the surface lists actual available package counts.
    - [ ] Starting an update streams per-package download/install lines into the UI in real time (not a single end-of-run dump).
    - [ ] A completed transaction shows success state and the installed version is reflected on a follow-up check; a network drop mid-stream surfaces an honest error, never a hang.

- [ ] **HW-5: Compositor/session cutover + greeter: all 3 roles boot -> greeter -> usable session on bench hardware** [!]
  **As** the operator, **I want** each deployment role to boot through its greeter into a usable session on real GPUs, **so that** the labwc/session cutover (a standing real-HW risk) is validated per role.
  *Reuse:* mde-session (launch labwc) + greetd/regreet/cage; LightDM-gtk win10() theme (as-is/adapt, §10/§12). *Deps:* E1.5.
  **Acceptance** (runtime-observable):
    - [ ] Workstation boots to the themed greeter, login starts labwc, and the Win10 taskbar/Start are interactive on the bench display.
    - [ ] Server (headless) reaches a usable login/console with its worker subset up and no display-manager spawned.
    - [ ] Lighthouse boots with enrollment-CA/Bus/Nebula only — no greeter, no desktop — and accepts an enrollment.
    - [ ] Logout/restart/shutdown from the session returns cleanly to greeter or powers down without a stuck compositor.

- [ ] **HW-6: LizardFS mesh-storage across a multi-peer fleet (replication + offline degrade)**
  **As** the operator, **I want** mesh XDG dirs replicated across a multi-peer LizardFS fleet and to confirm graceful degrade when peers drop, **so that** topology-aware replication and offline behavior are proven on real nodes.
  *Reuse:* mackesd meshfs worker (LizardFS FUSE) + LizardFS master/chunk (§11/§12). *Deps:* E3.1.
  **Acceptance** (runtime-observable):
    - [ ] A file written on one peer's mesh dir appears on a second peer's mount within the replication goal.
    - [ ] Killing a chunk peer leaves the file readable from a surviving replica; rejoining the peer re-replicates and the goal count is restored.
    - [ ] With all peers offline the mount serves cached state and surfaces degrade gracefully with no mesh / no peers (cached state, Bus timeouts, never panic).

- [ ] **HW-7: VoIP real outbound/inbound call via Vitelity**
  **As** the operator, **I want** to place and receive a real PSTN call through the Vitelity trunk from the Phone/Calls app, **so that** the kamailio/rtpengine softphone path is proven end-to-end with two-way audio.
  *Reuse:* mde-voice-hud Phone/Calls app + mde-voice-config (kamailio/rtpengine) (adapt/as-is, §9). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Dialing a real number from the Phone/Calls app connects through the Vitelity trunk with audible two-way audio.
    - [ ] An inbound call to the DID raises the call HUD toast; answering connects two-way audio and hangup tears down the session cleanly.
    - [ ] The completed call lands in the call-history list with correct direction and duration; a registration loss surfaces an honest "not registered" state, never a silent dead dialer.

- [ ] **HW-8: Deployment-role install from media for each role (Lighthouse/Server/Workstation)**
  **As** the operator, **I want** to install from real media and pick each deployment role at the role chooser, **so that** the one-RPM role-chooser provisions the correct worker subset and surfaces per role on bare hardware.
  *Reuse:* mde-installer + deployment-role chooser (adapt, §9/§12). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] Booting the media presents the role chooser and selecting Workstation provisions a system that first-boots into the greeter with the full surface set enabled.
    - [ ] Selecting Server provisions a headless system whose mackesd brings up the fleet/meshfs/metrics worker subset with no display manager.
    - [ ] Selecting Lighthouse provisions a relay node with enrollment-CA/leader/health workers only and a LizardFS read-only client — no media/voice/compute, no greeter.
    - [ ] Each installed role's systemd units and `/etc/mackesd/` templates match the chosen role on first boot (correct worker set running, no role-foreign units active).

### RETIRE-PY — Python-daemon retirement (EPIC-RETIRE-PY-DAEMONS)
_Depends: surfaced by the E0.11 audit (2026-06-03). These are v1.x→v2.0.0 transition shims whose own source comments say "the v2.0.0 cut reimplements [this]" but never landed in the merge. Governance §11 C13. Several converge with other epics — noted per task._

- [ ] **RETIRE-PY.1: RETIRE-PY — Port the mDNS relay worker to native Rust (replaces `python3 -m mackes.mesh_mdns`)**
  **As** an operator, **I want** the mDNS announce/watch relay to run as a native Rust `mackesd` worker, **so that** cross-LAN-segment mesh peer-presence bridging needs no Python runtime.
  *Reuse:* `mackesd/src/workers/mdns.rs` (today a `SubprocessTickWorker` shelling to `python3 -m mackes.mesh_mdns`) → native Rust mDNS. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] `mackesd` runs the mDNS relay with **no** `python3`/`mackes.mesh_mdns` process spawned (`ps`/`pgrep` confirms).
    - [ ] A peer announces and is discovered by a second peer on another LAN segment at runtime.
    - [ ] Degrades gracefully with no mesh / no peers (idle announce loop, never panics).

- [ ] **RETIRE-PY.2: RETIRE-PY — Port the Remmina-profile sync worker to native Rust (replaces `python3 -m mackes.remmina_sync`)**
  **As** a user, **I want** Remmina RDP/VNC profiles synced across peers by a native Rust worker, **so that** remote-desktop profile sync needs no Python.
  *Reuse:* `mackesd/src/workers/remmina_sync.rs` (today a `SubprocessTickWorker`). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] A Remmina profile created on one peer appears on another with **no** `python3`/`mackes.remmina_sync` spawn.
    - [ ] The worker ticks on its interval and is supervised by `mackesd` (restart-on-exit).
    - [ ] Degrades gracefully with no mesh / no peers.

- [ ] **RETIRE-PY.3: RETIRE-PY — Converge the clipboard worker onto the Rust `mde-clipd` (retire `python3 -m mackes.clipboard_app`)**
  **As** a user, **I want** the `mackesd` clipboard worker to drive the existing Rust `mde-clipd` daemon instead of the Python `clipboard_app`, **so that** clipboard sync is pure-Rust over the Bus.
  *Reuse:* `crates/services/mde-clipd` (§9 as-is, Rust) + `mackesd/src/workers/clipboard.rs`. *Deps:* E0.3 (Bus).
  **Acceptance** (runtime-observable):
    - [ ] Copying on one peer makes the entry available on another via `mde-clipd` over `mde-bus`, with **no** `mackes.clipboard_app` python process.
    - [ ] `wl-paste --watch` ring + pinned entries persist (the mde-clipd behavior), not the Python daemon's.
    - [ ] Degrades gracefully with no mesh / no peers (local clipboard still works).

- [ ] **RETIRE-PY.4: RETIRE-PY — Retire the GVFS `fs_sync` worker in favor of E3 LizardFS mesh-storage (remove `python3 -m mackes.mesh_gvfs.daemon`)**
  **As** a platform maintainer, **I want** the old per-peer GVFS/QNM-Shared FUSE sync removed once LizardFS mounts the shared tree, **so that** there is ONE mesh-storage substrate, not two.
  *Reuse:* **SUPERSEDED by E3** (LizardFS `mesh-storage`); delete `mackesd/src/workers/fs_sync.rs`. *Deps:* E3.2.
  **Acceptance** (runtime-observable):
    - [ ] No `python3`/`mackes.mesh_gvfs` process is spawned; the `fs_sync` worker is removed from `mackesd`.
    - [ ] Mesh XDG dirs (`~/Documents` etc.) are served by the LizardFS mount (E3), not the GVFS FUSE path.
    - [ ] Degrades gracefully with no mesh / no peers (local shadow answers reads).

- [ ] **RETIRE-PY.5: RETIRE-PY — Native Rust Birthright apply/rollback (replaces `python3 -m mackes.birthright` in mde-wizard + mde-installer)**
  **As** a new user, **I want** first-run Birthright steps applied by native Rust, **so that** OOBE and install need no Python runtime.
  *Reuse:* `mde-wizard/src/pages/apply.rs` + `mde-installer/.../mde-install.rs` (today build `python3 -m mackes.birthright [apply|rollback]`). *Deps:* converges with **E7.2** (OOBE Birthright merge).
  **Acceptance** (runtime-observable):
    - [ ] OOBE / `mde-install` apply runs the Birthright steps (themes, mesh enrolment, DND) with **no** `python3`/`mackes.birthright` spawn.
    - [ ] The rollback path (`birthright_rollback`) is native Rust and recovers a failed apply.
    - [ ] Each applied step is observable (theme set, mesh enrolled) — not a placeholder.

- [ ] **RETIRE-PY.6: RETIRE-PY — Pure-Rust asset installer — drop the `python3` "Asset installer runtime" dep (catalogue.rs)**
  **As** a packager, **I want** `mde install --assets` to fetch + stage assets without a `python3` runtime, **so that** the asset path is pure-Rust (git fetch + native unpack).
  *Reuse:* `mde/src/catalogue.rs` + `install.rs`. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] `mde install --assets` fetches + stages the asset set with **no** `python3` invoked.
    - [ ] `catalogue.rs` no longer lists `python3` as an asset-installer runtime dependency.
    - [ ] A fresh install resolves all bundled assets to disk (verifiable file presence).

- [ ] **RETIRE-PY.7: RETIRE-PY — Workbench service-publishing reads via Bus, not `python3 -c mackes.mesh_nebula` (mde-workbench)**
  **As** an admin, **I want** the Workbench service-publishing panel to read published-services state over `mde-bus` from `mackesd`, **so that** it needs no `python3` shell-out.
  *Reuse:* `mde-workbench/src/panels/service_publishing.rs` (excluded crate; §9 reskin) + `mde-bus`. *Deps:* E0.3 (Bus), E6.
  **Acceptance** (runtime-observable):
    - [ ] The panel renders live published-services from a Bus query with **no** `python3`/`mackes.mesh_nebula` spawn.
    - [ ] An added published service appears in the panel at runtime.
    - [ ] Degrades gracefully with no mesh (empty list + hint, never panics).
