//! `mde-popover` — Iced + wlr-layer-shell popover host.
//!
//! v3.0.2 panel-host wiring: the panel (`mde-panel`) spawns this
//! binary on every clickable zone press. Each popover is a separate
//! layer-shell overlay surface that anchors above the panel edge,
//! dismisses on Esc / outside-click / close-button, and exits cleanly
//! when the user commits or cancels.
//!
//! ```text
//!   mde-popover start-menu         # M button → app launcher
//!   mde-popover audio              # ♫ click → volume slider
//!   mde-popover notifications      # bell click → notification list
//!   mde-popover clock              # clock click → calendar
//!   mde-popover network            # network click → connection list
//! ```
//!
//! Per-kind ports: start-menu, audio, clock, notifications all ship
//! working today with the v3.0.3 close-button + the panel-side
//! toggle dedup + zombie reap fixes. The network kind is
//! grandfathered as an exit-0 stub under §0.12 until the v3.0.3
//! network-popover task closes.

#![forbid(unsafe_code)]

mod admin_menu;
mod audio;
mod clipboard;
mod clock;
mod dismiss;
mod farewell;
mod fonts;
mod hostname_info;
mod icon_mapper;
mod lock;
mod network;
mod notifications;
mod start_menu;
mod status;
mod toasts;
mod urgent;
mod watermark;
mod weather;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mde-popover",
    about = "Mackes Desktop Environment popover overlay surfaces"
)]
struct Cli {
    /// Which popover to mount.
    #[arg(value_enum)]
    kind: Kind,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    StartMenu,
    Audio,
    Notifications,
    Clock,
    Network,
    /// v3.0.3 — right-click-on-Start admin menu (9 actions, 5
    /// sections, foot --hold + pkexec).
    AdminMenu,
    /// v3.0.3 — long-running bottom-right watermark surface.
    /// Polls dnf every 4 hours; visible only when updates pending;
    /// click invokes `pkexec dnf upgrade`. Spawned at session
    /// start via data/sway/config rather than from the panel.
    Watermark,
    /// v3.0.3 — Super+V clipboard history popover. Reads
    /// ~/.cache/mde/clipboard.json (mesh-synced by mackesd's
    /// clipboard worker); click an entry to copy it back via
    /// wl-copy.
    Clipboard,
    /// v3.0.3 — long-running toast render surface (Layer::Top).
    /// Tails ~/.cache/mde/toasts.jsonl for emit events and stacks
    /// up to STACK_LIMIT=3 toasts above the panel. Spawned at
    /// session start via data/sway/config.
    Toast,
    /// v3.0.3 E.19 — icon-mapper glyph picker. Reads target
    /// via MDE_ICON_MAPPER_APP_ID env var; writes the picked
    /// Material Symbols glyph to ~/.local/share/applications/
    /// <app>.desktop's X-MDE-Icon= line. Spawned via the
    /// "Customize Icon..." entry on the WM-3 window-actions
    /// popover.
    IconMapper,
    /// Portal-6.c — hostname-info tooltip. Small card at the
    /// bottom-left corner above the Dock showing hostname,
    /// uptime, primary IP, and mesh role. Spawned when the
    /// user clicks the Dock's hostname segment.
    HostnameInfo,
    /// Portal-9.b — status-zone slide-up strip. Full-width
    /// 180 px strip above the Dock with Volume / Brightness /
    /// Power tabs. Spawned by clicking the volume or
    /// brightness glyph in the Dock's status segment.
    Status,
    /// Portal-25 — lock-screen layer-shell overlay. Fullscreen
    /// Layer::Overlay surface that captures keyboard exclusively
    /// and paints the MDE lock visual (M › hostname breadcrumb,
    /// big clock, date, mesh/net/battery/weather indicators).
    /// Bind via `bindsym $mod+l exec mde-popover lock`. Esc or
    /// Enter dismisses.
    Lock,
    /// BUS-2.5 — theater takeover for `urgent` Bus messages.
    /// Fullscreen Layer::Overlay rendering a centered urgent card
    /// (⚠ + title + body) read from MDE_URGENT_TITLE / MDE_URGENT_BODY;
    /// Esc / Enter / click dismisses. Spawned on a `priority=urgent`
    /// `fleet/announce` Bus segment. (E4.20: the mde-portal Dock that
    /// owned this `fleet/announce` subscription was retired; re-homing
    /// the spawner is tracked as E4.21 in `docs/PROJECT_WORKLIST.md`.)
    Urgent,
    // E4.22 — `WhichKey` removed. The sway binding-mode overlay (Q55) was
    // never wired to a spawner and is architecturally inapplicable under
    // the locked labwc compositor (plan §0 Q8), which has no sway-style
    // binding modes to set `MDE_SWAY_MODE`. Retired per §0.12 (no dead
    // code) — the newer labwc lock supersedes the sway-era surface.
    //
    // E0.16 — the sway-mode window surfaces (`overview`, `expose`,
    // `app-switcher`, `window-actions`, `snap-assist`, `minimized`) were
    // retired the same way: each shelled out to `swaymsg -t get_tree`/
    // scratchpad IPC that labwc does not provide, was bound only in the
    // dead `data/sway/*` config, and is superseded by the integrated
    // wlr-foreign-toplevel shell (`mde task-view`, E4.7) + labwc-native
    // edge-snap. Porting them would duplicate `task-view` (§2.7), so they
    // were dropped, not ported.
    /// ANIM-7.c — session-end fade-out overlay (Q40). Fullscreen
    /// Layer::Overlay that fades transparent → opaque charcoal
    /// (~200 ms) then executes the session action (logout/restart/
    /// shutdown). Reads `--action <slug>` CLI arg. Bound to
    /// `bindsym $mod+Shift+e exec mde-popover farewell --action logout`
    /// in data/sway/config.d/mackes-defaults.conf.
    Farewell,
}

fn main() -> iced_layershell::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("MDE_POPOVER_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("mde_popover=info,warn")),
        )
        .json()
        .init();

    let cli = Cli::parse();
    tracing::info!(kind = ?cli.kind, "mde-popover spawned");

    match cli.kind {
        Kind::StartMenu => start_menu::run(),
        Kind::Audio => audio::run(),
        Kind::Notifications => notifications::run(),
        Kind::Clock => clock::run(),
        Kind::AdminMenu => admin_menu::run(),
        Kind::Watermark => watermark::run(),
        Kind::Clipboard => clipboard::run(),
        Kind::Toast => toasts::run(),
        Kind::Network => network::run(),
        Kind::IconMapper => icon_mapper::run(),
        Kind::HostnameInfo => hostname_info::run(),
        Kind::Status => status::run(),
        Kind::Lock => lock::run(),
        Kind::Urgent => urgent::run(),
        Kind::Farewell => farewell::run(),
    }
}
