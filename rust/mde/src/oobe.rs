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
    Privacy,
    Personalize,
    Finalize,
}

/// The stages in flow order. `next`/`prev` index into this, so adding a stage is a
/// one-line edit and forward/back can never desync.
const FLOW: &[Stage] = &[
    Stage::Region,
    Stage::Keyboard,
    Stage::Privacy,
    Stage::Personalize,
    Stage::Finalize,
];

impl Stage {
    fn pos(self) -> usize {
        FLOW.iter().position(|&s| s == self).unwrap_or(0)
    }
    fn next(self) -> Option<Stage> {
        FLOW.get(self.pos() + 1).copied()
    }
    fn prev(self) -> Option<Stage> {
        self.pos().checked_sub(1).and_then(|i| FLOW.get(i).copied())
    }
}

/// The four UI accent choices (Personalize, E11.9) — the icon_color keys + their
/// Win10 accent swatch (`palette::icon_accent`, the one accent edge).
const ACCENTS: &[(&str, &str)] = &[
    ("blue", "Blue"),
    ("orange", "Orange"),
    ("red", "Red"),
    ("neutral", "Neutral"),
];

struct Oobe {
    stage: Stage,
    /// Echo backend commands instead of running them (`--dry-run`).
    dry: bool,
    region: usize,
    layout: usize,
    /// Privacy stage (E11.7): the four toggles, seeded on (Win10 defaults).
    p_location: bool,
    p_diagnostics: bool,
    p_find: bool,
    p_ads: bool,
    /// Personalize stage (E11.9): accent index into ACCENTS + light/dark.
    accent: usize,
    light: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    PickRegion(usize),
    PickLayout(usize),
    TogglePrivacy(u8),
    PickAccent(usize),
    SetMode(bool), // true = light
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
            "privacy" => Some(Stage::Privacy),
            "personalize" => Some(Stage::Personalize),
            "finalize" => Some(Stage::Finalize),
            _ => None,
        })
        .unwrap_or(Stage::Region);

    let init = Oobe {
        stage,
        dry,
        region: detected_region(),
        layout: detected_layout(),
        p_location: st.privacy_location,
        p_diagnostics: st.privacy_diagnostics,
        p_find: st.privacy_find_device,
        p_ads: st.privacy_ads,
        accent: ACCENTS
            .iter()
            .position(|(k, _)| *k == st.icon_color)
            .unwrap_or(0),
        light: st.theme_mode == "light",
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

/// Persist the four Privacy toggles to `menu.json` and apply the configs they each
/// control (E11.7) — echoed under dry-run. `find_my_device`/Advertising are pure
/// state flags; Location drives a geoclue opt-out marker and Diagnostics a telemetry
/// opt-out marker (small files the toggle owns), so each switch does something real.
fn commit_privacy(state: &Oobe) {
    if state.dry {
        println!("  + privacy: location={} diagnostics={} find_my_device={} ads={}\n  + geoclue: {}\n  + telemetry: {}",
            state.p_location, state.p_diagnostics, state.p_find, state.p_ads,
            if state.p_location { "enabled" } else { "opt-out marker written" },
            if state.p_diagnostics { "enabled" } else { "opt-out marker written" });
        return;
    }
    let mut st = crate::state::load();
    st.privacy_location = state.p_location;
    st.privacy_diagnostics = state.p_diagnostics;
    st.privacy_find_device = state.p_find;
    st.privacy_ads = state.p_ads;
    let _ = crate::state::save(&st);
    // Each external toggle owns one opt-out marker under the config dir.
    if let Some(dir) = crate::state::config_path().and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        write_optout(&dir.join("no-geolocation"), !state.p_location);
        write_optout(&dir.join("no-telemetry"), !state.p_diagnostics);
    }
}

/// Create (opt-out on) or remove (opt-out off) a marker file.
fn write_optout(path: &std::path::Path, present: bool) {
    if present {
        let _ = std::fs::write(path, b"opted out by MDE-Retro OOBE\n");
    } else {
        let _ = std::fs::remove_file(path);
    }
}

/// Persist the Personalize choices (accent + light/dark) to `menu.json` (E11.9);
/// applied at the next surface launch (`main.rs` reads them at startup).
fn commit_personalize(state: &Oobe) {
    let accent = ACCENTS[state.accent].0;
    let mode = if state.light { "light" } else { "dark" };
    if state.dry {
        println!("  + personalize: accent={accent} mode={mode}");
        return;
    }
    let mut st = crate::state::load();
    st.icon_color = accent.to_string();
    st.theme_mode = mode.to_string();
    let _ = crate::state::save(&st);
}

fn update(state: &mut Oobe, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::PickRegion(i) => state.region = i,
        Msg::PickLayout(i) => state.layout = i,
        Msg::TogglePrivacy(which) => match which {
            0 => state.p_location = !state.p_location,
            1 => state.p_diagnostics = !state.p_diagnostics,
            2 => state.p_find = !state.p_find,
            _ => state.p_ads = !state.p_ads,
        },
        Msg::PickAccent(i) => state.accent = i,
        Msg::SetMode(light) => state.light = light,
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
                Stage::Privacy => commit_privacy(state),
                Stage::Personalize => commit_personalize(state),
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

/// One Privacy row: an On/Off toggle button + a label/description, on the chrome.
fn privacy_row<'a>(on: bool, which: u8, label: &'a str, desc: &'a str) -> Element<'a, Msg> {
    let pill = ibutton(
        text(if on { "On" } else { "Off" })
            .size(metrics::UI_PX)
            .color(white()),
    )
    .padding(pad(4.0, 14.0, 4.0, 14.0))
    .on_press(Msg::TogglePrivacy(which))
    .style(move |_, _| ibutton::Style {
        background: Some(if on {
            Background::Color(palette::color(palette::HIGHLIGHT))
        } else {
            Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.10))
        }),
        text_color: white(),
        border: iced::Border {
            radius: 2.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });
    Row::new()
        .spacing(14.0)
        .align_y(iced::alignment::Vertical::Center)
        .push(container(pill).width(Length::Fixed(70.0)))
        .push(
            Column::new()
                .push(text(label).size(metrics::UI_PX).color(white()))
                .push(text(desc).size(metrics::BADGE_PX).color(dim())),
        )
        .into()
}

fn privacy_body(state: &Oobe) -> Element<'_, Msg> {
    scrollable(
        Column::new()
            .spacing(12.0)
            .padding(pad(12.0, 8.0, 0.0, 8.0))
            .push(privacy_row(
                state.p_location,
                0,
                "Location",
                "Let apps use your location and location history.",
            ))
            .push(privacy_row(
                state.p_diagnostics,
                1,
                "Diagnostic data",
                "Send diagnostic and usage data to help improve the system.",
            ))
            .push(privacy_row(
                state.p_find,
                2,
                "Find my device",
                "Use location to help you find your device if you lose it.",
            ))
            .push(privacy_row(
                state.p_ads,
                3,
                "Tailored experiences",
                "Use diagnostic data for tips and recommendations.",
            )),
    )
    .height(Length::Fill)
    .style(mde_ui::scrollbar)
    .into()
}

/// A color swatch button for the Personalize accent picker.
fn swatch(i: usize, selected: bool) -> Element<'static, Msg> {
    let rgb = palette::icon_accent(i as u8, true);
    let fill = iced::Color::from_rgb8(rgb.0, rgb.1, rgb.2);
    ibutton(
        text(if selected { "●" } else { " " })
            .size(metrics::UI_PX)
            .color(white()),
    )
    .width(Length::Fixed(44.0))
    .height(Length::Fixed(44.0))
    .on_press(Msg::PickAccent(i))
    .style(move |_, _| ibutton::Style {
        background: Some(Background::Color(fill)),
        text_color: white(),
        border: iced::Border {
            color: white(),
            width: if selected { 2.0 } else { 0.0 },
            radius: 3.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn personalize_body(state: &Oobe) -> Element<'_, Msg> {
    let mut swatches = Row::new().spacing(10.0);
    for (i, _) in ACCENTS.iter().enumerate() {
        swatches = swatches.push(swatch(i, i == state.accent));
    }
    let mode = Row::new()
        .spacing(8.0)
        .push(
            mde_ui::button(text("Light").size(metrics::UI_PX))
                .on_press(Msg::SetMode(true))
                .default(state.light)
                .width(Length::Fixed(96.0)),
        )
        .push(
            mde_ui::button(text("Dark").size(metrics::UI_PX))
                .on_press(Msg::SetMode(false))
                .default(!state.light)
                .width(Length::Fixed(96.0)),
        );
    Column::new()
        .spacing(18.0)
        .padding(pad(16.0, 8.0, 0.0, 8.0))
        .push(text("Accent color").size(metrics::UI_PX).color(dim()))
        .push(swatches)
        .push(text("Choose your mode").size(metrics::UI_PX).color(dim()))
        .push(mode)
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
        Stage::Privacy => frame(
            "Choose privacy settings for your device",
            "You're in control. Turn off anything you'd rather not share; you can change these later in Settings.",
            privacy_body(state),
            actions(true, "Accept", Msg::Next),
        ),
        Stage::Personalize => frame(
            "Now personalize your device",
            "Pick an accent color and a light or dark look. You can change this any time.",
            personalize_body(state),
            actions(true, "Next", Msg::Next),
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
        // FLOW is a single ordered list; next/prev walk it and terminate cleanly.
        assert_eq!(Stage::Region.next(), Some(Stage::Keyboard));
        assert_eq!(Stage::Region.prev(), None);
        assert_eq!(FLOW.last().copied().unwrap().next(), None);
        // Round-trip every adjacent pair: prev(next(s)) == s.
        for w in FLOW.windows(2) {
            assert_eq!(w[0].next(), Some(w[1]));
            assert_eq!(w[1].prev(), Some(w[0]));
        }
    }

    #[test]
    fn detected_region_falls_back_to_us() {
        // An unknown/empty LANG must not panic and lands on a valid index.
        assert!(detected_region() < REGIONS.len());
        assert!(detected_layout() < crate::keyboard::LAYOUTS.len());
    }
}
