//! The Windows 10 Action Center pane (E3) — a right-anchored, full-height
//! layer-shell surface that shows the notification history grouped by app, read
//! from the `notifyd` mirror (`~/.config/mde/notifications.json`). Clearing a
//! card (or a whole group) calls the standard `CloseNotification` on the daemon.
//!
//!   mde action-center   open the slide-in pane (Win10 era; WINKEY+A)
//!
//! The quick-action tile grid (E3.5/6) and inline notification actions (which need
//! a daemon action bridge) layer on in later stories.

use std::process::{exit, ExitCode};
use std::time::SystemTime;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, mouse_area, row, scrollable, text, Column, Row, Space};
use iced::{
    event, keyboard, Background, Border, Color, Element, Event, Length, Padding, Shadow, Task,
};
use iced_layershell::build_pattern::{application, MainSettings};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::{to_layer_message, Appearance};

use mde_ui::{metrics, palette};

use crate::notifyd::{self, Notif};

const PANE_W: f32 = 360.0;

struct Center {
    notes: Vec<Notif>,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Clear(u32),         // dismiss one notification
    ClearGroup(String), // dismiss every notification from one app
    Close,
    Event(Event),
}

pub fn run_center(_args: &[String]) -> ExitCode {
    match launch() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde action-center: {e}");
            ExitCode::FAILURE
        }
    }
}

fn launch() -> Result<(), iced_layershell::Error> {
    application(namespace, update, view)
        .style(style)
        .subscription(|_: &Center| {
            event::listen_with(|event, _status, _window| match event {
                Event::Keyboard(_) => Some(Message::Event(event)),
                _ => None,
            })
        })
        .font(mde_ui::font::REGULAR_BYTES)
        .font(mde_ui::font::BOLD_BYTES)
        .font(mde_ui::font::PLEX_REGULAR_BYTES)
        .font(mde_ui::font::PLEX_BOLD_BYTES)
        .default_font(mde_ui::font::ui())
        .settings(MainSettings {
            layer_settings: LayerShellSettings {
                // Full-screen transparent catcher; the pane hugs the right edge.
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                exclusive_zone: 0,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(|| {
            (
                Center {
                    notes: notifyd::load_file().notifications,
                },
                Task::none(),
            )
        })
}

fn namespace(_: &Center) -> String {
    "mde-action-center".to_string()
}

fn style(_: &Center, _: &iced::Theme) -> Appearance {
    Appearance {
        background_color: Color::TRANSPARENT,
        text_color: palette::color(palette::WINDOW_TEXT),
    }
}

fn update(state: &mut Center, message: Message) -> Task<Message> {
    match message {
        Message::Clear(id) => {
            dbus_close(id);
            state.notes.retain(|n| n.id != id);
        }
        Message::ClearGroup(app) => {
            for n in state.notes.iter().filter(|n| n.app_name == app) {
                dbus_close(n.id);
            }
            state.notes.retain(|n| n.app_name != app);
        }
        Message::Close => exit(0),
        Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        })) => exit(0),
        _ => {}
    }
    Task::none()
}

/// Dismiss a notification via the standard freedesktop `CloseNotification` on
/// whatever daemon owns the name (our `notifyd` in the Win10 era).
fn dbus_close(id: u32) {
    if let Ok(conn) = zbus::blocking::Connection::session() {
        if let Ok(proxy) = zbus::blocking::Proxy::new(
            &conn,
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
        ) {
            let _ = proxy.call::<_, _, ()>("CloseNotification", &(id,));
        }
    }
}

// --- view --------------------------------------------------------------------

fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding {
        top: t,
        right: r,
        bottom: b,
        left: l,
    }
}

fn clear_x(msg: Message) -> Element<'static, Message> {
    mouse_area(
        container(
            text("\u{f00d}") // fa-times (×)
                .size(metrics::UI_PX)
                .font(mde_ui::font::NERD)
                .color(palette::color(palette::GRAY_TEXT)),
        )
        .padding(pad(2.0, 6.0, 2.0, 6.0)),
    )
    .on_press(msg)
    .into()
}

fn view(state: &Center) -> Element<'_, Message> {
    let body: Element<Message> = if state.notes.is_empty() {
        container(
            text("No new notifications")
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let mut col = Column::new().spacing(8.0).width(Length::Fill);
        // Group by app, preserving first-seen order; newest cards first within.
        let mut groups: Vec<(String, Vec<&Notif>)> = Vec::new();
        for n in state.notes.iter().rev() {
            match groups.iter_mut().find(|(a, _)| *a == n.app_name) {
                Some((_, v)) => v.push(n),
                None => groups.push((n.app_name.clone(), vec![n])),
            }
        }
        for (app, notes) in groups {
            // Group header: app icon + name + a group Clear "x".
            let label: String = if app.is_empty() {
                "Notifications".into()
            } else {
                app.clone()
            };
            col = col.push(
                Row::new()
                    .align_y(Vertical::Center)
                    .push(crate::icons::icon_any(
                        &[app.to_lowercase().as_str(), "dialog-information"],
                        16,
                    ))
                    .push(Space::with_width(Length::Fixed(6.0)))
                    .push(
                        text(label)
                            .size(metrics::UI_PX)
                            .font(mde_ui::font::ui_bold())
                            .width(Length::Fill),
                    )
                    .push(clear_x(Message::ClearGroup(app.clone()))),
            );
            for n in notes {
                col = col.push(card(n));
            }
        }
        scrollable(col).style(mde_ui::scrollbar).into()
    };

    // The flat dark pane: a fixed-width column on a Carbon/Win10 layer surface.
    let pane = container(container(body).padding(10.0))
        .width(Length::Fixed(PANE_W))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(palette::color(palette::MENU))),
            border: Border {
                color: palette::color(palette::WINDOW_FRAME),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color {
                    a: 0.35,
                    ..Color::BLACK
                },
                offset: iced::Vector::new(-2.0, 0.0),
                blur_radius: 12.0,
            },
            ..container::Style::default()
        });

    iced::widget::stack![
        mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Message::Close),
        container(pane)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Top),
    ]
    .into()
}

/// One notification card: icon + summary (bold) + body + relative time, with a
/// per-card Clear "x".
fn card(n: &Notif) -> Element<'static, Message> {
    let head = Row::new()
        .align_y(Vertical::Center)
        .push(
            text(n.summary.clone())
                .size(metrics::UI_PX)
                .font(mde_ui::font::ui_bold())
                .width(Length::Fill),
        )
        .push(
            text(rel_time(n.timestamp))
                .size(metrics::UI_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        )
        .push(clear_x(Message::Clear(n.id)));
    let mut inner = Column::new().spacing(2.0).width(Length::Fill).push(head);
    if !n.body.is_empty() {
        inner = inner.push(
            text(n.body.clone())
                .size(metrics::UI_PX)
                .color(palette::color(palette::WINDOW_TEXT)),
        );
    }
    let icon = crate::icons::icon_any(&[n.app_icon.as_str(), "dialog-information"], 24);
    container(row![icon, inner].spacing(8.0).align_y(Vertical::Top))
        .width(Length::Fill)
        .padding(8.0)
        .style(|_| container::Style {
            background: Some(Background::Color(palette::color(palette::WINDOW))),
            border: Border {
                color: palette::color(palette::WINDOW_FRAME),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// A coarse "Nm ago" / "Nh ago" / "now" relative timestamp.
fn rel_time(t: SystemTime) -> String {
    match SystemTime::now().duration_since(t) {
        Ok(d) => {
            let s = d.as_secs();
            if s < 60 {
                "now".to_string()
            } else if s < 3600 {
                format!("{}m ago", s / 60)
            } else if s < 86_400 {
                format!("{}h ago", s / 3600)
            } else {
                format!("{}d ago", s / 86_400)
            }
        }
        Err(_) => "now".to_string(),
    }
}
