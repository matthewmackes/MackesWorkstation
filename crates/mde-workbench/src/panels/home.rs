//! v4.0.1 WB-2.a — Dashboard landing page.
//!
//! Workbench's default route when launched without `--focus`.
//! Renders an identity strip (MDE X.Y.Z · Fedora N · hostname)
//! plus a 4-card stat grid: mesh peers / pending updates /
//! snapshots / drift count. Each card links to the matching
//! panel.
//!
//! Backend integration is intentionally light at this version:
//! - peers / snapshots / drift = 0 until KDC2 / Phase G backends
//!   land — the panel renders an honest "—" rather than lying;
//! - pending updates reads `~/.cache/mde/dnf-updates.count`
//!   (the BUG-11 watermark daemon's cache file).
//!
//! Chrome influence (per iteration skill Phase 0.8): Win11
//! Settings → Home dashboard tile layout. Icon source: Carbon
//! Icon Set per the iconography lock.

use std::path::PathBuf;

use iced::widget::{button, column, container, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Task, Theme};
use mde_theme::{mde_icon, FontSize, Icon, IconSize, Palette, TypeRole};

use crate::model::Group;

/// Pure-data snapshot of what the dashboard shows. Built fresh
/// every time `HomePanel::load` runs; the view function reads
/// the cached snapshot from `self.snapshot`.
#[derive(Debug, Clone, Default)]
pub struct HomeSnapshot {
    pub mde_version: String,
    pub fedora_release: String,
    pub hostname: String,
    /// Counts. `None` = "we don't know" → renders "—" in the
    /// stat card; `Some(n)` = the literal count.
    pub mesh_peers: Option<u32>,
    pub pending_updates: Option<u32>,
    pub snapshot_count: Option<u32>,
    pub drift_count: Option<u32>,
}

impl HomeSnapshot {
    /// Best-effort load from local filesystem only. No tokio /
    /// no D-Bus — the dashboard renders synchronously on every
    /// `HomePanel::load` invocation.
    #[must_use]
    pub fn load() -> Self {
        Self {
            mde_version: env!("CARGO_PKG_VERSION").to_string(),
            fedora_release: read_fedora_release().unwrap_or_else(|| "44".into()),
            hostname: read_hostname(),
            mesh_peers: None,
            pending_updates: Some(read_dnf_count()),
            snapshot_count: count_snapshots(),
            drift_count: None,
        }
    }
}

fn read_fedora_release() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VERSION_ID=") {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "fedora".into())
}

fn read_dnf_count() -> u32 {
    // ~/.cache/mde/dnf-updates.count — populated by the BUG-11
    // headless watermark daemon's poll thread. Mirrors what
    // start_menu's footer chip reads, but the workbench crate
    // doesn't take a dep on mde-popover so we resolve the path
    // via $XDG_CACHE_HOME / $HOME directly.
    let cache = std::env::var("XDG_CACHE_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_default();
    std::fs::read_to_string(cache.join("mde/dnf-updates.count"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn count_snapshots() -> Option<u32> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    let dir = home.join(".local/share/mackes-shell/snapshots");
    let entries = std::fs::read_dir(&dir).ok()?;
    let n = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .count();
    n.try_into().ok()
}

/// Workbench panel state for the dashboard.
#[derive(Debug, Clone, Default)]
pub struct HomePanel {
    pub snapshot: HomeSnapshot,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refreshed(HomeSnapshot),
}

impl HomePanel {
    #[must_use]
    pub fn new() -> Self {
        Self {
            snapshot: HomeSnapshot::load(),
        }
    }

    pub fn load() -> Task<crate::Message> {
        // Sync load from FS; bounce through Task::perform so
        // iced's executor stays happy.
        Task::perform(async { HomeSnapshot::load() }, |snap| {
            crate::Message::Home(Message::Refreshed(snap))
        })
    }

    pub fn update(&mut self, msg: Message) -> Task<crate::Message> {
        match msg {
            Message::Refreshed(snap) => {
                self.snapshot = snap;
                Task::none()
            }
        }
    }

    /// Render the dashboard.
    pub fn view(&self) -> Element<'_, crate::Message> {
        let palette = Palette::dark();
        let sizes = FontSize::defaults();

        let title = text("Dashboard")
            .size(TypeRole::Display.size_in(sizes))
            .color(palette.text.into_iced_color());

        let identity = text(format!(
            "MDE {ver} · Fedora {rel} · {host}",
            ver = self.snapshot.mde_version,
            rel = self.snapshot.fedora_release,
            host = self.snapshot.hostname,
        ))
        .size(TypeRole::Body.size_in(sizes))
        .color(palette.text_muted.into_iced_color());

        let cards = row![
            stat_card(
                "Mesh peers",
                self.snapshot.mesh_peers,
                Icon::Peer,
                Group::Fleet,
                "inventory",
                palette,
            ),
            Space::with_width(Length::Fixed(12.0)),
            stat_card(
                "Updates pending",
                self.snapshot.pending_updates,
                Icon::Update,
                Group::Maintain,
                "snapshots", // landing on Maintain hub if system_update isn't routed
                palette,
            ),
            Space::with_width(Length::Fixed(12.0)),
            stat_card(
                "Snapshots",
                self.snapshot.snapshot_count,
                Icon::Snapshot,
                Group::Maintain,
                "snapshots",
                palette,
            ),
            Space::with_width(Length::Fixed(12.0)),
            stat_card(
                "Drift events",
                self.snapshot.drift_count,
                Icon::Repair,
                Group::Maintain,
                "drift",
                palette,
            ),
        ];

        container(
            column![
                title,
                Space::with_height(Length::Fixed(4.0)),
                identity,
                Space::with_height(Length::Fixed(24.0)),
                cards,
            ]
            .spacing(2),
        )
        .padding(Padding::from([24u16, 32u16]))
        .width(Length::Fill)
        .into()
    }
}

fn stat_card<'a>(
    label: &'a str,
    value: Option<u32>,
    icon: Icon,
    target_group: Group,
    target_panel: &'a str,
    palette: Palette,
) -> Element<'a, crate::Message> {
    let resolved = mde_icon(icon, IconSize::PanelHeader);
    let icon_widget: Element<'a, crate::Message> = if let Some(svg_bytes) = resolved.svg_bytes() {
        use iced::widget::svg as widget_svg;
        let muted = palette.text_muted.into_iced_color();
        widget_svg(widget_svg::Handle::from_memory(svg_bytes))
            .width(Length::Fixed(resolved.size_px()))
            .height(Length::Fixed(resolved.size_px()))
            .style(move |_t: &Theme, _s: widget_svg::Status| widget_svg::Style {
                color: Some(muted),
            })
            .into()
    } else {
        text(resolved.fallback_glyph)
            .size(resolved.size_px())
            .color(palette.text_muted.into_iced_color())
            .into()
    };
    let value_display = match value {
        Some(n) => n.to_string(),
        None => "—".into(),
    };
    let value_text = text(value_display)
        .size(28)
        .color(palette.text.into_iced_color());
    let label_text = text(label.to_string())
        .size(12)
        .color(palette.text_muted.into_iced_color());
    let card_panel_slug: &'static str = match target_panel {
        "snapshots" => "snapshots",
        "drift" => "drift",
        "inventory" => "inventory",
        _ => "snapshots",
    };
    let card = column![
        icon_widget,
        Space::with_height(Length::Fixed(4.0)),
        value_text,
        Space::with_height(Length::Fixed(2.0)),
        label_text,
    ]
    .spacing(0)
    .align_x(iced::alignment::Horizontal::Left);

    let bg = palette.raised.into_iced_color();
    let border = palette.border.into_iced_color();
    let muted_text = palette.text_muted.into_iced_color();
    button(card)
        .width(Length::Fill)
        .padding(Padding::from([16u16, 16u16]))
        .style(move |_t: &Theme, status: iced::widget::button::Status| {
            let hover_bg = Color {
                r: bg.r * 1.08,
                g: bg.g * 1.08,
                b: bg.b * 1.08,
                a: bg.a,
            };
            iced::widget::button::Style {
                background: Some(Background::Color(match status {
                    iced::widget::button::Status::Hovered => hover_bg,
                    _ => bg,
                })),
                text_color: muted_text,
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .on_press(crate::Message::SelectPanel {
            group: target_group,
            panel: card_panel_slug,
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_loads_with_version() {
        let s = HomeSnapshot::load();
        assert!(!s.mde_version.is_empty());
        assert!(!s.hostname.is_empty());
        assert!(!s.fedora_release.is_empty());
    }

    #[test]
    fn view_renders_without_panic() {
        let panel = HomePanel::new();
        let _ = panel.view();
    }
}
