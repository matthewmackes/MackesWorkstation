# Mackes Workstation — Fusion Plan (PLANNING ONLY)

> **Status:** discovery + architecture + decision-space complete; **blocked on the
> 50 questions below** before it becomes an executable, sequenced plan. Nothing in
> here is built. Interim home in MDE-Retro `docs/`; the real home depends on **Q1**
> (repo shape). Generated 2026-06-03 from an exhaustive parallel review of both
> repos (119 features inventoried across MDE + MDE-Retro).

## 0. Decisions (answered by the owner)

*Strategic positioning (2026-06-03) — THE FRAMING:*
- **Mackes Workstation is the SUCCESSOR.** It **preempts and EOLs both** MDE
  (MackesDE for Workgroups / MDE4WG) **and** MDE-Retro. They do **not** continue as
  independent projects — both are retired/archived; the Mackes Workstation monorepo
  is the single go-forward codebase. *(Resolves Q2/Q3 definitively: subsumed + EOL,
  not coexisting.)* Implication: no parallel maintenance of the old repos; their
  packages (MDE `mde-core`/`mde-desktop`, the MDE-Retro RPM) are superseded by the
  Mackes Workstation deployment-role RPM; all future epics target the successor.

*Review pass (2026-06-03) — four locks from the plan walkthrough:*
- **Repo name → `MackesWorkstation`** (GitHub repo, matching the MDE / MackesDE
  capitalization). The cargo workspace + crate names stay kebab-case (`mackes-*` / `mde-*`)
  per Rust convention.
- **Settings architecture (resolves Q14) → HYBRID.** A lightweight metadata registry
  (category, title, icon, deep-link) drives the Settings home grid + nav rail; each page
  still renders as its **own `mde settings --page X` process** (fits §7's per-process theme
  model, stays grim-capturable). Registry for discovery; separate processes for rendering —
  no monolithic match.
- **KDE Connect TLS (refines Q48) → MUTUAL TLS both directions.** Inbound (host 3b.2e, done)
  binds the peer's presented client-cert fingerprint to the paired device's pinned value
  before accepting — a deliberate, *more-secure* divergence from the reference's
  `no_client_auth` (flagged for the owner's adversarial audit). **Follow-up:** make the
  outbound `connect_pinned_tls` present our cert too, so real-device interop is symmetric.
- **Crate-prefix policy → keep mixed `mde-*`/`mackes-*` for v1** (the
  `mde-config`/`mde-mesh-types`/`mded` facades already bridge); unify to `mackes-*` in a
  post-v1 rename.

*Load-bearing four — 2026-06-03:*
- **Q8 Compositor → labwc.** Keep MDE-Retro's labwc substrate; MDE's sway-specific
  bits adapt to labwc/wlroots. The Win10 shell is the primary UX.
- **Q16 mesh-storage → PRECONDITION, on LizardFS (NOT Gluster).** *(Revised
  2026-06-03 against current MDE @ `6459e17`.)* Shared **mesh-storage** is a
  precondition (the Workstation is not single-box-only), implemented by **LizardFS**
  — `docs/design/v5.0.0-mesh-storage-lizardfs.md` (locked 2026-05-29) supersedes the
  GlusterFS design **wholesale** (Gluster never shipped; no migration). Depends on
  the Nebula fabric + the Bus. Drop **all** Gluster assumptions; the MESHFS-* tasks
  are the LizardFS path. Mesh-storage `XDG` dirs are LizardFS-mounted.

*Resolved by the refresh against current MDE (see §0.1):*
- **Q4 terminology → MDE = "MackesDE for Workgroups" (a.k.a. MDE4WG)**, the platform,
  releasing as **v10.0.0**. "Mackes Workstation" is the new Win10-shell fusion on top.
- **Q11 D-Bus vs Bus → Bus.** The platform is retiring all MDE-internal D-Bus to Bus
  topics (EPIC-RETIRE-DBUS; only FDO interop like `org.freedesktop.Notifications`
  stays). Mackes Workstation surfaces consume the **Bus**, not internal D-Bus.

*Batch 2 — 2026-06-03:*
- **Surfaces → the Win10 shell REPLACES `mde-portal`.** One daily-driver shell
  (MDE-Retro's Win10). The Portal is retired/absorbed — its functions reappear as
  Win10 idioms (Start/Settings/Explorer/Action Center); its distinctive power-user
  layers (mesh roster, tags, library) move to the **Workbench**.
- **Q9 daemon → `mackesd` as a supervised service.** It owns the workers + mesh/CA
  state; the Win10 shell + Workbench consume state and send actions over the **Bus**
  (no in-process worker pool in the shell).
- **Q26 KDE Connect → `MDE-KDECnt-Rust` is canonical.** The monorepo depends on the
  extracted proto+host crate; MDE's in-tree `mde-kdc` host converges onto it. (The
  in-progress host 3b work is already on the canonical path.)
- **Q40 Workbench → RE-SKIN: Windows-10 design concepts in the Microsoft Server
  2003 "Manage Your Server" mold, wearing the platform's color + icon theme.**
  *(Refined by owner 2026-06-03.)* The Workbench is a **task/role-oriented admin
  console** — the "Manage Your Server" pattern: a left nav (the 9 groups / 43
  panels) + a main pane of **role/section cards**, each with a heading, a short
  descriptive paragraph, and **action links** ("Add…", "Manage…", "Configure…"),
  plus a "Tools / See also" sidebar — rendered with **Win10 design concepts** (flat
  surfaces, accent, the Win10 control idiom). It **inherits the overall platform's
  color theme** (the `palette::color` Carbon/Win10 palette — no separate Material
  palette) **and icon theme** (the platform icon set via `icon_any`). *Implications:*
  unify on the MDE-Retro `mde-ui` flat-Carbon widget kit + palette + icons across
  both surfaces (resolves Q23/Q24 → one design system); the 43 Workbench panels are
  re-laid-out into the role-card/action-link "Manage Your Server" structure, not just
  re-colored. Keep the Material/ChromeOS-Classic content OUT.
- **Q1 Repo → MONOREPO.** One cargo workspace absorbs MDE's platform crates +
  MDE-Retro's shell + the Workbench as members. The existing repos become upstream
  history. (This is the final home for *this* plan once the repo exists.)
- **Q5 v1 scope → EVERYTHING.** v1 = the full fusion: mesh, fleet/playbooks, VoIP,
  compute (KVM/Podman), music, all 43 Workbench panels, + the Win10 shell. The RPM
  is held until all of it is ready; hardware bench follows release.

*Implications:* labwc + Gluster-mandatory + monorepo + full-scope means the plan
optimizes for one large integrated workspace, not a minimal shell. Per-crate reuse
is maximal (shared crates in one workspace). Remaining questions refine the daemon
model, the bus, KDE Connect convergence, Win10-vs-Workbench placement, and the
Workbench's look.

*Batch 3 — placement, 2026-06-03:*
- **Q34 first-run → merged Win10 OOBE + mesh enrolment.** The shipped Win10 OOBE
  gains Birthright's stages (mesh enrolment/pairing, DND) as Win10-styled steps —
  one first-run that also enrols the node on the mesh.
- **Q31 VoIP → a Win10 "Phone/Calls" app + incoming-call toast/HUD** (reuse
  `mde-voice` + `mde-voice-hud`).
- **Q32 compute → a Workbench role** ("Virtualization & Containers" — KVM + Podman),
  fitting the Manage-Your-Server console.
- **Q29 files → Win10 Explorer primary; mesh/peer locations + mesh-storage fold into
  Quick access** (reuse `mde-files`' mesh-browse backend; retire its separate UI).

*Batch 4 — placement + KDC, 2026-06-03:*
- **Q48 KDE Connect → finish the inbound listener (host 3b.2e).** Full bidirectional
  KDE Connect for v1; resume the crate work in `MDE-KDECnt-Rust`.
- **Q30 music → a Win10 "Media Player" app** (the AIR-* maxi-player re-skinned Win10;
  reuse `mde-music`/`mde-musicd` + MPRIS).
- **Q33 drawer → fold into the Win10 Action Center** (one quick-actions surface;
  reuse the drawer's action backends; retire the separate overlay).
- **Q41 Workbench entry → a Start tile + a Control-Panel "Manage Workstation" app**
  (deep-linkable to a role/panel), per the Manage-Your-Server idiom.

*Batch 5 — packaging, legal, more placement, 2026-06-03:*
- **Q49 packaging → ONE RPM + an install-time DEPLOYMENT-ROLE chooser:**
  **1. Lighthouse** (mesh relay/coordination node) · **2. Server (headless)** ·
  **3. Workstation (full desktop)**. The chosen role selects which `mackesd` workers
  + which surfaces are enabled (maps to MDE's lighthouse/host/peer worker model).
  *Implication:* the Win10 shell + Workbench install only under the **Workstation**
  role; Lighthouse/Server are headless-ish. First-run + role pick happen early in
  install.
- **Q50 licensing → GPL-3.0, Win10-*inspired* (not a clone), original assets.**
  Avoid MS trademarks + pixel-exact copies; the `DISCLAIMER.md` covers risk.
- **Q36 maintain → Workbench, + snapshots also as Win10 "System Restore"** (Settings
  ▸ Recovery; one snapshot backend, two entry points; reuse `restore.rs`/E17).
- **Q37 network → ALL network lives in Win10 Settings; NO Workbench Network group.**
  The Workbench's 13 Network panels (Wi-Fi, mesh control/topology/federation, VPN,
  firewall, remote desktop, service publishing, SSH, services, Bus) **migrate into
  Win10 Settings ▸ Network & Internet** (atop the 9 native E15 pages already built).
  *Implication:* Win10 Settings must scale to many more pages → favors a
  **registered-module Settings registry** (resolves Q14 → modular pages); the
  Workbench drops Network entirely.

*Batch 6 — governing rule + process/versioning, 2026-06-03:*
- **Q15 PLACEMENT RULE → "Mirror Windows 10."** If Windows 10 has an equivalent
  surface, the feature lives there (Win10 shell / Settings / an app); only **novel
  platform features** (fleet/playbooks, compute, mesh ops, maintain/drift, presets,
  deployment-role management) go to the **Workbench**. This rule settles the
  remaining per-feature placements.
- **Q12 process model → one multiplexed `mde <subcommand>` binary** for all GUI
  surfaces + **`mackesd` daemon** for long-lived workers (MDE-Retro's proven model).
- **Q39 applets → map the 17 `mde-applets` to Win10 tray items + Action-Center
  tiles** (reuse backends; no separate applet host).
- **Q7 versioning → share the platform's v10.0.0** (one version across the monorepo).

## 0.1 Information currency

The original review snapshot was **65 commits stale**; refreshed against current MDE
**`6459e17`** (2026-06-03). Material deltas since the snapshot, folded into this plan:
- **GlusterFS → LizardFS** for mesh-storage (Gluster fully retired).
- Platform release is **v10.0.0 "MackesDE for Workgroups"** (= MDE / MDE4WG).
- **D-Bus retirement** in progress (EPIC-RETIRE-DBUS) → Bus topics; FDO interop only.
- **`mde-portal` has grown into a major surface** (Library / Control / Network layers,
  a tag system, the Birthright wizard rendered inside Portal-full) — relevant to the
  Win10-shell-vs-Portal-vs-Workbench surface question.
- **QNM-Shared → `workgroup_root`** (EPIC-RETIRE-QNM); a large native **music**
  player (`mde-music`/`mde-musicd`, AIR-*) shipped.
- **Before producing the executable plan, the full feature inventory should be
  re-derived against `6459e17`** — the architecture decisions hold, but per-crate
  details (Portal scope, music, mesh-storage) moved.

## 1. Vision

**Mackes Workstation** is a new project that fuses two existing codebases:

- **MDE** (`github.com/matthewmackes/MDE`) — a Rust *platform*: ~37 crates providing
  encrypted mesh (Nebula), KDE Connect, a supervised daemon (`mackesd` + ~40
  workers), GlusterFS mesh-home, a pub/sub bus, voice/VoIP, music, files, and a
  power-user **Workbench** console.
- **MDE-Retro** (this repo) — a Windows 10 / IBM Carbon **shell**: the daily-driver
  desktop UX (taskbar, Start, Settings, Explorer, Action Center, Search, Task View,
  OOBE) on labwc, with a mature reusable design system.

The result: **MDE's platform sits *underneath* to empower the MDE-Retro Win10
shell, with reuse as the spine.** Anything that doesn't fit the Windows 10 idiom
goes to the **Workbench** surface (MDE's `mde-workbench`). **REUSE IS KEY** — new
code is glue, not reimplementation.

## 2. Architecture (proposed — every fork maps to a question below)

```
┌─ Win10 Shell (MDE-Retro, primary UX) ──────────┐   ┌─ Workbench (mde-workbench) ──────┐
│ taskbar · Start · Settings · Explorer · Action │   │ ChromeOS-Classic/PatternFly      │
│ Center · Search · Task View · OOBE · flyouts   │   │ console for non-Win10 features:  │
│ surfaces platform features in the Win10 idiom  │   │ fleet · mesh · maintain · netadmin│
└───────────────┬────────────────────────────────┘   │ presets · compute · metrics       │
                │  launched from Start ("Mackes Workbench") └──────────────┬──────────────────┘
┌───────────────┴──── Platform substrate ("underneath") ──────────────────┴──────────────────┐
│ mackesd + workers · mde-bus · KDE Connect (mde-kdc-proto, shared) · Nebula mesh · Gluster    │
│ mesh-home · config · metrics — consumed as libraries + a supervised daemon                   │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

## 3. Feature inventory (119 features, by target layer)

**Platform — underneath (40):** `mackesd` daemon + ~40 workers · `mde-bus` (pub/sub,
replacing internal D-Bus) · `mde-session` · `mde-config`/`mackes-config` · Nebula
mesh (`mackes-nebula-https-tunnel` + CA/lighthouse in `mackesd`) · **KDE Connect**
(`mde-kdc-proto` + host — already being extracted to `MDE-KDECnt-Rust`) · GlusterFS
mesh-home · Netdata metrics + `mde-alert-emit` · `mde-musicd` · `mde-voice-config` ·
`mde-clipd` · `mde-installer` · plus MDE-Retro's platform bits (one-binary
dispatcher, accuracy harness, `wlr.rs` window control, outputs/bluez/cups/mount/etc.).

**Win10 surface (38):** the whole MDE-Retro Win10 set — panel/taskbar, tiled Start,
Settings, Explorer, Action Center, Search, Task View + virtual desktops, OOBE,
security dashboard, network flyout, clipboard/snip, lock/power, browser, About,
System Properties, Run, context menus, tray.

**Workbench surface (20):** `mde-workbench` — 240px sidebar, **9 groups / 43 panels**:
Dashboard · Apps · Devices (9) · **Fleet** (inventory/playbooks/run-history/settings/
revisions) · Look & Feel (4) · **Maintain** (hub/snapshots/debloat/health/repair/
drift) · **Network** (13: wifi/mesh-control/pending/history/join/ssh/topology/services/
bus/federation/service-publishing/vpn/firewall/remote-desktop) · System (5) · Help ·
plus the **Preset/drift engine** (Hashbang/Mackes/Daylight/Vanilla/Node variants).

**Shared libs (15):** MDE's `mde-theme` · `mde-iced-components` · `mde-card` schema ·
mesh-types; MDE-Retro's `palette.rs` color() edge · `metrics.rs` · flat Carbon
widget set · bevel · icon-resolution chain · tolerant-serde `state.rs` · font system.

**Ambiguous (6) — placement is a question:** `mde-files` (mesh-first manager) ·
`mde-music` · **`mde-voice`** (PJSIP softphone/VoIP) · **`mde-virtual`** (KVM+Podman
compute) · `mde-drawer` (quick actions) · `mde-wizard` (Birthright first-run).

## 4. Reuse strategy (REUSE IS KEY)

- **New repo `MackesWorkstation`** wires the three layers together; new code is glue.
- **Shared crates as the spine:** `mde-kdc-proto` is already shared; the platform
  crates become libraries (+ `mackesd` as a supervised service); MDE-Retro's
  theme/widget/state/icon kits power the Win10 shell; the Workbench ports in largely
  as-is.
- **Two design languages coexist by design:** Win10/Carbon for the shell, Material/
  ChromeOS-Classic for the Workbench (the deliberate "power-user contrast").
- Per-crate reuse classification (as-is / adapt / rebuild) is the **next deliverable**
  (pending the load-bearing answers).

## 5. Standing requirements (owner-set, 2026-06-03)

- **Disclaimer on every surface.** The Mackes Workstation Warning/Disclaimer/Mission
  ([`../DISCLAIMER.md`](../DISCLAIMER.md)) appears on all About + informational
  surfaces. Already wired in MDE-Retro (About dialog, System Properties, READMEs,
  embedded via `disclaimer.rs`/`include_str!`). Any **new** surface in Mackes
  Workstation (Workbench About/Help, the platform daemon `--version`/banner, the
  installer) must pull from the same single source.
- **Hold the RPM** until **all features are ready**; do not cut a release before then.
- **Hardware-bench testing happens *after* release** — so hardware-/interactive-gated
  features (KDE Connect phone round-trips, BlueZ pairing, PAM unlock, live dnf
  streaming, vertical-taskbar UX) are *build-complete* now; their device/UX bench is
  a post-release step, not a blocker for "ready."

## 6. The 50 questions (the decision gate)

### A · Strategy, repo & scope
1. Repo shape: monorepo absorbing both codebases, or a thin integration repo that git-deps them?
2. Does MDE-Retro continue independently, or is it subsumed (and archived)?
3. Does MDE continue shipping independently, or become "the platform layer of Mackes Workstation"?
4. Terminology: which is "underneath" (MDE) vs "on top" (MDE-Retro)? What does **MDE4WG** stand for?
5. MVP for v1: which features must ship first vs deferred?
6. Audience: personal/home, small-fleet ops, or a distributable product?
7. Versioning: own line, or track MDE's planned 1.0 "brand reset"?

### B · The fusion architecture
8. **Compositor:** labwc (MDE-Retro) vs sway (MDE) — standardize on one (which?) or stay compositor-agnostic via layer-shell only?
9. Daemon model: adopt `mackesd` as the single supervisor, import `mackesd_core` as a lib, or shell out to CLI subcommands?
10. Session bootstrap: keep MDE-Retro's labwc session, MDE's `mde-session`, or a new launcher?
11. D-Bus vs `mde-bus`: target the stable-but-deprecated D-Bus API or the future Bus?
12. One multiplexed binary for surfaces + separate supervised services for daemons?
13. Live theme switching via a `ThemeChanged` Bus signal, or keep "relaunch to switch"?
14. Settings as a registered-module registry (extensible) vs the current monolithic match?
15. Win10↔Workbench boundary: a hard rule, or case-by-case per feature?

### C · Platform substrate (mesh, storage, scale)
16. **Gluster mesh-home:** precondition, or optional (local-only state first-class)?
17. Nebula mesh: mandatory substrate or optional capability?
18. 8-peer cap: applies to the Workstation? target or hard limit?
19. QNM-Shared / coordination files: keep, or local state for single-box?
20. Fleet/playbooks (multi-machine): in scope for a "Workstation," or MDE-server territory?
21. Netdata/metrics: ship it? surface in the Win10 shell or only the Workbench?
22. Must every platform feature degrade gracefully with no mesh / no peers (standalone first-class)?

### D · Reuse mechanics
23. Two design languages coexist (Win10 shell + Material Workbench), confirmed?
24. Two widget kits (`mde-ui` flat-Carbon + `mde-iced-components`), or unify?
25. Per-crate: as-is / re-skinned / rebuilt — produce the table once 8/23 land.
26. Is `MDE-KDECnt-Rust`'s host the canonical KDE Connect, with MDE's `mde-kdc` converging onto it (one host, not two)?
27. Config: unify on MDE's TOML `mde-config` + `mackesd`, or keep MDE-Retro's `menu.json` + bridge?
28. Extend MDE-Retro's accuracy harness to cover the Workbench + platform surfaces?

### E · Win10-surface vs Workbench placement (per feature)
29. File manager: merge MDE's mesh-first `mde-files` into the Win10 Explorer, or keep it separate/Workbench?
30. Music (`mde-music`): Win10 app or Workbench?
31. Voice/VoIP (`mde-voice`, PJSIP softphone): Win10 "Phone" app, tray/flyout, or Workbench? Include in v1 at all?
32. Compute manager (`mde-virtual`, KVM+Podman): Workbench panel, standalone admin app, or out of scope?
33. Quick-actions drawer (`mde-drawer`): fold into the Win10 Action Center, or keep separate?
34. First-run: MDE's Birthright wizard, MDE-Retro's Win10 OOBE, or a merged flow?
35. Device surfaces: do "Your Phone"/Mobile Devices and the Workbench mesh views share one device model?
36. Maintain (snapshots/debloat/health/repair/drift): all Workbench, or do any surface in Win10 Settings (e.g. snapshots→System Restore)?
37. Network admin (VPN/firewall/remote-desktop/service-publishing): Workbench, or consumer bits in the Win10 flyout + admin in Workbench?
38. Presets/drift: Workbench-only, or a "restore my setup" in the Win10 shell?
39. Applets (17 `mde-applets`): map to Win10 tray/Action-Center tiles, or remain a separate applet system?

### F · The Workbench surface
40. Keep the ChromeOS-Classic/PatternFly identity (deliberate contrast), or re-skin to a Win10 "Computer Management" look?
41. Entry point from the Win10 shell: Start tile, a "Mackes Workbench" app, a keybind, or Settings "advanced"?
42. Nav model: keep the fixed 9-group/43-panel tree, or make it dynamic?
43. Scope: are the 11 known Workbench port-gaps v1 blockers or v1.1?

### G · Connectivity & phone
44. KDE Connect pairing store: one shared store across shell + `mackesd`, or per-binary?
45. Phone identity: are the Nebula cert (mesh CA) and KDE Connect fingerprint bound to one identity, or two?
46. Phone transport preference: KDC-TLS (battery) vs Nebula — what drives it, and is it user-visible?
47. Phone data (battery/ring/find): from KDE Connect plugins, Nebula probes, or both (authoritative source)?
48. **Finish the KDE Connect inbound listener (host 3b.2e) now, or is outbound-`open` enough for v1's phone flows?** *(KDE Connect crate work is parked on this.)*

### H · Migration, packaging, licensing
49. Packaging relative to MDE's `mde-core`/`mde-desktop` split — new flavor, drop-in replacement, or conflicting? (RPM **held until ready**.)
50. Licensing/IP: both GPL-3.0; the "Win10 look" raises trade-dress questions — guidance on resemblance, branding, attribution?

## 7. Load-bearing four (answering these unblocks ~80%)

**Q8** (compositor) · **Q16** (Gluster precondition) · **Q1** (repo shape) · **Q5** (MVP scope).

## 8. Executable plan — STATUS: READY (assembled 2026-06-03)

The decision gate is closed (27 answers + the governing **"Mirror Windows 10"** rule,
§0). §9–§12 below are the **executable plan**, synthesized against the current code:
MDE `@6459e17`, MDE-Retro `main`, and MDE-KDECnt-Rust. **Still PLANNING ONLY** — no
`MackesWorkstation` repo is created and nothing is built until the owner says
"execute." Reading order: §9 *what we keep* → §10 *what changes in the shell* →
§11 *the order we build it* → §12 *where it all lives + how it ships*.

Mackes Workstation is the **successor** (§0): it EOLs both MDE and MDE-Retro; their
repos become upstream history under the one monorepo.

## 9. Per-crate reuse table (REUSE IS KEY)

All ~35 workspace crates (33 MDE + 2 MDE-Retro — plus the canonical KDE Connect host imported
from the separate MDE-KDECnt-Rust repo at E0; exact count firms up at import), each classified
**as-is** (move unchanged) /
**adapt** (reuse backend, reskin/rewire) / **rebuild-or-reskin** / **retire-absorb**
(functions fold into a Win10/Workbench surface, crate retires post-v10). Target layer:
`shared-lib` · `platform-daemon` · `win10-surface` · `app` · `workbench`.

| Crate | Disposition | Layer | Notes |
|---|---|---|---|
| mde (MDE-Retro) | as-is | win10-surface | The shell — **PRIMARY** Win10-inspired surface; **replaces** mde-portal |
| mde-ui | as-is | shared-lib | **Canonical** design system (Win2000+Win10 widgets, Carbon palette) |
| mde-portal | **retire-absorb** | win10-surface | Unified shell → Win10 shell replaces; power layers → Workbench/Settings |
| mde-drawer | **retire-absorb** | win10-surface | Quick-actions → fold into Win10 Action Center tiles |
| mde-virtual | **retire-absorb** | workbench | KVM+Podman → Workbench "Compute" role (novel, not Win10) |
| mde-workbench | **rebuild-or-reskin** | workbench | → Server-2003 "Manage Your Server" + Win10 design, consume mde-ui |
| mde-files | adapt | win10-surface | Mesh file mgr → Win10 Explorer + mesh quick-access |
| mde-music | adapt | app | Airsonic GUI → Win10 "Media Player" app (MPRIS+Bus) |
| mde-musicd | adapt | platform-daemon | Airsonic REST client → supervised service; GUI via Bus/MPRIS |
| mde-voice-hud | adapt | win10-surface | PJSIP softphone → Win10 "Phone/Calls" app + call HUD |
| mde-voice-config | as-is | shared-lib | Pure-fn kamailio/rtpengine config gen |
| mde-theme | adapt | shared-lib | Design tokens → consolidate onto mde-ui Carbon/Win10 |
| mde-kdc | adapt | platform-daemon | KDE Connect host → converge onto MDE-KDECnt-Rust canonical |
| mde-kdc-proto | as-is | shared-lib | KDE Connect protocol (pure Rust+ring) |
| mde-applets | adapt | win10-surface | 17 applets → Win10 tray + Action-Center tiles |
| mde-panel | adapt | win10-surface | Top bar+dock → Win10 Taskbar+Tray |
| mde-popover | adapt | win10-surface | Layer-shell popover → Win10 Start Menu+popovers |
| mde-peer-card | adapt | win10-surface | Peer Connection Card → Workbench + Win10 Network modal |
| mde-wizard | adapt | app | First-run → Win10 OOBE + Birthright mesh enrolment |
| mde-installer | adapt | platform-daemon | + deployment-role chooser (Lighthouse/Server/Workstation) |
| mde-logout-dialog | as-is | app | Logout/restart/shutdown → Win10 reskin |
| mde-session | as-is | platform-daemon | Session orchestrator → launch labwc; keep D-Bus (FDO carve-out) |
| mde-bus | as-is | shared-lib | Mesh pub/sub (ntfy/Nebula) — **platform IPC backbone** |
| mackesd | as-is | platform-daemon | **Control plane** — supervised service + workers + mesh/CA |
| mde-clipd | as-is | platform-daemon | Clipboard daemon (wlr-data-control → Bus) |
| mde-alert-emit | as-is | platform-daemon | Netdata alert translator |
| mde-iced-components | as-is | shared-lib | Shared iced widgets (Object Card) |
| mde-card | as-is | shared-lib | Universal cards subsystem + mesh probe |
| mackes-mesh-types | as-is | shared-lib | Canonical mesh-resource types |
| mackes-transport | as-is | shared-lib | Transport trait + capability model |
| mackes-config | as-is | shared-lib | Serde TOML schema (panel.toml) |
| mackes-theme | as-is | shared-lib | Carbon→cosmic CSS adapter → merge into mde-ui or deprecate |
| mackes-nebula-https-tunnel | as-is | shared-lib | Covert TLS (Nebula UDP over HTTPS) mesh fallback |
| mde-config / mde-mesh-types / mded | as-is | shared-lib | Re-export facades → merge into mackes-config / mackes-mesh-types / mackesd post-transition |

**Platform substrate retained wholesale:** Bus, mackesd, Nebula, LizardFS mesh-storage,
KDE Connect proto. **Design unifies** on `mde-ui` (flat-Carbon + Win10 palette, one
system — no separate Material/ChromeOS look).

## 10. MDE-Retro shell changes (the Win10 surface build, E0–E20 + substrate)

Each item is reachable from an `mde <subcommand>`, themed only via `palette::color()`
(no raw hex), metrics via `metrics::UI_PX`. Era-gated on `palette::theme() ==
Theme::Windows10` so Carbon stays the default and Win2000/BeOS are untouched.

**E0 — Era foundation.** `Theme::Windows10` (THEME atomic = 3) + `win10(rgb)` remap
(accent `#0078d4`/`#2899f5`, Win10 greys), wired through `palette::color()`,
`font::family()`, `state.rs` (`"windows10"`), `main.rs` startup. Win10 panel = bottom
anchor. Display ▸ Appearance gains a "Windows 10" picker (labwc themerc rewriter).
Pinned by `windows10_remap_pins` in `checklist.rs`.

**E1 — Tiled Start** (`mde start-win10`). Full-screen layer-shell overlay: left icon-rail
(account/folders/Settings/Power, hover-expands), center (Recently-Added/Suggested/All-Apps
A–Z), right tile grid (`StartTile` in `state.rs`). Right-click Pin/Unpin/Resize/Uninstall.
Headless CLI `--pin/--unpin/--resize/--list-tiles`. Reuses `menu.rs` launch/context.

**E2 — Win10 taskbar** (`panel.rs` `view_win10()`). Bottom-anchored: Start tile, Search
box (E5), Task View (E4), app buttons (accent underline on focus), tray, two-line clock,
Action Center button + unread badge (reads `notifications.json`). New surfaces: `mde
search`/`taskview`/`action-center`/`jumplist`. Win+A → Action Center.

**E3 — Action Center + notification daemon** (`notifyd.rs`, `action_center.rs`). zbus
daemon claims `org.freedesktop.Notifications` (hosted in the panel process, persists across
restarts), store mirrored to `notifications.json`. `mde action-center` (right pane, Win+A)
+ `mde toast <id>` (bottom-right transient). Quick-action tile grid
(Wi-Fi/BT/Airplane/Brightness/Volume/Night-light/Focus) backed by NM/BlueZ/wlsunset.
Feeds E2 badge; toast source for E9/E12/E13/E15/E16/E17.

**E4 — Multitasking** (`task_view.rs`, `workspace.rs`). Win+Tab full-screen grid (icon+title
tiles from `wlr.rs`, no pixel thumbnails). Virtual desktops via `ext-workspace-v1` with an
honest fallback ladder. Snap = labwc rc.xml edge-snap keybinds (mde never owns geometry);
Snap Assist = `mde task-view --snap-assist <side>` (focus-only, labwc chain-snaps).

**E5 — Search + Quick Access** (`search.rs`). Win+S overlay, tabs All/Apps/Documents/Web/
Settings (apps via `apps::programs()`, docs via `fd`, web via DuckDuckGo). Win+X Quick
Access menu (`popup.rs items_for("quickaccess")`) → System/Device-Mgr/Disk/Power/Event-Viewer/
Network/Task-Mgr/Terminal/Run. Both Win10-gated.

**E6 — Modern Settings app** (`settings.rs`, `mde settings`, Win+I). Category grid (System,
Devices, Phone, Network, Personalization, Apps, Accounts, Time & Language, Ease of Access,
Privacy, Update & Security) + left rail. **Replaces Control Panel in Win10 era only**;
Win2000/Carbon keep `mde control-panel`. M1 live pages: Display, About, Printers, Colors,
Background. Reuses `control_panel.rs` shape + `fedora::TOOLS`.

**E7 — Personalization** (`settings/personalization.rs`). Colors (Light/Dark/Custom +
accent grid → `set_dark`/`set_accent`, new `win10_accent`), Background (Picture/Solid/
Slideshow, reuse `display.rs` wallpaper helpers), Themes, Lock screen (Spotlight-style local
rotation), Start, Taskbar pages. New `#[serde(default)]` state fields.

**E8 — File Explorer** (`files.rs` Win10 routing). Quick Access landing (Frequent + Recent),
This PC (mounts from `/proc/mounts`), Network (SMB via gio/smbclient), **Cloud Files = paired
KDE Connect devices** (remote browse via sftp/gio), mesh-storage LizardFS mounts. Breadcrumb
+ flat command row. `mde mount <uri>`.

**E9 — Your Phone** (`connect.rs` zbus client + `phone.rs`). `mde phone --view=messages|photos|
calls|notifications --device=<id>`. Three-region: device picker rail + per-view pane
(Notifications/Messages/Photos/Calls/Settings). Toasts via E3 filtered to KDE Connect.
Depends on E3 + MDE-KDECnt-Rust + `mde connect` daemon.

**E10 — Accounts / Lock / Sign-in.** Settings ▸ Accounts (Your-info → `~/.face`, Sign-in
options incl. argon2 PIN at `~/.config/mde/pin.hash`, Family & other users via useradd/usermod
behind pkexec). `mde lock` (Win+L) layer-shell lock face (PIN argon2 / password via PAM).
LightDM-gtk greeter theme generated from `win10()` tokens. New dep: `argon2`.

**E11 — OOBE** (extends `installer.rs` + `tui_setup.rs`). `OobeEra::{Classic,Win10}` from
theme. Stages Region/Keyboard/Network/Account/PIN/Privacy/Your-Phone/Personalize/Finalize.
GUI+TUI share pickers. `oobe_done` state. *(Note: an `oobe.rs` scaffold + state fields
already landed in MDE-Retro; this epic merges them with the Birthright mesh enrolment.)*

**E12 — Settings ▸ Devices** (`settings/devices.rs`, `bluez.rs` bg-thread). Bluetooth
(BlueZ via zbus: power/discover/pair/remove), Printers (lpinfo/lpadmin/lpstat), Mouse/Touchpad/
Typing (labwc libinput config), AutoPlay (udisks2), Project/second-display (`mde project`,
Win+P). Deep-links `mde settings --page devices[:bluetooth|...]`.

**E13 — Windows Update** (`settings/update.rs`). dnf-backed: check (`dnf check-update`),
install (`pkexec dnf upgrade`), feature-update probe, pause (≤35d), active hours, history
(`dnf history`), uninstall (`history undo`), advanced toggles. Promotes the
`system_properties.rs` auto-update stub to a real `sysinfo::set_auto(AutoMode)` persisted in
state (shared backend, screenshot-parity).

**E14 — Security dashboard** (`security.rs` + `security_probe.rs`, `mde security`, Win10-only).
Tiles: Virus & threat (ClamAV optional), Firewall (firewalld zones), Device encryption (LUKS
status + recovery-key backup; turn-on is destructive-confirm, never auto-runs), Find-my-device
(KDE Connect ring/lock), Secure Boot/TPM read-only probes. New `STATUS_OK/WARN/RISK` roles
pinned in checklist.

**E15 — Networking** (`net_flyout.rs` + `settings/network.rs` + `nm.rs` pure backend). Panel
net-glyph flyout (Wi-Fi list/connect, Airplane). Settings pages: Status, Wi-Fi, Ethernet, VPN,
Mobile hotspot, Proxy, Data usage, Airplane. Action-Center toggles call `nm::set_*`.
Win2000/Carbon keep nm-connection-editor.

**E16 — Clipboard history + Screenshots** (`clipboard.rs` + `snip.rs`). `wl-paste --watch`
ring buffer (`~/.local/share/mde/clipboard/`, 25 unpinned + pinned), `mde clipboard` (Win+V)
overlay. `mde snip` (Win+Shift+S) over grim+slurp: rect/window/full/clip, PrintScreen family
mapped, toast via E3.

**E17 — Storage / Backup / Recovery** (`settings/{storage,backup,recovery}.rs`). Storage Sense
(systemd timer + dnf/journald clean), usage breakdown, Apps drill-in. Backup = Timeshift
(add-drive/schedule/retention/back-up-now/restore). **System Restore** browser with green
`RESTORE_PRIMARY` "Restore to original location". Recovery = Reset-this-PC (two-mode,
typed-destructive confirm), Advanced startup, Create recovery drive.

**E18 — Edge → Firefox** (`browser.rs` + `browser_jumplist.rs`). `default_browser()` via
xdg-settings, `recent_sites()` read-only over places.sqlite, jump list (New/Private/Recent).
Default-apps "Web browser" row. Label is always "Firefox" (never fake Edge brand, §2.4). New
dep: `rusqlite` (read-only).

**E19 — Power / Session** (extends `dialogs.rs`). `Choice::Lock` + `mde lock` (Win+L, loginctl
lock-session → swaylock). Win10 flat-flyout rows (Sleep/Shutdown/Restart, Lock/Sign-out);
Win2000/Carbon keep the dropdown.

**E20 — Polish + accuracy.** Pins `Theme::Windows10` + `win10()` + roles into `checklist.rs`;
dynamic `[capture.win10-*]` accuracy points (accent at Start/taskbar/Action-Center/Settings);
per-era `gallery.sh` captures (era-aware crops); documented rc.xml keybind table; focus-ring
checks. Win10 reaches parity with Win2000/BeOS/Carbon in the accuracy gate.

### Cross-cutting substrate (not a single surface)

- **Registered-module Settings registry (decided — hybrid, §0).** A lightweight metadata
  registry (category/title/icon/deep-link) drives the Settings home grid + nav rail; each page
  renders as its own `mde settings --page X` process (per-process theme model, grim-capturable).
  Epics register page metadata only — they never edit a central match tree.
- **Bus client integration** — Win10 surfaces talk to mackesd workers over `mde-bus` (not
  private D-Bus). Prove theme/accent signal delivery first, then E3/E9/E15 action↔state loops.
- **Mesh/peer Quick Access + LizardFS FUSE** — E8 Explorer enumerates LizardFS mesh mounts;
  installer ensures the FUSE mount is live before any surface browses. Blocked on the LizardFS
  binding landing.
- **mackesd supervisor + role chooser** — mackesd is the authority for long-lived state;
  surfaces **degrade gracefully** if workers are absent (cached state, Bus timeouts, never
  panic). Install-time role gates which workers + surfaces install.
- **Disclaimer everywhere** — every new About/Info/Help surface (Settings, Security,
  Storage/Backup, Workbench) pulls the single `DISCLAIMER.md` via `disclaimer.rs`
  `include_str!` — never copy-paste.
- **Workbench re-skin** — `mde-workbench` (9 groups / 43 panels) → Server-2003 "Manage Your
  Server" mold (left-nav role cards + description + action links + Tools/See-also sidebar)
  wearing `win10()` + `icon_any` (no separate Material set). Power-user only; deferred to a
  post-v1-shell task (not a ship blocker).
- **Snapshots as System Restore** — one Timeshift backend, two entry points (Settings ▸
  Recovery + Workbench Maintain).

## 11. Epic breakdown & milestone sequence

Eight ordered epics. **E0** is foundational (blocks all). **E1–E3** are parallel substrate.
**E4–E5** the UI surface layer. **E6–E7** complete the user model. **E8** gates the held RPM.
Every epic enforces §3 Definition of Done (no stubs, runtime-reachable, disclaimer embedded).

| Epic | Scope | Depends on | Unblocks |
|---|---|---|---|
| **E0 Monorepo Bootstrap** | Cargo workspace absorbs ~35 workspace crates (platform + shell + Workbench); v10.0.0/GPL-3.0; wire `mde-bus` (retire D-Bus); import labwc config; EOL/archive old repos; disclaimer embedding; mackesd systemd unit; verify `mde <sub>` dispatch | none | E1–E7 |
| **E1 Deployment-Role Install** | RPM install-time role chooser (Lighthouse/Server/Workstation) → mackesd worker subset + role-gated surfaces; role-aware systemd units + `/etc/mackesd/` templates; wire selector into installer | E0 | E2,E4,E5,E7 |
| **E2 KDE Connect Convergence** | Finish MDE-KDECnt-Rust **inbound listener (3b.2e)** → bidirectional; converge in-tree `mde-kdc` onto canonical crate; pairing store + host in mackesd; sftp mount for Explorer Cloud Files; verify round-trip with a real phone | E0,E1 | E5,E9 |
| **E3 Mesh-Storage LizardFS** | LizardFS master+chunk daemons; mount mesh XDG dirs owned by mackesd; topology-aware replication; offline graceful degrade (Gluster retired) | E0,E1 | E4,E5,E6 |
| **E4 Win10 Shell Replaces mde-portal** | Retire portal; functions reappear as Win10 idioms; port 13 network panels → Settings ▸ Network; **Settings registry**; migrate power-user net/CA/fleet → Workbench; scale Settings to 20+ pages | E0,E1,E3 | E5,E6 |
| **E5 New Apps + Explorer + Action Center** | Phone/Calls app (voice+KDC), Media Player (music+MPRIS), Explorer mesh Quick-Access, drawer→Action Center, 17 applets→tray/tiles | E0,E1,E2,E3,E4 | E7,E8 |
| **E6 Workbench Re-skin** | Server-2003 "Manage Your Server" + Win10 design; 43 panels → role/section cards + action links; fold compute/fleet/maintain/presets/role-mgmt; retire network panels (→E4); Start tile + "Manage Workstation" entry | E0,E1,E3,E4,E5 | E7,E8 |
| **E7 Merged OOBE + Mesh Enrolment** | Win10 OOBE ∪ Birthright wizard; role picker early; Nebula cert/CA enrolment step; optional KDC phone pair; disclaimer "read before proceeding" step | E0,E1,E2,E4,E5,E6 | E8 |
| **E8 Polish + Held RPM Release** | Accuracy harness over Workbench + new apps; screenshot all 43 panels + new surfaces; **disclaimer audit sweep** across every About surface; per-era pixel compliance; full build/test/clippy/fmt; verify E1–E7 §3-complete; **cut RPM v10.0.0** + CHANGELOG | E0–E7 | release gate |

## 12. Monorepo workspace layout + deployment-role architecture

**One** Cargo workspace, `version = "10.0.0"`, `license = "GPL-3.0-only"`, `edition 2021`,
`rust-version 1.85`. Members grouped:

```
crates/platform/*   mde-bus, CA/enrollment
crates/mesh/*       mackesd (lib+bin), mackes-mesh-types, mackes-nebula-https-tunnel,
                    mackes-config, mackes-transport
crates/shell/*      mde (multiplexed dispatcher), mde-ui, mde-installer, mde-wizard,
                    mde-session, mde-logout-dialog, mde-peer-card, mde-popover, mde-card,
                    mde-alert-emit  (+ retiring: mde-portal, mde-drawer)
crates/workbench/*  mde-workbench, mde-virtual
crates/shared/*     mde-theme, mde-iced-components
crates/applets/*    mde-applets (17 widgets)
crates/services/*   mde-musicd, mde-music, mde-files, mde-clipd, mde-voice-config, mde-voice-hud
crates/kdc/*        mde-kdc, mde-kdc-proto  (canonical host = MDE-KDECnt-Rust)
```

**Deployment roles — ONE RPM, install-time chooser** (`mde setup --profile=<role>` or OOBE
menu; stored immutable in `/var/lib/mde/role.toml`; upgrade allowed, downgrade blocked). Each
role is a strict superset:

1. **Lighthouse** (rank 0) — VPS relay. mackesd (enrollment CA only) + mde-bus + Nebula +
   LizardFS read-only client. No desktop, no media/voice/compute, no display manager.
2. **Server** (rank 1, headless) — Lighthouse + LizardFS chunk brick + fleet (ansible-pull) +
   monitoring. Still no desktop.
3. **Workstation** (rank 2, full desktop) — Server + sway/labwc + `mde` shell + all GUI + 17 applets +
   service daemons + greetd/regreet/cage + libvirt/qemu + kamailio/rtpengine + fonts.

**mackesd worker subsets by role:** Lighthouse = enrollment(CA)+leader+health · Server = +
fleet+meshfs(LizardFS FUSE)+metrics · Workstation = + voice coordinator + media stack.

**Surfaces gated by role:** CLI subcommands (`mde panel/menu/files/net-flyout/filedialog`)
available everywhere; desktop-only (`mde settings/start-win10/action-center/security/oobe/
installer`) ENOENT on non-Workstation; Workbench binary installs only under Workstation.

**RPM subpackage structure** (one spec, conditional — package ids, not role names): `mde-core`
(all roles — `mde`, `mackesd`, `mde-bus`, `mde-kdc`, systemd `mackesd.service`+`mde-bus.service`)
· `mde-headless` (Server+Workstation — lizardfs, ansible-pull.timer) · `mde-desktop`
(Workstation only, `Requires: mde-core` — sway/labwc/
regreet/cage, `mde-workbench`, applets, icons/themes/fonts, `mde-session.service`+`greetd.service`).
`Provides:` legacy names; `Obsoletes:` the old xfce/i3 packages.

**Versioning:** single `[workspace.package] version = "10.0.0"`; all crates inherit; one git tag
`MackesWorkstation-v10.0.0`. **Disclaimer pre-flight gate:** `DISCLAIMER.md` must exist + be
non-empty before any RPM build.

---

*Plan complete and executable. Awaiting "execute" to create the `MackesWorkstation` monorepo
and begin E0. The RPM (E8) stays held until all features are §3-complete; hardware bench is
post-release.*
