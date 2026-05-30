//! Start menu — a layer-shell popup anchored bottom-left, above the taskbar.
//!
//! A raised Win2000 menu: a navy side-banner on the left, then the system tools
//! grouped by category (from `fedora.rs`) as flat items that highlight navy on
//! hover, plus Log Off / Restart / Shut Down. Launching a tool (CLI tools open
//! in foot at 150%) or pressing Esc closes the menu (the process exits).

use std::process::{exit, Command, ExitCode};

use iced::widget::{button, container, scrollable, text, Column, Row, Space};
use iced::{event, keyboard, Background, Border, Element, Event, Length, Padding, Shadow, Task};
use iced_layershell::build_pattern::{application, MainSettings};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::{to_layer_message, Appearance};

use mde_ui::{frame, metrics, palette};

use crate::fedora;

const BOLD: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..iced::Font::DEFAULT
};

#[derive(Default)]
struct Menu;

#[derive(Debug, Clone, Copy)]
enum Power {
    LogOff,
    Restart,
    Shutdown,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Launch(usize),
    Power(Power),
    Event(Event),
}

pub fn run(_args: &[String]) -> ExitCode {
    match launch() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde menu: {e}");
            ExitCode::FAILURE
        }
    }
}

fn launch() -> Result<(), iced_layershell::Error> {
    application(namespace, update, view)
        .style(style)
        .subscription(subscription)
        .settings(MainSettings {
            layer_settings: LayerShellSettings {
                size: Some((230, 460)),
                exclusive_zone: 0,
                anchor: Anchor::Bottom | Anchor::Left,
                margin: (0, 0, metrics::TASKBAR_HEIGHT as i32, 0),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

fn namespace(_state: &Menu) -> String {
    "mde-menu".to_string()
}

fn style(_state: &Menu, _theme: &iced::Theme) -> Appearance {
    Appearance {
        background_color: palette::color(palette::MENU),
        text_color: palette::color(palette::MENU_TEXT),
    }
}

fn subscription(_state: &Menu) -> iced::Subscription<Message> {
    event::listen().map(Message::Event)
}

fn update(_state: &mut Menu, message: Message) -> Task<Message> {
    match message {
        Message::Launch(i) => {
            if let Some(tool) = fedora::TOOLS.get(i) {
                let _ = fedora::launch(tool);
            }
            exit(0);
        }
        Message::Power(p) => {
            match p {
                Power::LogOff => drop(Command::new("swaymsg").arg("exit").spawn()),
                Power::Restart => drop(Command::new("systemctl").arg("reboot").spawn()),
                Power::Shutdown => drop(Command::new("systemctl").arg("poweroff").spawn()),
            }
            exit(0);
        }
        Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        })) => exit(0),
        _ => Task::none(),
    }
}

fn menu_item_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let hot = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: hot.then(|| Background::Color(palette::color(palette::HIGHLIGHT))),
        text_color: if hot {
            palette::color(palette::HIGHLIGHT_TEXT)
        } else {
            palette::color(palette::MENU_TEXT)
        },
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

fn pad(top: f32, right: f32, bottom: f32, left: f32) -> Padding {
    Padding { top, right, bottom, left }
}

fn item<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(text(label).size(11.0))
        .on_press(message)
        .width(Length::Fill)
        .padding(pad(2.0, 10.0, 2.0, 10.0))
        .style(menu_item_style)
        .into()
}

fn header(label: &str) -> Element<'_, Message> {
    container(text(label).size(11.0).font(BOLD))
        .padding(pad(4.0, 8.0, 1.0, 8.0))
        .into()
}

fn separator() -> Element<'static, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .padding(pad(3.0, 6.0, 3.0, 6.0))
        .into()
}

fn view(_state: &Menu) -> Element<'_, Message> {
    // Left navy banner (rotated "Windows 2000" text is a later refinement).
    let banner = container(Space::new(Length::Fixed(0.0), Length::Fill))
        .width(Length::Fixed(24.0))
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(palette::color(palette::ACTIVE_TITLE))),
            ..container::Style::default()
        });

    let mut list = Column::new().spacing(0).padding([4, 2]);
    for category in fedora::categories() {
        list = list.push(header(category));
        for (i, tool) in fedora::TOOLS.iter().enumerate() {
            if tool.category == category {
                list = list.push(item(tool.name, Message::Launch(i)));
            }
        }
    }
    list = list
        .push(separator())
        .push(item("Log Off...", Message::Power(Power::LogOff)))
        .push(item("Restart", Message::Power(Power::Restart)))
        .push(item("Shut Down...", Message::Power(Power::Shutdown)));

    let body = Row::new()
        .push(banner)
        .push(scrollable(list).width(Length::Fill).height(Length::Fill));

    iced::widget::stack![frame::raised(), body].into()
}
