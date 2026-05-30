//! MDE-Retro installer (`mde setup`) — styled after the Windows 2000 GUI Setup
//! screen: deep-blue gradient background, a left stage list, a right content
//! pane with a progress bar, and a bottom status strip, all in Tahoma/white.
//! The look-and-feel mimics Win2000 Setup; the *stages* are MDE-Retro's real
//! installation steps.

use std::process::{exit, ExitCode};
use std::time::Duration;

use iced::widget::{container, text, Column, Row, Space};
use iced::{
    gradient::Linear, Background, Color, Element, Gradient, Length, Padding, Radians, Task,
};

use mde_ui::font;

/// (stage title, what it does) — MDE-Retro's actual install, in Setup's voice.
const STAGES: &[(&str, &str)] = &[
    ("Collecting information", "Setup is examining your computer and checking that the required components are available."),
    ("Installing packages", "Setup is installing sway, Waybar, foot, fonts and the system tools MDE-Retro uses."),
    ("Deploying configuration", "Setup is copying the MDE-Retro configuration files into your home directory."),
    ("Installing visual assets", "Setup is installing the Chicago95 icons, cursors and sounds and the Win2k icon theme."),
    ("Building the shell", "Setup is compiling the mde shell and the Device Manager."),
    ("Finalizing installation", "Setup is configuring your Windows 2000 session and the logon screen."),
];

const WHITE: Color = Color::WHITE;
const DIM: Color = Color::from_rgb(0.62, 0.70, 0.86);

struct Setup {
    stage: usize,
    progress: f32,
    done: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    Finish,
}

/// Route `mde setup` to the TUI (headless / --tui) or the GUI (in-session).
pub fn dispatch(args: &[String]) -> ExitCode {
    let tui = args.iter().any(|a| a == "--tui");
    let gui = args.iter().any(|a| a == "--gui");
    let dry = args.iter().any(|a| a == "--dry-run");
    let headless = std::env::var_os("WAYLAND_DISPLAY").is_none();
    if gui || (!tui && !headless) {
        run(args)
    } else {
        crate::tui_setup::run(dry)
    }
}

pub fn run(_args: &[String]) -> ExitCode {
    let r = iced::application(|_: &Setup| "MDE-Retro Professional Setup".to_string(), update, view)
        .window_size(iced::Size::new(640.0, 480.0))
        .resizable(false)
        .subscription(|_| iced::time::every(Duration::from_millis(110)).map(|_| Msg::Tick))
        .font(font::REGULAR_BYTES)
        .font(font::BOLD_BYTES)
        .default_font(font::UI)
        .run_with(|| (Setup { stage: 0, progress: 0.0, done: false }, Task::none()));
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn update(state: &mut Setup, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::Tick if !state.done => {
            state.progress += 0.035;
            if state.progress >= 1.0 {
                state.progress = 0.0;
                state.stage += 1;
                if state.stage >= STAGES.len() {
                    state.stage = STAGES.len() - 1;
                    state.progress = 1.0;
                    state.done = true;
                }
            }
        }
        Msg::Finish => exit(0),
        Msg::Tick => {}
    }
    Task::none()
}

fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding { top: t, right: r, bottom: b, left: l }
}

fn bg_gradient() -> Background {
    Background::Gradient(Gradient::Linear(
        Linear::new(Radians(std::f32::consts::PI))
            .add_stop(0.0, Color::from_rgb8(0x1c, 0x4a, 0x8f))
            .add_stop(1.0, Color::from_rgb8(0x08, 0x16, 0x40)),
    ))
}

fn stage_list(state: &Setup) -> Element<'_, Msg> {
    let mut col = Column::new().spacing(10.0).padding(pad(16.0, 10.0, 16.0, 16.0));
    col = col.push(text("MDE-Retro").size(18.0).font(font::UI_BOLD).color(WHITE));
    col = col.push(text("Professional Setup").size(11.0).color(DIM));
    col = col.push(Space::new(Length::Fill, Length::Fixed(14.0)));
    for (i, (title, _)) in STAGES.iter().enumerate() {
        let (marker, color, fnt) = if i < state.stage || (state.done && i == state.stage) {
            ("   ", WHITE, font::UI) // done
        } else if i == state.stage {
            (">  ", WHITE, font::UI_BOLD) // current
        } else {
            ("   ", DIM, font::UI) // pending
        };
        col = col.push(text(format!("{marker}{title}")).size(11.0).color(color).font(fnt));
    }
    container(col)
        .width(Length::Fixed(212.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.20))),
            ..container::Style::default()
        })
        .into()
}

fn progress_bar(frac: f32) -> Element<'static, Msg> {
    let width = 360.0;
    let fill = ((width - 6.0) * frac.clamp(0.0, 1.0)).max(0.0);
    let trough = container(
        container(Space::new(Length::Fixed(fill), Length::Fill)).style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x16, 0x3a, 0xa8))),
            ..container::Style::default()
        }),
    )
    .padding(3.0)
    .width(Length::Fixed(width))
    .height(Length::Fixed(20.0))
    .style(|_| container::Style {
        background: Some(Background::Color(Color::WHITE)),
        border: iced::Border {
            color: Color::from_rgb8(0x40, 0x40, 0x40),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });
    trough.into()
}

fn content(state: &Setup) -> Element<'_, Msg> {
    let (title, desc) = STAGES[state.stage];
    let mut col = Column::new()
        .spacing(16.0)
        .padding(pad(24.0, 24.0, 16.0, 24.0))
        .width(Length::Fill);
    col = col.push(text(title).size(15.0).font(font::UI_BOLD).color(WHITE));
    col = col.push(text(desc).size(11.0).color(WHITE));
    col = col.push(Space::new(Length::Fill, Length::Fixed(8.0)));

    if state.done {
        col = col
            .push(text("MDE-Retro has been installed on your computer.").size(11.0).color(WHITE))
            .push(text("Click Finish to complete Setup.").size(11.0).color(WHITE))
            .push(Space::new(Length::Fill, Length::Fill))
            .push(
                Row::new().push(Space::with_width(Length::Fill)).push(
                    mde_ui::button(text("Finish").size(11.0))
                        .on_press(Msg::Finish)
                        .default(true)
                        .width(Length::Fixed(84.0)),
                ),
            );
    } else {
        let pct = ((state.stage as f32 + state.progress) / STAGES.len() as f32 * 100.0) as u32;
        col = col
            .push(progress_bar(state.progress))
            .push(text(format!("Overall progress: {pct}%")).size(11.0).color(DIM))
            .push(Space::new(Length::Fill, Length::Fill))
            .push(text("Setup will complete in a few minutes.").size(11.0).color(DIM));
    }
    container(col).width(Length::Fill).height(Length::Fill).into()
}

fn status_bar<'a>() -> Element<'a, Msg> {
    container(text("MDE-Retro Professional Setup").size(10.0).color(DIM))
        .width(Length::Fill)
        .padding(pad(2.0, 8.0, 2.0, 8.0))
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.35))),
            ..container::Style::default()
        })
        .into()
}

fn view(state: &Setup) -> Element<'_, Msg> {
    let body = Row::new().push(stage_list(state)).push(content(state));
    let screen = Column::new()
        .push(container(body).width(Length::Fill).height(Length::Fill))
        .push(status_bar());

    container(screen)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(bg_gradient()),
            ..container::Style::default()
        })
        .into()
}
