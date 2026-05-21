//! `mde-applet-drawer` — Iced drawer overlay binary.
//!
//! Phase E.8.1 + E.8.2 skeleton. Boots an Iced window with the
//! four locked drawer sections; per-section interactivity wires
//! in as Phase E.8.2 + E.2 (layer-shell) complete.

#![forbid(unsafe_code)]

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding, Size, Theme};

use mde_drawer::{DrawerSection, QuickToggle, DRAWER_WIDTH_PX};

#[derive(Debug, Clone)]
enum Message {
    Dismiss,
    ToggleQuickAction(QuickToggle),
}

#[derive(Default)]
struct DrawerApp;

impl DrawerApp {
    fn run() -> iced::Result {
        iced::application(Self::title, Self::update, Self::view)
            .theme(Self::theme)
            .window_size(Size::new(f32::from(DRAWER_WIDTH_PX), 1080.0))
            .run()
    }

    fn title(&self) -> String {
        "MDE drawer".into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(16).padding(Padding {
            top: 24.0,
            right: 16.0,
            bottom: 24.0,
            left: 16.0,
        });
        for section in DrawerSection::ordered() {
            col = col.push(section_header(section));
            col = col.push(section_body(section));
            col = col.push(Space::with_height(Length::Fixed(8.0)));
        }
        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn section_header<'a>(section: DrawerSection) -> Element<'a, Message> {
    text(section.label()).size(18).into()
}

fn section_body<'a>(section: DrawerSection) -> Element<'a, Message> {
    match section {
        DrawerSection::QuickActions => quick_actions_body(),
        DrawerSection::Sliders => placeholder("Brightness · Volume sliders (Phase E.6 wiring)"),
        DrawerSection::Notifications => placeholder("Unread notifications (Phase E.8.2 wiring)"),
        DrawerSection::Hardware => placeholder("Battery · CPU · Network (upower over zbus)"),
    }
}

fn quick_actions_body<'a>() -> Element<'a, Message> {
    let mut r = row![].spacing(8).align_y(Alignment::Center);
    for toggle in QuickToggle::ordered() {
        r = r.push(text(format!("[{}]", toggle.label())).size(14));
    }
    r.into()
}

fn placeholder<'a>(text_body: &'static str) -> Element<'a, Message> {
    text(text_body).size(13).into()
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("MDE_DRAWER_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("mde_drawer=info,warn")),
        )
        .init();
    DrawerApp::run()
}
