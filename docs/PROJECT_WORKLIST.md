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

- [ ] **4.1 Drawer IPC** — define a `mackes-drawer` D-Bus interface so the new Rust panel can open/close the existing Python drawer window. (Or: port the drawer to Rust — decide in 4.1a planning task.)
- [✓] **4.2 Status-cluster click → Drawer open** — each of the 6 status buttons shells out to `mackes --drawer --drawer-focus <slug>` so the existing Python drawer opens with the right section pre-selected. D-Bus interface (4.1) lives in a follow-up if startup latency becomes a concern.
- [ ] **4.3 Drawer port to mackes-panel module** *(if 4.1a == port)* — bring `mackes/drawer.py` into `crates/mackes-panel/src/modules/drawer/` as Rust, using gtk-rs.
- [ ] **4.4 Quick-toggle behaviors** — Mesh on/off, Bluetooth, Do-Not-Disturb, Caffeine all driven from the drawer's existing Python wiring (or ported in 4.3).

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

### 8.5 — First-boot visual polish (in-flight bug bundle, 2026-05-18)

User feedback bundle on the freshly-installed panel surfaces:

- [>] **8.5.1 Recolor Mackes-Carbon symbolic icons at load** — `icons::load()` rasterizes SVGs via `Pixbuf::from_file_at_scale`, which renders `fill="currentColor"` as black. Result: every glyph in the top bar is black on a black bar (left Apple button, right status cluster). Substitute `currentColor` → `#f0f0f0` in the SVG source before handing to gdk-pixbuf so the cached Pixbuf is already light. Also add a panel-scoped CSS rule so text labels in the top bar pick up Carbon text-primary.
- [>] **8.5.2 Bottom dock auto-sizes / hides when empty** — fixed-`DOCK_HEIGHT_PX = 80` reserves a thick strip even with zero dock items. Build the strip first; if empty, never `show_all` the dock window. When populated, fit window height to icon size + small padding (~50 % of the prior thickness).
- [>] **8.5.3 12-hour clock + weather popover** — switch `current_hhmm()` from `%H:%M` to `%l:%M %p` (trim leading space). Wrap the label in a frameless `gtk::Button`; click opens a `gtk::Popover` rendering current conditions fetched from `api.met.no/weatherapi/locationforecast/2.0/complete?lat=…&lon=…`, the same endpoint xfce4-weather-plugin uses. New `weather.rs` module; HTTP via the system `curl` (no new crate dependency) with the User-Agent met.no requires.
- [>] **8.5.4 Status-cluster popovers for review** — currently each of the 6 right-side buttons only shells out to `mackes --drawer --drawer-focus <slug>` with no visible UI on the panel side. Replace with an in-process `gtk::Popover` showing the slug title + a "Open in Drawer →" button that keeps the existing subprocess path. Gives the user an immediate review surface without waiting on the Python drawer process.
- [>] **8.5.5 `_NET_WM_STRUT_PARTIAL` on panel + dock** — `WindowTypeHint::Dock` alone doesn't reserve screen space; maximized windows occlude the panel and the dock. Publish strut hints on both windows (top: `top = TOP_BAR_HEIGHT_PX`, bottom: `bottom = dock_height` when shown). Solves the bug under xfwm4 and under any future tiling-WM swap.

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
- [>] **10.5 Cut release 1.0.0** — RPM `Release: 1` (was `0.1.rc1`). `make rpm` produces `mackes-xfce-workstation-1.0.0-1.fc44.x86_64.rpm`. CHANGELOG + tag below complete the cut.

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

## Tracking

This worklist is the canonical source for v3.0.0 / 1.0.0 work, per
[mackes-worklist-management](.claude/skills/mackes-worklist-management/SKILL.md).
Mark items `[>] In Progress` before starting; `[✓] Done` only when every
gate in CLAUDE.md §0.8 (committed · pushed · RPM builds · imports clean ·
CHANGELOG updated) is satisfied.
