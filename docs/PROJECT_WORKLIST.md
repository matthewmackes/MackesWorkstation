# Project Worklist — Mackes XFCE Workstation 1.0.0

**Canonical worklist for the v3.0.0 / 1.0.0 rewrite.**
**Status legend:** `[ ] Open` · `[>] In Progress` · `[✓] Done` · `[!] Blocked` · `[~] Deferred`
**Design source:** `docs/design/v3.0.0-mackes-xfce-workstation.md` (50-question lock, 2026-05-18)

Estimated total effort: ~5–6 months. M1 ships the **full v1 vision** in a
single coherent release (Q47 lock — no partial cuts).

**Performance budget (Q41 revised 2026-05-18):**
- Cold start: **< 200 ms** (Xvfb-measured from systemd `Started mackes-panel.service` to first paint)
- Idle CPU: **< 1%** (averaged over 60 s with no input)
- RSS: **≤ 150 MB** (resident memory after 5 minutes runtime, drawer + dock fully populated)

These are CI gates, not aspirations. A PR that regresses any of them is
blocked until fixed. See Phase 9.4 below.

---

## Phase 0 — Foundations (1–2 weeks)

- [✓] **0.1 Rust toolchain in repo** — `rust-toolchain.toml` pins Fedora 44's Rust 1.95.0. Workspace `Cargo.toml` at repo root. `make rust` / `make rust-check` wired. CI's `rust (Fedora 44)` job green on `cargo fmt --check` + `clippy -D warnings` + `cargo check` + `cargo test`. Landed in `440c190`.
- [✓] **0.2 Cargo workspace skeleton** — three crates now: `mackes-panel` (skeleton bin), `mackes-config` (4 tests, TOML schema for panel.toml top_bar/dock/mesh sections), `mackes-mesh-types` (3 tests, MeshResource Peer/MountedShare/Service). 7 unit tests, fmt/clippy/check clean. Landed in `570146e`.
- [✓] **0.3 Build/packaging plumbing** — `cargo build --release --workspace` runs in `%build`, `target/release/mackes-panel` installs to `/usr/bin/mackes-panel`, MANIFEST.in ships Cargo.{toml,lock} + crates/ in the sdist. Verified: RPM build succeeds, ELF binary present. Landed in `e9cfc35`. `Obsoletes: mackes-shell < 3` deferred to 10.1 (the actual rename commit).
- [✓] **0.4 First boot: empty top bar** — 20 px GTK3 ApplicationWindow with Dock type-hint, PatternFly-dark, clean SIGTERM/SIGINT. Landed in `cc5a122`.
- [✓] **0.5 First boot: empty bottom dock** — second Dock-hint window, 80 px, bottom of primary monitor. `FallbackGeometry` factored out, `apply_placeholder_style` reused across surfaces. Landed in `196cbb6`.
- [✓] **0.6 Wallpaper rendering** — Desktop-hint window with scaled wallpaper from state.json (or branding/ fallback). Pure GTK stacking. Release 558 KB / 433 KB stripped. Landed in `9c51124`. **Phase 0 complete.**
- [✓] **0.7 Repair the latent pytest suite uncovered by ci.yml fix** — ci.yml YAML-bug fixed in `d379914`. Then `f96044e` purged stale `mackes.mesh_*` from sys.modules in `conftest.isolated_xdg`, fixed `test_list_presets_ships_five`, and added cairo/textual to CI deps. `8eb3eb7` added a Typelib/namespace filter to `test_every_non_gui_module_imports`. `32cf2f1` dropped the redundant import-smoke shell step. CI run `26052513245` green: ✓ python (F43) · ✓ python (F44) · ✓ rust (F44). First green CI since 0.2.0.

## Phase 1 — Visual chrome (3–4 weeks)

- [✓] **1.1 PatternFly tokens loaded** — panel reads `/usr/share/mackes-shell/data/css/tokens.css` + `mackes.css` at startup via screen-wide `Gtk.CssProvider`, plus an inline backup so the chrome renders on uninstalled trees. Per-window `#mackes-*` IDs reserved.
- [✓] **1.2 Top bar layout slots** — left/center/right slots via `gtk::Box::set_center_widget`. 1 px hairline border at bottom via inline CSS. Slots named `#mackes-top-{left,center,right}` ready for Phase 1.5–1.7 widgets.
- [✓] **1.3 Dock layout slots** — single centered slot `#mackes-dock-strip`. Hairline border at top. Phase 5.1+ populates it.
- [✓] **1.4 Mackes-Carbon icon loader** — `icons::load(name, size_px)` resolves freedesktop names + `-symbolic` variants under `/usr/share/icons/Mackes-Carbon/scalable/{actions,status,devices,places,emblems,categories,mimetypes,apps}/`. Thread-local `HashMap<(name,size), Pixbuf>` cache. 3 unit tests for the name-candidate logic.
- [✓] **1.5 Clock widget (center)** — `top_bar::clock()` returns a `gtk::Label` showing `HH:MM`. First tick is scheduled at the next-minute boundary (wall-clock synced), then every 60 s. Calendar dropdown is deferred to a follow-up sub-step.
- [✓] **1.6 Status cluster (right)** — 6-item horizontal box with Carbon-loaded glyphs for mesh/clipboard/volume/battery/notifications/user. Per-item click handlers stubbed; Phase 4.2 replaces them with the Drawer-open signal.
- [✓] **1.7 Apple-menu button (left)** — Mackes-mark button (`applications-system-symbolic` placeholder) with stub click handler. Phase 3 wires the dropdown.
- [✓] **1.8 Dock module dispatch** — `DockModule` trait with `id / icon_name / tooltip / state / on_click`. `DockState` enum: Idle / Running / Focused / Urgent{unread}. `render_module()` builds the widget tree.
- [✓] **1.9 State indicators on dock icons** — `state_dot()` (1 px under-icon with class muted/accent/alert) + `unread_badge()` (top-right corner number, 99+ cap) per Q16. 2 unit tests cover the state→class + unread-skip-zero mapping.

## Phase 2 — Configuration & mesh sync (2–3 weeks)

- [✓] **2.1 panel.toml schema** — shipped early in Phase 0.2 (commit `570146e`). `crates/mackes-config/` holds `PanelConfig` / `TopBarConfig` / `DockConfig` / `MeshConfig` / `DockItem` with serde + 4 unit tests including unknown-section tolerance.
- [✓] **2.2 Default panel.toml** — `config_store::load_or_default()` reads `XDG_CONFIG_HOME/mackes-panel/panel.toml` (with `$HOME/.config` fallback) and writes the default via `mackes_config::default_config()` on first launch. Malformed TOML is logged + falls back to defaults so the panel always starts. 2 new unit tests (default round-trip via TOML, six-item status cluster).
- [✓] **2.3 inotify-driven hot reload** — `config_store::watch(callback)` attaches a `gio::FileMonitor` (inotify-backed on Linux) and re-parses on `ChangesDoneHint`. Atomic-save patterns (delete + create + done-hint) reload once, not three times. Diff-and-apply against the UI lands in Phase 2.5 once the live `PanelConfig` is held in a stable place.
- [✓] **2.4 QNM-Shared mirror** — `mesh_sync::mirror(src)` copies `panel.toml` to `~/.qnm-sync/mackes-panel/panel.toml`. Content-aware: skips the write when bytes already match, so QNM-Shared inotify doesn't echo. Callers wire it after every save (callable from Phase 2.5+ when the watcher triggers a save).
- [✓] **2.5 Drift detection** — `mesh_sync::compute_drift()` SHA-256-hashes the local mirror and each `peers/<peer>/panel.toml` under the same root. Returns a `DriftSummary` with per-peer `InSync` / `Drifted` / `Missing` / `Unreadable`. Empty mesh → vacuously in-sync. 3 unit tests.
- [✓] **2.6 Look & Feel → Panel → Sync status row** — new `mackes/workbench/look_and_feel/panel.py` ships `PanelLookFeelPanel` with a single-line drift summary ("In sync with N peers" / "Drifted from N peers · M in sync" / "Not replicated"). Hashing mirrors `mackes_panel::mesh_sync::compute_drift` (same SHA-256 over `~/.qnm-sync/mackes-panel/peers/<peer>/panel.toml`). Sidebar registration + click-through inspector are a small follow-up; the panel module compiles and the data is correct.

## Phase 3 — Apple menu + app discovery (2 weeks)

- [✓] **3.1 .desktop scanner** — `desktop_files::scan()` walks `/usr/share/applications/`, `/usr/local/share/applications/`, `~/.local/share/applications/` and parses each entry. Honors `NoDisplay`/`Hidden`. User-side shadows system-side by basename. `parse_text()` is public so 8 unit tests exercise the parser without filesystem hits.
- [✓] **3.2 Applications submenu builder** — `apple_menu::build(entries)` groups DesktopEntry items into 8 canonical buckets (Internet / Multimedia / Graphics / Office / Development / Games / System / Utilities) plus an `Other` catch-all. Each bucket carries its Mackes-Carbon icon name + entries sorted case-insensitively by Name. First-match wins on Categories with multiple tags. 5 unit tests (bucketing, sort, dedup, Other fallback, empty input).
- [✓] **3.3 Apple-menu chrome** — clicking the Mackes button pops a real `gtk::Menu` with the Q24 ordering: About / Settings / Software Update / Applications → / Force Quit / Sleep / Restart / Shut Down / Lock / Sign Out, all wired to `Command::spawn`. Separators in the right places. Submenu glyphs deferred to a polish pass.
- [✓] **3.4 Recent Items source** — `recents::load(10)` parses `recently-used.xbel` from `$XDG_DATA_HOME` (or `$HOME/.local/share`), sorts by modified timestamp desc, returns top 10 as `RecentItem { uri, label, modified }`. Apple menu inserts a `Recent Items →` submenu between Software Update and Applications; empty placeholder when no recents exist. 4 unit tests cover the parser.
- [✓] **3.5 System action wiring** — `loginctl suspend|reboot|poweroff|lock-session` directly; Sign Out via `xfce4-session-logout --logout`. PolicyKit prompts for reboot/poweroff handled by the system policy (no AdminSession indirection — we're a real binary now, not a Python subprocess).
- [ ] **3.6 Super+Space global hotkey** — XGrabKey on Super+Space → toggles the Apple menu.

## Phase 4 — Notification Drawer integration (2 weeks)

- [~] **4.1 Drawer IPC** — *Superseded by 4.3 (port to Rust).* A
  D-Bus interface is no longer needed once the drawer is in-process
  with the panel.
- [✓] **4.2 Status-cluster click → Drawer open** — each of the 6 status buttons shells out to `mackes --drawer --drawer-focus <slug>` so the existing Python drawer opens with the right section pre-selected. D-Bus interface (4.1) lives in a follow-up if startup latency becomes a concern.
- [>] **4.3 Drawer port to mackes-panel module** — **Locked 2026-05-18
  via single-question survey.** Bring `mackes/drawer.py` (1142 lines
  of Python/GTK3) into `crates/mackes-panel/src/drawer/` as Rust
  modules using gtk-rs. **Substeps:**
  - [ ] **4.3.1 Crate scaffolding** — new `drawer/` module tree:
    `mod.rs` (public `toggle()` API + `DrawerWindow` widget),
    `state.rs` (`LiveState` mirror of the Python dataclass),
    `sections/` (one file per drawer section: header, quick_toggles,
    sliders, mesh, fleet, services, notifications, battery, hardware).
  - [ ] **4.3.2 Live-data probes** — port `_audio_volume`,
    `_brightness`, `_bluetooth`, `_dnd_state`, `_caffeine`,
    `_read_battery`, `_read_hardware`, `_read_mesh` (tailscale --json),
    `_remote_sessions`, `_playing_count` (MPRIS DBus). Each as a
    `pub fn probe_X() -> X` returning a value type; subprocess via
    `std::process::Command` with the same 2 s timeout and
    silent-degradation semantics as the Python version.
  - [ ] **4.3.3 Quick toggles** — Mesh / BT / DND / Caffeine. Each is
    a `gtk::ToggleButton` with `connect_toggled` calling the matching
    `set_X()` mutator (tailscale up/down, bluetoothctl power on/off,
    xfconf-query notifyd, xfce4-power-manager presentation-mode).
  - [ ] **4.3.4 Sliders** — Volume + Brightness as `gtk::Scale` widgets,
    `connect_value_changed` debounced to 100 ms before invoking
    `pactl set-sink-volume @DEFAULT_SINK@ N%` and
    `/usr/local/bin/mackes-brightness set N`.
  - [ ] **4.3.5 Mesh + Fleet sections** — reuse `mackes-mesh-types`
    crate already in the workspace. Render peer list + a 2×2
    fleet grid. Click a peer → spawn `xdg-open ~/QNM-Shared/<peer>/`
    (same as the dock's `MeshModule`).
  - [ ] **4.3.6 Notifications list** — port the dismiss + clear-all
    flow. Source = `~/.cache/mackes/notifications.json` (existing
    file format). `dismiss(id)` rewrites the JSON; `clear_all()`
    truncates to `[]`.
  - [ ] **4.3.7 Header + battery + hardware** — small read-only
    surfaces. Header reads `state.json` for the active preset name;
    battery from `/sys/class/power_supply/BAT*`; hardware from
    `/proc/{stat,meminfo,loadavg}`.
  - [ ] **4.3.8 Wire `mackes-panel --drawer`** — add a `--drawer`
    flag to the panel binary that opens the new Rust drawer window
    instead of launching the panel surfaces. `--drawer-focus <slug>`
    pre-scrolls to the matching section.
  - [ ] **4.3.9 Swap the apple-menu + status-cluster + plugin entry
    points** — replace the `mackes --drawer` subprocess in
    `top_bar.rs` status-cluster popovers with an in-process
    `DrawerWindow::toggle()` call. Update the C panel plugin
    (`data/panel-plugins/mackes-drawer/`) to spawn
    `mackes-panel --drawer` instead of `mackes --drawer`.
  - [ ] **4.3.10 Retire `mackes/drawer.py`** — delete the Python
    drawer once the Rust drawer ships and the C plugin points at
    the panel binary. RPM `Obsoletes:` not needed (same package).
    The `mackes --drawer` Python flag stays as a one-line shim
    that execs `mackes-panel --drawer` for legacy callers.
  - **Why a port (not IPC):** the Python drawer ships ~7 s of probe
    latency on cold open even with the threaded refresh added in
    1.0.7 (sidebar pattern). The Rust drawer can probe lazily
    per-section and amortize across the panel's already-running
    process, so a click → render path is ~50 ms instead of seconds.
    Also simplifies packaging: one binary owns every panel surface.
  - **Risk:** the C panel plugin (`data/panel-plugins/mackes-drawer/`)
    is in the legacy 10.6 cleanup path. Coordinate with 10.6 so we
    don't ship a plugin pointing at a binary that no longer exists.
- [ ] **4.4 Quick-toggle behaviors** — covered by 4.3.3; this row
  stays open as a polish gate (tooltips, accelerators, error toasts).

## Phase 5 — Dock behaviors (3–4 weeks)

- [✓] **5.1 Pinned-app launchers** — `AppModule` (concrete `DockModule`) wraps a `DesktopEntry`. `build_dock_strip(cfg)` walks `cfg.dock.items` of kind `App`, looks up each in a one-shot `desktop_files::scan()` index, renders via `dock::render_module`, and binds `button-release` to `launch_exec`. Mesh items skipped with a warning until Phase 5.4 lands `MeshModule`. 5 unit tests cover the `AppModule` accessors.
- [>] **5.2 Running-app detection** — `windows::list_open_windows()` shells out to `wmctrl -lp` and parses `(window_id, pid, title)` tuples (libwnck has no maintained safe Rust binding — only raw `wnck-sys` FFI). `app_is_running(name, exec, windows)` matches by title contains-Name, title contains-Exec-basename, or `/proc/<pid>/comm` contains-basename. 6 unit tests cover the parser + matcher. RPM `Requires: wmctrl`. AppModule's state mutation lands when the dock holds a long-lived handle (a small refactor of build_dock_strip).
- [✓] **5.3 Window switching** — pinned-app click now scans `windows::list_open_windows()` first: a matching window → `windows::toggle_window(id)` (`wmctrl -i -a` to raise; second click → `xdotool windowminimize`). No match → `launch_exec` as before. Mackes installs already ship `wmctrl`; `xdotool` falls back to plain re-activate if missing.
- [✓] **5.4 Mesh-resource enumeration** — `mesh_module::parse_id` is the inverse of `MeshResource::id()` (peer:NAME / share:PEER:BUCKET / svc:PEER:SLUG → typed `MeshResource`). `MeshModule` implements `DockModule` and renders via the shared path. Peer click → `xdg-open ~/QNM-Shared/<peer>/`; share click → its bucket; service click → the service URL. 6 unit tests. Periodic re-enumeration against Headscale + service catalog lives in Phase 5.5.
- [✓] **5.5 Mesh-resource interleaving** — `build_dock_strip` walks `cfg.dock.items` in render order, instantiating `AppModule` or `MeshModule` per entry. No segmentation, no separator — matches Q10. Live online/offline state for peers lands when Headscale is wired (Phase 5.5b, deferred).
- [✓] **5.6 Peer-click action popover** — `mesh_module::build_peer_popover` returns a `gtk::Popover` with six buttons: Files (Thunar at `~/QNM-Shared/<peer>/`), SSH (`xfce4-terminal -e ssh <peer>.mesh`), RDP (`remmina -c rdp://<peer>.mesh`), VNC (`remmina -c vnc://<peer>.mesh`), Services (`mackes --services --peer <peer>`), Send file (zenity file-picker → cp into `~/QNM-Shared/<peer>/`). Clicking a peer dock item now opens the popover; shares and services keep the simple xdg-open click. Phase 5.6 acceptance met.
- [>] **5.7 Drag-to-pin / drag-to-reorder** — data layer landed: `mackes_config::pin_app(cfg, desktop)` (idempotent by id) + `mackes_config::reorder_dock(cfg, from, to)` (clamped, no-op on equal). 4 unit tests cover append, idempotency, in-bounds move, out-of-range clamp. GTK drag-source/drop-target wiring on the dock widgets is the visual follow-up.

## Phase 6 — Window management (2 weeks)

- [ ] **6.1 Super+Tab app switcher** — modal overlay strip with live window thumbnails. Hold Super, tap Tab to cycle. Release Super to switch.
- [ ] **6.2 Exposé grid (F3 / hot-corner)** — fullscreen overlay that arranges every visible window in a non-overlapping tile grid. Click to focus.
- [✓] **6.3 Workspaces disabled** — `workspace_count: 1` baked into every preset (hashbang/daylight/mackes). `mackes.presets.apply_system` writes `xfwm4/general/workspace_count = 1` via xfconf at apply-time. Single desktop per Q29; app-switching via Cmd+Tab (Phase 6.1).
- [ ] **6.4 Other 6 default hotkeys** — Super+Q quit · Super+W close · Super+L lock · Super+V clipboard · Super+E Thunar · F3 Exposé. All via XGrabKey + backup-on-conflict.

## Phase 7 — Iconography + theming (1–2 weeks)

- [✓] **7.1 App → Carbon icon mapping table** — `icons::resolve()` maps common `.desktop Icon=` values to Mackes-Carbon symbolic glyphs (firefox→earth, thunar→folder--open, vlc→play--filled-alt, etc.). `AppModule::icon_name()` routes through it so well-known apps wear Carbon by default. ~45 entries, case-insensitive, strips paths + extensions. 3 unit tests.
- [✓] **7.2 Inline Nerd Font glyphs** — `data/css/tokens.css` gains `.mackes-nerd / .nerd-glyph / .mackes-apple-menu-status / .mackes-drawer-mini` selectors with a Nerd Font fallback stack (RedHatMono Nerd Font → JetBrainsMono Nerd Font → Symbols Nerd Font → Red Hat Mono). Inline-text places that need a glyph at sub-16 px size apply one of these classes. CSS lints clean.
- [✓] **7.3 Force monochrome on all dock icons** — already shipping. `AppModule::icon_name()` routes every `.desktop Icon=` through `icons::resolve()` (Phase 7.1), and unmapped names land in `icons::load()` which only resolves under `/usr/share/icons/Mackes-Carbon/` (Phase 1.4). A `.desktop` shipping a colorful PNG never reaches the dock — it's either mapped to a Carbon glyph or falls back to `applications-other-symbolic`.

## Phase 8 — Continuity surfaces (1–2 weeks)

- [✓] **8.1 LightDM greeter look** — `mackes.lightdm.configure_greeter` writes `panel-position = top` (was `bottom`), `clock-format = %H:%M` (was full date), and a slimmed `indicators` line that mirrors mackes-panel's right-side cluster (clock + session + a11y + power). The greeter now renders a strip at the top of the screen matching the panel's 20 px top bar — boot → greeter → desktop have continuous visual language per Q36.
- [✓] **8.2 Plymouth rebuild** — `data/plymouth/mackes/mackes.script` rewritten: black Carbon Gray 100 background, centered Mackes logo (~22% screen width), 20 px full-width progress strip pinned to the bottom edge with a 1 px hairline above (matches mackes-panel's dock position + dock border). Accent orange fills the strip as boot progresses. Status-message line shifted to sit above the strip.
- [✓] **8.3 xfdesktop removal** — RPM ships `/etc/xdg/autostart/mackes-panel.desktop` (so every XFCE session brings up the Rust panel) and `/etc/xdg/autostart/xfdesktop.desktop` overrides upstream's autostart with `Hidden=true` + `X-XFCE-Autostart-enabled=false`. On install: log out / log in → mackes-panel owns wallpaper + dock + top bar; xfdesktop never starts. Verified in fresh `make rpm` build — both entries present at the right paths.
- [ ] **8.4 Root right-click menu** — XGrabButton on the root window, right-click opens a Mackes-themed menu (Change wallpaper / Open mesh share / Send file to peer / Display settings).

### 8.5 — First-boot visual polish (shipped in 1.0.6, 2026-05-18)

- [✓] **8.5.1 Recolor Mackes-Carbon symbolic icons at load** — shipped 1.0.6. `icons::load()` substitutes `currentColor` → `#f0f0f0` before rasterizing; `data/css/mackes.css` forces panel chrome to Carbon text-primary.
- [✓] **8.5.2 Bottom dock auto-sizes / hides when empty** — shipped 1.0.6. Empty dock never shows the window; populated dock sizes to `DOCK_ICON_PX + 8 px`. 1.0.7 expands this with a Plank-parity tasklist segment (see 1.0.7 below).
- [✓] **8.5.3 12-hour clock + weather popover** — shipped 1.0.6. `%l:%M %p` clock + frameless button + `gtk::Popover` rendering current temperature from `api.met.no/weatherapi/locationforecast/2.0/complete`. `crates/mackes-panel/src/weather.rs`; HTTP via system `curl`; 3 unit tests on the JSON parser.
- [✓] **8.5.4 Status-cluster popovers for review** — shipped 1.0.6. Each of the 6 right-side buttons opens an in-process `gtk::Popover` with title + summary + "Open in Drawer →" delegate.
- [✓] **8.5.5 `_NET_WM_STRUT_PARTIAL` on panel + dock** — shipped 1.0.6. `crates/mackes-panel/src/strut.rs` looks up XID via `xdotool search --name` and publishes both `_NET_WM_STRUT_PARTIAL` (12-cardinal) and `_NET_WM_STRUT` (legacy) via `xprop -id`. 1.0.7 adds a 500 ms allocated-height poll because GTK3's `size-allocate` doesn't fire reliably on Dock-hint toplevels.

### 8.6 — 1.0.7 panel + drawer work (in flight, 2026-05-18)

Bundle currently in the working tree (18 modified files + 4 untracked,
~2,060 line insertions). Lands as 1.0.7 once the user authorizes the
version bump + tag.

- [>] **8.6.1 Plank-parity dock — pinned launchers + tasklist** — `refresh_dock()` rebuilds both segments every 2 s from a `DockSnapshot` (open windows + `WM_CLASS` + active window id). Pinned launchers group every window sharing their `StartupWMClass`; un-grouped windows go to the right-side tasklist. Multi-window launchers show a 1/2/3+ tick indicator under the icon. Left-click activates (or launches if no window); right-click opens a per-launcher context menu (Open New / Bring to Front: «title» / Close All).
- [>] **8.6.2 Tasklist right-click menu** — Bring to Front / Close / Maximize / Restore / Minimize / Pin to Dock. Pin path reads `WM_CLASS`, finds the `.desktop` whose `StartupWMClass` matches, appends to `panel.toml:[dock.items]`, saves through `mesh_sync::mirror`.
- [>] **8.6.3 i3 WM live-switch** — `bin/mackes-wm {i3|xfwm4|status}` is a 70-line bash script that uses `i3 --replace` / `xfwm4 --replace` for handover. Auto-stops `mackes-maximizer.service` under i3. Workbench → System → Window Manager exposes a toggle row + (for i3) an 8-cell layout-preset grid (Maximized / Side by Side / Split-in-4 / Master+Stack / Tabbed / Stacking / Focus / Floating) driven by `i3-msg`. RPM gains `Requires: i3 i3status dmenu`; default `data/i3/config` installs to `/usr/share/mackes-shell/i3/config` and is seeded into `~/.config/i3/config` on first switch.
- [>] **8.6.4 About Mackes window** — `mackes/about.py` + `data/ABOUT.txt` (credits, licenses, upstream attributions). Wired via `mackes --about` and the apple-menu's "About Mackes" item.
- [>] **8.6.5 Drawer live-data wiring pass** — replaced every mocked data source in `mackes/drawer.py` with live probes (`pactl`, `bluetoothctl`, `xfconf-query notifyd`, `xfce4-power-manager presentation-mode`, `tailscale status --json`, `who -u`, MPRIS DBus, `/sys/class/power_supply`, `/proc`). Removed sections that depended on subsystems not yet implemented (Drift / Shared storage / Daemons grid / Footer-power) rather than ship placeholder data.
- [>] **8.6.6 Drawer process hold/release fix** — `app.hold()` before `toggle()` so the GApplication survives past `do_activate`; `release()` on drawer hide so a second invocation can quit cleanly. Was a hot bug: drawer closed on first click because the GApp exited.
- [>] **8.6.7 Non-blocking sidebar status refresh** — first `_refresh_status_bar` call now runs on a background thread (saved ~7 s of `__init__` blocking — headscale + fleet + drift probes were synchronous).
- [>] **8.6.8 `python3 -P` mackes wrapper** — RPM-installed `/usr/bin/mackes` now invokes `python3 -P -m mackes` so the cwd's `mackes/` checkout never shadows the installed `mackes/` package. Cold start from `~/Desktop/files`: 17 s → 1.5 s.
- [>] **8.6.9 Top-bar + dock height-tracking poll** — initial strut hint is set with the requested size; a 500 ms timer notices when the realized window grows past the request and republishes `_NET_WM_STRUT_PARTIAL` to match. Fixes the 4-px occlusion delta on first paint.
- [>] **8.6.10 Status cluster — icon + numeric (read-only, 2 s)** — **Locked 2026-05-18 via 5-question survey.** Replaces the popover-only stubs with six live read-only indicators: Mesh=online peers · Clipboard=item count · Volume=% · Battery=% · Notifications=unread · User=sessions. Inline icon-left/number-right layout. 2 s poll (matches dock). Click → drawer focused. Probe failure → em-dash + dimmed icon + tooltip with reason. New module `crates/mackes-panel/src/status_cluster.rs` (probe_mesh/clipboard/volume/battery/notifications/user). CSS in `PLACEHOLDER_CSS` + production tokens. 2 unit tests cover `cache_dir` XDG resolution + battery probe sanity.

## Phase 9 — Test pyramid (continuous; ratchet to green before M1)

- [ ] **9.1 Unit tests** — every pure-logic module (config parsing, mesh-resource scoring, icon lookup, hotkey parser). Target: 80% line coverage.
- [ ] **9.2 GTK widget tests** — gtk-test harness around dock, status cluster, Apple menu, calendar dropdown. Headless via Xvfb in CI.
- [ ] **9.3 E2E tests** — xdotool-driven smoke: launch panel, click Mackes button, navigate Applications submenu, launch Firefox via dock, verify running indicator appears. Runs nightly.
- [>] **9.4 Performance benchmarks** — `install-helpers/bench-panel.sh` launches the panel under a clean Xvfb, samples `/proc/<pid>/{stat,status}` for cold-start / RSS / idle-CPU, gates at the Q41 revised targets and exits 1 on regression. **First measurement run 2026-05-18 vs commit `99e2680`: cold start 5 ms · RSS 85 MB · idle CPU 0.0% — all three gates pass with significant margin.** CI integration (run on every push) lands in a follow-up.

## Phase 10 — Migration + cutover (2 weeks)

- [✓] **10.1 RPM rename** — `Name: mackes-xfce-workstation`, `Provides: mackes-shell = %{version}-%{release}`, `Obsoletes: mackes-shell < 3.0`. Source tarball still ships under the legacy `mackes-shell-%{version}.tar.gz` filename so the build pipeline doesn't need a rename. Verified: `make rpm` produces `mackes-xfce-workstation-1.0.0-0.1.rc1.fc44.x86_64.rpm`; `rpm -q --obsoletes` shows the Obsoletes line. Filesystem paths intentionally unchanged (Q44 brand-only rename).
- [ ] **10.2 First-launch wizard** — detect `~/.config/mackes-shell/` leftovers from 2.x; import preset + active wallpaper + pinned apps into `~/.config/mackes-panel/panel.toml`. Show what's being migrated.
- [✓] **10.3 Brand surfacing** — `data/applications/mackes-shell.desktop:Name` now "Mackes XFCE Workstation" (was "Mackes Shell"). Plymouth Description updated to v1.0.0 wording (Phase 8.2). RPM Summary line updated (Phase 10.1). About dialog and greeter banner will pick up the new label via these same strings. About-dialog text lives in `mackes/workbench/help.py` — already pulls from `__version__`, so the 1.0.0 bump cascades through.
- [✓] **10.4 CHANGELOG 1.0.0 section** — `CHANGELOG.md` carries the full "1.0.0 — Mackes XFCE Workstation (2026-05-18)" entry: what's new (icon theme, panel + dock + wallpaper, config + mesh sync, boot continuity, perf gates, workspaces dropped), post-1.0 roadmap (global hotkeys via x11rb, Cmd+Tab/Exposé overlays, drawer Rust port, GTK widget + E2E test pyramid, first-launch wizard, root right-click menu), migration story.
- [✓] **10.5 Cut release 1.0.0** — RPM `Release: 1` (was `0.1.rc1`). `make rpm` produces `mackes-xfce-workstation-1.0.0-1.fc44.x86_64.rpm`. CHANGELOG + tag below complete the cut. **Shipped 2026-05-18 as `v1.0.0`; patch line continues through `v1.0.6` (1.0.1–1.0.5 were held by legacy 2.x tags on origin so we jumped to 1.0.2 → 1.0.6).**

### 10.6 — Birthright removal sequence (replaces incumbent panel + desktop)

Per Q2 / Q5 / Q29 / Q39 we replace xfce4-panel, xfdesktop, the
Whisker-menu plugin, and the legacy mackes-shell Python entry points
with the unified mackes-panel binary. Order matters — a peer can't
lose its panel before the replacement is running. Each substep is a
new birthright step in `mackes.birthright` (placed after the existing
14 v1.x steps so legacy installs still wash through them cleanly):

- [✓] **10.6.1-4 Panel-swap sequence** — `mackes.birthright.apply_panel_swap` is one idempotent birthright step that: (1) starts `mackes-panel`, (2) quits `xfce4-panel` and writes a Hidden autostart override at `~/.config/autostart/xfce4-panel.desktop`, (3) quits `xfdesktop` (system-side override already shipped in Phase 8.3), (4) rebinds `<Super>l` and `<Super>Space` xfconf shortcuts to `mackes-panel --apple-menu`, backing up any prior values to `~/.config/mackes-panel/keybindings.backup.toml`. Each step is best-effort and aborts the rest on failure rather than half-applying.
- [✓] **10.6.5 Remove xfwm4 workspaces** — already baked into every preset (workspace_count = 1 per Phase 6.3). `mackes.presets.apply_system` writes the xfconf key at apply-time.
- [ ] **10.6.6 Uninstall the now-orphaned packages** (final removal step, only after 10.6.1–10.6.5 succeed). Single dnf call: `dnf remove -y xfce4-panel xfdesktop xfce4-whiskermenu-plugin xfce4-docklike-plugin xfce4-pulseaudio-plugin xfce4-power-manager-plugin`. Side effect: the legacy mackes-launcher / mackes-clipboard / mackes-drawer C plugin RPMs (which BuildRequire xfce4-panel-devel) are obsoleted by the renamed mackes-xfce-workstation RPM in 10.1.
- [✓] **10.6.7 Clean leftover xfce4-panel-profiles snapshots** — `mackes.birthright.apply_panel_archive` copies `~/.config/xfce4/panel/` to `~/.config/mackes-panel/legacy-xfce-panel/` on first run. Idempotent — second runs detect the existing archive and no-op. First-launch wizard summary picks it up via the standard apply-step log surface.
- [ ] **10.6.8 Rollback path** — every removal step writes a `~/.config/mackes-panel/rollback/<step>.json` with the previous state. If `mackes-panel` segfaults or the daemon-stop wedges, `mackes-panel --recover` reads the most-recent rollback and reverses everything in 10.6.1–10.6.6 (re-install xfce4-panel + xfdesktop, restore layout snapshot, re-enable Whisker hotkey). Rollback paths land alongside each forward step, not as one big final task.

---

## Phase 11 — Production polish (/goal directive 2026-05-18)

User issued a `/goal` directive: transform the GTK app into a polished,
production-quality desktop application while preserving its core
purpose. Items below extend Phases 1–10 with the gaps still open per
the goal's eight pillars. Work autonomously; bundle related items.

- [ ] **11.1 AppStream metainfo** — write `data/metainfo/shell.mackes.Panel.metainfo.xml` (and a companion for the Python `mackes` entry point) so GNOME Software / KDE Discover / `appstreamcli validate` know what we are. Includes screenshots, release entries pulled from `CHANGELOG.md`, project_license, content_rating, and a launchable= line per `mackes-xfce-workstation.desktop`. Hook into RPM `%files` + `appstream-util validate` in CI.
- [ ] **11.2 Accessibility pass** — every interactive widget gets `set_tooltip_text` + an AT-SPI name via `set_accessible_name`. Focus order audited per panel (sidebar → content → footer). `Escape` closes every dialog. Status cluster items announce as "Mesh: 3 online peers" rather than just "Mesh". `make a11y` runs `accerciser`/`dogtail` smoke if available.
- [ ] **11.3 Wayland-readiness audit** — `mackes-panel` currently hard-depends on X11 paths: `wmctrl`, `xdotool`, `xprop`, `_NET_WM_STRUT_PARTIAL`, `XGrabKey`. Survey the gap: which Wayland compositors expose equivalents (wlroots layer-shell for the bar/dock, ext-foreign-toplevel for the dock tasklist). Output: a `docs/design/wayland-readiness.md` with per-feature replacement plan and a `[wayland]` section in `panel.toml` for runtime-switching once the work lands. **Scope:** audit only — actual port is multi-phase.
- [✓] **11.4 Keyboard shortcuts catalog + cheat-sheet** — `docs/help/keyboard-shortcuts.md` ships every binding (WM-owned, panel, workbench, drawer) plus a CLI-flag mirror and the `~/.config/mackes-panel/panel.toml:[keybindings]` override syntax. Phase 6.x bindings flagged as "pending" rather than omitted so the doc stays accurate. Accelerator labels next to menu items remains a follow-up under 11.4b.
- [ ] **11.5 Empty + error state pass** — every sidebar panel + drawer section needs (a) an empty state with a CTA, (b) an error state with the actionable next step. Audit pass: `mackes/workbench/**`, `mackes/drawer.py`. No more silent `pass`-on-error; every probe degrades to a labeled empty state.
- [ ] **11.6 README + dev docs refresh** — `README.md` currently assumes legacy 2.x mental model. Rewrite around the 1.0.x workstation framing: `make rpm`, `make rust`, `make test-nodeps`, `python3 -P -m mackes`, the panel binary's CLI flags, the i3 switcher. Add a "Smoke test" section with the exact commands to verify a fresh checkout builds + runs.
- [✓] **11.7 pytest smoke baseline** — `tests/test_panel_instantiation_smoke.py` discovers every `*Panel(Gtk.Box)` subclass under `mackes.workbench.**`, instantiates each headless under Xvfb, asserts the panel produces at least one child widget, and surfaces slow constructors (> 100 ms) as informational test output (tracked under 11.9). 49 panels discovered; 45 pass; 1 daemon-dependent (FirewallPanel) and 4 state-required panels are skipped with explicit reasons. Full pytest run under Xvfb: **118 passed, 5 skipped, 0 failed** in ~100 s.
- [ ] **11.8 GSettings schema (optional)** — survey whether mackes-panel should publish a GSettings schema for the panel.toml fields (`org.mackes.panel.*`). Pros: GNOME Settings + dconf integration. Cons: duplicates the existing TOML, adds a new dep (`gsettings-desktop-schemas`). Decide via a follow-up survey before implementing.
- [>] **11.9 Reliability sweep** — **In progress 2026-05-19.** Canonical helper landed: `mackes.workbench._async.async_probe(probe, on_result, on_error=None)` — runs `probe()` on a daemon thread, marshals result to GTK main thread via `GLib.idle_add`, swallows probe-side AND callback-side exceptions so a buggy panel can't crash GLib's main loop. 6 unit tests in `tests/test_async_probe.py`. **Converted (no longer block main thread):** FirewallPanel (was hanging > 5 s when firewalld down — now < 100 ms with 2 s per-call timeout), DependenciesPanel (rpm -qa probe), MeshVpnPanel (was 15 s — tailscale + headscale probes), MeshSshPanel (was 7 s — headscale_list_peers). **Remaining slow constructors** (surfaced by `tests/test_panel_instantiation_smoke.py`): FleetInventoryPanel (8 s), FleetRunHistoryPanel (7 s), RemoteDesktopPanel (6.5 s), AppsPanel (2.5 s), DebloatPanel (1.6 s), AppearancePanel (500 ms), DateTimePanel (280 ms), DisplaysPanel/DefaultAppsPanel/HealthCheckPanel (~150 ms each). Each gets the same `async_probe` pattern; the helper is generic and the conversion is a 5-line skeleton-then-fill change per panel. Valgrind leak-check pass against the Rust panel still open.

## Phase 12 — Enterprise Mesh Backend & GUI (/goal directive 2026-05-19)

User issued a second `/goal` directive: elevate the Mesh networking
implementation from a loose collection of probes into a **production-
grade enterprise control plane** with a backend that is the
authoritative single source of truth for every Mesh fact (nodes,
identities, peers, routes, policies, telemetry, configuration history),
a versioned + validated + rollback-capable configuration model, and a
GUI whose topology drawings reflect the real operating network rather
than static assumptions.

**Scope clarification — what this *isn't*.** We're not rebuilding
WireGuard or replacing Tailscale + Headscale. Those continue to do
the actual encryption, route exchange, and packet forwarding. What
Phase 12 builds is the **control plane on top of them** — the layer
that owns declarative config, drift detection, audit, policy, and the
GUI surface. Equivalent to what Twingate / Nebula's control plane do
on top of their data plane.

**Existing surface to consolidate.** 18 Python modules under
`mackes/mesh_*.py` + 9 workbench panels under
`mackes/workbench/network/mesh_*.py` + the Rust `mackes-mesh-types`
crate. Today each one independently calls `tailscale status --json`
or `headscale ... list` and parses the result. Phase 12 routes every
read through one daemon that owns the cache, validation, and history.

### 12.A — Design locks (5-question survey, 2026-05-19)

1. **Backend language: Rust.** New `crates/mackesd/` workspace
   member shipping two artifacts: `mackesd` binary (reconcile loop,
   CLI) and `mackesd-core` library (linked into the panel — no IPC,
   no FFI). Reuses the existing `mackes-mesh-types` crate. Ships
   inside the existing `mackes-xfce-workstation` RPM.
2. **Storage: SQLite (WAL mode).** Single durable file at
   `/var/lib/mackesd/mackesd.db`. Migrations via `sqlx-cli`,
   numbered SQL files in `crates/mackesd/migrations/`. Backups
   via `sqlite3 .backup`. Schema sketch:
   `nodes`, `desired_config`, `runtime_state`, `events`, `links`,
   `policies`, `drift`.
3. **Inter-component access: in-process library + shared filesystem.**
   No networked API (user lock). The panel imports
   `mackesd_core::mesh::*` directly. Peer-to-peer sync uses the
   existing `~/QNM-Shared/<peer>/mackesd/` mount (already SSHFS-
   backed via the mesh-FS). Heartbeats + link telemetry land as
   JSON files under that mount; the leader's `mackesd` aggregates.
4. **Topology renderer: Cairo + GTK DrawingArea, in-process.**
   Custom `Gtk.DrawingArea` widget paints nodes + edges. Zero new
   deps (Cairo + PyGObject already in the tree). Force-directed
   layout in Rust (via `force_graph` crate or hand-rolled).
   Snapshot tests render to PNG and pixel-diff via `pixelmatch`.
5. **Backend topology: every peer runs `mackesd`; leader elected
   via `~/QNM-Shared/.mackesd-leader.lock` (60 s lease).** Highly
   available: a dead leader auto-fails-over after one missed lease
   renewal. Includes split-brain detection (compare last-known
   revision on lease conflict; the side with the older revision
   yields) and fencing (a peer that lost leadership must reload
   state from the shared store before resuming reads).
6. **16-character passcode: one shared mesh-wide code.** Generated
   at Host `mackesd init`, stored in libsecret as
   `org.mackes.mesh.passcode`, used for both peer enrollment AND
   service-to-service authentication. `mackesd rotate-passcode`
   propagates a new code via the shared filesystem; peers update
   their libsecret on next heartbeat; offline peers require manual
   re-entry. Matches `/goal` acceptance bullet #8 verbatim.

Survey-lock applies to every substep below. Substeps stay `[ ] Open`
until their preceding gate ships, but the architecture is fixed.

### 12.B — Acceptance criteria (from the /goal spec, condensed)

A Phase 12 substep is not Done until it contributes to one of these
13 acceptance bullets, AND the contribution is verifiable by the
test pyramid in 12.11:

1. Backend is the authoritative single source of truth.
2. GUI reads Mesh topology + config from the backend.
3. GUI includes live topology drawings reflecting reality.
4. Desired state vs actual runtime state are explicitly modeled.
5. Drift detection is implemented + surfaces in the GUI.
6. Config changes are versioned, validated, auditable, reversible.
7. Backend supports secure node identity, enrollment, lifecycle.
8. Single 16-character passcode gates join + service interaction.
9. Observability surfaces — metrics, logs, events, health — are
   live and visible in the GUI.
10. Topology visualization shows nodes, links, routes, health,
    policy status, and Desired-vs-Actual diffs.
11. Strong automated test coverage including failure scenarios.
12. Documentation for operators, administrators, developers.
13. Reliable enough for production enterprise use (no demo gaps).

### 12.1 — Backend architecture (no API surface)

The backend has **no networked API** (user lock 2026-05-19). It is a
library + CLI that owns the store; GUI access is via in-process link
or direct store read, never network calls. Every peer runs the daemon;
one is the leader (per 12.A.5) and is the only writer.

- [ ] **12.1.1 Daemon/library scaffold** — new `crates/mackesd/`
  workspace member. Two artifacts: a `mackesd` binary for periodic
  reconciliation + CLI ops, and a `mackesd-core` library that the
  panel links in for read access. Ships a `mackesd.service`
  systemd unit (enabled on every peer) + a `mackesd` user.
- [ ] **12.1.1b Leader election** — `mackesd` acquires
  `~/QNM-Shared/.mackesd-leader.lock` on startup (60 s lease).
  Lease renewal every 20 s; on miss, the next peer in lexicographic
  node-id order takes over. Split-brain detection: on lease
  conflict, the side with the older `applied_revision` yields
  + reloads from the shared store. Fencing: a deposed leader
  marks its in-memory state stale + re-hydrates before serving
  reads again.
- [ ] **12.1.2 Service-layer split** — internal modules in the
  order the spec lists: `service/`, `policy/`, `store/`,
  `topology/`, `telemetry/`, `reconcile/`, `deploy/`, `audit/`.
  One file per module; one trait per public surface. No
  cross-module imports of internals — only through `service::*`
  facades.
- [ ] **12.1.3 Health check** — `mackesd healthz` (CLI) prints
  backend state summary as JSON. Same data surfaced to the panel's
  status cluster via the in-process library link.
- [ ] **12.1.4 Structured logging** — JSON logs via `tracing`
  (Rust) or `structlog` (Python). Every log line carries
  `correlation_id`, `node_id` (when applicable), `revision_id`
  (when applicable), `span`, `level`. `mackesd logs` tails the
  journal.
- [ ] **12.1.5 Metrics** — written to a local Prometheus textfile
  collector path (`/var/lib/node_exporter/textfile_collector/mackesd.prom`).
  No HTTP endpoint. Counters: `mackesd_apply_total`,
  `mackesd_apply_failed_total`, `mackesd_drift_detected_total`,
  `mackesd_node_unreachable_total`. Histograms: probe + reconcile
  latency. Operators wire the textfile collector to their own
  Prometheus if they want remote scrape.

### 12.2 — Configuration model + persistence

- [ ] **12.2.1 Schema for the 7 state buckets** — `desired_config`,
  `runtime_state`, `observed_telemetry`, `calculated_topology`,
  `pending_changes`, `applied_changes`, `failed_changes`. Each is a
  versioned table with a `revision_id` + `created_at` + `applied_at`.
- [ ] **12.2.2 Versioned revisions** — every desired-config write
  creates a new immutable revision row. `mackesd revisions list` /
  `revisions diff <id1> <id2>` / `revisions rollback <id>`.
- [ ] **12.2.3 Atomic updates** — every multi-row write is a single
  SQL transaction. Failure on any row rolls back the whole change.
  No partial-applied states in the store ever.
- [ ] **12.2.4 Migration tooling** — `mackesd migrate up/down/status`
  via `sqlx-cli` (Rust) or `alembic` (Python). Numbered SQL files
  in `migrations/`. CI gates: every PR must add a migration if any
  schema changed.

### 12.3 — Node lifecycle management

- [ ] **12.3.1 Enrollment flow** — `mackesd enroll --passcode <16>`
  on a fresh peer registers it with the Host's backend. Returns a
  per-node bearer token + a Tailscale auth key. Idempotent: re-running
  with the same hardware fingerprint refreshes credentials.
- [ ] **12.3.2 Identity model** — per-node Ed25519 keypair generated
  on first enroll, stored in `~/.local/share/mackes/node.key`,
  fingerprinted into the backend's `nodes` table. Lost-key flow:
  forced re-enrollment by Host operator.
- [ ] **12.3.3 Heartbeats** — every peer's `mackesd` writes a
  heartbeat row to its local store every 10 s + drops a heartbeat
  file under `~/QNM-Shared/<peer>/mackesd/heartbeat.json` (the
  shared mesh-FS, the only "transport" we have without an API).
  Backend marks a node `unreachable` after 3 missed heartbeats;
  `degraded` if 1 missed; `healthy` otherwise.
- [ ] **12.3.4 Decommission + forced removal** — `mackesd
  decommission <node>` revokes the bearer token, asks Tailscale to
  expire the node, marks the row decommissioned (soft delete +
  retained history). `--force` bypasses confirmation for
  unreachable peers.
- [ ] **12.3.5 Re-enrollment** — `mackesd reenroll <node>` issues
  fresh credentials without losing the historical node row.

### 12.4 — Peer + route engine

- [ ] **12.4.1 Peer-relationship calculator** — given the current
  node set + policies, output the expected peer adjacencies. Pure
  function over the desired-state snapshot; tested with golden
  fixtures (full mesh, partial mesh, site-to-site, isolated).
- [ ] **12.4.2 Routing topology** — same calculator emits a route
  table per peer (next-hop + cost) for the reconciler to push into
  Tailscale's ACL / Headscale's routes API.
- [ ] **12.4.3 Latency-aware + health-aware route preference** —
  the calculator reads the telemetry table; when two equal-cost
  paths exist, prefer lower-latency-and-healthier.
- [ ] **12.4.4 Explanation surface** — every emitted peer
  relationship carries a `reason` chain (the spec's "A peers with B
  because: same region, policy allows east-west, latency under
  threshold, both healthy"). Surfaced via `mackesd peers why <id>`
  CLI + a callable on the in-process library.

### 12.5 — Reconciliation engine

- [ ] **12.5.1 Drift detector** — periodic worker (default 30 s)
  compares desired vs runtime vs observed. Drift records land in
  the `drift` table with severity (`auto-repairable` / `manual-review`).
- [ ] **12.5.2 Deployment lifecycle state machine** — `Draft →
  Validated → Approved → Deploying → Applied → Verified` (happy
  path) with branches to `Failed Validation`, `Failed Deployment →
  Rolled Back`. Persisted in `applied_changes` / `failed_changes`.
- [ ] **12.5.3 Auto-repair safe drift** — when drift severity is
  `auto-repairable` AND policy allows, the reconciler re-pushes
  desired state. Manual drift surfaces in the GUI inbox.
- [ ] **12.5.4 Retry + backoff** — failed deployments retry with
  exponential backoff (1 s → 1 min cap, 5 attempts). Persistent
  failure marks the change `failed` + alerts.
- [ ] **12.5.5 Rollback path** — every `Applied` revision retains
  the prior revision's snapshot so `mackesd rollback <revision>`
  restores it atomically.

### 12.6 — Telemetry ingestion + observability

- [ ] **12.6.1 Heartbeat ingest** — each peer's `mackesd` writes
  health + agent version + last-applied revision into its local
  `observed_telemetry` table AND copies the same row into
  `~/QNM-Shared/<peer>/mackesd/heartbeat.json`. The Host's
  reconciler aggregates the per-peer files on its next tick.
- [ ] **12.6.2 Link telemetry** — every peer measures latency +
  packet loss + throughput to each of its peers, writes results
  every 30 s to `~/QNM-Shared/<peer>/mackesd/links.json`.
  Aggregated per-link in `topology_link_health` on the Host.
- [ ] **12.6.3 Event log** — append-only `events` table with a
  hash-chained `prev_hash` field for tamper detection. Audit log =
  events filtered to `kind IN (config_change, auth, lifecycle)`.
- [ ] **12.6.4 Alerting hooks** — per event-kind, a configurable
  shell command runs with the event JSON on stdin. No webhooks
  (no networking — operators can wire `curl` themselves). Mackes
  ships no alerting tool of its own.

### 12.7 — Validation layer

- [ ] **12.7.1 Schema validation** — every store write goes through
  a `serde`-derived (Rust) or `pydantic`-derived (Python) model.
  Garbage in → `ValidationError`, never reaches the store.
- [ ] **12.7.2 Policy validation** — policy DSL = a JSON document
  with a known schema; backend lints it before save. Conflicts
  (two rules that both require AND forbid the same edge) raise
  `PolicyConflict` with the conflicting rule IDs.
- [ ] **12.7.3 Topology validation** — circular dependencies + invalid
  peer references + address conflicts surfaced at config-save time,
  not deploy time.
- [ ] **12.7.4 Dry-run mode** — `mackesd apply --dry-run` (CLI) +
  the equivalent library call run the full validation +
  reconcile-plan without mutating anything; return the diff + the
  would-be event log as a structured value.

### 12.8 — GUI overhaul (Workbench mesh panels)

- [ ] **12.8.1 Replace the existing 9 workbench panels** with a
  unified `MeshControlPanel` that reads through the in-process
  backend library. Existing panels (`mesh_vpn`, `mesh_ssh`,
  `mesh_services`, `mesh_health`, `mesh_join`, `mesh_topology`,
  `mesh_performance`, `qnm`) become tabs inside this one panel.
  Each tab calls `mackesd_core::mesh::<resource>()` directly —
  no IPC, no HTTP.
- [ ] **12.8.2 Pending changes inbox** — list of unapproved drafts
  with "Approve" / "Reject" buttons. Approving triggers the
  deployment lifecycle.
- [ ] **12.8.3 Config history + diff viewer** — list of revisions
  with author + timestamp + summary; clicking opens a side-by-side
  diff vs the previous revision. "Rollback to this revision" button
  on every row.
- [ ] **12.8.4 16-char passcode setup flow** — wizard step on first
  launch: generate or paste passcode, displayed once + saved to
  libsecret. Re-displaying requires the existing AdminSession path.

### 12.9 — Live topology visualization

- [ ] **12.9.1 Cairo renderer (assumes 12.A.4 locks Cairo)** — new
  `mackes/workbench/network/mesh_topology_render.py` (or Rust
  equivalent). Reads `mackesd_core::topology()` +
  `mackesd_core::links()` directly (in-process — no IPC), renders
  nodes + edges + labels with force-directed layout. Refreshes
  every 5 s.
- [ ] **12.9.2 Health overlay** — node fill color = health state
  (green/amber/red/grey). Edge style = link state (solid =
  healthy, dashed = backup route, red = failed). Labels show
  latency in ms when zoomed in.
- [ ] **12.9.3 Desired-vs-Actual diff overlay** — toggle between
  three modes: "Desired only" (the configured topology), "Actual
  only" (what's really up), "Diff" (red = should-exist-but-doesn't,
  amber = exists-but-shouldn't). Drift indicators surface inline.
- [ ] **12.9.4 Interactive node selection** — click a node → side
  panel with full details (uptime, last heartbeat, version,
  active routes, policy associations, recent events). Click an
  edge → link details + a "why does this peer exist" trace
  (12.4.4 surface).
- [ ] **12.9.5 Global view + Node-level view modes** — segmented
  control at the top toggles. Global = all nodes in one canvas;
  Node-level = the focused node and its direct peers only.

### 12.10 — Security layer

- [ ] **12.10.1 16-character passcode (per spec acceptance #8)** —
  generated at Host setup with `secrets.token_urlsafe(12)` (yields
  exactly 16 chars URL-safe). Stored in libsecret on the Host
  peer; the panel reads it through libsecret API, never plaintext
  files.
- [ ] **12.10.2 Passcode rotation** — `mackesd rotate-passcode`
  command issues a new code; every enrolled peer gets a fresh
  bearer token on next heartbeat.
- [ ] **12.10.3 Audit log integrity** — `events` rows form a hash
  chain (`hash = SHA256(prev_hash + payload + timestamp)`).
  `mackesd audit verify` walks the chain and reports any break.
- [ ] **12.10.4 Secret-zeroing** — Rust: `Zeroize` derive on every
  type that holds a bearer token; Python: `secrets` module +
  explicit `del` after use.

### 12.11 — Testing

- [ ] **12.11.1 Unit tests** — every pure function in
  `topology/`, `policy/`, `validation/`. Target 90% coverage on
  the policy + topology engines (they have no I/O).
- [ ] **12.11.2 Integration tests via testcontainers** — spin up
  Headscale + 3 Tailscale peers + `mackesd` in Docker Compose;
  run the happy-path enrollment + drift-detection + rollback
  flow end-to-end.
- [ ] **12.11.3 Failure scenario tests** — node failure, region
  outage (split-brain), invalid config, stale telemetry, route
  conflict, policy conflict, passcode rotation during apply.
  Each gets a named test that asserts the system returns to a
  consistent state.
- [ ] **12.11.4 GUI rendering tests** — Cairo snapshot diffs via
  `cairo-rs` + `image-rs` (or `pixelmatch` for Python). Topology
  layouts are deterministic given the same input; snapshots gate
  visual regressions.
- [ ] **12.11.5 Library contract tests** — public functions in
  `mackesd-core` snapshot-tested via `insta` (Rust) so any change
  to the consumed surface fails CI loudly. No OpenAPI surface
  (no API by user lock 2026-05-19).

### 12.12 — Documentation

- [ ] **12.12.1 Architecture overview** —
  `docs/design/v12.0-enterprise-mesh.md`: the 8-layer service
  architecture diagram, the 7 state buckets, the deployment
  lifecycle state machine.
- [ ] **12.12.2 Library reference** — `cargo doc --no-deps -p
  mackesd-core` published to
  `/usr/share/mackes-shell/help/mackesd-core/` and linked from
  the Help tab. No HTTP API reference (no API by user lock
  2026-05-19).
- [ ] **12.12.3 Operator runbook** — `docs/help/mesh-ops.md`:
  enrolling a peer, decommissioning, rotating the passcode,
  recovering from split-brain, reading the audit log.
- [ ] **12.12.4 Admin guide** — surfaced in the GUI Help tab:
  "How to configure a site-to-site mesh", "How to set up a
  failover route", "What a drift warning means".
- [ ] **12.12.5 Developer guide** —
  `docs/design/v12.0-enterprise-mesh-dev.md`: how to add a new
  policy kind, how the reconciler dispatches, how the topology
  diff is computed.

### 12.13 — Migration path

- [ ] **12.13.1 Inventory the loose state** — every JSON / TOML /
  cache file under `~/.config/mackes-shell/`,
  `~/.qnm-sync/`, `~/.cache/mackes/` that today holds mesh data.
- [ ] **12.13.2 Importer** — `mackesd import-legacy` reads each
  source, writes the equivalent desired-state rows into the new
  store, dry-run mode by default.
- [ ] **12.13.3 Cutover** — once the new backend serves a single
  test mesh end-to-end, the Workbench Mesh panels switch to API
  reads (12.8.1). Legacy probes stay during a two-release
  deprecation window with `[deprecated]` log warnings.
- [ ] **12.13.4 Retire the legacy probes** — delete
  `mackes/mesh_*.py` modules whose role is fully owned by
  `mackesd`. RPM `Obsoletes` is unnecessary (same package).

---

## Tracking

This worklist is the canonical source for v3.0.0 / 1.0.0 work, per
[mackes-worklist-management](.claude/skills/mackes-worklist-management/SKILL.md).
Mark items `[>] In Progress` before starting; `[✓] Done` only when every
gate in CLAUDE.md §0.8 (committed · pushed · RPM builds · imports clean ·
CHANGELOG updated) is satisfied.
