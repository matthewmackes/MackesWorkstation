# v3.x Runtime Integration Audit

**Date:** 2026-05-22
**Author:** Audit run after live bug reports — start menu won't close,
notification panel won't close, missing window-management buttons,
right-click M button does nothing.
**Scope:** `crates/mde-panel`, `crates/mde-popover`, `crates/mackesd`.
**Outcome:** Many worklist `[✓] Done` entries shipped **data-layer
helpers + tests only**, with the actual runtime wiring deferred to a
follow-up that never happened. This document inventories the gap so
the v3.0.3 integration pass has a single source of truth.

---

## TL;DR

- **Panel:** 13 of 18 `crates/mde-panel/src/*.rs` modules are dead
  code at runtime — declared `pub mod`, fully implemented, fully
  unit-tested, but **never referenced from `lib.rs`'s `update()` or
  `view()`**. Each corresponds to a Phase E.x `[✓] shipped` entry
  whose fine print said "the widget renders / subscription lands /
  popover opens when Phase E.2 (or E.3) wires up." Phase E.2 then
  shipped at v3.0.2 — but no integration sweep followed.
- **Popovers:** all four working popovers (start_menu, audio, clock,
  notifications) share an identical dismiss-handler defect:
  Esc-only close, `KeyboardInteractivity::OnDemand`, and the panel's
  `spawn_popover()` has zero dedup so each click stacks a new
  instance on top of the last. Zombies accumulate (18 observed in a
  single session) because the spawned `Child` handle is dropped
  without `wait()`.
- **Daemon:** 6 of the workers under `crates/mackesd/src/workers/`
  implement the `Worker` trait but are **never spawned**.
  `run_serve()` only registers the legacy reconcile worker. Phase B
  is marked `[✓]` end-to-end but only the trait + supervisor
  scaffolding actually run.

---

## Tier 1 — Live user-visible bugs (verified 2026-05-22)

### 1A. All popovers fail to close on outside-click

**Affected:** `mde-popover start-menu`, `mde-popover audio`,
`mde-popover clock`, `mde-popover notifications`.

Each popover's `subscription()` is:

```rust
fn subscription(&self) -> iced::Subscription<Message> {
    iced::keyboard::on_key_press(|key, _| {
        if matches!(key, Key::Named(Named::Escape)) {
            Some(Message::Exit)
        } else { None }
    })
}
```

with `keyboard_interactivity: KeyboardInteractivity::OnDemand` in
the layer-shell settings. Two failure modes:

1. wlr-layer-shell never delivers pointer events from outside the
   layer surface to the layer surface — the compositor swallows
   them. There is no outside-click dismiss anywhere in the code.
2. `OnDemand` keyboard interactivity means the popover only
   receives key events when the compositor decides to grant focus.
   If focus is on the panel or another window, Esc never reaches
   the popover.

**Fix shape:** add either a Close button inside the popover view,
a `KeyboardInteractivity::Exclusive` mode for the duration the
popover is open, or a transparent backdrop layer surface that
absorbs clicks and signals dismiss. (The backdrop approach matches
xdg-popup behavior most closely.)

### 1B. Popovers stack instead of toggling

`crates/mde-panel/src/lib.rs:286-293`:

```rust
fn spawn_popover(kind: &str) {
    let _ = Command::new("mde-popover")
        .arg(kind)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
```

No process-existence check, no kill-on-second-click. Comment at
line 282 says *"Per-popover dedup is left to the popover process
itself"* — but the popover binaries have zero dedup logic either.
Three live `mde-popover start-menu` instances were observed in a
single session.

**Fix shape:** track running popover PIDs in panel state; toggle
behavior (second click sends SIGTERM to the existing instance).

### 1C. Zombie accumulation

`spawn_popover()` drops the `Child` handle. When the popover exits
(user clicks an app, presses Esc, etc.) it becomes a zombie because
the panel never calls `wait()`. The four stub-style click targets
(audio / network / clock / notifications were stubs pre-v3.0.2,
network still is) made this dramatically worse — they exited
immediately, creating a zombie per click. 18 zombies observed in
one session.

**Fix shape:** either ignore `SIGCHLD` once at panel startup so
Linux auto-reaps, or hold child handles and reap on subscription
tick.

### 1D. Right-click M button does nothing

`crates/mde-panel/src/top_bar.rs:142-150` — the Start button uses
Iced's `button` widget with `.on_press(Message::StartClicked)`.
Iced's built-in button is left-click only. `admin_menu.rs` is fully
implemented (219 LOC, 8 tests, full action set, `spawn_action()`,
`build_foot_argv()`, sudo-cached probe) but is **never wired** to
any UI event.

**Fix shape:** replace the `button` with a custom mouse-area or
event-subscription widget that distinguishes left vs. right press
and emits `Message::StartLeftClicked` / `Message::StartRightClicked`.

### 1E. Window management buttons absent from center of panel

`crates/mde-panel/src/top_bar.rs::view` lays out
`start | dock | [fill] | cluster | [fill] | tray | clock`. There is
no min/max/close widget anywhere. The v8.7 design lock (
`memory/project_v8_7_window_buttons.md`) specifies far-right
min/max/close buttons in i3. With the v8.8 transition to sway-only,
the same functional contract should apply via
`wlr-foreign-toplevel-management`. Neither the widget nor the
subscription that would feed it exists in the runtime.

---

## Tier 2 — Panel modules shipped as helpers only

13 modules are declared `pub mod` in `crates/mde-panel/src/lib.rs`
but never used elsewhere in the workspace. Counted with:

```
grep -l "${mod}::" *.rs | grep -v "^${mod}.rs$"
```

| Module | Phase | LOC | Tests | Wired? | Worklist note (fine print) |
|---|---|---|---|---|---|
| `admin_menu` | E.13 | 219 | 8 | no | no integration mentioned |
| `clipboard` | E.5 | 158 | 8 | no | "retires once mded's clipboard worker flips to wl-paste subscription" |
| `dock_dnd` | E.9 | 228 | 12 | no | "widget integration lands when the dock applet adds drag recognition" |
| `expose` | E.4.4 | 191 | 10 | no | "Iced fullscreen overlay UI...lands alongside Phase E.3" |
| `hero` | E.4.2 | 204 | 12 | no | "subscription that calls set_focused() lands when Phase E.3 wires foreign-toplevel events" |
| `icon_mapper` | E.19 | 260 | 10 | no | "Iced popover itself lands when the dock applet gets a right-click handler" |
| `layer_shell` | E.2 | 174 | 7 | n/a | superseded by iced_layershell at v3.0.2 — module is now moot |
| `root_menu` | E.14 | 195 | 9 | no | no integration mentioned (wallpaper is swaybg's surface, may not be wireable here) |
| `sliders` | E.6.1+6.2 | 258 | 15 | no | "drawer (E.8) and start menu consume these helpers when their quick-action slider widgets render" |
| `toasts` | E.20 | 221 | 10 | no | no integration mentioned |
| `toplevels` | E.3 | 273 | 11 | no | "actual SCTK subscription that emits these events into an Iced channel lands alongside E.2's surface integration" |
| `watermark` | E.18 | 332 | 13 | no | "Iced widget itself renders into a separate Layer::Background surface as part of Phase E.2 layer-shell wiring" |
| `weather` | E.17fu | 344 | 14 | no | no integration mentioned |

**Total dead code:** ~3,057 LOC, 139 tests, 12 production-relevant
modules. All compile and pass tests. None of their public functions
are called from the panel binary's runtime path.

`layer_shell` is the only module that's legitimately retired —
`iced_layershell 0.13.7` at v3.0.2 took over the surface
integration, so the module's pure-fn helpers became unused at the
moment they would have been needed.

---

## Tier 3 — Daemon workers never spawned

`crates/mackesd/src/bin/mackesd.rs::run_serve` (lines 1283-1340)
spawns exactly one worker: the legacy reconcile worker on a
`std::thread`. The comment at line 1308 says *"Future Phase B
workers slot in alongside it via the async supervisor"* — but the
supervisor is constructed (in `workers/mod.rs`'s `Supervisor`
type), never instantiated in `run_serve`.

Workers that implement the `Worker` trait but are not spawned:

| Worker | Phase | LOC | Worker trait impl |
|---|---|---|---|
| `clipboard` | B.1 | 140 | yes |
| `mdns` | B.2 | 139 | yes |
| `fs_sync` | B.3 | 176 | yes |
| `heartbeat` | B.8 | 109 | yes |
| `mesh_router` | (KDC2-3) | 348 | yes |
| `notification_relay` | B.9 | 369 | yes |

Workers that are misclassified — module exists as `workers/<name>.rs`
but is actually a helper library (no `Worker` impl, no `run()`):

`nats` (B.11), `perf` (B.11), `thumbnailer` (B.11), `derp`,
`remmina_sync`, `media_sync`, `ansible_pull`, `wol`,
`lan_discovery`, `kdc_host`, `metrics_flush`, `subprocess_tick`.

The helper classification may be correct for some (e.g. `nats` and
`perf` are read-only sysfs probes) but `kdc_host` and
`mesh_router` are listed as workers in the worklist that need to
spawn — verify on a per-worker basis before assuming "no impl =
intentional helper."

User-visible consequence: anything that depends on these workers
running silently fails to function. Notifications can't relay
across the mesh, clipboard doesn't sync, mDNS peer discovery
doesn't run.

---

## Tier 5 — Other-crate dead modules (audit-2 sweep, same day)

Initial Phase 0.1 grep had a false-negative bug: it matched any
same-named module across the workspace, so e.g. mde-panel's
`admin_menu` looked "wired" because legacy `mackes-panel` has its
own `admin_menu` module. Corrected scope (refs within the declaring
crate only) re-ran across all `crates/*/src/{lib,main}.rs` roots
and surfaced 10 additional dead modules + 1 pure-scaffold
directory:

| Crate :: Module | Phase | LOC | Wired? | Worklist [✓]? | Notes |
|---|---|---|---|---|---|
| `mackesd::deploy/mod.rs` | 12.1.2 | 16 (all doc) | n/a | none | 658-byte pure-doc scaffold reserving directory layout — exactly the §0.12 pattern. **Deleted** 2026-05-22; comes back with Phase G submodules in one commit. |
| `mackesd::logging` | 12.1.4 | 64 | no | [✓] line 3529 | `LogContext` helper; daemon binary never imports it |
| `mackesd::stun` | 12.17 | ~80 | no | [✓] line 4042 | RFC 5389/8489 STUN client; transport/handshake never invokes it |
| `mackesd::https_fallback` | 12.18 | ~100 | no | [✓] line 4057 | Fallback policy layer (3-failed-cycle activation); transport supervisor never consults it |
| `mde-files::search` | 1.8 | ~60 | no | [✓] line 4463 | Pure-fn filter; Iced view never switches to results layout |
| `mde-files::grid` | 1.9 | ~80 | no | [✓] line 4475 | Tile-layout math; Iced widget tree never consumes it |
| `mde-files::dbus_backend` | 2.3 | ~150 | partial | [✓] line 4505 | Parsers + structs ship; `impl Backend for DBusBackend` self-documented as deferred to Phase G (never closed) |
| `mde-files::a11y_labels` | 5.3 | ~50 | no | [✓] line 4652 | Label table; Iced view never calls `Element::accessibility_label` |
| `mde-kdc::tls` | KDC2-2.8 | ~70 | no | [✓] line 5659 | TLS fingerprint-pinning helper; KDC host transport bypasses it |
| `mde-kdc::dbus` | KDC2-3.3 | ~50 | bus only | [✓] line 5777 | Bus acquisition only; concrete methods explicitly deferred to 3.4/3.5/3.6/3.9 |

**Aggregate (audit + audit-2):** 23 dead-at-runtime modules across
4 crates + 1 pure scaffold deleted. All re-cued to `[>]` in
`docs/PROJECT_WORKLIST.md`. v3.0.3 integration pass now carries
17 panel/popover/daemon tasks (audit-1) + 9 mackesd/mde-files/mde-kdc
tasks (audit-2) + the `deploy/` scaffold deletion.

### Phase 0.1 grep bug fix

Old (broken) detection looked workspace-wide, so a module dead in
crate X looked live if any other crate had a same-named module:

```bash
# WRONG — false negatives
refs=$(grep -rln "${mod}::" --include='*.rs' crates/ | grep -v "^${modfile}$" | wc -l)
```

Corrected — refs scoped to the declaring crate only:

```bash
# RIGHT — same-named modules in other crates don't pollute
refs=$(grep -rln "${mod}::\|crate::${mod}\b\|self::${mod}\b" \
  --include='*.rs' "crates/$crate" 2>/dev/null \
  | grep -v "^${targetfile}$" | wc -l)
```

This fix is now in `.claude/skills/iteration/SKILL.md` Phase 0.1.

## Tier 4 — Popover crate

| Popover | Status | Notes |
|---|---|---|
| `start-menu` | works | dismiss bug (Tier 1A), dedup bug (Tier 1B) |
| `audio` | works | dismiss bug (Tier 1A) |
| `clock` | works | dismiss bug (Tier 1A) |
| `notifications` | works | dismiss bug (Tier 1A) |
| `network` | explicit stub | worklist matches reality — v3.1 scope |

---

## Dependency-ordered integration plan (v3.0.3)

The user-selected sweep order:

1. **Popover dismiss + dedup + zombie reaping** (Tier 1A + 1B + 1C)
   — single bundle. Touches `lib.rs::spawn_popover` and all four
   popover modules. Highest UX impact. Independent of every other
   item.
2. **Toplevels subscription** (Tier 2 `toplevels`) — wlr-foreign-
   toplevel-management subscription emitting `ToplevelEvent` into
   the panel's `update()`. Unblocks 3, 4, and the v8.7 window
   buttons.
3. **Hero widget + window-management buttons** (Tier 1E + Tier 2
   `hero`) — slot the focused-window display into `top_bar::view`
   and add the min/max/close cluster at far right (or far left of
   the clock, per design).
4. **Watermark surface + toasts render layer** (Tier 2 `watermark` +
   `toasts`) — secondary layer-shell surfaces or in-panel overlays
   for the dnf-update watermark and transient toast stack.
5. **Admin menu wiring** (Tier 1D + Tier 2 `admin_menu`) — custom
   right-clickable widget for the M button, dispatching to
   `admin_menu::spawn_action`.
6. **Icon mapper popover** (Tier 2 `icon_mapper`) — dock applet
   right-click handler (depends on the dock applet's own UI
   refresh; coordinate with v3.1 dock work).
7. **Sliders into drawer + clipboard wiring + dock_dnd in dock
   applet + expose F3 overlay + weather popover + root menu** —
   independent bundles, each closes one Tier 2 row.

Daemon worker registration (Tier 3) is a separate workstream — it
needs per-worker review (some may be intentionally dormant pending
upstream readiness). Not in the v3.0.3 panel pass.

---

## Why this happened (process note)

The Phase E.x worklist entries used "shipped 2026-05-21" to mean
"helper module + tests committed." The actual integration work
(subscription emit-sites, widget placement in `view()`, message
handlers in `update()`) was systematically deferred with phrases
like "lands alongside Phase E.2." When Phase E.2 itself shipped at
v3.0.2, no follow-up swept back through the deferred integrations.

To avoid recurrence, future Phase-style entries should either:

- be split into `Phase X-helpers (shipped)` + `Phase X-wiring (open)`
  at write-time, OR
- not be marked `[✓]` until the runtime can call the new code via
  user input — i.e. the user-visible feature works end-to-end.

The Definition of Done in `.claude/CLAUDE.md` §0.8 already requires
"all module imports clean" — extending it to "module is reachable
from the binary's runtime path" would catch this class of gap.
