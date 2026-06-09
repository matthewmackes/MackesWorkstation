//! On-screen display (OSD) — a native, Carbon-styled volume / brightness /
//! mic bar that pops up on the media keys, the cosmic-osd equivalent.
//!
//!   mde osd volume up|down|mute
//!   mde osd mic mute
//!   mde osd brightness up|down
//!
//! Each invocation APPLIES the change (PipeWire `wpctl`, falling back to
//! PulseAudio `pactl`; `brightnessctl` for backlight) and then shows a small
//! bottom-centre overlay bar with the new level — replacing the optional
//! `swayosd` dependency the labwc media keys used to shell out to.
//!
//! Singleton + live update: the bar is one layer-shell surface that auto-
//! dismisses ~1.3 s after the last key. A repeated key while it's showing
//! does NOT spawn a second window — the new process applies the change, writes
//! the shared state file (`$XDG_RUNTIME_DIR/mde-osd.state`), and exits; the
//! already-running bar polls that file every 80 ms, adopts the new value, and
//! resets its dismiss timer. (The singleton pid-file guard from
//! `start_common` arbitrates which process owns the window.)

use std::process::{Command, ExitCode};
use std::time::{Duration, Instant, SystemTime};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, text, Row, Space};
use iced::{Background, Border, Color, Element, Length, Task};
use iced_layershell::build_pattern::{application, MainSettings};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::{to_layer_message, Appearance};

use mde_ui::{metrics, palette};

/// How long the bar lingers after the last key before it dismisses.
const LINGER: Duration = Duration::from_millis(1300);
/// Bar surface geometry + how far it floats above the taskbar.
const W: u32 = 320;
const H: u32 = 64;
const BOTTOM_MARGIN: i32 = 140;
/// The level track width inside the bar.
const TRACK_W: f32 = 196.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Volume,
    Brightness,
    Mic,
}

impl Kind {
    fn tag(self) -> &'static str {
        match self {
            Self::Volume => "volume",
            Self::Brightness => "brightness",
            Self::Mic => "mic",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "volume" => Some(Self::Volume),
            "brightness" => Some(Self::Brightness),
            "mic" => Some(Self::Mic),
            _ => None,
        }
    }
}

/// What the bar is showing: which control, the 0–100 level, and mute state.
#[derive(Debug, Clone, Copy)]
struct Snap {
    kind: Kind,
    value: u8,
    muted: bool,
    /// Monotonic stamp (ns since epoch) of the invocation that produced this —
    /// the running bar treats a changed stamp as "a new key was pressed".
    stamp: u128,
}

pub fn run(args: &[String]) -> ExitCode {
    let kind = args.first().map(String::as_str).unwrap_or("");
    let action = args.get(1).map(String::as_str).unwrap_or("");
    let Some(kind) = Kind::parse(kind) else {
        eprintln!("usage: mde osd <volume|brightness|mic> <up|down|mute>");
        return ExitCode::FAILURE;
    };

    // 1. Apply the change and read back the resulting level.
    let level = match kind {
        Kind::Volume => {
            apply_sink(action);
            read_volume("@DEFAULT_AUDIO_SINK@")
        }
        Kind::Mic => {
            apply_source(action);
            read_volume("@DEFAULT_AUDIO_SOURCE@")
        }
        Kind::Brightness => {
            apply_brightness(action);
            read_brightness().map(|v| (v, false))
        }
    };
    // No readable level (e.g. no backlight on a desktop) → the change is done,
    // there's just nothing to draw. Done.
    let Some((value, muted)) = level else {
        return ExitCode::SUCCESS;
    };

    // 2. Publish the new level for any already-running bar to adopt.
    let snap = Snap {
        kind,
        value,
        muted,
        stamp: now_ns(),
    };
    write_state(snap);

    // 3. No display, or a bar is already up → we're the messenger, exit.
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return ExitCode::SUCCESS;
    }
    if !crate::start_common::acquire_singleton("mde-osd") {
        return ExitCode::SUCCESS;
    }

    // 4. We own the window — draw the bar until it lingers out.
    match launch(snap) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde osd: {e}");
            ExitCode::FAILURE
        }
    }
}

// --- apply / read backends -------------------------------------------------

fn ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Apply a sink (output) change via wpctl, falling back to pactl.
fn apply_sink(action: &str) {
    let done = match action {
        "up" => ok(
            "wpctl",
            &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"],
        ),
        "down" => ok("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]),
        "mute" => ok("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]),
        _ => true,
    };
    if !done {
        let _ = match action {
            "up" => ok("pactl", &["set-sink-volume", "@DEFAULT_SINK@", "+5%"]),
            "down" => ok("pactl", &["set-sink-volume", "@DEFAULT_SINK@", "-5%"]),
            "mute" => ok("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "toggle"]),
            _ => true,
        };
    }
}

/// Apply a source (mic) change. Only mute-toggle is wired (the mic key).
fn apply_source(action: &str) {
    if action == "mute" && !ok("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]) {
        let _ = ok("pactl", &["set-source-mute", "@DEFAULT_SOURCE@", "toggle"]);
    }
}

fn apply_brightness(action: &str) {
    let arg = match action {
        "up" => "7%+",
        "down" => "7%-",
        _ => return,
    };
    let _ = ok("brightnessctl", &["set", arg]);
}

/// Read a PipeWire device's volume as (0–100, muted), wpctl then pactl.
fn read_volume(node: &str) -> Option<(u8, bool)> {
    if let Ok(o) = Command::new("wpctl").args(["get-volume", node]).output() {
        if o.status.success() {
            // "Volume: 0.45 [MUTED]"
            let s = String::from_utf8_lossy(&o.stdout);
            let muted = s.contains("MUTED");
            if let Some(v) = s
                .split_whitespace()
                .nth(1)
                .and_then(|t| t.parse::<f32>().ok())
            {
                return Some(((v * 100.0).round().clamp(0.0, 100.0) as u8, muted));
            }
        }
    }
    // pactl fallback keys off @DEFAULT_SINK@/@DEFAULT_SOURCE@.
    let (mute_obj, vol_obj, mute_cmd, vol_cmd) = if node.contains("SOURCE") {
        (
            "@DEFAULT_SOURCE@",
            "@DEFAULT_SOURCE@",
            "get-source-mute",
            "get-source-volume",
        )
    } else {
        (
            "@DEFAULT_SINK@",
            "@DEFAULT_SINK@",
            "get-sink-mute",
            "get-sink-volume",
        )
    };
    let muted = Command::new("pactl")
        .args([mute_cmd, mute_obj])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("yes"))
        .unwrap_or(false);
    let pct = Command::new("pactl")
        .args([vol_cmd, vol_obj])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split('/')
                .nth(1)
                .and_then(|t| t.trim().trim_end_matches('%').parse::<u8>().ok())
        })?;
    Some((pct, muted))
}

/// Read the backlight as 0–100, or None when there's no backlight.
fn read_brightness() -> Option<u8> {
    // `brightnessctl -m` → CSV "device,class,current,percent,max"; field 4 = "NN%".
    let o = Command::new("brightnessctl").arg("-m").output().ok()?;
    String::from_utf8_lossy(&o.stdout)
        .split(',')
        .nth(3)?
        .trim()
        .trim_end_matches('%')
        .parse()
        .ok()
}

// --- shared state file -----------------------------------------------------

fn state_path() -> std::path::PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(dir).join("mde-osd.state")
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn write_state(s: Snap) {
    let line = format!(
        "{} {} {} {}",
        s.kind.tag(),
        s.value,
        u8::from(s.muted),
        s.stamp
    );
    let _ = std::fs::write(state_path(), line);
}

fn read_state() -> Option<Snap> {
    let raw = std::fs::read_to_string(state_path()).ok()?;
    let mut it = raw.split_whitespace();
    let kind = Kind::parse(it.next()?)?;
    let value: u8 = it.next()?.parse().ok()?;
    let muted = it.next()? == "1";
    let stamp: u128 = it.next()?.parse().ok()?;
    Some(Snap {
        kind,
        value,
        muted,
        stamp,
    })
}

// --- GUI -------------------------------------------------------------------

struct Osd {
    snap: Snap,
    /// When the bar should dismiss if no newer key arrives.
    deadline: Instant,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Tick,
}

fn launch(initial: Snap) -> Result<(), iced_layershell::Error> {
    application(namespace, update, view)
        .style(style)
        .subscription(|_: &Osd| iced::time::every(Duration::from_millis(80)).map(|_| Message::Tick))
        .font(mde_ui::font::REGULAR_BYTES)
        .font(mde_ui::font::BOLD_BYTES)
        .font(mde_ui::font::PLEX_REGULAR_BYTES)
        .font(mde_ui::font::PLEX_BOLD_BYTES)
        .default_font(mde_ui::font::ui())
        .settings(MainSettings {
            layer_settings: LayerShellSettings {
                // Bottom-anchored only → the compositor centres it horizontally;
                // a small surface (not a screen catcher) on the Overlay layer with
                // no keyboard grab, so it floats over apps without stealing input.
                anchor: Anchor::Bottom,
                layer: Layer::Overlay,
                size: Some((W, H)),
                margin: (0, 0, BOTTOM_MARGIN, 0),
                exclusive_zone: 0,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(move || {
            // Prefer the freshest state on disk (a key may have landed between the
            // write and the window opening); fall back to what we launched with.
            let snap = read_state().unwrap_or(initial);
            (
                Osd {
                    snap,
                    deadline: Instant::now() + LINGER,
                },
                Task::none(),
            )
        })
}

fn namespace(_: &Osd) -> String {
    "mde-osd".to_string()
}

fn style(_: &Osd, _: &iced::Theme) -> Appearance {
    Appearance {
        background_color: Color::TRANSPARENT,
        text_color: palette::color(palette::WINDOW_TEXT),
    }
}

fn update(state: &mut Osd, _message: Message) -> Task<Message> {
    // Adopt any newer key (resets the linger), then dismiss once it lapses.
    if let Some(fresh) = read_state() {
        if fresh.stamp != state.snap.stamp {
            state.snap = fresh;
            state.deadline = Instant::now() + LINGER;
        }
    }
    if Instant::now() >= state.deadline {
        std::process::exit(0);
    }
    Task::none()
}

/// The control glyph (Nerd Font): speaker / mic / sun, with the muted variants.
fn glyph(s: Snap) -> &'static str {
    match s.kind {
        Kind::Volume => {
            if s.muted {
                "\u{f026}" // fa-volume-off
            } else if s.value <= 50 {
                "\u{f027}" // fa-volume-down
            } else {
                "\u{f028}" // fa-volume-up
            }
        }
        Kind::Mic => {
            if s.muted {
                "\u{f131}" // fa-microphone-slash
            } else {
                "\u{f130}" // fa-microphone
            }
        }
        Kind::Brightness => "\u{f0335}", // mdi-brightness-7 (sun)
    }
}

fn view(state: &Osd) -> Element<'_, Message> {
    let s = state.snap;
    let dim = s.muted;
    let fill_color = if dim {
        palette::color(palette::GRAY_TEXT)
    } else {
        palette::accent()
    };
    let icon = text(glyph(s))
        .size(metrics::PANEL_GLYPH_PX)
        .font(mde_ui::font::NERD)
        .color(palette::color(palette::WINDOW_TEXT));

    // Level track: a gray bar with an accent fill sized to the level.
    let fill_w = (TRACK_W * f32::from(s.value) / 100.0).clamp(0.0, TRACK_W);
    let track = container(
        Row::new()
            .push(
                container(Space::new(Length::Fixed(fill_w), Length::Fixed(6.0))).style(move |_| {
                    container::Style {
                        background: Some(Background::Color(fill_color)),
                        border: Border {
                            radius: 3.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
            )
            .push(Space::new(Length::Fill, Length::Fixed(6.0))),
    )
    .width(Length::Fixed(TRACK_W))
    .height(Length::Fixed(6.0))
    .style(|_| container::Style {
        background: Some(Background::Color(palette::color(palette::WINDOW_FRAME))),
        border: Border {
            radius: 3.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    });

    let label = if dim {
        "Muted".to_string()
    } else {
        format!("{}%", s.value)
    };
    let pct = text(label)
        .size(metrics::UI_PX)
        .color(palette::color(palette::WINDOW_TEXT))
        .width(Length::Fixed(44.0))
        .align_x(Horizontal::Right);

    let bar = container(
        Row::new()
            .spacing(metrics::SPACING_04)
            .align_y(Vertical::Center)
            .push(icon)
            .push(track)
            .push(pct),
    )
    .padding(metrics::SPACING_04)
    .center_y(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(palette::color(palette::MENU))),
        border: Border {
            color: palette::color(palette::WINDOW_FRAME),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    });

    container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
}
