//! `mde lock` — the Windows 10 lock screen (E10.8).
//!
//! An exclusive top-layer `iced_layershell` overlay (the popup.rs pattern) that
//! covers the screen with the wallpaper, a large clock + date, and a PIN field.
//! A correct PIN — checked against the Argon2 hash from [`crate::pin`] — calls
//! `loginctl unlock-session` and dismisses the overlay.
//!
//! Era-gated (E10.7): only the Windows 10 theme draws this overlay. Under the
//! classic eras, and when **no PIN is enrolled** (so the overlay couldn't be
//! unlocked — never trap the user), `mde lock` falls back to the headless
//! `loginctl lock-session` path in [`crate::dialogs::lock`].
//!
//! Scope (E10.8 core): the **PIN** unlock is the implemented, harness-verifiable
//! path. Unlocking with the *account password* (PAM reauth) is a separate,
//! not-headless-observable path tracked as E10.8a.

use std::process::{exit, Command, ExitCode};
use std::time::Duration;

use iced::widget::{container, image, stack, text, text_input, Column, Space};
use iced::{Length, Task};
use iced_layershell::build_pattern::{application, MainSettings};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::{to_layer_message, Appearance};

use mde_ui::{metrics, palette};

struct Lock {
    offset: i32,
    time: String,
    date: String,
    user: String,
    wallpaper: String,
    entry: String,
    error: bool,
    field: text_input::Id,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Tick,
    Entry(String),
    Submit,
}

/// Dispatch for `mde lock`. Win10 + an enrolled PIN → the overlay; otherwise the
/// classic logind lock (so a PIN-less or classic session is never trapped).
pub fn run(_args: &[String]) -> ExitCode {
    if !palette::is_windows10() {
        return crate::dialogs::lock();
    }
    if !crate::pin::is_set() {
        eprintln!(
            "mde lock: no PIN enrolled — set one in Settings ▸ Accounts ▸ Sign-in options for \
             the Windows 10 lock screen; falling back to loginctl lock-session."
        );
        return crate::dialogs::lock();
    }
    match draw() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde lock: could not open the lock overlay: {e}");
            ExitCode::FAILURE
        }
    }
}

fn draw() -> Result<(), iced_layershell::Error> {
    let field = text_input::Id::unique();
    let widget_field = field.clone();
    application(namespace, update, view)
        .style(style)
        .subscription(|_: &Lock| iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick))
        .font(mde_ui::font::REGULAR_BYTES)
        .font(mde_ui::font::BOLD_BYTES)
        .font(mde_ui::font::PLEX_REGULAR_BYTES)
        .font(mde_ui::font::PLEX_BOLD_BYTES)
        .default_font(mde_ui::font::ui())
        .settings(MainSettings {
            layer_settings: LayerShellSettings {
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                layer: Layer::Overlay,
                exclusive_zone: -1,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(move || (Lock::new(widget_field), text_input::focus(field)))
}

impl Lock {
    fn new(field: text_input::Id) -> Self {
        let offset = utc_offset_secs();
        let st = crate::state::load();
        let user = if st.display_name.is_empty() {
            std::env::var("USER").unwrap_or_else(|_| "User".into())
        } else {
            st.display_name.clone()
        };
        // MenuState doesn't persist the live desktop wallpaper, so use the first
        // picture the Background page would offer (the system background scan);
        // an empty result falls back to the solid dark base in `style`.
        let wallpaper = crate::wallpaper::scan()
            .into_iter()
            .next()
            .unwrap_or_default();
        let now = epoch_now();
        Lock {
            offset,
            time: clock_time(now, offset),
            date: clock_date(now, offset),
            user,
            wallpaper,
            entry: String::new(),
            error: false,
            field,
        }
    }
}

fn namespace(_: &Lock) -> String {
    "mde-lock".to_string()
}

fn style(_: &Lock, _: &iced::Theme) -> Appearance {
    // Opaque dark base so a missing/!found wallpaper still yields a solid screen.
    Appearance {
        background_color: palette::color(palette::WINDOW),
        text_color: palette::color(palette::TITLE_TEXT),
    }
}

fn update(state: &mut Lock, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            let now = epoch_now();
            state.time = clock_time(now, state.offset);
            state.date = clock_date(now, state.offset);
            Task::none()
        }
        Message::Entry(s) => {
            // PIN is numeric (matches enrolment in Sign-in options).
            state.entry = s.chars().filter(char::is_ascii_digit).take(8).collect();
            state.error = false;
            Task::none()
        }
        Message::Submit => {
            if crate::pin::verify(&state.entry) {
                // Clear the logind lock state, then dismiss the overlay.
                let _ = Command::new("loginctl").arg("unlock-session").status();
                exit(0)
            }
            state.error = true;
            state.entry.clear();
            text_input::focus(state.field.clone())
        }
        _ => Task::none(),
    }
}

fn view(state: &Lock) -> Element<'_, Message> {
    // A dark scrim (the window background at reduced alpha) over the wallpaper so
    // the clock and field stay legible on any picture — colour stays palette-sourced.
    let mut scrim = palette::color(palette::WINDOW);
    scrim.a = 0.55;
    let light = palette::color(palette::TITLE_TEXT);
    let dim = palette::color(palette::GRAY_TEXT);

    let prompt = if state.error {
        text("Incorrect PIN. Try again.")
            .size(metrics::UI_PX)
            .color(palette::accent())
    } else {
        text("Enter your PIN, then press Enter")
            .size(metrics::UI_PX)
            .color(dim)
    };

    let content = Column::new()
        .align_x(iced::alignment::Horizontal::Center)
        .spacing(8.0)
        .push(
            text(state.time.clone())
                .size(metrics::LOCK_CLOCK_PX)
                .color(light),
        )
        .push(
            text(state.date.clone())
                .size(metrics::INFO_TITLE_PX)
                .color(light),
        )
        .push(Space::new(Length::Shrink, Length::Fixed(48.0)))
        .push(
            text(state.user.clone())
                .size(metrics::INFO_TITLE_PX)
                .color(light),
        )
        .push(
            text_input("PIN", &state.entry)
                .id(state.field.clone())
                .on_input(Message::Entry)
                .on_submit(Message::Submit)
                .secure(true)
                .size(metrics::UI_PX)
                .width(Length::Fixed(220.0)),
        )
        .push(prompt);

    let foreground = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(scrim)),
            ..container::Style::default()
        });

    // Wallpaper behind the scrim, when one is set and readable.
    if !state.wallpaper.is_empty() && std::path::Path::new(&state.wallpaper).is_file() {
        stack![
            image(state.wallpaper.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover),
            foreground,
        ]
        .into()
    } else {
        foreground.into()
    }
}

type Element<'a, M> = iced::Element<'a, M, iced::Theme, iced::Renderer>;

// --- time helpers (pure, no chrono) ----------------------------------------

fn epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn utc_offset_secs() -> i32 {
    Command::new("date")
        .arg("+%z")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| parse_utc_offset(s.trim()))
        .unwrap_or(0)
}

fn parse_utc_offset(s: &str) -> Option<i32> {
    let sign = if s.starts_with('-') { -1 } else { 1 };
    let d = s.trim_start_matches(['+', '-']);
    if d.len() < 4 {
        return None;
    }
    let h: i32 = d.get(0..2)?.parse().ok()?;
    let m: i32 = d.get(2..4)?.parse().ok()?;
    Some(sign * (h * 3600 + m * 60))
}

/// Big 12-hour clock, e.g. "3:58 PM".
fn clock_time(epoch_secs: i64, offset_secs: i32) -> String {
    let day = (epoch_secs + offset_secs as i64).rem_euclid(86_400);
    let h = (day / 3600) as u32;
    let m = ((day % 3600) / 60) as u32;
    let (ampm, h12) = if h < 12 { ("AM", h) } else { ("PM", h - 12) };
    let h12 = if h12 == 0 { 12 } else { h12 };
    format!("{h12}:{m:02} {ampm}")
}

/// Date line, e.g. "Tuesday, June 2".
fn clock_date(epoch_secs: i64, offset_secs: i32) -> String {
    let days = (epoch_secs + offset_secs as i64).div_euclid(86_400);
    let (_, month, day) = civil_from_days(days);
    const WD: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    const MO: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    // 1970-01-01 was a Thursday → (days + 4) mod 7 with 0 = Sunday.
    let wd = (days + 4).rem_euclid(7) as usize;
    format!("{}, {} {day}", WD[wd], MO[(month - 1) as usize])
}

/// Days since 1970-01-01 → (year, month [1-12], day [1-31]). Howard Hinnant's
/// civil-from-days algorithm (proleptic Gregorian, pure integer math).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_offset_parses_signs() {
        assert_eq!(parse_utc_offset("+0000"), Some(0));
        assert_eq!(parse_utc_offset("-0500"), Some(-5 * 3600));
        assert_eq!(parse_utc_offset("+0530"), Some(5 * 3600 + 30 * 60));
        assert_eq!(parse_utc_offset("bad"), None);
    }

    #[test]
    fn clock_time_is_twelve_hour() {
        // 2021-01-01 00:00:00 UTC = epoch 1609459200.
        assert_eq!(clock_time(1_609_459_200, 0), "12:00 AM");
        assert_eq!(clock_time(1_609_459_200 + 13 * 3600 + 5 * 60, 0), "1:05 PM");
        assert_eq!(clock_time(1_609_459_200 + 12 * 3600, 0), "12:00 PM");
    }

    #[test]
    fn civil_and_date_known_values() {
        // epoch 0 = 1970-01-01 (Thursday).
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(clock_date(0, 0), "Thursday, January 1");
        // 2021-01-01 = epoch day 18628 (Friday).
        let day = 1_609_459_200 / 86_400;
        assert_eq!(civil_from_days(day), (2021, 1, 1));
        assert_eq!(clock_date(1_609_459_200, 0), "Friday, January 1");
    }
}
