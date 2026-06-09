# Mesh Decoupling — "Magic Mesh" on Cosmic (and standalone)

> **Status:** LOCKED via a 124-question operator survey + directives, 2026-06-08.
> Captures the pivot from the integrated **MDE** desktop to a
> **separable mesh platform ("Magic Mesh")** that runs as an add-on to the
> **Cosmic** desktop or standalone — and the **EOL of the MDE desktop GUI**.
> This doc is the source of truth for the new **E11+** epic line. Where it
> conflicts with older locks, **this wins** (CLAUDE.md "newest wins"); the
> `AI_GOVERNANCE.md` rewrite (Q85) ratifies it platform-wide.
>
> **Brand (operator decision, 2026-06-08):** the platform is **Magic** ("Magic
> Mesh"). The rebrand is **brand/display-only** — internal technical identifiers
> (`mde`, `mackesd`, `dev.mackes.MDE.*`, `~/.config/mde`, existing crate names)
> are **retained**; only product/brand names + user-visible text become "Magic".

---

## 0. The decision in one paragraph

The labwc/Win-era **MDE desktop GUI is EOL** (hard cutover, fresh-install only).
The platform becomes **Magic Mesh** — the mesh/infra stack (mackesd, the ntfy
Bus, Nebula, LizardFS, KDC, fleet, compute, voice, CA) plus its **GUIs
(Workbench, mde-files)** — installed as an **add-on on stock Fedora-Cosmic** or
as a **self-contained minimal-GUI appliance**, and manageable **headless**.
**Workbench** is the single control surface; **mde-files** becomes the default
file manager; a **D-Bus bridge + native cosmic-applet** connect the Bus to
Cosmic; notifications route through Cosmic's daemon; and a **full Carbon visual
identity** (icons/theme/fonts/accent/wallpaper/boot) is applied on install. One
feature-flagged codebase, **split into its own `magic-mesh` repo**, guarded by
a hard crate-dependency boundary gate against the dying shell.

---

## 1. Locks (Q1–Q124 + directives)

### Scope & boundary (Q1–Q7)
| # | Lock |
|---|---|
| Q1 | Carry-forward (mesh) set confirmed: mackesd, mde-bus, Nebula, LizardFS, KDC, Workbench, mde-files, notification bridge, fleet, compute, CA. Dropped: labwc shell, panel, menu, OSD, action-center, Win-era Settings, themes, window-mgmt, lock/greeter. |
| Q2 | Mesh-only settings **fold into Workbench** (no separate settings app). |
| Q3 | **Core headless**; GUI features (voice HUD, mde-files, Workbench) may assume a session. |
| Q4 | Host detected by an **explicit install-time flag** (`host = mde \| cosmic \| headless`). |
| Q5 | `voice-hud --agent` carries forward (mesh); `clipboard` + `devices-monitor` are desktop → dropped. |
| Q6 | **Hard CI boundary gate** forbidding mesh crates depending on the shell. |
| Q7 | **Single feature-flagged codebase** (desktop/mesh-only/headless by build+install features). |

### Product & packaging (Q8–Q13)
| # | Lock |
|---|---|
| Q8 | Name: **Magic Mesh**. |
| Q9 | **One RPM + install-time host/role chooser**. |
| Q10 | Add-on install path: **RPM / COPR on Fedora-Cosmic**. |
| Q11 | Standalone = **self-contained minimal-Workbench GUI appliance**. |
| Q12 | **Independent version line** from the (dying) desktop. |
| Q13 | Target distro: **Fedora-Cosmic only**. |

### Bus ↔ Cosmic bridge (Q14–Q20)
| # | Lock |
|---|---|
| Q14 | Bridge **both directions** (surface state + accept actions). |
| Q15 | Primitive: a **D-Bus bridge + a native cosmic-applet** consuming it. |
| Q16 | **Yes** — a native cosmic-applet shows mesh status. |
| Q17 | **Full event set** crosses the bridge. |
| Q18 | Auth: **loopback + same-user trust**. |
| Q19 | Bridge is **bespoke/internal** (no public-API commitment). |
| Q20 | ntfy broker **localhost-bound under mackesd**. |

### Notifications (Q21–Q26)
| # | Lock |
|---|---|
| Q21 | On Cosmic, **drop mde notifyd**; use Cosmic's FDO daemon. |
| Q22 | Mesh notifications via a **Cosmic-native path**. |
| Q23 | DND **defers to the host** and syncs from it. |
| Q24 | Notification **action buttons call back into the Bus**. |
| Q25 | SIP call HUD carries forward as a **standalone layer-shell app**. |
| Q26 | History: **Cosmic shows it; mesh logs to Persist**. |

### Workbench (Q27–Q33)
| # | Lock |
|---|---|
| Q27 | Workbench stays on **iced**, **Carbon always**. |
| Q28 | Theming: **Carbon always** (Magic identity holds on any host). |
| Q29 | Form: **toplevel app + the cosmic-applet**. |
| Q30 | **Remove desktop settings links; fold mesh settings in**. |
| Q31 | **Mesh/infra groups only** (drop LookAndFeel/Apps-via-dnf/datetime). |
| Q32 | Launch via **.desktop + an autostart applet**. |
| Q33 | Workbench is the **single pane of glass** for all mesh config. |

### File manager — mde-files (Q34–Q39, Q63–Q66)
| # | Lock |
|---|---|
| Q34 | mde-files **replaces Cosmic Files**. |
| Q35 | **Registers as the default** `inode/directory` handler. |
| Q36 | Backends: **LizardFS + SMB/Network + KDC Cloud-Files** (all three). |
| Q37/Q63 | **Full hard cut** to default — full general-FM parity, no phasing. |
| Q38 | **Artifacts as a first-class view** (mde-card). |
| Q39 | **Bus file events + notifications**. |
| Q64 | **Native LizardFS client** (not just FUSE). |
| Q65 | **Native-Rust** local file ops (trash/mounts/archives/properties). |
| Q66 | **Freedesktop thumbnailers + a content/name indexer**. |

### Headless / session (Q40–Q45)
| # | Lock |
|---|---|
| Q40 | Headless control via a **remote Workbench** from any desktop node. |
| Q41 | Nodes are **self-managed locally AND remote-manageable**. |
| Q42 | User-agents run as **systemd user units**. |
| Q43 | mackesd stays a **per-machine system service**. |
| Q44 | First-run **enrollment is a Workbench view**. |
| Q45 | Presence rebinds to **systemd-logind signals**. |

### Identity / risk (Q46–Q50)
| # | Lock |
|---|---|
| Q46 | Config namespace stays **`~/.config/mde`**. |
| Q47 | **Full Magic branding** on Cosmic. |
| Q48 | MDE **dogfoods then EOLs** (transitional host only). |
| Q49 | Boundary gate at **crate-dependency level**. |
| Q50 | **Lead risk: mde-files as a full-parity default.** |

### MDE EOL & salvage (Q51–Q54)
| # | Lock |
|---|---|
| Q51 | **Hard cutover** — next release is Cosmic+Mesh only; labwc shell removed. |
| Q52 | **Fresh install only** (no migration; re-enroll). |
| Q53 | **Absorb mesh-relevant surfaces** (birthright, voice HUD, mesh status); **delete desktop chrome**. |
| Q54 | **Drop `mde osd`** — Cosmic provides the OSD. |

### Cosmic integration depth (Q55–Q57)
| # | Lock |
|---|---|
| Q55 | **Stock Cosmic behavior** (ship no cosmic-comp/keybind/panel config) — *except tiling, Q90*. |
| Q56 | **No cosmic-settings integration**; all mesh config in Workbench. |
| Q57 | Applet built on **libcosmic / cosmic-applet** (styled Carbon). |

### Carbon visual identity (Q58–Q62, Q109–Q112)
| # | Lock |
|---|---|
| Q58 | Installer sets **full Carbon defaults** (icons + GTK/Qt theme + IBM Plex + cursor + Blue 60 accent). |
| Q59 | **Build a real Carbon freedesktop icon theme** from the Carbon Design icon library. |
| Q60 | Theme **GTK + Qt + the Cosmic accent**. |
| Q61 | Installer **writes cosmic-config + gsettings/dconf defaults** at install. |
| Q62 | Carbon is **default-on but reversible**. |
| Q109 | Branding reaches **boot → lock → apps → notifications**. |
| Q110 | **Magic plymouth boot theme**. |
| Q111 | **Magic Carbon wallpaper + lock background**. |
| Q112 | **Full runtime identity** (notifications/applet/Workbench/About). |

### Mesh services (Q67–Q70, Q75–Q78, Q119)
| # | Lock |
|---|---|
| Q67 | Nebula = **always-on system tunnel**. |
| Q68 | KDC in Workbench + FDO notifications. **+Directive: new-device pairing/enrollment gets a dedicated Workbench interface.** |
| Q69 | **Full Compute group** in Workbench. |
| Q70 | LizardFS **auto-mounts a standard path**. |
| Q75 | Call audio: **PipeWire-native, auto-route**. |
| Q76 | SIP config **in Workbench**. |
| Q77 | Incoming = **notification + layer-shell HUD**. |
| Q78 | Outbound: **Workbench + click-to-call + `tel:`/`sip:` handler**. |
| Q119 | **Mesh remote desktop (RDP/VNC) + SSH-across-mesh** both carry forward. |

### Bridge implementation (Q71–Q74)
| # | Lock |
|---|---|
| Q71 | Bridge keeps the **`dev.mackes.MDE.*`** D-Bus namespace. |
| Q72 | **Signals (events) + methods (actions) + properties (state)**. |
| Q73 | **Auto-reconnect + cache last state**. |
| Q74 | Applet is **event-driven** off the bridge's signals/properties. |

### Distribution / ops (Q79–Q82, Q87, Q117–Q118)
| # | Lock |
|---|---|
| Q79 | Updates via **dnf + an on-demand Workbench trigger**. |
| Q80 | **Signed RPM downloads** (no repo) — *but* COPR per Q10; reconcile at packaging stage. |
| Q81 | CI: **build + test + a Fedora-Cosmic install smoke-test**. |
| Q82 | **Local diagnostics bundle + opt-in crash reports**. |
| Q87 | Ship **both a turnkey ISO and the add-on COPR**. |
| Q117 | Maintain group = **full suite** (Snapshots/Health/Repair/Debloat/Drift/Updates/Hub). |
| Q118 | Update posture: **fully automatic** (with snapshot/rollback). |

### Repo / governance / roadmap (Q83–Q86, Q96–Q98)
| # | Lock |
|---|---|
| Q83 | **Split Magic Mesh into its own `magic-mesh` repo**. |
| Q84 | **Go public after the pivot lands** (operator-gated flip). |
| Q85 | **Rewrite `AI_GOVERNANCE.md`** for the Mesh-on-Cosmic identity. |
| Q86/Q96 | **New E11+ epic line**, led by **E11 = boundary gate + Bus bridge + cosmic-applet**. |
| Q97 | Repo `magic-mesh`, product **Magic Mesh**. |
| Q98 | Done = **runs fully on stock Fedora-Cosmic with the crate-dep gate green** (zero shell deps), all surfaces functional. |

### First-boot / input (Q88–Q90)
| # | Lock |
|---|---|
| Q88 | **Workbench first-run wizard** (enroll + Carbon defaults + tour). |
| Q89 | **No Magic keybinds** (pure stock). |
| Q90 | **Enable Cosmic tiling-by-default** — the one deliberate exception to stock config. |

### Data / security (Q91–Q94, +Directive)
| # | Lock |
|---|---|
| Q91 | **Mesh-replicated identity** (lost nodes rejoin automatically). |
| Q92 | **Single primary user** per node. |
| Q93 | Secrets in the **OS keyring, else mackesd-encrypted files**. |
| Q94 | **LizardFS replication + Workbench-managed snapshots**. |
| **+Directive** | **Maximum crypto:** use the strongest available encryption + key complexity for the mesh — Nebula cipher/curve, KDC TLS ciphers, CA key strength. Exact parameters pinned in a security-design task (E11.x). |

### Scope / spike (Q95, Q99–Q100, Q120)
| # | Lock |
|---|---|
| Q95 | **Full feature set in v1** (matches the hard-cut direction). |
| Q99 | First de-risking spike: the **Carbon-on-Cosmic install** (identity + distribution end-to-end). |
| Q100 | Carry-forward features (birthright, debloat, drift, branding) **all live inside Workbench**. |
| Q120 | **Inherit Cosmic a11y; i18n-ready, English first**. |

### Carried-forward systems (Q101–Q108, Q113–Q116, Q121–Q124)
| # | Lock |
|---|---|
| Q101 | birthright = **ongoing live health** (no one-time ceremony). |
| Q102 | birthright attests **mesh + Cosmic-integration** state. |
| Q103 | birthright **auto-remediates** failures. |
| Q104 | birthright **pushes health to a fleet dashboard**. |
| Q105 | Debloat = **curated allowlist + user picks**. |
| Q106 | Debloat **snapshot + dnf-history undo** (reversible). |
| Q107 | Drift watches **config vs fleet baseline**. **+Directive: ALL OS functions** (full config-management). |
| Q108 | Drift **auto-heals to baseline with audit**. |
| Q113 | Fleet push via **Workbench playbooks + a CLI/GitOps path**. |
| Q114 | New nodes **enroll → auto-join + inherit baseline**. |
| Q115 | **Version-aware revisions** for mixed-version fleets. |
| Q116 | **Peer-to-peer, no fixed controller** (honors the §0 master rule). |
| Q121 | Fleet sync covers the **full OS desired-state** (access, packages, services, files, networking/firewall, storage, time/locale, scheduled tasks, sysctl/kernel, certs, logging). |
| Q122 | Sync engine: **wrap Ansible** (mackesd orchestrates playbooks across the mesh). *Deliberate exception to the pure-Rust ethos — adds a Python/Ansible dependency.* |
| Q123 | Baselines authored as **declarative YAML desired-state** revisions. |
| Q124 | Conflict policy: **baseline wins, declared local exceptions allowed**. |

---

## 2. Resulting architecture

- **Magic Mesh = services + GUIs**, one feature-flagged codebase in a new
  `magic-mesh` repo. Install-time flag selects `cosmic` / `headless` (the
  `mde` desktop host is transitional, removed at EOL).
- **mackesd** (per-machine system service) supervises the **ntfy Bus** (loopback),
  **Nebula** (always-on system tunnel, max crypto), **LizardFS** (auto-mounted),
  the **KDC host**, and the **Ansible-wrapping fleet sync** engine.
- **Bus ↔ Cosmic:** a **D-Bus bridge** (`dev.mackes.MDE.*`, signals/methods/
  properties, auto-reconnect+cache) feeds a **libcosmic cosmic-applet** (Carbon-
  styled) for panel status; actions round-trip back to the Bus.
- **Notifications:** mesh emits via a Cosmic-native path to Cosmic's daemon;
  action buttons call back to the Bus; DND defers to the host; mesh keeps a
  Persist log.
- **Workbench** (iced, Carbon, toplevel + applet) is the single control surface:
  mesh/infra groups + folded-in mesh settings + first-run enrollment + **KDC
  device-pairing UI** + birthright (live health) + Maintain (snapshots/health/
  repair/debloat/drift/updates/hub) + Compute + voice/SIP + remote access +
  fleet (peer-to-peer, playbooks + CLI/GitOps).
- **mde-files** is the default file manager: native-Rust, full general-FM parity,
  native LizardFS client + SMB + KDC Cloud-Files, artifacts-as-a-view, Bus events.
- **Identity:** full Carbon (icon theme, GTK/Qt, IBM Plex, cursor, Blue 60 accent,
  plymouth, wallpaper/lock) applied by the installer (default-on, reversible);
  full runtime branding.
- **Distribution:** turnkey ISO **and** add-on (signed RPM/COPR); dnf updates +
  Workbench trigger; CI install-smoke on Fedora-Cosmic.

### 2a. Fleet sync = a Magic "Automation Mesh"

Ref: Red Hat **Ansible Automation Mesh**
(<https://www.redhat.com/en/technologies/management/ansible/automation-mesh>).
Its model is the right shape for our Q116 (no-fixed-center) + Q122 (wrap-Ansible)
+ Q67 (always-on Nebula overlay) locks, so the fleet-sync engine adopts it:

- **Execution near the endpoint.** Each node runs its own automation locally
  (`ansible-runner`, Podman-isolated) to converge its full OS desired-state
  (Q121) against its assigned YAML baseline (Q123) — instead of a central
  controller SSH-ing into every box. This removes the SSH-proxy/jump-host
  pattern and is resilient to latency/disconnects.
- **The Nebula overlay is the transport** (our analog of Automation Mesh's
  receptor overlay): a peer-to-peer fabric work routes over, with hop-style
  relaying through the lighthouse/peers when two nodes lack direct connectivity.
- **No fixed control node** (Q116): nodes are "hybrid" (originate *and* execute);
  **any** node's Workbench can author a revision and fan it out over the mesh.
  The lighthouse may relay but is **not** a required controller — native peering
  + hop-relay give fault tolerance and independent scaling.
- **Targeting = mesh/fleet membership** (the Instance-Group analog): a revision
  targets a fleet/group; matching nodes pick it up and self-apply, with
  version-aware revisions (Q115) and declared local exceptions (Q124).
- **Security:** TLS + the mesh CA's RBAC over the overlay — pinned to the
  **maximum-crypto** directive.

This makes "fleet sync" a true distributed automation mesh, not a hub-and-spoke
push, and it reuses Nebula we already run rather than standing up receptor.

---

## 3. Acceptance (the "decoupled" bar, Q98)

- [ ] Magic Mesh installs + runs on **stock Fedora-Cosmic** with the
  **crate-dependency boundary gate green** (no mesh crate depends on the `mde`
  shell).
- [ ] Workbench, the cosmic-applet, mde-files (default handler), notifications,
  Nebula, LizardFS, KDC (incl. pairing UI), voice/SIP, Compute, fleet sync, and
  birthright are all functional on Cosmic.
- [ ] Full Carbon defaults applied + reversible; full runtime branding present.
- [ ] Headless node manageable both locally and via a remote Workbench.
- [ ] Fleet sync converges a node's full OS desired-state to a YAML baseline
  (Ansible-backed), auto-healing drift with audit + declared exceptions.

---

## 4. Lead risk + mitigations (Q50/Q99)

**mde-files as a native-Rust, full-parity default file manager** is the biggest
bet (full hard cut, native LizardFS client, native ops). Mitigation: the **first
spike is the Carbon-on-Cosmic install** (proves identity + distribution cheaply)
while a **parallel spike** validates native full-parity file management +
LizardFS before committing the full-v1 build. Secondary risks: Bus exposure to a
foreign DE (loopback trust), notification/DND fidelity via the Cosmic-specific
path, and the Ansible dependency cutting against pure-Rust.

---

## 5. Open follow-ups (not yet locked)

- **Security parameters** — pin the exact Nebula cipher/curve, KDC TLS ciphers,
  and CA key strength for "maximum crypto" (E11 security-design task).
- **Q80 vs Q10** — reconcile "signed RPM downloads (no repo)" with "RPM/COPR";
  likely a signed COPR + downloadable signed RPMs.
- **`AI_GOVERNANCE.md` rewrite** (Q85) — ratify this pivot platform-wide; retire
  the labwc/§1-Carbon-shell locks.
- **Repo split mechanics** (Q83) — history, CI, and the boundary gate move to
  `magic-mesh`.

---

## 6. Out of scope (this design)

- The MDE desktop shell (panel/menu/OSD/action-center/Win-era Settings/themes/
  window-mgmt/lock/greeter) — **EOL, deleted at cutover**.
- Non-Cosmic desktops, non-Fedora distros (Q13).
- Multi-user-per-node (Q92), full a11y/multi-locale at launch (Q120).
