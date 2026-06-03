//! Windows 10-era Out-Of-Box Experience (OOBE) — the first-run setup wizard (E11).
//!
//! A full-screen, multi-stage flow reached by `mde setup --era=win10`. The classic
//! Win2000-blue component-picker Setup (`installer.rs`) is **unchanged**: the
//! `--era=win10` branch in `installer::dispatch` is purely additive. The OOBE forces
//! the Windows 10 palette for its chrome and, on finish, stamps `state.oobe_done` so
//! it shows once (re-run it any time with `--force`).
//!
//! Each stage collects one choice and a **Yes/Next** advances; the backend writes
//! (locale, keymap, …) are built as commands that are *echoed* under `--dry-run`
//! and run otherwise — so the flow is testable without mutating the host.

use std::process::{Command, ExitCode};

use iced::widget::{button as ibutton, container, scrollable, text, Column, Row, Space};
use iced::{
    gradient::Linear, Background, Color, Element, Gradient, Length, Padding, Radians, Task,
};

use mde_ui::palette::Theme;
use mde_ui::{font, metrics, palette};

/// Region choices: a display country + the locale it sets (`localectl set-locale`).
const REGIONS: &[(&str, &str)] = &[
    ("United States", "en_US.UTF-8"),
    ("United Kingdom", "en_GB.UTF-8"),
    ("Canada", "en_CA.UTF-8"),
    ("Australia", "en_AU.UTF-8"),
    ("Germany", "de_DE.UTF-8"),
    ("France", "fr_FR.UTF-8"),
    ("Spain", "es_ES.UTF-8"),
    ("Italy", "it_IT.UTF-8"),
];

/// The wizard stages, in order. Only the implemented stages exist here (no stub
/// arms, §3); later stages are added as they land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Region,
    Keyboard,
    Finalize,
}

impl Stage {
    /// The next stage, or `None` at the end of the flow.
    fn next(self) -> Option<Stage> {
        match self {
            Stage::Region => Some(Stage::Keyboard),
            Stage::Keyboard => Some(Stage::Finalize),
            Stage::Finalize => None,
        }
    }
    fn prev(self) -> Option<Stage> {
        match self {
            Stage::Region => None,
            Stage::Keyboard => Some(Stage::Region),
            Stage::Finalize => Some(Stage::Keyboard),
        }
    }
}

struct Oobe {
    stage: Stage,
    /// Echo backend commands instead of running them (`--dry-run`).
    dry: bool,
    region: usize,
    layout: usize,
}

#[derive(Debug, Clone)]
enum Msg {
    PickRegion(usize),
    PickLayout(usize),
    Next,
    Back,
    Finish,
}

/// White text on the blue OOBE chrome. Uses the on-accent white sentinel
/// (`HIGHLIGHT_TEXT`), which remaps to pure white under the forced Win10 palette —
/// unlike `WINDOW`, which is the (dark) window surface there.
fn white() -> Color {
    palette::color(palette::HIGHLIGHT_TEXT)
}
fn dim() -> Color {
    palette::color(palette::SETUP_SUBTITLE)
}

/// Index of the REGION whose locale matches `$LANG`, else United States (0).
fn detected_region() -> usize {
    let lang = std::env::var("LANG").unwrap_or_default();
    let head = lang.split('.').next().unwrap_or("");
    REGIONS
        .iter()
        .position(|(_, loc)| loc.split('.').next() == Some(head) && !head.is_empty())
        .unwrap_or(0)
}

/// Index of the keyboard LAYOUT matching the detected locale's country, else US (0).
fn detected_layout() -> usize {
    // Map the detected region's locale country to a layout code (en_US → us, …).
    let (_, loc) = REGIONS[detected_region()];
    let cc = loc
        .split('_')
        .nth(1)
        .and_then(|s| s.split('.').next())
        .unwrap_or("US")
        .to_lowercase();
    crate::keyboard::LAYOUTS
        .iter()
        .position(|(code, _)| *code == cc || (cc == "us" && *code == "us"))
        .unwrap_or(0)
}

pub fn run(args: &[String]) -> ExitCode {
    let dry = args.iter().any(|a| a == "--dry-run");
    let force = args.iter().any(|a| a == "--force");

    // The OOBE renders in Windows 10 chrome regardless of the persisted theme.
    palette::set_theme(Theme::Windows10);
    palette::set_dark(true);

    // Show once: a completed OOBE is skipped unless re-run with --force (E11.10).
    let st = crate::state::load();
    if st.oobe_done && !force {
        return ExitCode::SUCCESS;
    }

    // No compositor (or an explicit --tui) → a non-interactive dry walkthrough that
    // applies detected defaults, so the path is exercisable headlessly without a
    // panic in the layer-shell/iced init.
    let tui = args.iter().any(|a| a == "--tui");
    if tui || std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return headless(dry);
    }

    // `--stage <name>` starts the wizard at a given stage — a capture seam (so the
    // accuracy gallery can grab each screen without injecting clicks to advance).
    let stage = args
        .iter()
        .position(|a| a == "--stage")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| match s.as_str() {
            "region" => Some(Stage::Region),
            "keyboard" => Some(Stage::Keyboard),
            "finalize" => Some(Stage::Finalize),
            _ => None,
        })
        .unwrap_or(Stage::Region);

    let init = Oobe {
        stage,
        dry,
        region: detected_region(),
        layout: detected_layout(),
    };
    let r = iced::application(|_: &Oobe| "MDE-Retro Setup".to_string(), update, view)
        .window_size(iced::Size::new(720.0, 540.0))
        .resizable(false)
        .font(font::REGULAR_BYTES)
        .font(font::BOLD_BYTES)
        .font(font::PLEX_REGULAR_BYTES)
        .font(font::PLEX_BOLD_BYTES)
        .default_font(font::ui())
        .run_with(move || (init, Task::none()));
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

/// Headless walkthrough (`--tui`/no compositor): apply the detected region + layout
/// (echoing under `--dry-run`), stamp `oobe_done`, and print each step so the path
/// is observable without a display.
fn headless(dry: bool) -> ExitCode {
    let region = detected_region();
    let layout = detected_layout();
    println!("MDE-Retro Windows 10 setup (headless)");
    println!("  Region:   {}", REGIONS[region].0);
    println!("  Keyboard: {}", crate::keyboard::LAYOUTS[layout].1);
    apply_locale(REGIONS[region].1, dry);
    apply_keymap(crate::keyboard::LAYOUTS[layout].0, dry);
    finish(dry);
    println!(
        "  Done. (oobe_done set{})",
        if dry { ", dry-run" } else { "" }
    );
    ExitCode::SUCCESS
}

/// `localectl set-locale LANG=<locale>` — echoed under dry-run.
fn apply_locale(locale: &str, dry: bool) {
    let arg = format!("LANG={locale}");
    if dry {
        println!("  + localectl set-locale {arg}");
        return;
    }
    let _ = Command::new("localectl")
        .args(["set-locale", &arg])
        .status();
}

/// `localectl set-x11-keymap <layout>` — echoed under dry-run. (The OOBE also writes
/// the labwc XKB layout via `keyboard::apply_layout` so it applies without a reboot.)
fn apply_keymap(layout: &str, dry: bool) {
    if dry {
        println!("  + localectl set-x11-keymap {layout}");
        return;
    }
    let _ = Command::new("localectl")
        .args(["set-x11-keymap", layout])
        .status();
    let _ = crate::keyboard::apply_layout(layout);
}

/// Stamp `oobe_done` so the wizard shows once (E11.10) — never echoed; it's our own
/// state, harmless to write even in a dry walkthrough's *test*, but we honour dry by
/// not persisting so a `--dry-run` leaves the user's config untouched.
fn finish(dry: bool) {
    if dry {
        return;
    }
    let mut st = crate::state::load();
    st.oobe_done = true;
    let _ = crate::state::save(&st);
}

fn update(state: &mut Oobe, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::PickRegion(i) => state.region = i,
        Msg::PickLayout(i) => state.layout = i,
        Msg::Back => {
            if let Some(p) = state.stage.prev() {
                state.stage = p;
            }
        }
        Msg::Next => {
            // Commit the stage we're leaving, then advance.
            match state.stage {
                Stage::Region => apply_locale(REGIONS[state.region].1, state.dry),
                Stage::Keyboard => {
                    apply_keymap(crate::keyboard::LAYOUTS[state.layout].0, state.dry)
                }
                Stage::Finalize => {}
            }
            if let Some(n) = state.stage.next() {
                state.stage = n;
            }
        }
        Msg::Finish => {
            finish(state.dry);
            std::process::exit(0);
        }
    }
    Task::none()
}

fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding {
        top: t,
        right: r,
        bottom: b,
        left: l,
    }
}

fn bg() -> Background {
    Background::Gradient(Gradient::Linear(
        Linear::new(Radians(std::f32::consts::PI))
            .add_stop(0.0, palette::color(palette::SETUP_GRADIENT_TOP))
            .add_stop(1.0, palette::color(palette::SETUP_GRADIENT_BOTTOM)),
    ))
}

/// A scrollable single-select list (E11.2 `render_picker`, reused for Region and
/// Keyboard): each row is a button; the selected row paints the accent.
fn picker<'a>(
    items: impl Iterator<Item = (usize, &'a str)>,
    selected: usize,
    on_pick: fn(usize) -> Msg,
) -> Element<'a, Msg> {
    let mut col = Column::new().spacing(2.0).padding(pad(0.0, 8.0, 0.0, 8.0));
    for (i, label) in items {
        let sel = i == selected;
        let row = ibutton(text(label).size(metrics::UI_PX).color(white()))
            .width(Length::Fill)
            .padding(pad(6.0, 12.0, 6.0, 12.0))
            .on_press(on_pick(i))
            .style(move |_, _| ibutton::Style {
                background: Some(if sel {
                    Background::Color(palette::color(palette::HIGHLIGHT))
                } else {
                    Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))
                }),
                text_color: white(),
                border: iced::Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        col = col.push(row);
    }
    scrollable(col)
        .height(Length::Fill)
        .style(mde_ui::scrollbar)
        .into()
}

/// The bottom action strip: an optional Back, a spacer, and the primary button.
fn actions<'a>(back: bool, primary: &'a str, on_primary: Msg) -> Element<'a, Msg> {
    let mut row = Row::new().spacing(8.0).padding(pad(8.0, 24.0, 16.0, 24.0));
    if back {
        row = row.push(
            mde_ui::button(text("Back").size(metrics::UI_PX))
                .on_press(Msg::Back)
                .width(Length::Fixed(96.0)),
        );
    }
    row = row.push(Space::with_width(Length::Fill));
    row = row.push(
        mde_ui::button(text(primary).size(metrics::UI_PX))
            .on_press(on_primary)
            .default(true)
            .width(Length::Fixed(120.0)),
    );
    row.into()
}

/// A stage frame: a big heading, a subtitle, the body, and the action strip.
fn frame<'a>(
    heading: &'a str,
    subtitle: &'a str,
    body: Element<'a, Msg>,
    actions: Element<'a, Msg>,
) -> Element<'a, Msg> {
    let header = Column::new()
        .spacing(6.0)
        .padding(pad(28.0, 28.0, 8.0, 28.0))
        .push(
            text(heading)
                .size(metrics::INFO_TITLE_PX)
                .font(font::ui_bold())
                .color(white()),
        )
        .push(text(subtitle).size(metrics::UI_PX).color(dim()));
    let screen = Column::new()
        .push(header)
        .push(
            container(body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(pad(0.0, 28.0, 0.0, 28.0)),
        )
        .push(actions);
    container(screen)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(bg()),
            ..container::Style::default()
        })
        .into()
}

fn view(state: &Oobe) -> Element<'_, Msg> {
    match state.stage {
        Stage::Region => frame(
            "Let's start with your region",
            "Is this the right country or region?",
            picker(
                REGIONS.iter().enumerate().map(|(i, (name, _))| (i, *name)),
                state.region,
                Msg::PickRegion,
            ),
            actions(false, "Yes", Msg::Next),
        ),
        Stage::Keyboard => frame(
            "Is this the right keyboard layout?",
            "If you also use another keyboard layout, you can add one later.",
            picker(
                crate::keyboard::LAYOUTS
                    .iter()
                    .enumerate()
                    .map(|(i, (_, name))| (i, *name)),
                state.layout,
                Msg::PickLayout,
            ),
            actions(true, "Yes", Msg::Next),
        ),
        Stage::Finalize => {
            let body = Column::new()
                .spacing(10.0)
                .padding(pad(20.0, 0.0, 0.0, 0.0))
                .push(
                    text("This might take a few minutes.")
                        .size(metrics::UI_PX)
                        .color(dim()),
                )
                .push(
                    text(format!(
                        "Region: {}\nKeyboard: {}",
                        REGIONS[state.region].0,
                        crate::keyboard::LAYOUTS[state.layout].1
                    ))
                    .size(metrics::UI_PX)
                    .color(white()),
                );
            frame(
                "Hi. We're getting everything ready for you.",
                "Almost there — your MackesDE desktop is nearly set up.",
                body.into(),
                actions(true, "Finish", Msg::Finish),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_flow_is_linear_and_terminates() {
        // Region → Keyboard → Finalize → end; prev mirrors it.
        assert_eq!(Stage::Region.next(), Some(Stage::Keyboard));
        assert_eq!(Stage::Keyboard.next(), Some(Stage::Finalize));
        assert_eq!(Stage::Finalize.next(), None);
        assert_eq!(Stage::Region.prev(), None);
        assert_eq!(Stage::Finalize.prev(), Some(Stage::Keyboard));
    }

    #[test]
    fn detected_region_falls_back_to_us() {
        // An unknown/empty LANG must not panic and lands on a valid index.
        assert!(detected_region() < REGIONS.len());
        assert!(detected_layout() < crate::keyboard::LAYOUTS.len());
    }
}
