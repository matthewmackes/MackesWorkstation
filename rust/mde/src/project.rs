//! The "Project" pane — `mde project`, bound to Win+P (E12.10).
//!
//! A right-docked layer-shell flyout (the `net_flyout.rs` pattern) with the four
//! Windows projection modes. Each builds a [`outputs::Desired`] from the live
//! outputs and applies it through [`outputs::apply_live`] (transient, like Win10 —
//! not persisted, so a reboot returns to the saved Display layout):
//!   - **PC screen only** — primary on at 0,0; the second output off.
//!   - **Duplicate** — both outputs at 0,0 (mirrored).
//!   - **Extend** — primary at 0,0; the second output at x = primary width.
//!   - **Second screen only** — primary off; the second output on at 0,0.
//!
//! Win10-era only (the classic eras use Display Properties); the keybind is inert
//! under them, like the other Win10 flyouts.

use std::process::{exit, ExitCode};

use iced::widget::{button, container, text, Column};
use iced::{Color, Element, Length, Task};
use iced_layershell::build_pattern::{application, MainSettings};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::{to_layer_message, Appearance};

use mde_ui::{metrics, palette};

use crate::outputs::{self, Desired, DesiredOutput, Output};

/// The four Windows projection modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    PcOnly,
    Duplicate,
    Extend,
    SecondOnly,
}

impl Mode {
    const ALL: [Mode; 4] = [
        Mode::PcOnly,
        Mode::Duplicate,
        Mode::Extend,
        Mode::SecondOnly,
    ];
    fn label(self) -> &'static str {
        match self {
            Mode::PcOnly => "PC screen only",
            Mode::Duplicate => "Duplicate",
            Mode::Extend => "Extend",
            Mode::SecondOnly => "Second screen only",
        }
    }
}

/// Position/enable the desired outputs for `mode`, in place. Pure — unit-tested.
/// `douts[0]` is the primary; `douts[1]` (if any) the second screen.
pub fn arrange(mode: Mode, douts: &mut [DesiredOutput]) {
    if douts.is_empty() {
        return;
    }
    let primary_w = douts[0].width;
    let two = douts.len() >= 2;
    match mode {
        Mode::PcOnly => {
            douts[0].enabled = true;
            douts[0].x = 0;
            douts[0].y = 0;
            if two {
                douts[1].enabled = false;
            }
        }
        Mode::Duplicate => {
            for d in douts.iter_mut() {
                d.enabled = true;
                d.x = 0;
                d.y = 0;
            }
        }
        Mode::Extend => {
            douts[0].enabled = true;
            douts[0].x = 0;
            douts[0].y = 0;
            if two {
                douts[1].enabled = true;
                douts[1].x = primary_w;
                douts[1].y = 0;
            }
        }
        Mode::SecondOnly => {
            if two {
                douts[0].enabled = false;
                douts[1].enabled = true;
                douts[1].x = 0;
                douts[1].y = 0;
            } else {
                // Only one screen — never black it out.
                douts[0].enabled = true;
                douts[0].x = 0;
                douts[0].y = 0;
            }
        }
    }
}

/// Build the full `Desired` for `mode` from the live outputs.
pub fn build(mode: Mode, live: &[Output]) -> Desired {
    let mut douts = outputs::desired_from(live);
    arrange(mode, &mut douts);
    Desired {
        outputs: douts,
        wallpaper: None,
        screensaver: None,
        scheme: None,
    }
}

struct Project {
    outputs: Vec<Output>,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Pick(Mode),
    Close,
}

pub fn run(_args: &[String]) -> ExitCode {
    // Win10-era only; classic eras use Display Properties (the keybind is inert).
    if !palette::is_windows10() || std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return ExitCode::SUCCESS;
    }
    match launch() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde project: {e}");
            ExitCode::FAILURE
        }
    }
}

fn launch() -> Result<(), iced_layershell::Error> {
    application(namespace, update, view)
        .style(style)
        .subscription(|_: &Project| {
            iced::event::listen_with(|event, _s, _w| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::Close),
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
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Right,
                size: Some((360, 0)),
                exclusive_zone: 0,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(|| {
            (
                Project {
                    outputs: outputs::query(),
                },
                Task::none(),
            )
        })
}

fn namespace(_: &Project) -> String {
    "mde-project".to_string()
}

fn style(_: &Project, _: &iced::Theme) -> Appearance {
    Appearance {
        background_color: Color::TRANSPARENT,
        text_color: palette::color(palette::WINDOW_TEXT),
    }
}

fn update(state: &mut Project, message: Message) -> Task<Message> {
    match message {
        Message::Pick(mode) => {
            outputs::apply_live(&build(mode, &state.outputs));
            exit(0);
        }
        Message::Close => exit(0),
        _ => {}
    }
    Task::none()
}

fn mode_button(mode: Mode) -> Element<'static, Message> {
    button(
        text(mode.label())
            .size(metrics::UI_PX)
            .color(palette::color(palette::WINDOW_TEXT)),
    )
    .on_press(Message::Pick(mode))
    .width(Length::Fill)
    .padding(iced::Padding::from([12.0, 14.0]))
    .style(mde_ui::button_ghost)
    .into()
}

fn view(state: &Project) -> Element<'_, Message> {
    let mut col = Column::new().spacing(2.0).push(
        text("Project")
            .size(metrics::INFO_TITLE_PX)
            .color(palette::color(palette::WINDOW_TEXT)),
    );
    if state.outputs.len() < 2 {
        col = col.push(
            text("Only one display detected — connect another to extend or duplicate.")
                .size(metrics::BADGE_PX)
                .color(palette::color(palette::GRAY_TEXT)),
        );
    }
    for mode in Mode::ALL {
        col = col.push(mode_button(mode));
    }

    let card = container(col.spacing(6.0).padding(16.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(palette::color(palette::WINDOW).into()),
            text_color: Some(palette::color(palette::WINDOW_TEXT)),
            border: iced::Border {
                color: palette::color(palette::WINDOW_FRAME),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });
    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(name: &str, w: i32) -> DesiredOutput {
        DesiredOutput {
            name: name.into(),
            width: w,
            height: 1080,
            refresh_mhz: 60000,
            scale: 1.0,
            transform: "normal".into(),
            x: 999,
            y: 999,
            enabled: true,
        }
    }

    #[test]
    fn extend_places_second_right_of_primary() {
        let mut d = vec![out("DP-1", 1920), out("HDMI-A-1", 1280)];
        arrange(Mode::Extend, &mut d);
        assert!(d[0].enabled && d[1].enabled);
        assert_eq!((d[0].x, d[0].y), (0, 0));
        assert_eq!(d[1].x, 1920); // second screen at x == primary width
        assert_eq!(d[1].y, 0);
    }

    #[test]
    fn pc_only_disables_second() {
        let mut d = vec![out("DP-1", 1920), out("HDMI-A-1", 1280)];
        arrange(Mode::PcOnly, &mut d);
        assert!(d[0].enabled);
        assert!(!d[1].enabled);
        assert_eq!((d[0].x, d[0].y), (0, 0));
    }

    #[test]
    fn second_only_disables_primary() {
        let mut d = vec![out("DP-1", 1920), out("HDMI-A-1", 1280)];
        arrange(Mode::SecondOnly, &mut d);
        assert!(!d[0].enabled);
        assert!(d[1].enabled);
        assert_eq!((d[1].x, d[1].y), (0, 0));
    }

    #[test]
    fn duplicate_overlaps_both_at_origin() {
        let mut d = vec![out("DP-1", 1920), out("HDMI-A-1", 1280)];
        arrange(Mode::Duplicate, &mut d);
        assert!(d[0].enabled && d[1].enabled);
        assert_eq!((d[0].x, d[0].y), (0, 0));
        assert_eq!((d[1].x, d[1].y), (0, 0));
    }

    #[test]
    fn single_display_never_blacks_out() {
        let mut d = vec![out("DP-1", 1920)];
        arrange(Mode::SecondOnly, &mut d);
        assert!(d[0].enabled); // can't disable the only screen
    }
}
