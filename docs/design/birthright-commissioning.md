# Birthright Commissioning Dashboard (`mde birthright`)

> **Epic:** folds into **E7 — Merged OOBE + Mesh Enrolment** (new tasks E7.3–E7.6).
> **Status:** scoped 2026-06-08 (operator 20-Q survey). No code yet.
> **Authority:** this doc is a per-epic lock (below `CLAUDE.md` / `AI_GOVERNANCE.md`).
> The §0 master rule still governs: *Secure, Simple, No-Fixed-Center Workgroup.*

## 1. Intent

After install, the operator needs a single screen that **attests the node is
actually commissioned** — not just that the installer exited 0. `mde birthright`
is that screen: a Carbon status dashboard that confirms, with live re-runnable
checks, that

1. **the full desktop is built** — labwc is the compositor, `mde panel` is up,
   the expected applets are registered (the exact regression that shipped a
   black desktop on second login — see `mde-session` `seed_missing_files`),
2. **the mesh is online** — `mackesd` (+ its workers), the Nebula overlay, the
   LizardFS mount, and `mde-bus` are each up,
3. **the VoIP/SIP softphone is listening** — the E5.4 SIP agent is registered
   AND its inbound listener socket is bound (a call *would* ring), and
4. **network reporting is available** — readouts from the platform's network
   tools (Nebula roster + RTT, fleet inventory, a LAN nmap probe, and
   NetworkManager/net-flyout connectivity).

"Birthright" is this repo's existing term for the OOBE/enrolment lineage (the
`mde-wizard` Birthright pages, folded into OOBE by E7.2). The commissioning
dashboard is the natural close of that lineage: enrolment *joins* the fleet
(E7.2); birthright *proves the join took* and the box is whole.

## 2. Locked decisions (20-Q survey, 2026-06-08)

| # | Decision | Lock |
|---|---|---|
| 1 | Relationship to oobe/setup | **Final step of OOBE** — birthright is the page OOBE Finalize hands off to; also re-launchable standalone. |
| 2 | Name / command | **`mde birthright`** — "Birthright Commissioning". |
| 3 | First-boot trigger scope | **Both** a machine-first-boot marker (`/var/lib/mde`) and a per-user-first-login marker (`~/.config/mde`). |
| 4 | Re-surface mechanism | **labwc autostart entry**, gated on the per-user `~/.config/mde` flag (same pattern as the existing first-run `mde setup`). |
| 5 | "Show at startup" persistence | Checkbox **default ON**; unchecking writes the flag in `~/.config/mde` (per-user). |
| 6 | When every check is green | **Window stays open** and foregrounds a prominent "Don't show this at startup again" control. No auto-close. |
| 7 | Refresh model | **Live updates + a manual "Re-check all" button.** |
| 8 | Liveness transport | **Event-driven over `mde-bus`** — `mackesd` health/status changes push; rows update on event (no fixed poll). Manual Re-check forces a re-probe. |
| 9 | Desktop check content | **panel + applets + compositor** all up (labwc is compositor, `mde panel` alive, expected applets registered, autostart completed). |
| 10 | Mesh section granularity | **Per-component rows**: `mackesd` (+ each worker), Nebula overlay, LizardFS mount, `mde-bus`. Each independently Pass/Degraded/Fail. |
| 11 | SIP check depth | **Registered + inbound listener bound.** No automatic test-call (a live loopback call is an explicit opt-in button, not an auto-check). |
| 12 | Network section | **All four** readouts: Nebula roster + `voip_rtt` RTT · `fleet` inventory · LAN `probe_nmap` scan · NetworkManager / net-flyout connectivity. |
| 13 | Failure remediation | **Actionable fix buttons** per failed row: Start service (mackesd worker), Mount (LizardFS), Register (SIP), **Re-enroll** (triggers the E7.2 Nebula enrolment flow), Open Settings. |
| 14 | Status states | **Pass / Degraded / Fail + a transient "checking…".** Degraded covers partial health (mesh up but a peer unreachable; network up but no overlay). |
| 15 | UI form | **Standalone Carbon status dashboard** — sectioned cards (Desktop / Mesh / Voice / Network), full-window layer-shell or xdg-toplevel iced app. Launched as OOBE's final step and re-launchable on its own. |
| 16 | Deployment roles | **Workstation only.** Server/Lighthouse (headless) do not run it. |
| 17 | Export | **Copy diagnostics** (clipboard) + **Save report** (timestamped JSON/txt), reusing `mackesd` `health.rs` `to_json_line`. |
| 18 | Post-commissioning escalation | After the box is unchecked, a later regression of a previously-green subsystem raises a **panel health badge + a toast** that deep-links back to `mde birthright`. |
| 19 | Non-goals | Not a continuous monitor · not a Settings replacement · not a network analyzer (see §5). |
| 20 | Epic slot | **Fold into E7** (Merged OOBE + Mesh Enrolment), as tasks E7.3–E7.6. |

**Deliberate non-lock:** "not mesh enrollment" was *not* selected as a non-goal.
Birthright does not own enrolment, but its **Re-enroll fix button triggers
E7.2's enrolment flow** when the Nebula row is red — remediation, not ownership.

## 3. Architecture

- **Surface:** new `crates/shell/mde/src/birthright.rs`, dispatched by a
  `"birthright" => birthright::run(rest)` arm in `main.rs` (a sibling of
  `oobe::run`). Full-window iced app, Carbon-only (Gray 100 default), all color
  via `palette::color()`, all sizing via `metrics` — §2.1/§2.3.
- **OOBE hand-off:** the OOBE `Finalize` stage (`oobe.rs`) launches `mde
  birthright` as its closing page instead of just stamping `oobe_done`.
- **Re-surface:** a guarded line in the skel `autostart`
  (`crates/shell/mde/skel/.config/labwc/autostart`) launches `mde birthright`
  when the per-user flag is set and the role is Workstation — mirroring the
  existing `mde setup` first-run block.
- **Checks read existing producers (reuse is the spine, §2.7):**
  - Desktop — query labwc (`wlr-foreign-toplevel` / compositor name), the
    `mde panel` process + applet registration.
  - Mesh — `mackesd` `health.rs` `HealthReport` + `preflight.rs`, surfaced over
    `mde-bus` (per-worker, Nebula, LizardFS mount, bus self-ping).
  - Voice — the E5.4 persistent SIP agent registration state + inbound listener
    socket bound.
  - Network — `nebula_roster.rs`, `voip_rtt.rs`, `fleet.rs`, `probe_nmap.rs`,
    and NetworkManager / `net_flyout.rs`.
- **Liveness:** subscribe to `mde-bus` health/status topics; an iced
  subscription folds Bus events into row state. "Re-check all" publishes a
  re-probe request and/or re-runs the on-demand probes (nmap, RTT).
- **Persistence:** per-user flag + dual first-boot markers in `~/.config/mde`
  (and the machine marker under `/var/lib/mde`), every field
  `#[serde(default)]`, atomic `save()` — §2.5.
- **Escalation:** the panel (`panel.rs`) gains a health-badge fed by the same
  Bus health topic; a regression after commissioning raises a toast
  (`mde toast`) deep-linking to `mde birthright`.

## 4. Acceptance (the no-stubs floor, §3) — see worklist E7.3–E7.6

Each task is a runtime-reachable vertical slice (a section + its lifecycle), so
each ships complete in one commit per §3.

- **E7.3** — dashboard shell + lifecycle + **Desktop** section.
- **E7.4** — **Mesh** section (per-component rows + fix buttons).
- **E7.5** — **Voice** + **Network** sections.
- **E7.6** — attestation extras: Copy/Save report + panel-badge/toast escalation.

## 5. Out of scope (non-goals — locked)

- **Not a continuous health daemon.** `mackesd` owns ongoing health; birthright
  *reads* it. Birthright is commissioning-time + on-demand attestation.
- **Not a Settings replacement.** Fix buttons trigger discrete actions or
  deep-link into Settings; birthright does not edit config.
- **Not a network analyzer.** It surfaces existing tools' readouts; it adds no
  new packet-capture/analysis beyond `probe_nmap`/`voip_rtt`/etc.
- **Not on headless roles.** Workstation only (Server/Lighthouse skip it).
- **Does not perform enrolment.** It verifies enrolment health and can *trigger*
  E7.2's flow (Re-enroll), but enrolment itself stays E7.2.

## 6. Risks

- **Liveness fan-in:** four sections × several producers over the Bus — keep
  each check cheap and event-driven; gate the expensive probes (nmap, test-call)
  behind explicit actions, never the auto-poll (§2 simplicity).
- **OOBE coupling:** birthright launching from OOBE Finalize must not block
  `oobe_done` from being stamped — a birthright crash must not strand OOBE.
- **Badge noise:** the post-commissioning escalation (E7.6) must debounce so a
  flapping peer doesn't toast-spam — single toast per new failure, panel badge
  carries steady state.
