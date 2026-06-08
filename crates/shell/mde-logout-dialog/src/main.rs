//! Phase D.2 — Iced GUI for the MDE logout / restart / shutdown
//! confirmation dialog.
//!
//! Layout (per the design spec):
//!
//!   ┌───────────────────────────────────────────────────┐
//!   │  <title — bold, 18 pt>                            │
//!   │                                                   │
//!   │  <body — wraps>                                   │
//!   │                                                   │
//!   │                       [Cancel]   [Primary]        │
//!   └───────────────────────────────────────────────────┘
//!
//! Escape and the Cancel button both return Choice::Cancel. The
//! primary button (Log out / Restart / Shut down) gets the
//! destructive-action style. Exit codes:
//!
//!   * `0`  — user confirmed; the parent should run the action.
//!   * `10` — user cancelled; the parent should do nothing.
//!   * `2`  — bad CLI args (clap exit code).

#![forbid(unsafe_code)]

use std::cell::Cell;

use clap::Parser;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Size, Task, Theme};

use mde_logout_dialog::{
    body, cancel_button_label, exit_code, primary_button_label, title, Action, Choice,
};

#[derive(Debug, Parser)]
#[command(
    name = "mde-logout-dialog",
    about = "MDE logout / restart / shutdown confirmation dialog",
    version
)]
struct Cli {
    /// What action to confirm. One of `logout` | `restart` |
    /// `shutdown`.
    #[arg(long, value_parser = parse_action)]
    action: Action,
}

fn parse_action(s: &str) -> Result<Action, String> {
    Action::from_slug(s)
        .ok_or_else(|| format!("unknown action {s:?} (expected logout|restart|shutdown)"))
}

// Iced 0.13's application() doesn't surface a non-zero process exit
// code, so we shuttle the user's choice through this thread-local
// Cell and translate it into a process exit after the event loop
// returns.
thread_local! {
    static OUTCOME: Cell<Choice> = const { Cell::new(Choice::Cancel) };
}

/// The dialog theme — the canonical IBM Carbon dark palette, sourced from
/// `mde_theme::Palette::dark()` (E9.9, §1 — no raw values here). The old
/// `tokens.css` read is gone: that file is the Material-indigo GTK-era palette
/// (and shared), so seeding from it gave this dialog a non-Carbon look.
fn load_theme() -> Theme {
    let c = mde_theme::Palette::dark();
    let to = |x: mde_theme::Rgba| Color {
        r: f32::from(x.r) / 255.0,
        g: f32::from(x.g) / 255.0,
        b: f32::from(x.b) / 255.0,
        a: 1.0,
    };
    Theme::custom(
        "MDE".to_string(),
        iced::theme::Palette {
            background: to(c.background),
            text: to(c.text),
            primary: to(c.accent),
            success: to(c.success),
            danger: to(c.danger),
        },
    )
}

fn main() -> iced::Result {
    let cli = Cli::parse();
    let action = cli.action;
    let dialog_theme = load_theme();

    let result = iced::application(
        move |state: &State| title(state.action).to_string(),
        move |state: &mut State, message: Message| state.update(message),
        State::view,
    )
    .theme(move |_state: &State| dialog_theme.clone())
    .window_size(Size::new(420.0, 180.0))
    .run_with(move || (State::new(action), Task::none()));

    let choice = OUTCOME.with(Cell::get);
    if result.is_ok() {
        std::process::exit(exit_code(choice));
    }
    result
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Cancel,
    Confirm,
}

struct State {
    action: Action,
}

impl State {
    fn new(action: Action) -> Self {
        Self { action }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let choice = match message {
            Message::Cancel => Choice::Cancel,
            Message::Confirm => Choice::Confirm,
        };
        OUTCOME.with(|c| c.set(choice));
        iced::exit()
    }

    fn view(&self) -> Element<'_, Message> {
        let title_widget = text(title(self.action)).size(18);
        let body_widget = text(body(self.action)).size(14);
        let cancel_btn = button(text(cancel_button_label())).on_press(Message::Cancel);
        let primary_btn =
            button(text(primary_button_label(self.action))).on_press(Message::Confirm);

        let buttons = row![
            Space::with_width(Length::Fill),
            cancel_btn,
            Space::with_width(Length::Fixed(12.0)),
            primary_btn,
        ]
        .align_y(Alignment::Center);

        container(
            column![
                title_widget,
                Space::with_height(Length::Fixed(12.0)),
                body_widget,
                Space::with_height(Length::Fill),
                buttons,
            ]
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
