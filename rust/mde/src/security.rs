//! Windows 10 "Windows Security" dashboard home (E14.4).
//!
//! A small iced window showing the 6 posture tiles (icon + title + status line +
//! an OK/WARN/RISK glyph), fed by [`crate::security_probe`] off the UI thread via
//! an async `Loaded` (the `system_properties.rs` pattern), so the window paints at
//! once and the probes fill in. Era-gated to Windows 10 (E14.10). The per-tile
//! detail pages land in E14.5–E14.9; this is the home grid.

use std::process::ExitCode;

use iced::widget::{button, column, container, mouse_area, text, Column, Row, Space};
use iced::{Element, Length, Padding, Task};

use crate::security_probe::{self, BlockDev, FirewallDetail, Level, SecurityStatus, Tile};
use mde_ui::{metrics, palette};

/// Which pane the dashboard is showing — the home grid or a tile's detail page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Home,
    Firewall,
    Encryption,
}

struct Security {
    status: Option<SecurityStatus>,
    pane: Pane,
    /// Live firewall detail, loaded when the Firewall page opens (E14.5).
    fw: Option<FirewallDetail>,
    /// Block-device list for the Encryption page (E14.6).
    devs: Vec<BlockDev>,
    /// A plain device the user asked to "turn on" — shows the advisory luksFormat
    /// command (never auto-run).
    confirm_format: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Box<SecurityStatus>),
    Open(Pane),
    Back,
    Advanced, // launch firewall-config
    TurnOn(String),
    CancelConfirm,
    BackupKey(String),
    KeyBackedUp(String, Option<String>), // (device, chosen save path)
}

pub fn run(args: &[String]) -> ExitCode {
    // Era gate (E14.10): the Security dashboard is a Windows 10 surface.
    if !palette::is_windows10() {
        eprintln!(
            "mde security: Windows Security is a Windows 10-era surface — use the Control Panel \
             security tools in this theme."
        );
        return ExitCode::SUCCESS;
    }
    // Deep-link: `mde security <page>` opens straight to a detail page.
    let deep = args.iter().find_map(|a| match a.as_str() {
        "firewall" => Some(Pane::Firewall),
        "encryption" => Some(Pane::Encryption),
        _ => None,
    });
    let r = iced::application(|_: &Security| "Windows Security".to_string(), update, view)
        .window_size(iced::Size::new(540.0, 420.0))
        .resizable(false)
        .theme(|_| palette::iced_theme())
        .font(mde_ui::font::REGULAR_BYTES)
        .font(mde_ui::font::BOLD_BYTES)
        .font(mde_ui::font::PLEX_REGULAR_BYTES)
        .font(mde_ui::font::PLEX_BOLD_BYTES)
        .default_font(mde_ui::font::ui())
        .run_with(move || {
            // The probes shell out (firewall-cmd/mokutil/lsblk/clamscan), so run
            // them off-thread and let the window appear immediately.
            let (pane, fw, devs) = match deep {
                Some(Pane::Firewall) => (
                    Pane::Firewall,
                    Some(security_probe::firewall_detail()),
                    Vec::new(),
                ),
                Some(Pane::Encryption) => {
                    (Pane::Encryption, None, security_probe::encryption_detail())
                }
                _ => (Pane::Home, None, Vec::new()),
            };
            (
                Security {
                    status: None,
                    pane,
                    fw,
                    devs,
                    confirm_format: None,
                },
                Task::perform(async { Box::new(security_probe::probe()) }, Message::Loaded),
            )
        });
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn update(state: &mut Security, message: Message) -> Task<Message> {
    match message {
        Message::Loaded(s) => state.status = Some(*s),
        Message::Open(Pane::Firewall) => {
            // Load the live firewall detail when the page opens (a quick local query).
            state.fw = Some(security_probe::firewall_detail());
            state.pane = Pane::Firewall;
        }
        Message::Open(Pane::Encryption) => {
            state.devs = security_probe::encryption_detail();
            state.confirm_format = None;
            state.pane = Pane::Encryption;
        }
        Message::Open(pane) => state.pane = pane,
        Message::Back => {
            state.pane = Pane::Home;
            state.confirm_format = None;
        }
        Message::Advanced => {
            let _ = std::process::Command::new("firewall-config").spawn();
        }
        Message::TurnOn(dev) => state.confirm_format = Some(dev),
        Message::CancelConfirm => state.confirm_format = None,
        Message::BackupKey(dev) => {
            // Pick a save path via the shared file dialog, then back up the header.
            return Task::perform(async move { (dev.clone(), save_dialog()) }, |(d, p)| {
                Message::KeyBackedUp(d, p)
            });
        }
        Message::KeyBackedUp(dev, Some(path)) => {
            let _ = std::process::Command::new("pkexec")
                .args(security_probe::luks_header_backup_cmd(&dev, &path))
                .spawn();
        }
        Message::KeyBackedUp(_, None) => {}
    }
    Task::none()
}

/// Run the shared Save file dialog (`mde filedialog --save`) → chosen path.
fn save_dialog() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let o = std::process::Command::new(exe)
        .args([
            "filedialog",
            "--save",
            "--title",
            "Back up LUKS header",
            "--filename",
            "luks-header.img",
        ])
        .output()
        .ok()?;
    if o.status.success() {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

/// The OK/WARN/RISK glyph + its palette colour (E14.2 STATUS roles).
fn level_mark(level: Level) -> (&'static str, palette::Rgb) {
    match level {
        Level::Ok => ("\u{f058}", palette::STATUS_OK), // check-circle
        Level::Warn => ("\u{f071}", palette::STATUS_WARN), // exclamation-triangle
        Level::Risk => ("\u{f057}", palette::STATUS_RISK), // times-circle
    }
}

/// One status tile card.
fn tile_card<'a>(icon: &'static str, t: &Tile) -> Element<'a, Message> {
    let (mark, mark_role) = level_mark(t.level);
    let head = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .push(
            text(icon)
                .font(mde_ui::font::NERD)
                .size(metrics::TILE_GLYPH_PX)
                .color(palette::color(palette::WINDOW_TEXT)),
        )
        .push(Space::new(Length::Fill, Length::Shrink))
        .push(
            text(mark)
                .font(mde_ui::font::NERD)
                .size(metrics::BUTTON_GLYPH_PX)
                .color(palette::color(mark_role)),
        );
    container(
        Column::new()
            .spacing(6.0)
            .push(head)
            .push(
                text(t.title.clone())
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::WINDOW_TEXT)),
            )
            .push(
                text(t.status.clone())
                    .size(metrics::BADGE_PX)
                    .color(palette::color(palette::GRAY_TEXT)),
            ),
    )
    .width(Length::Fixed(metrics::SECURITY_TILE))
    .height(Length::Fixed(metrics::SECURITY_TILE))
    .padding(12.0)
    .style(|_| container::Style {
        border: iced::Border {
            color: palette::color(palette::WINDOW_FRAME),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// The advisory "App & browser control" tile (E14.9 expands these); no fake
/// control, just real status text (§3).
fn advisory_tile() -> Tile {
    Tile {
        title: "App & browser control".to_string(),
        status: "Reputation-based controls are handled by the browser.".to_string(),
        level: Level::Ok,
    }
}

fn view(state: &Security) -> Element<'_, Message> {
    let Some(s) = &state.status else {
        return column![
            text("Security at a glance")
                .size(metrics::INFO_TITLE_PX)
                .color(palette::color(palette::WINDOW_TEXT)),
            text("Checking your device's security…")
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        ]
        .spacing(12.0)
        .padding(16.0)
        .into();
    };
    match state.pane {
        Pane::Home => home_view(s),
        Pane::Firewall => firewall_view(state.fw.as_ref()),
        Pane::Encryption => encryption_view(state),
    }
}

fn home_view(s: &SecurityStatus) -> Element<'_, Message> {
    let heading = text("Security at a glance")
        .size(metrics::INFO_TITLE_PX)
        .color(palette::color(palette::WINDOW_TEXT));

    // The 6 home tiles: five probed checks + one advisory; an icon and (for tiles
    // with a detail page) a navigation target. Only Firewall has a page so far
    // (E14.5); the rest gain theirs in E14.6–E14.9.
    let advisory = advisory_tile();
    let tiles: [(&'static str, &Tile, Option<Pane>); 6] = [
        ("\u{f188}", &s.antivirus, None),
        ("\u{f132}", &s.firewall, Some(Pane::Firewall)),
        ("\u{f0ac}", &advisory, None),
        ("\u{f023}", &s.encryption, Some(Pane::Encryption)),
        ("\u{f084}", &s.secureboot, None),
        ("\u{f2db}", &s.tpm, None),
    ];

    let mut grid = Column::new().spacing(12.0);
    for chunk in tiles.chunks(3) {
        let mut r = Row::new().spacing(12.0);
        for (icon, t, nav) in chunk {
            let card = tile_card(icon, t);
            let cell: Element<Message> = match nav {
                Some(pane) => mouse_area(card).on_press(Message::Open(*pane)).into(),
                None => card,
            };
            r = r.push(cell);
        }
        grid = grid.push(r);
    }

    Column::new()
        .spacing(14.0)
        .padding(Padding::from(16.0))
        .push(heading)
        .push(grid)
        .into()
}

/// Firewall tile detail (E14.5): live firewalld state + zones, Advanced → firewall-config.
fn firewall_view(fw: Option<&FirewallDetail>) -> Element<'_, Message> {
    let back = button(text("← Back").size(metrics::UI_PX))
        .on_press(Message::Back)
        .padding(Padding::from([4.0, 12.0]))
        .style(mde_ui::button_ghost);
    let heading = text("Firewall & network protection")
        .size(metrics::INFO_TITLE_PX)
        .color(palette::color(palette::WINDOW_TEXT));

    let mut col = Column::new().spacing(10.0).push(back).push(heading);

    if let Some(fw) = fw {
        let (mark, role) = level_mark(if fw.running { Level::Ok } else { Level::Risk });
        col = col.push(
            Row::new()
                .spacing(8.0)
                .align_y(iced::alignment::Vertical::Center)
                .push(
                    text(mark)
                        .font(mde_ui::font::NERD)
                        .size(metrics::BUTTON_GLYPH_PX)
                        .color(palette::color(role)),
                )
                .push(
                    text(if fw.running {
                        "Firewall is on."
                    } else {
                        "Firewall is off."
                    })
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::WINDOW_TEXT)),
                ),
        );
        col = col.push(
            text(format!("Default zone: {}", fw.default_zone))
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        );
        if fw.zones.is_empty() {
            col = col.push(
                text("No active network zones.")
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::GRAY_TEXT)),
            );
        } else {
            for z in &fw.zones {
                col = col.push(
                    text(format!(
                        "{} — {z} (active)",
                        security_probe::win10_zone_label(z)
                    ))
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::WINDOW_TEXT)),
                );
            }
        }
    }

    col.push(Space::new(Length::Shrink, Length::Fixed(8.0)))
        .push(
            button(text("Advanced settings").size(metrics::UI_PX))
                .on_press(Message::Advanced)
                .padding(Padding::from([4.0, 12.0]))
                .style(mde_ui::button_primary),
        )
        .padding(Padding::from(16.0))
        .into()
}

/// Encryption tile detail (E14.6): per-device LUKS state, with "Back up recovery
/// key" (luksHeaderBackup via the Save dialog) for encrypted devices and a
/// "Turn on" that shows — never runs — the exact `cryptsetup luksFormat` command.
fn encryption_view(state: &Security) -> Element<'_, Message> {
    let back = button(text("← Back").size(metrics::UI_PX))
        .on_press(Message::Back)
        .padding(Padding::from([4.0, 12.0]))
        .style(mde_ui::button_ghost);
    let mut col = Column::new().spacing(10.0).push(back).push(
        text("Device encryption")
            .size(metrics::INFO_TITLE_PX)
            .color(palette::color(palette::WINDOW_TEXT)),
    );

    // Inline "Turn on" advisory: the exact luksFormat command, never auto-run.
    if let Some(dev) = &state.confirm_format {
        col = col.push(
            container(
                Column::new()
                    .spacing(6.0)
                    .push(
                        text(format!(
                            "Encrypting /dev/{dev} ERASES it. Run this as root from a terminal:"
                        ))
                        .size(metrics::UI_PX)
                        .color(palette::color(palette::WINDOW_TEXT)),
                    )
                    .push(
                        text(security_probe::luks_format_cmd(dev))
                            .size(metrics::UI_PX)
                            .font(mde_ui::font::NERD)
                            .color(palette::color(palette::STATUS_WARN)),
                    )
                    .push(
                        button(text("Close").size(metrics::UI_PX))
                            .on_press(Message::CancelConfirm)
                            .padding(Padding::from([3.0, 10.0]))
                            .style(mde_ui::button_ghost),
                    ),
            )
            .padding(10.0)
            .style(|_| container::Style {
                border: iced::Border {
                    color: palette::accent(),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..container::Style::default()
            }),
        );
    }

    if state.devs.is_empty() {
        col = col.push(
            text("No block devices found.")
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        );
    } else {
        for dev in &state.devs {
            let (glyph, role) = if dev.encrypted {
                ("\u{f023}", palette::STATUS_OK) // lock
            } else {
                ("\u{f09c}", palette::STATUS_WARN) // unlock
            };
            let label = if dev.encrypted {
                "Encrypted (LUKS)".to_string()
            } else if dev.fstype.is_empty() {
                "Not encrypted".to_string()
            } else {
                format!("Not encrypted — {}", dev.fstype)
            };
            let action = if dev.encrypted {
                button(text("Back up recovery key").size(metrics::UI_PX))
                    .on_press(Message::BackupKey(dev.name.clone()))
            } else {
                button(text("Turn on").size(metrics::UI_PX))
                    .on_press(Message::TurnOn(dev.name.clone()))
            }
            .padding(Padding::from([3.0, 10.0]))
            .style(mde_ui::button_ghost);
            col = col.push(
                Row::new()
                    .spacing(10.0)
                    .align_y(iced::alignment::Vertical::Center)
                    .push(
                        text(glyph)
                            .font(mde_ui::font::NERD)
                            .size(metrics::TILE_GLYPH_PX)
                            .color(palette::color(role)),
                    )
                    .push(
                        Column::new()
                            .width(Length::Fill)
                            .push(
                                text(format!("/dev/{}", dev.name))
                                    .size(metrics::UI_PX)
                                    .color(palette::color(palette::WINDOW_TEXT)),
                            )
                            .push(
                                text(label)
                                    .size(metrics::BADGE_PX)
                                    .color(palette::color(palette::GRAY_TEXT)),
                            ),
                    )
                    .push(action),
            );
        }
    }

    col.padding(Padding::from(16.0)).into()
}
