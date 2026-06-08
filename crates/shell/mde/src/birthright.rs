//! `mde birthright` — the commissioning dashboard (E7.3+).
//!
//! After install, the operator needs one screen that *attests the node came up
//! whole* — not merely that the installer exited 0. Birthright is that screen: a
//! Carbon status dashboard of live, re-runnable checks. It is launched as the
//! final step of the OOBE (see `oobe::Msg::Finish`) and re-surfaced at each login
//! by the labwc autostart while `state.birthright_show_at_startup` is true; the
//! operator unchecks "Show this at startup" to dismiss it for good.
//!
//! E7.3 ships the dashboard shell + lifecycle + the **Desktop** section (the live
//! checks that would have caught the second-login black-desktop regression: is
//! labwc up, is `mde panel` up, did the autostart's background services run). The
//! Mesh / Voice / Network sections land in E7.4 / E7.5 — this surface renders only
//! sections that are genuinely live (no placeholder cards, CLAUDE.md §3).
//!
//! Workstation-role only: `main.rs` gates `birthright` through
//! [`crate::role_gate`] (`DESKTOP_ONLY`), so a headless Server/Lighthouse refuses
//! it before a window is ever created.

use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use iced::widget::{button, checkbox, container, scrollable, text, Column, Row, Space};
use iced::{Element, Length, Subscription, Task};

use mde_ui::{metrics, palette};

/// How often the dashboard re-probes while open (live attestation). The Desktop
/// checks are cheap `/proc` scans, so a brisk cadence is fine; the expensive
/// probes that land in E7.5 (nmap, RTT) will be gated behind the Re-check button.
const POLL: Duration = Duration::from_secs(2);

/// A check's state — Carbon tri-state plus a transient "checking…".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A probe is in flight (initial open + manual Re-check).
    Checking,
    /// Healthy.
    Pass,
    /// Up but partial (e.g. an optional service down) — not a hard failure.
    Degraded,
    /// Down / not detected.
    Fail,
}

impl Status {
    /// The palette role for the status dot/label (remapped to Carbon by
    /// `palette::color`). `GRAY_TEXT` reads as "in progress / unknown".
    fn color(self) -> palette::Rgb {
        match self {
            Status::Checking => palette::GRAY_TEXT,
            Status::Pass => palette::STATUS_OK,
            Status::Degraded => palette::STATUS_WARN,
            Status::Fail => palette::STATUS_RISK,
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Status::Checking => "…",
            Status::Pass => "OK",
            Status::Degraded => "!",
            Status::Fail => "X",
        }
    }
}

/// Worst-of rollup for a section / the whole dashboard: any `Fail` dominates,
/// then `Degraded`, then a still-in-flight `Checking`, else `Pass`.
fn rollup(checks: &[Check]) -> Status {
    if checks.iter().any(|c| c.status == Status::Fail) {
        Status::Fail
    } else if checks.iter().any(|c| c.status == Status::Degraded) {
        Status::Degraded
    } else if checks.iter().any(|c| c.status == Status::Checking) {
        Status::Checking
    } else {
        Status::Pass
    }
}

/// One attestation row.
#[derive(Debug, Clone)]
struct Check {
    label: &'static str,
    status: Status,
    detail: String,
}

impl Check {
    fn checking(label: &'static str) -> Self {
        Check {
            label,
            status: Status::Checking,
            detail: "checking…".into(),
        }
    }
}

struct Birthright {
    desktop: Vec<Check>,
    show_at_startup: bool,
}

#[derive(Debug, Clone)]
enum Message {
    /// User pressed "Re-check all": flash Checking, then re-probe.
    Recheck,
    /// Run the probes (immediately after open / Recheck, and on each tick).
    Probe,
    /// Periodic live refresh.
    Tick,
    /// "Show this at startup" toggled — persisted to menu.json.
    ToggleStartup(bool),
    /// Close the dashboard.
    Close,
}

/// The three Desktop rows in their initial (pre-probe) Checking state.
fn desktop_checking() -> Vec<Check> {
    vec![
        Check::checking("Compositor (labwc)"),
        Check::checking("Taskbar (mde panel)"),
        Check::checking("Session services"),
    ]
}

pub fn run(args: &[String]) -> ExitCode {
    // `--autostart`: the labwc autostart launches us this way every login. Honour
    // the per-user "show at startup" flag and exit silently when it's off, so an
    // operator who dismissed the dashboard isn't nagged. (A manual `mde birthright`
    // always shows.) The Workstation-role gate already ran in main.rs dispatch.
    if args.iter().any(|a| a == "--autostart") && !crate::state::load().birthright_show_at_startup {
        return ExitCode::SUCCESS;
    }

    let r = iced::application(
        |_: &Birthright| "Birthright Commissioning".to_string(),
        update,
        view,
    )
    .theme(|_| palette::iced_theme())
    .window_size(iced::Size::new(560.0, 640.0))
    .subscription(subscription)
    .font(mde_ui::font::REGULAR_BYTES)
    .font(mde_ui::font::BOLD_BYTES)
    .font(mde_ui::font::PLEX_REGULAR_BYTES)
    .font(mde_ui::font::PLEX_BOLD_BYTES)
    .default_font(mde_ui::font::ui())
    .run_with(|| {
        (
            Birthright {
                desktop: desktop_checking(),
                show_at_startup: crate::state::load().birthright_show_at_startup,
            },
            Task::done(Message::Probe),
        )
    });
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde birthright: {e}");
            ExitCode::FAILURE
        }
    }
}

fn subscription(_state: &Birthright) -> Subscription<Message> {
    iced::time::every(POLL).map(|_| Message::Tick)
}

fn update(state: &mut Birthright, message: Message) -> Task<Message> {
    match message {
        Message::Recheck => {
            state.desktop = desktop_checking();
            return Task::done(Message::Probe);
        }
        Message::Probe | Message::Tick => {
            state.desktop = probe_desktop();
        }
        Message::ToggleStartup(on) => {
            state.show_at_startup = on;
            let mut st = crate::state::load();
            st.birthright_show_at_startup = on;
            let _ = crate::state::save(&st);
        }
        Message::Close => std::process::exit(0),
    }
    Task::none()
}

// --- probes (Desktop section) ----------------------------------------------

/// The basename of an argv[0] (strips any directory part).
fn basename(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}

/// True if this argv is the labwc compositor.
fn argv_is_labwc(argv: &[String]) -> bool {
    argv.first().is_some_and(|a| basename(a) == "labwc")
}

/// True if this argv is the canonical shell taskbar (`mde panel`, or the legacy
/// `mde-panel` basename). Deliberately does NOT match `mde birthright` itself.
fn argv_is_mde_panel(argv: &[String]) -> bool {
    let Some(a0) = argv.first().map(|s| basename(s)) else {
        return false;
    };
    (a0 == "mde" && argv.iter().any(|t| t == "panel")) || a0 == "mde-panel"
}

/// Read every process's argv from `/proc` (NUL-separated `cmdline`). Best-effort:
/// unreadable entries are skipped, never fatal.
fn proc_cmdlines() -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir("/proc") else {
        return out;
    };
    for entry in rd.flatten() {
        let name = entry.file_name();
        let is_pid = name
            .to_str()
            .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()));
        if !is_pid {
            continue;
        }
        if let Ok(raw) = std::fs::read(entry.path().join("cmdline")) {
            let argv: Vec<String> = raw
                .split(|b| *b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect();
            if !argv.is_empty() {
                out.push(argv);
            }
        }
    }
    out
}

/// Is the clipboard-history daemon (an autostart-launched background service)
/// alive? Reuses its PID lockfile — a live PID proves the autostart block ran,
/// which is the exact thing the black-desktop regression broke.
fn clipboard_daemon_alive() -> bool {
    crate::clipboard::dir()
        .map(|d| d.join("daemon.lock"))
        .and_then(|lock| std::fs::read_to_string(lock).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .is_some_and(|pid| Path::new(&format!("/proc/{pid}")).exists())
}

/// Probe the three Desktop rows from live system state.
fn probe_desktop() -> Vec<Check> {
    let procs = proc_cmdlines();
    let labwc = procs.iter().any(|a| argv_is_labwc(a));
    let panel = procs.iter().any(|a| argv_is_mde_panel(a));
    let clip = clipboard_daemon_alive();

    vec![
        Check {
            label: "Compositor (labwc)",
            status: if labwc { Status::Pass } else { Status::Fail },
            detail: if labwc {
                "labwc is running".into()
            } else {
                "labwc not detected — the Wayland session is not up".into()
            },
        },
        Check {
            label: "Taskbar (mde panel)",
            status: if panel { Status::Pass } else { Status::Fail },
            detail: if panel {
                "mde panel is running".into()
            } else {
                "mde panel is not running — the desktop autostart did not launch it".into()
            },
        },
        Check {
            // Optional background services: down is Degraded, not a hard Fail.
            label: "Session services",
            status: if clip { Status::Pass } else { Status::Degraded },
            detail: if clip {
                "clipboard-history daemon running (autostart completed)".into()
            } else {
                "clipboard-history daemon not running — autostart may be incomplete".into()
            },
        },
    ]
}

// --- view -------------------------------------------------------------------

fn label(s: impl text::IntoFragment<'static>) -> iced::widget::Text<'static> {
    text(s).size(metrics::UI_PX)
}

/// One status row: a fixed-width status chip, the label, and the detail line.
fn check_row(c: &Check) -> Element<'static, Message> {
    let chip = container(
        text(c.status.glyph())
            .size(metrics::UI_PX)
            .font(mde_ui::font::ui_bold())
            .color(palette::color(c.status.color())),
    )
    .width(Length::Fixed(28.0));

    Row::new()
        .spacing(metrics::SPACING_03)
        .align_y(iced::Alignment::Center)
        .push(chip)
        .push(
            Column::new()
                .spacing(metrics::SPACING_01)
                .push(label(c.label).font(mde_ui::font::ui_bold()))
                .push(label(c.detail.clone()).color(palette::color(palette::GRAY_TEXT))),
        )
        .into()
}

/// A titled section card with a rolled-up status chip in its header.
fn section_card(title: &'static str, checks: &[Check]) -> Element<'static, Message> {
    let roll = rollup(checks);
    let header = Row::new()
        .spacing(metrics::SPACING_02)
        .align_y(iced::Alignment::Center)
        .push(
            text(roll.glyph())
                .size(metrics::UI_PX)
                .font(mde_ui::font::ui_bold())
                .color(palette::color(roll.color())),
        )
        .push(
            text(title)
                .size(metrics::INFO_TITLE_PX)
                .font(mde_ui::font::ui_bold()),
        );

    let mut body = Column::new().spacing(metrics::SPACING_03).push(header);
    for c in checks {
        body = body.push(check_row(c));
    }

    container(
        body.spacing(metrics::SPACING_03)
            .padding(metrics::SPACING_04),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(palette::color(palette::WINDOW))),
        ..container::Style::default()
    })
    .into()
}

fn view(state: &Birthright) -> Element<'_, Message> {
    let header = Column::new()
        .spacing(metrics::SPACING_01)
        .push(
            text("Birthright Commissioning")
                .size(metrics::INFO_TITLE_PX)
                .font(mde_ui::font::ui_bold()),
        )
        .push(
            label("Confirms this workstation came up whole.")
                .color(palette::color(palette::GRAY_TEXT)),
        );

    let sections = scrollable(
        Column::new()
            .spacing(metrics::SPACING_04)
            .push(section_card("Desktop", &state.desktop)),
    )
    .height(Length::Fill);

    let footer = Row::new()
        .spacing(metrics::SPACING_04)
        .align_y(iced::Alignment::Center)
        .push(
            checkbox("Show this at startup", state.show_at_startup)
                .on_toggle(Message::ToggleStartup)
                .size(metrics::UI_PX)
                .text_size(metrics::UI_PX),
        )
        .push(Space::with_width(Length::Fill))
        .push(
            button(label("Re-check all"))
                .on_press(Message::Recheck)
                .height(Length::Fixed(metrics::BUTTON_MD)),
        )
        .push(
            button(label("Close"))
                .on_press(Message::Close)
                .height(Length::Fixed(metrics::BUTTON_MD)),
        );

    let body = Column::new()
        .spacing(metrics::SPACING_05)
        .padding(metrics::SPACING_05)
        .push(header)
        .push(sections)
        .push(footer);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::color(palette::MENU))),
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_string()).collect()
    }

    #[test]
    fn basename_strips_dirs() {
        assert_eq!(basename("/usr/bin/labwc"), "labwc");
        assert_eq!(basename("labwc"), "labwc");
        assert_eq!(basename("/usr/bin/mde"), "mde");
    }

    #[test]
    fn labwc_argv_matches_only_labwc() {
        assert!(argv_is_labwc(&s(&["/usr/bin/labwc"])));
        assert!(argv_is_labwc(&s(&["labwc", "-C", "/etc/labwc"])));
        assert!(!argv_is_labwc(&s(&["/usr/bin/mde", "panel"])));
        assert!(!argv_is_labwc(&[]));
    }

    #[test]
    fn panel_argv_matches_mde_panel_not_birthright() {
        assert!(argv_is_mde_panel(&s(&["/usr/bin/mde", "panel"])));
        assert!(argv_is_mde_panel(&s(&["mde", "panel"])));
        assert!(argv_is_mde_panel(&s(&["/usr/bin/mde-panel"])));
        // Must NOT match the dashboard itself, nor other mde subcommands.
        assert!(!argv_is_mde_panel(&s(&["/usr/bin/mde", "birthright"])));
        assert!(!argv_is_mde_panel(&s(&["/usr/bin/mde", "files"])));
        assert!(!argv_is_mde_panel(&[]));
    }

    #[test]
    fn rollup_is_worst_of() {
        let mk = |st: Status| Check {
            label: "x",
            status: st,
            detail: String::new(),
        };
        assert_eq!(rollup(&[mk(Status::Pass), mk(Status::Pass)]), Status::Pass);
        assert_eq!(
            rollup(&[mk(Status::Pass), mk(Status::Degraded)]),
            Status::Degraded
        );
        assert_eq!(
            rollup(&[mk(Status::Fail), mk(Status::Degraded)]),
            Status::Fail
        );
        assert_eq!(
            rollup(&[mk(Status::Checking), mk(Status::Pass)]),
            Status::Checking
        );
        // Fail dominates Checking.
        assert_eq!(
            rollup(&[mk(Status::Checking), mk(Status::Fail)]),
            Status::Fail
        );
    }

    #[test]
    fn desktop_checking_seeds_three_rows() {
        let rows = desktop_checking();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|c| c.status == Status::Checking));
    }
}
