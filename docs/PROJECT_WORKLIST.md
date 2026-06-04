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

- [>] **E0.3.1: E0.3 — Migrate the Nebula.Status D-Bus service -> Bus (SelfNode / ListPeers)** *(READS done + retired; WRITES + allowlist-drop = E0.3.1.b)*
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

- [>] **E0.3.1.b: E0.3 — Migrate the Nebula.Status WRITES (RegenCerts / Enroll) + signals -> Bus; drop the allowlist entry**
  **As** a platform dev, **I want** the remaining `dev.mackes.MDE.Nebula.Status` D-Bus surface (`RegenCerts`, `Enroll`, + the `peer_state_changed`/`transport_changed`/`enrollment_completed` signals) moved onto the Bus, **so that** the whole Nebula.Status interface retires and `lint-dbus-shape` drops nebula.rs. *Reuse:* mde-session's action/reply precedent for the writes; an event/notify Bus topic for the signals (the Overview subscription + applets are the consumers). *Deps:* E0.3.1.a. *Sub-steps:*
    - [✓] **RegenCerts → Bus.** Added `action/nebula/regen-certs` to the responder (`ACTION_VERBS` + `build_reply` → `{ "ok", "message" }` over the extracted `regen_certs_inner`); `mesh_control::run_rotate_ca` now calls `crate::dbus::nebula_request_with_timeout("regen-certs", 30s)` (spawn_blocking; writes shell `nebula-cert`, hence the longer budget) + `parse_regen_reply`. The `#[interface] regen_certs` method is removed. mackesd 101 + mde-workbench 747 tests green; clippy clean.
    - [✓] **Enroll → removed (DEAD).** The D-Bus `enroll()` method had NO consumer — panels (mesh_join, mesh_pending) shell the `mackesd enroll` CLI, which drives the CSR-watcher path. Removed the `#[interface] enroll()` wrapper; `enroll_inner` (the CLI's entry) + the CSR-watcher's `EnrollmentCompleted` emission are untouched. mackesd 101 nebula tests green (enroll tests exercise `enroll_inner`).
    - [ ] **Signals → Bus events.** *(Design worked out 2026-06-03 — next session executes.)* The dispatcher (`spawn_signal_dispatcher`, nebula.rs:533) is the single mpsc→D-Bus indirection and the clean cut-point. EMIT side: rewrite it to drop the `conn`/`iface_ref` args and instead run on a **dedicated `std::thread`** ("nebula-signal-dispatcher") with a current-thread runtime that holds one `Persist` (rusqlite isn't `Send` — same pattern as `serve_bus`; a `tokio::spawn` can't hold `Persist` across `rx.recv().await`). For each `NebulaSignal`, `persist.write` to a single `event/nebula/signals` topic with body `{"kind":"peer-state-changed|transport-changed|enrollment-completed", ...payload}`. Signature becomes `spawn_signal_dispatcher(slot: &SignalSenderSlot) -> NebulaSignalSender` (sync, no `zbus::Result`). SUBSCRIBE side: rewrite `home.rs`'s iced Subscription (the D-Bus `MatchRule` listener + `PeerStateChanged`/`TransportChanged`/`EnrollmentCompleted` match loop, ~1340–1450) to a cursor-based `persist.list_since("event/nebula/signals", cursor)` poll on an interval → map `kind` → the existing `DbusEvent` enum → `reprobe_for_event`. Workers + `NebulaSignal` + the mpsc stay unchanged. *Caller:* mackesd.rs (~3113) drops `conn` from the dispatcher call. *New pattern:* this is the Bus's first fire-and-forget EVENT topic (vs request/reply) — multiple subscribers each keep their own `list_since` cursor.
    - [ ] **Retire interface + allowlist.** With reads + RegenCerts + Enroll gone and the signals on the Bus, the `#[interface(name = "dev.mackes.MDE.Nebula.Status")]` block is empty — remove it entirely **plus** `register_nebula_status_on` (nothing left to register on the object server; verify the shared `conn` is still used by the other ipc/ services and keep it). Drop nebula.rs from the `lint-dbus-shape` `ipc/` allowlist (verify the gate then reports nebula.rs clean). Flip **E0.3.1** `[✓]`.
  **Acceptance** (runtime-observable): `RegenCerts` answered over Bus action/reply (done); `Enroll` D-Bus method gone; the three signals delivered over a Bus event topic with the same subscribers re-probing; the `#[interface]` block gone from nebula.rs; `lint-dbus-shape` allowlist drops the nebula.rs entry. *(Round-trip = Bus bench.)*

- [ ] **E0.3.2: E0.3 — Migrate the 5 file-op D-Bus services -> Bus; rewire mde-files**
  **As** a user, **I want** Inbox/Outbox/Downloads/FileOperations/Fleet.Files served over Bus, **so that** Explorer mesh ops need no private D-Bus. *Reuse:* mackesd/src/ipc/files.rs + mde-files/src/dbus_backend.rs (-> a bus_backend). *Deps:* E0.3.1. *(Largest sub-task.)*
  **Acceptance** (runtime-observable): file ops route over Bus action/reply; mde-files mesh-browse works via the Bus backend (no D-Bus); allowlist drops files.rs + dbus_backend.rs. *(Round-trip = Bus bench.)*
  **Rescue finding (2026-06-03):** `mackesd::orchestrator` (the Send-To state machine — Pending→Validating→Executing→Verifying, ProgressEvent stream, the "engine behind dev.mackes.MDE.Shell.Send") is a **dead module** (zero external refs; allowlisted by lint-runtime-reachability). It is intended infra, not speculative — **wire it as the engine behind the file-op flow** as part of this migration (the D-Bus file-op services bypass it today). FINISH, not REMOVE.

- [ ] **E0.3.3: E0.3 — Migrate the Fleet D-Bus service -> Bus**
  **As** an admin, **I want** `dev.mackes.MDE.Fleet` over Bus. *Reuse:* mackesd/src/ipc/fleet.rs (the Workbench fleet panels already call the `mackesd` CLI per E0.13, so D-Bus consumers may be few). *Deps:* E0.3.1.
  **Acceptance** (runtime-observable): fleet queries answered over Bus; allowlist drops fleet.rs. *(Round-trip = Bus bench.)*

- [ ] **E0.3.4: E0.3 — Migrate the Settings D-Bus service -> Bus**
  **As** a surface, **I want** `dev.mackes.MDE.Settings` over Bus. *Reuse:* mackesd/src/ipc/settings.rs. *Deps:* E0.3.1.
  **Acceptance** (runtime-observable): settings get/set answered over Bus; allowlist drops settings.rs. *(Round-trip = Bus bench.)*

- [ ] **E0.3.5: E0.3 — Migrate the Shell (+ root dev.mackes.MDE) D-Bus service -> Bus**
  **As** a surface, **I want** the Shell control surface over Bus. *Reuse:* mackesd/src/ipc/shell.rs. *Deps:* E0.3.1.
  **Acceptance** (runtime-observable): shell verbs answered over Bus; allowlist drops shell.rs. *(Round-trip = Bus bench.)*

- [ ] **E0.3.6: E0.3 — Migrate the Connect (org.mde.Connect) D-Bus -> Bus**
  **As** a user, **I want** the KDE Connect roster surface over Bus. *Reuse:* mde/src/connect.rs. *Deps:* E0.3.1; converges with **E2** (KDC). *Consumers:* mde-peer-card, notifications applet.
  **Acceptance** (runtime-observable): the device roster reaches consumers over Bus (no org.mde.Connect D-Bus); allowlist drops connect.rs. *(Round-trip = Bus bench.)*

- [ ] **E0.3.7: E0.3 — Final D-Bus retirement sweep**
  **As** a maintainer, **I want** the lint-dbus-shape allowlist empty, **so that** only FDO interop remains. *Reuse:* install-helpers/lint-dbus-shape.allowlist. *Deps:* E0.3.1–E0.3.6.
  **Acceptance** (runtime-observable): `lint-dbus-shape` passes with an EMPTY allowlist; a tree grep finds only `org.freedesktop.*` / `org.mpris.*` / `org.kde.StatusNotifier*` `#[interface]` blocks.

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

- [>] **E0.9: E0 — GitHub Actions CI: cargo check/test/clippy/fmt on push/PR, with gtk3-devel + alsa-lib-devel so the full workspace builds**
  **As** a maintainer, **I want** CI running check/test/clippy/fmt on push and PR with the system dev libs installed, **so that** every change is gated on a full-workspace green build.
  *Reuse:* new glue (adapt provenance `.github/workflows/ci.yml`). *Deps:* E0.2.
  **Acceptance** (runtime-observable):
    - [ ] A push and a PR each trigger the workflow; it installs `gtk3-devel` + `alsa-lib-devel` and runs `cargo check --workspace`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`.
    - [ ] The job goes red on a clippy warning, a failing test, or an fmt violation (a deliberately broken commit fails CI).
    - [ ] A clean `main` commit produces an all-green run with the four ALSA/Workbench crates compiled (full workspace, no excluded members).
  **Note (2026-06-03):** `.github/workflows/ci.yml` written + YAML-valid; cargo cmds match the green local baseline. Fedora container installs gtk3-devel/alsa-lib-devel (forward-ready for E0.2). CI-green confirmation is **push-gated** → flips to [✓] after the first push.

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

- [ ] **E1.1: E1 — Deployment-role chooser (Lighthouse/Server/Workstation) -> /var/lib/mde/role.toml (upgrade-allow, downgrade-block)**
  **As** an operator installing the one RPM, **I want** to pick a deployment role once and have it pinned immutably, **so that** the box always boots as the rank I chose and can only be upgraded to a richer role, never silently downgraded.
  *Reuse:* `mde-installer` (§9 adapt — + deployment-role chooser). *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] `mde setup --profile=lighthouse|server|workstation` writes `/var/lib/mde/role.toml` and a second `mde setup --profile=...` with the same or higher rank exits 0 (upgrade), printing the new rank.
    - [ ] Re-running `mde setup --profile=` with a lower rank than the pinned role exits non-zero with a "downgrade blocked" message and leaves `role.toml` byte-for-byte unchanged.
    - [ ] Any code path reads role solely via the loader; `mde setup --show` prints the live role rank (0/1/2) parsed back from the on-disk `role.toml`.
    - [ ] A malformed or absent `role.toml` causes role-dependent commands to fail closed (lowest privilege / ENOENT), never defaulting to Workstation.

- [ ] **E1.2: E1 — Role-gated mackesd worker subsets + role-gated surface install (desktop surfaces ENOENT on non-Workstation)**
  **As** an operator on a headless box, **I want** mackesd to spawn only the workers my role permits and desktop surfaces to be genuinely absent, **so that** a Lighthouse/Server never runs media/voice workers or exposes GUI entry points it cannot satisfy.
  *Reuse:* `mackesd` (§9 as-is — control plane) + `mde` dispatcher (new role-gate glue). *Deps:* E1.1.
  **Acceptance** (runtime-observable):
    - [ ] On Lighthouse, `mackesd` supervises only enrollment(CA)+leader+health workers; on Server it additionally runs fleet+meshfs+metrics; on Workstation it adds the voice coordinator + media stack — verifiable via `mackesd`'s worker-status listing over `mde-bus`.
    - [ ] On a non-Workstation role, `mde settings`, `mde start-win10`, `mde action-center`, `mde security`, `mde oobe`, and `mde installer` exit with ENOENT/not-available, while `mde panel/menu/files/net-flyout/filedialog` run on every role.
    - [ ] On Workstation all of the above desktop subcommands launch their surface.
    - [ ] Degrades gracefully with no mesh / no peers (cached state, Bus timeouts, never panic) — worker-gating still resolves and CLI surfaces still answer when the mesh is unreachable.

- [ ] **E1.3: E1 — Role-aware systemd units + /etc/mackesd/ templates**
  **As** an operator, **I want** systemd to start the unit set matching my role, **so that** `systemctl` shows exactly the services that role requires and nothing it forbids.
  *Reuse:* `mde-session` (§9 as-is — session orchestrator) + `mackesd` unit (§9 as-is) + new packaging glue. *Deps:* E1.1, E1.2.
  **Acceptance** (runtime-observable):
    - [ ] All roles report `mackesd.service` and `mde-bus.service` active under `systemctl is-active`; `mde-session.service` and `greetd.service` are active only on Workstation and absent/inactive otherwise.
    - [ ] `mde-headless` units (lizardfs, ansible-pull.timer) are enabled on Server+Workstation and not present on Lighthouse.
    - [ ] mackesd reads its runtime config from `/etc/mackesd/` templates; editing a template value and restarting the service produces the changed behavior at runtime (e.g. an altered worker/relay setting observable over `mde-bus`).
    - [ ] A role mismatch (unit enabled for a role the box is not) is rejected at unit start, logging the role conflict to the journal rather than starting a forbidden service.

- [ ] **E1.4: E1 — Wire the role selector into mde-installer**
  **As** an installer, **I want** the role chooser surfaced during install/OOBE, **so that** the role is picked and pinned before first boot without a manual `mde setup` invocation.
  *Reuse:* `mde-installer` (§9 adapt) + `mde-wizard` (§9 adapt — Win10 OOBE) glue. *Deps:* E1.1, E1.3.
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

- [ ] **E4.1: E4 — Win10 era foundation: Theme::Windows10 + win10(rgb) remap (accent #0078d4/#2899f5) through palette::color(), font::family(), state.rs, main.rs startup; bottom panel anchor; Display > Appearance "Windows 10" picker; pin in checklist.rs** [M1]
  **As** a daily-driver user, **I want** to switch the whole shell into a Windows 10 era look, **so that** every surface adopts the Win10 accent and chrome from one toggle.
  *Reuse:* `mde` (as-is, §9) + `mde-ui` Win10 palette/widgets (as-is); adapt `display.rs` Appearance, `state.rs`, `main.rs`, `checklist.rs`. *Deps:* none.
  **Acceptance** (runtime-observable):
    - [ ] `mde` launched with `Theme::Windows10` (THEME atomic = 3) renders accent `#0078d4`/`#2899f5` everywhere via `palette::color()` — no raw hex anywhere in the era path; metrics via `metrics::UI_PX`.
    - [ ] The shell panel anchors to the bottom edge (not top) only when the era is Windows10; Carbon/Win2000/BeOS keep their existing anchors.
    - [ ] Display ▸ Appearance shows a selectable "Windows 10" picker that rewrites labwc themerc and flips `state.rs` `"windows10"`, and the choice survives a restart (`main.rs` reads it at startup).
    - [ ] `windows10_remap_pins` appears in `checklist.rs` so the accuracy gate verifies the remap is live.

- [ ] **E4.2: E4 — Bus client foundation: prove theme/accent action<->state delivery over mde-bus end-to-end (the simplest loop) before surfaces depend on it**
  **As** a shell developer, **I want** the simplest theme/accent action↔state loop proven over `mde-bus` first, **so that** later surfaces (Action Center, Network, Phone) can trust the transport before depending on it.
  *Reuse:* `mde-bus` (as-is, §9 platform IPC backbone) + `mackesd` worker (as-is); new glue in `mde` Bus client. *Deps:* E4.1.
  **Acceptance** (runtime-observable):
    - [ ] An accent/theme change emitted from a surface travels over `mde-bus` (not private D-Bus) and the resulting state change is observed back in the surface within the round-trip.
    - [ ] A second running `mde` surface reflects the same accent within one Bus tick, proving fan-out delivery to multiple subscribers.
    - [ ] Degrades gracefully with no mesh / no peers / absent worker: surface falls back to cached state on Bus timeout and never panics.

- [ ] **E4.3: E4 — Settings registry foundation: metadata registry (category/title/icon/deep-link) drives the home grid + nav rail; pages register metadata, never edit a central match tree**
  **As** a settings author, **I want** a metadata registry that drives the home grid and nav rail, **so that** I can add pages by registering metadata instead of editing a central match tree.
  *Reuse:* adapt `control_panel.rs` shape + `settings.rs`; new glue registry module. *Deps:* E4.1.
  **Acceptance** (runtime-observable):
    - [ ] The Settings home grid and left nav rail render their tiles/rows from registered metadata (category/title/icon), with no hand-maintained central match arm per page.
    - [ ] `mde settings --page X` deep-links straight to a registered page (each page renders as its own grim-capturable process), and an unregistered key produces a clear miss, not a panic.
    - [ ] Adding a new page record makes it appear in both the home grid and rail at next launch without editing existing page code.

- [ ] **E4.4: E4 — Win10 taskbar (panel view_win10): Start tile, Search box, Task View, app buttons (accent underline on focus), tray, two-line clock, Action Center button + unread badge** [M1]
  **As** a desktop user, **I want** a bottom Win10 taskbar with Start, Search, Task View, app buttons, tray and a clock, **so that** I can launch, switch and monitor from one bar.
  *Reuse:* adapt `panel.rs` `view_win10()`, `tray.rs` (§9 mde-panel adapt). *Deps:* E4.1, E4.3.
  **Acceptance** (runtime-observable):
    - [ ] `panel.rs` `view_win10()` (era-gated on `Theme::Windows10`) shows the Start tile, Search box, Task View button, running-app buttons, tray and a two-line clock, all themed via `palette::color()` and metrics via `metrics::UI_PX`.
    - [ ] The focused/active app button draws the Win10 accent underline; clicking an app button raises that window via `wlr.rs`.
    - [ ] The Action Center button reads `notifications.json` and shows an unread badge whose count matches the stored unread notifications; Win+A opens Action Center.
    - [ ] Reachable from `mde search` / `taskview` / `action-center` subcommands; degrades gracefully with no worker / no peers (cached counts, never panic).

- [ ] **E4.5: E4 — Tiled Start menu (mde start-win10): icon-rail (account/folders/Settings/Power), Recently-Added/Suggested/All-Apps, tile grid with pin/unpin/resize/uninstall, headless CLI (--pin/--unpin/--resize/--list-tiles)** [M1]
  **As** a desktop user, **I want** a tiled Start menu with an icon rail, app lists and a pinnable tile grid, **so that** I can find, pin and launch apps the Win10 way.
  *Reuse:* adapt `start_win10.rs`, `menu.rs` launch/context (§9 mde-popover adapt). *Deps:* E4.1, E4.4.
  **Acceptance** (runtime-observable):
    - [ ] `mde start-win10` opens a full-screen layer-shell overlay with a left icon-rail (account/folders/Settings/Power), center Recently-Added/Suggested/All-Apps A–Z, and a right tile grid, themed via `palette::color()`.
    - [ ] Right-clicking a tile offers Pin/Unpin/Resize/Uninstall and the change persists to `StartTile` state across relaunch.
    - [ ] Headless `mde start-win10 --pin/--unpin/--resize/--list-tiles` mutate and print the same tile state the GUI shows.
    - [ ] Launching a tile starts the app via `menu.rs`; era-gated so Carbon/Win2000/BeOS Start surfaces are untouched.

- [ ] **E4.6: E4 — Action Center + notification daemon (notifyd claims org.freedesktop.Notifications, persists across restarts, mirrors to notifications.json) + toasts (mde toast) + quick-action tile grid (Wi-Fi/BT/Airplane/Brightness/Volume/Night-light/Focus) backed by NM/BlueZ/wlsunset**
  **As** a desktop user, **I want** a notification daemon, toasts and a quick-action grid, **so that** apps' notifications collect in one pane and I can flip Wi-Fi/BT/brightness fast.
  *Reuse:* adapt `notifyd.rs`, `action_center.rs`; absorb `mde-drawer` quick-actions (§9 retire-absorb); `nm.rs`/`bluez.rs` backends. *Deps:* E4.2, E4.4.
  **Acceptance** (runtime-observable):
    - [ ] `notifyd.rs` claims `org.freedesktop.Notifications` (FDO interop, hosted in the panel process), and a notification raised before a restart is still listed after the daemon restarts (mirrored to `notifications.json`).
    - [ ] `mde toast <id>` shows a bottom-right transient toast and `mde action-center` (Win+A) lists the stored notifications and feeds the E4.4 unread badge.
    - [ ] Quick-action tiles (Wi-Fi/BT/Airplane/Brightness/Volume/Night-light/Focus) reflect and change live system state via NM/BlueZ/wlsunset.
    - [ ] Reachable from `mde toast`/`action-center`; degrades gracefully with no mesh / no peers / absent backend (tiles show cached/disabled state, Bus timeouts, never panic).

- [ ] **E4.7: E4 — Multitasking: Task View (Win+Tab icon+title grid from wlr.rs), virtual desktops via ext-workspace-v1 with fallback ladder, labwc edge-snap keybinds, Snap Assist (focus-only)**
  **As** a desktop user, **I want** Task View, virtual desktops and edge-snap, **so that** I can organize and switch windows and workspaces.
  *Reuse:* adapt `task_view.rs`, `workspace.rs`, `wlr.rs`. *Deps:* E4.1, E4.4.
  **Acceptance** (runtime-observable):
    - [ ] Win+Tab (or `mde taskview`) opens a full-screen icon+title grid enumerated from `wlr.rs`; selecting a tile focuses that window (no pixel thumbnails).
    - [ ] Virtual desktops switch via `ext-workspace-v1`, and on a compositor without it the honest fallback ladder still presents a usable desktop count rather than failing.
    - [ ] labwc rc.xml edge-snap keybinds tile the focused window (mde never owns geometry), and `mde task-view --snap-assist <side>` offers focus-only Snap Assist that chain-snaps via labwc.

- [ ] **E4.8: E4 — Search + Quick Access: Win+S overlay (All/Apps/Documents/Web/Settings — apps + fd docs + DuckDuckGo) + Win+X Quick Access menu**
  **As** a desktop user, **I want** a Win+S search overlay and a Win+X power menu, **so that** I can find apps, docs, web results and system tools instantly.
  *Reuse:* adapt `search.rs`, `popup.rs` `items_for("quickaccess")`, `apps.rs`. *Deps:* E4.1, E4.3, E4.4.
  **Acceptance** (runtime-observable):
    - [ ] Win+S (or `mde search`) opens an overlay with All/Apps/Documents/Web/Settings tabs; Apps resolve via `apps::programs()`, Documents via `fd`, Web via DuckDuckGo, Settings via the E4.3 registry — each tab returns live matches for a query.
    - [ ] Selecting a result launches the app, opens the document, opens the browser to the web result, or deep-links the Settings page respectively.
    - [ ] Win+X opens the Quick Access menu (System/Device-Mgr/Disk/Power/Event-Viewer/Network/Task-Mgr/Terminal/Run) and each row launches its tool; both surfaces era-gated on `Theme::Windows10`.

- [ ] **E4.9: E4 — Modern Settings app (mde settings, Win+I): category grid + left rail + M1 pages (Display, About, Printers, Colors, Background). Replaces Control Panel in Win10 era only** [M1]
  **As** a desktop user, **I want** a modern Settings app with a category grid and the core pages, **so that** I can configure the system without the legacy Control Panel.
  *Reuse:* adapt `settings.rs`, `control_panel.rs` shape, `fedora::TOOLS`; `disclaimer.rs` for About. *Deps:* E4.1, E4.3.
  **Acceptance** (runtime-observable):
    - [ ] `mde settings` (Win+I) shows the category grid + left rail (System, Devices, Phone, Network, Personalization, Apps, Accounts, Time & Language, Ease of Access, Privacy, Update & Security), themed via `palette::color()`.
    - [ ] The M1 pages Display, About, Printers, Colors and Background each render live and apply real changes; About pulls `DISCLAIMER.md` via `disclaimer.rs` `include_str!` (single source).
    - [ ] In the Win10 era Settings replaces Control Panel, while Win2000/Carbon still reach `mde control-panel`; pages are reachable via `mde settings --page X`.

- [ ] **E4.10: E4 — Settings > Personalization: Colors (Light/Dark/Custom + accent grid), Background (Picture/Solid/Slideshow), Themes, Lock screen, Start, Taskbar pages**
  **As** a desktop user, **I want** Personalization pages, **so that** I can set my accent, light/dark mode, wallpaper, lock screen and taskbar.
  *Reuse:* adapt `settings/personalization.rs`, `display.rs` wallpaper helpers, `wallpaper.rs`. *Deps:* E4.1, E4.3, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] Colors page Light/Dark/Custom + accent grid call `set_dark`/`set_accent`/`win10_accent` and the chosen accent appears across the shell live and after restart.
    - [ ] Background page Picture/Solid/Slideshow changes the live wallpaper via `display.rs` helpers; Slideshow rotates the configured folder.
    - [ ] Themes, Lock screen, Start and Taskbar pages each persist their `#[serde(default)]` state fields and apply observable changes (e.g. Taskbar settings re-render the E4.4 bar).

- [ ] **E4.11: E4 — Accounts / Lock / Sign-in: Your-info (~/.face), argon2 PIN, Family & other users (useradd/usermod via pkexec), Win+L lock face (PIN/password via PAM), greeter theme from win10() tokens**
  **As** a desktop user, **I want** account info, a PIN, user management and a lock screen, **so that** I can sign in and secure my session the Win10 way.
  *Reuse:* adapt `lock.rs`, `pin.rs`, `greeter.rs`; new dep `argon2`. *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] Settings ▸ Accounts ▸ Your-info reads/sets `~/.face` and the avatar appears in Start and the lock face.
    - [ ] Setting a PIN writes an argon2 hash to `~/.config/mde/pin.hash`; `mde lock` (Win+L) unlocks on the correct PIN (argon2 verify) or password (PAM) and rejects a wrong one.
    - [ ] Family & other users adds/modifies a user via useradd/usermod behind pkexec, and the new user appears in the greeter whose theme is generated from `win10()` tokens.

- [ ] **E4.12: E4 — Settings > Devices: Bluetooth (BlueZ zbus), Printers (lpadmin/lpstat), Mouse/Touchpad/Typing (labwc libinput), AutoPlay (udisks2), Project/second-display (Win+P)**
  **As** a desktop user, **I want** Devices pages, **so that** I can pair Bluetooth, add printers, tune the mouse/touchpad and project to a second display.
  *Reuse:* adapt `settings/devices.rs`, `bluez.rs`, `cups.rs`, `mouse.rs`, `autoplay.rs`, `project.rs`. *Deps:* E4.1, E4.3, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] Bluetooth page powers/discovers/pairs/removes real adapters via BlueZ zbus (FDO interop) and the device list reflects live state.
    - [ ] Printers page enumerates via lpinfo/lpstat and adds a queue via lpadmin; Mouse/Touchpad/Typing changes write labwc libinput config and take effect.
    - [ ] AutoPlay reacts to a udisks2 media-insert event; `mde project` (Win+P) offers second-display modes that change the actual output layout.
    - [ ] Deep-linkable via `mde settings --page devices[:bluetooth|...]`; degrades gracefully with no peers / absent service (cached/empty state, never panic).

- [ ] **E4.13: E4 — Windows Update (settings/update.rs, dnf-backed): check, install (pkexec dnf upgrade), feature-update probe, pause (<=35d), active hours, history (dnf history), uninstall (history undo), advanced toggles**
  **As** a desktop user, **I want** a Windows Update page, **so that** I can check, install, pause and review updates from one screen.
  *Reuse:* adapt `settings/update.rs`, promote `system_properties.rs` auto-update stub to `sysinfo::set_auto`. *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] Check runs `dnf check-update` and lists available updates; Install runs `pkexec dnf upgrade` and the list/history update on completion.
    - [ ] Pause sets a ≤35-day window that visibly suppresses checks until it lapses; Active hours and advanced toggles persist via `sysinfo::set_auto(AutoMode)` in state.
    - [ ] History reads `dnf history`, and Uninstall performs `dnf history undo` for a selected transaction with the result reflected back in history.

- [ ] **E4.14: E4 — Security dashboard (mde security): Virus & threat (ClamAV optional), Firewall (firewalld), Device encryption (LUKS — turn-on typed-destructive-confirm only), Find-my-device (KDE Connect), Secure Boot/TPM read-only probes** [!]
  **As** a desktop user, **I want** a security dashboard, **so that** I can see virus, firewall, encryption and device-find status at a glance.
  *Reuse:* adapt `security.rs`, `security_probe.rs`; `disclaimer.rs` for the About/info. *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] `mde security` shows tiles for Virus & threat (ClamAV when present), Firewall (firewalld zones), Device encryption (LUKS), Find-my-device (KDE Connect) and read-only Secure Boot/TPM probes, each rendering a `STATUS_OK/WARN/RISK` role pinned in checklist.
    - [ ] Firewall tile reflects and toggles the live firewalld zone; LUKS turn-on requires a typed-destructive confirm and never auto-runs.
    - [ ] Degrades gracefully with no mesh / no peers (Find-my-device shows cached/offline, Secure Boot/TPM probes report "unknown" rather than crashing; never panic).

- [ ] **E4.15: E4 — Networking: panel net-flyout (Wi-Fi list/connect, Airplane) + Settings pages (Status/Wi-Fi/Ethernet/VPN/Mobile-hotspot/Proxy/Data-usage/Airplane) + Action-Center toggles; MIGRATE the 13 Workbench Network panels into Settings > Network here**
  **As** a desktop user, **I want** all networking in Settings plus a taskbar flyout, **so that** Wi-Fi, VPN, mesh and firewall live in one place and the Workbench Network group is gone.
  *Reuse:* adapt `net_flyout.rs`, `settings/network.rs`, `nm.rs` backend, `mde-peer-card` (§9 adapt); absorb the 13 `mde-workbench` Network panels. *Deps:* E4.1, E4.3, E4.6, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] The taskbar net-glyph flyout lists real Wi-Fi networks and connects/disconnects via `nm`, and Airplane mode toggles live radio state.
    - [ ] Settings ▸ Network shows the 9 native pages (Status/Wi-Fi/Ethernet/VPN/Mobile-hotspot/Proxy/Data-usage/Airplane) plus the 13 migrated Workbench panels (mesh control/topology/federation, VPN, firewall, remote desktop, service publishing, SSH, services, Bus, Wi-Fi), each registered via the E4.3 registry and reachable as `mde settings --page network:*`.
    - [ ] Action-Center network toggles call `nm::set_*` and reflect back into the flyout; the Workbench no longer exposes any Network group.
    - [ ] Degrades gracefully with no mesh / no peers (cached topology/peer state, Bus timeouts, never panic).

- [ ] **E4.16: E4 — Clipboard history + Screenshots: Win+V ring (25 unpinned + pinned, wl-paste --watch) + Win+Shift+S snip over grim+slurp (rect/window/full/clip), PrintScreen family mapped, toast**
  **As** a desktop user, **I want** clipboard history and a snipping tool, **so that** I can re-paste past copies and capture the screen quickly.
  *Reuse:* adapt `clipboard.rs`, `snip.rs`; `mde-clipd` (as-is, §9). *Deps:* E4.1, E4.6.
  **Acceptance** (runtime-observable):
    - [ ] `wl-paste --watch` fills a ring at `~/.local/share/mde/clipboard/` (25 unpinned + unlimited pinned); Win+V (or `mde clipboard`) shows it and re-pastes a chosen entry; pinned entries survive eviction.
    - [ ] Win+Shift+S (or `mde snip`) captures rect/window/full/clip via grim+slurp and the PrintScreen family keys map to the matching modes.
    - [ ] A capture emits a confirmation toast via E4.6 and the image lands in the clipboard/save target.

- [ ] **E4.17: E4 — Storage / Backup / Recovery: Storage Sense (timer + dnf/journald clean) + usage breakdown, Timeshift backup/schedule/restore + System Restore browser, Reset-this-PC (typed-destructive two-mode) + Advanced startup + recovery drive**
  **As** a desktop user, **I want** Storage, Backup and Recovery pages, **so that** I can free space, schedule backups and restore or reset the PC.
  *Reuse:* adapt `settings/{storage,backup,recovery}.rs`, `restore.rs`; `disclaimer.rs`. *Deps:* E4.1, E4.9.
  **Acceptance** (runtime-observable):
    - [ ] Storage Sense enables a systemd timer and a run-now frees space via dnf/journald clean; the usage breakdown reflects real disk figures and drills into Apps.
    - [ ] Backup adds a Timeshift drive, schedules/retains, runs back-up-now, and the System Restore browser lists snapshots with a green `RESTORE_PRIMARY` "Restore to original location" action.
    - [ ] Reset-this-PC offers the two modes behind a typed-destructive confirm (never auto-runs); Advanced startup and Create recovery drive each invoke their real action.

- [ ] **E4.18: E4 — Edge -> Firefox browser surface: default_browser() via xdg-settings, recent_sites() read-only over places.sqlite, jump list (New/Private/Recent); label always "Firefox" (never fake Edge brand)**
  **As** a desktop user, **I want** the browser surface to be honest Firefox with a jump list, **so that** I see recent sites and quick actions without any fake Edge branding.
  *Reuse:* adapt `browser.rs`, `browser_jumplist.rs`; new dep `rusqlite` (read-only). *Deps:* E4.1, E4.5.
  **Acceptance** (runtime-observable):
    - [ ] `default_browser()` resolves via xdg-settings and the Default-apps "Web browser" row sets it; the surface label always reads "Firefox" (never an Edge brand).
    - [ ] `recent_sites()` reads `places.sqlite` strictly read-only and the jump list shows New/Private/Recent; selecting an entry launches Firefox to it.
    - [ ] Degrades gracefully when `places.sqlite` is locked/absent (empty recent list, never panic).

- [ ] **E4.19: E4 — Power / Session: Win10 flat-flyout (Sleep/Shutdown/Restart, Lock/Sign-out) + mde lock (Win+L, loginctl lock-session)**
  **As** a desktop user, **I want** a Win10 power flyout and a lock command, **so that** I can sleep, shut down, restart, lock or sign out cleanly.
  *Reuse:* adapt `dialogs.rs`, `lock.rs`; `mde-logout-dialog` (as-is reskin, §9). *Deps:* E4.1, E4.5, E4.11.
  **Acceptance** (runtime-observable):
    - [ ] The Start/Power flat-flyout shows Sleep/Shutdown/Restart and Lock/Sign-out rows (Win10 era only; Win2000/Carbon keep the dropdown) and each row performs the real action.
    - [ ] `mde lock` (Win+L) issues `loginctl lock-session` and shows the lock face from E4.11.
    - [ ] `Choice::Lock` is reachable from the session surface and themed via `palette::color()`.

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
