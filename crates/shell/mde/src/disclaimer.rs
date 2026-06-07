//! The Mackes Workstation warning / disclaimer / mission statement, surfaced on
//! every About + informational surface (the About dialog, System Properties). The
//! canonical text + title/body split live in the toolkit-free `mde-disclaimer`
//! crate — **the single source of truth** (embedded from the repo-root
//! `DISCLAIMER.md`), shared with the installer and daemon banner so the GUI text
//! and the docs can never drift. This module adds only the shell's iced rendering.

use iced::widget::{container, scrollable, text, Column, Space};
use iced::{Element, Length};

use mde_ui::{metrics, palette};

/// Re-export of the canonical disclaimer text (embedded from `DISCLAIMER.md` by
/// the `mde-disclaimer` crate), so existing `crate::disclaimer::TEXT` callers and
/// tests keep working.
pub use mde_disclaimer::TEXT;

/// A self-contained, scrollable rendering of the disclaimer for an About /
/// informational surface — a bold title over the scrollable body. Non-interactive,
/// so it's generic over the caller's message type.
pub fn view<'a, M: 'a>() -> Element<'a, M> {
    let (title, body) = mde_disclaimer::split();
    let inner = Column::new()
        .spacing(metrics::SPACING_03)
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
    scrollable(container(inner).padding(metrics::SPACING_03))
        .height(Length::Fill)
        .style(mde_ui::scrollbar)
        .into()
}
