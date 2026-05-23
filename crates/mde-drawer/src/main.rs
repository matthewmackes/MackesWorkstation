//! `mde-applet-drawer` — Iced drawer overlay binary.
//!
//! Phase E.8.1 + E.8.2 skeleton. Boots an Iced window with the
//! four locked drawer sections; per-section interactivity wires
//! in as Phase E.8.2 + E.2 (layer-shell) complete.

#![forbid(unsafe_code)]

use iced::widget::{button, column, container, row, slider, text, Space};
use iced::{Alignment, Element, Length, Padding, Size, Theme};

use mde_drawer::{DrawerSection, QuickToggle, DRAWER_WIDTH_PX};
use mde_panel::sliders;

#[derive(Debug, Clone)]
enum Message {
    ToggleQuickAction(QuickToggle),
    /// v3.0.3 — brightness slider moved. Routes through
    /// `mde_panel::sliders::set_brightness_percent`.
    BrightnessChanged(u8),
    /// v3.0.3 — volume slider moved. Routes through
    /// `mde_panel::sliders::set_volume_percent`.
    VolumeChanged(u8),
    /// v3.0.3 — mute toggle. Routes through
    /// `mde_panel::sliders::toggle_mute`.
    MuteToggled,
}

/// v3.0.3 — drawer state. Holds the last-read brightness + volume
/// + mute snapshots so the sliders reflect the live values without
/// re-querying on every render.
struct DrawerApp {
    brightness: u8,
    volume: u8,
    muted: bool,
}

impl Default for DrawerApp {
    fn default() -> Self {
        Self {
            brightness: sliders::read_brightness_percent().unwrap_or(50),
            volume: sliders::read_volume_percent().unwrap_or(50),
            muted: sliders::read_mute().unwrap_or(false),
        }
    }
}

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

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ToggleQuickAction(toggle) => {
                // v3.0.3 — fire the QuickToggle's flag-file
                // round-trip per the existing lib helper. The
                // flag files live under $XDG_CACHE_HOME/mde/.
                let cache_root = quick_toggle_cache_root();
                let cur = toggle.is_on(&cache_root);
                if let Err(e) = toggle.set(&cache_root, !cur) {
                    tracing::warn!(toggle = ?toggle, error = %e, "drawer: toggle set failed");
                }
            }
            Message::BrightnessChanged(pct) => {
                let snapped = sliders::snap_to_step(pct);
                self.brightness = snapped;
                if let Err(e) = sliders::set_brightness_percent(snapped) {
                    tracing::warn!(error = %e, "drawer: set_brightness failed");
                }
            }
            Message::VolumeChanged(pct) => {
                self.volume = pct;
                if let Err(e) = sliders::set_volume_percent(pct) {
                    tracing::warn!(error = %e, "drawer: set_volume failed");
                }
            }
            Message::MuteToggled => {
                if let Err(e) = sliders::toggle_mute() {
                    tracing::warn!(error = %e, "drawer: toggle_mute failed");
                } else {
                    self.muted = !self.muted;
                }
            }
        }
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
            col = col.push(self.section_body(section));
            col = col.push(Space::with_height(Length::Fixed(8.0)));
        }
        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// v3.0.3 — slider section now renders real widgets bound to
    /// `mde_panel::sliders` helpers. Brightness snaps to the
    /// 7-step grid per the helper math; volume is continuous.
    fn section_body<'a>(&self, section: DrawerSection) -> Element<'a, Message> {
        match section {
            DrawerSection::QuickActions => quick_actions_body(),
            DrawerSection::Sliders => self.sliders_body(),
            DrawerSection::Notifications => {
                placeholder("Unread notifications (Phase E.8.2 wiring)")
            }
            DrawerSection::Hardware => placeholder("Battery · CPU · Network (upower over zbus)"),
        }
    }

    fn sliders_body<'a>(&self) -> Element<'a, Message> {
        let brightness_label = text(format!(
            "Brightness · {}% (step {})",
            self.brightness,
            sliders::step_index(self.brightness)
        ))
        .size(12);
        let brightness_slider = slider(0u8..=100u8, self.brightness, Message::BrightnessChanged)
            .step(1u8);

        let volume_label = text(format!(
            "Volume · {}% {}",
            self.volume,
            if self.muted { "(muted)" } else { "" }
        ))
        .size(12);
        let volume_slider = slider(0u8..=100u8, self.volume, Message::VolumeChanged).step(1u8);

        let mute_btn = button(text(if self.muted { "Unmute" } else { "Mute" }).size(11))
            .on_press(Message::MuteToggled);

        column![
            brightness_label,
            brightness_slider,
            Space::with_height(Length::Fixed(8.0)),
            volume_label,
            volume_slider,
            Space::with_height(Length::Fixed(4.0)),
            row![mute_btn].spacing(8),
        ]
        .spacing(6)
        .into()
    }
}

fn section_header<'a>(section: DrawerSection) -> Element<'a, Message> {
    text(section.label()).size(18).into()
}

/// v3.0.3 — quick-toggle flag files live under
/// `$XDG_CACHE_HOME/mde/`. Mirrors the path the panel + popover
/// processes use for shared state (clipboard.json, toasts.jsonl,
/// dnf-updates.count). Falls back to /tmp on systems without a
/// resolvable cache dir.
fn quick_toggle_cache_root() -> std::path::PathBuf {
    dirs::cache_dir()
        .map(|d| d.join("mde"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/mde"))
}

fn quick_actions_body<'a>() -> Element<'a, Message> {
    let mut r = row![].spacing(8).align_y(Alignment::Center);
    for toggle in QuickToggle::ordered() {
        // v3.0.3 — quick-action toggles are clickable buttons.
        // Each fires `Message::ToggleQuickAction(t)` which today
        // logs a debug event; subsequent commits wire the
        // `QuickToggle::set` calls per the drawer-section spec.
        let btn = button(text(format!("[{}]", toggle.label())).size(14))
            .on_press(Message::ToggleQuickAction(toggle));
        r = r.push(btn);
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
