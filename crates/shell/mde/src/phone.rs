//! "Your Phone" (Windows 10 era) / "Mobile Devices" (Win2000·Carbon) — the KDE
//! Connect device surface (E9.2: the window shell).
//!
//! An xdg-toplevel iced window (labwc-framed): a left rail (the **device picker** plus
//! the Notifications / Messages / Photos / Calls / Settings navigation) over a content
//! pane. The picker and the per-device **Overview** (name · online · battery) read the
//! live roster from the `mde connect` daemon ([`crate::connect::devices`]). The five
//! rich view panes land in E9.3–E9.9; until each does, its nav entry is shown
//! **reserved** (greyed, non-selectable) — the project's convention for not-yet-built
//! surfaces (cf. Settings reserved pages, the Control Panel greying missing tools) —
//! so nothing is faked (§3).
//!
//! `mde phone`  opens the window; per-era title via [`palette::theme`].

use std::process::ExitCode;

use iced::widget::{button as ibutton, container, scrollable, text, Column, Row, Space};
use iced::{Element, Length, Padding, Task};

use mde_ui::{font, group_box, metrics, palette};

use crate::connect::{self, DeviceInfo};

/// Selected/hover row button style. Moved here from the retired `files.rs`
/// (E10.6) — phone.rs was its only consumer.
fn row_style(
    selected: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let hot = selected
            || matches!(
                status,
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
            );
        iced::widget::button::Style {
            background: hot.then(|| iced::Background::Color(palette::color(palette::HIGHLIGHT))),
            text_color: if hot {
                palette::color(palette::HIGHLIGHT_TEXT)
            } else {
                palette::color(palette::WINDOW_TEXT)
            },
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
        }
    }
}

/// The rail's view entries (E9.3–E9.9). Each is shown reserved until its epic lands.
const VIEWS: &[&str] = &["Notifications", "Messages", "Photos", "Calls", "Settings"];

struct Phone {
    devices: Vec<DeviceInfo>,
    /// Index into `devices` of the selected handset (clamped on reload).
    selected: usize,
    /// False until the first roster query returns, so the pane shows "Loading…"
    /// instead of an empty state that reads like "no devices".
    loaded: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Vec<DeviceInfo>),
    SelectDevice(usize),
    Refresh,
}

/// E9.7 — the window title is the modern "Your Phone" under Carbon-only
/// (modern identity); the old Win2000/Carbon "Mobile Devices" era branch is gone.
fn window_title() -> String {
    "Your Phone".to_string()
}

pub fn run(_args: &[String]) -> ExitCode {
    // (E9.11 will parse --view / --device deep links here; the shell ignores them now.)
    let r = iced::application(|_: &Phone| window_title(), update, view)
        .window_size(iced::Size::new(720.0, 480.0))
        .resizable(true)
        .theme(|_| palette::iced_theme())
        .font(font::REGULAR_BYTES)
        .font(font::BOLD_BYTES)
        .font(font::PLEX_REGULAR_BYTES)
        .font(font::PLEX_BOLD_BYTES)
        .default_font(font::ui())
        .run_with(|| {
            (
                Phone {
                    devices: Vec::new(),
                    selected: 0,
                    loaded: false,
                },
                // Query the daemon off the first paint (a Bus round-trip; the sync
                // client runs on the blocking pool so it never freezes the UI).
                Task::perform(
                    async {
                        tokio::task::spawn_blocking(connect::devices)
                            .await
                            .unwrap_or_default()
                    },
                    Message::Loaded,
                ),
            )
        });
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn update(state: &mut Phone, message: Message) -> Task<Message> {
    match message {
        Message::Loaded(devices) => {
            state.devices = devices;
            if state.selected >= state.devices.len() {
                state.selected = 0;
            }
            state.loaded = true;
        }
        Message::SelectDevice(i) => state.selected = i,
        Message::Refresh => {
            return Task::perform(
                async {
                    tokio::task::spawn_blocking(connect::devices)
                        .await
                        .unwrap_or_default()
                },
                Message::Loaded,
            );
        }
    }
    Task::none()
}

fn view(state: &Phone) -> Element<'_, Message> {
    let body = Row::new()
        .push(rail(state))
        .push(
            container(content(state))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding::from(metrics::SPACING_04)),
        )
        .width(Length::Fill)
        .height(Length::Fill);
    container(body)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(palette::color(palette::WINDOW))),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The left rail: a device picker over the (reserved) view nav, plus Refresh.
fn rail(state: &Phone) -> Element<'_, Message> {
    let mut col = Column::new()
        .spacing(metrics::SPACING_01)
        .width(Length::Fill);

    col = col.push(text("Devices").size(metrics::UI_PX).font(font::ui_bold()));
    if state.devices.is_empty() {
        let msg = if state.loaded {
            "No paired devices"
        } else {
            "Loading…"
        };
        col = col.push(
            text(msg)
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        );
    } else {
        for (i, d) in state.devices.iter().enumerate() {
            let dot = if d.online { "● " } else { "○ " };
            col = col.push(
                ibutton(text(format!("{dot}{}", d.name)).size(metrics::UI_PX))
                    .width(Length::Fill)
                    .padding(Padding::from([metrics::SPACING_02, metrics::SPACING_03]))
                    .style(row_style(i == state.selected))
                    .on_press(Message::SelectDevice(i)),
            );
        }
    }

    col = col.push(Space::with_height(10.0));
    // The view nav: reserved (greyed, non-selectable) until E9.3–E9.9 wire each pane —
    // honest, not a mockup; the device Overview is the live content for now.
    for name in VIEWS {
        col = col.push(
            container(
                text(*name)
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::GRAY_TEXT)),
            )
            .padding(Padding::from([metrics::SPACING_02, metrics::SPACING_03])),
        );
    }

    // The scrollable holds only intrinsic-height content (Fill inside a scrollable
    // panics); Refresh sits below it, pinned to the rail's bottom by the scroller's
    // Fill height.
    let outer = Column::new()
        .push(scrollable(col).width(Length::Fill).height(Length::Fill))
        .push(
            ibutton(text("Refresh").size(metrics::UI_PX))
                .width(Length::Fill)
                .padding(Padding::from([metrics::SPACING_02, metrics::SPACING_03]))
                .style(row_style(false))
                .on_press(Message::Refresh),
        )
        .height(Length::Fill)
        .width(Length::Fill);

    container(outer)
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .padding(Padding::from(metrics::SPACING_03))
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(palette::color(
                palette::BUTTON_FACE,
            ))),
            ..Default::default()
        })
        .into()
}

/// The content pane: the selected device's Overview (live name/online/battery), or an
/// honest empty / loading state.
fn content(state: &Phone) -> Element<'_, Message> {
    if !state.loaded {
        return text("Loading devices…").size(metrics::UI_PX).into();
    }
    let Some(d) = state.devices.get(state.selected) else {
        return Column::new()
            .spacing(metrics::SPACING_03)
            .push(
                text("No paired devices")
                    .size(metrics::INFO_TITLE_PX)
                    .font(font::ui_bold()),
            )
            .push(
                text("Devices trusted over KDE Connect appear here once the connect service sees them.")
                    .size(metrics::UI_PX),
            )
            .into();
    };

    let (status, status_role) = if d.online {
        ("● Online", palette::HIGHLIGHT)
    } else {
        ("○ Offline", palette::GRAY_TEXT)
    };
    let battery = match d.battery {
        Some(p) => format!("Battery: {p}%"),
        None => "Battery: unknown".to_string(),
    };

    let card = Column::new()
        .spacing(metrics::SPACING_03)
        .push(
            text(d.name.clone())
                .size(metrics::INFO_TITLE_PX)
                .font(font::ui_bold()),
        )
        .push(
            text(status)
                .size(metrics::UI_PX)
                .color(palette::color(status_role)),
        )
        .push(text(battery).size(metrics::UI_PX))
        .push(Space::with_height(4.0))
        .push(
            text(format!("Device id: {}", d.id))
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        );

    group_box("Overview", card)
}
