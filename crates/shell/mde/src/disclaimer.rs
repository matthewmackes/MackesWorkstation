//! The Mackes Workstation warning / disclaimer / mission statement, surfaced on
//! every About + informational surface (the About dialog, System Properties, and
//! the docs). **Single source of truth:** the repo-root `DISCLAIMER.md`, embedded
//! at build time via `include_str!` so the GUI text and the documentation can never
//! drift — edit `DISCLAIMER.md` and every surface updates.

use iced::widget::{container, scrollable, text, Column, Space};
use iced::{Element, Length};

use mde_ui::{metrics, palette};

/// The full disclaimer text (Markdown), embedded from the canonical `DISCLAIMER.md`.
pub const TEXT: &str = include_str!("../../../../DISCLAIMER.md");

/// The heading (first markdown `#` line) for a compact title, and the body (the
/// remaining paragraphs) — split so a surface can show a bold title over the body.
fn split() -> (&'static str, &'static str) {
    match TEXT.split_once('\n') {
        Some((head, body)) => (head.trim_start_matches('#').trim(), body.trim_start()),
        None => ("Disclaimer", TEXT),
    }
}

/// A self-contained, scrollable rendering of the disclaimer for an About /
/// informational surface — a bold title over the scrollable body. Non-interactive,
/// so it's generic over the caller's message type.
pub fn view<'a, M: 'a>() -> Element<'a, M> {
    let (title, body) = split();
    let inner = Column::new()
        .spacing(8.0)
        .push(
            text(title)
                .size(metrics::UI_PX)
                .font(mde_ui::font::ui_bold())
                .color(palette::color(palette::WINDOW_TEXT)),
        )
        .push(
            text(body)
                .size(metrics::BADGE_PX)
                .color(palette::color(palette::WINDOW_TEXT)),
        )
        .push(Space::with_height(Length::Fixed(4.0)));
    scrollable(container(inner).padding(8.0))
        .height(Length::Fill)
        .style(mde_ui::scrollbar)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disclaimer_is_embedded_and_complete() {
        // The canonical text is embedded and carries its mission + the key
        // "as is" / "at your own risk" warranty waivers (so a surface always shows
        // the real disclaimer, not an empty/placeholder string).
        assert!(TEXT.contains("Mackes Workstation"));
        assert!(TEXT.contains("educational, experimental, open-source"));
        assert!(TEXT.contains("provided “as is”"));
        assert!(TEXT.contains("Use Mackes Workstation at your own risk."));
        assert!(TEXT.len() > 1500, "disclaimer text looks truncated");
    }

    #[test]
    fn split_extracts_title_and_body() {
        let (title, body) = split();
        assert_eq!(
            title,
            "Mackes Workstation — Warning, Disclaimer, and Mission Statement"
        );
        assert!(body.starts_with("Mackes Workstation is an educational"));
    }
}
