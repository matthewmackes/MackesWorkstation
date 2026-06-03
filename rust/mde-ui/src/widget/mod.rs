//! Win2000 Classic widgets for iced.
//!
//! The bevel model ([`bevel`]) is implemented and unit-tested. The iced
//! `Widget`/style wiring (3D button, sunken field, title bar, menubar, tree,
//! column list) lands as the components are built — see tasks for mde-ui.

pub mod bevel;
pub mod button;
pub mod frame;
pub mod groupbox;
pub mod infoband;
pub mod tabs;

pub use bevel::Bevel;
pub use button::{button, Button};
pub use frame::BevelFrame;
pub use groupbox::group_box;
pub use tabs::tab_strip;

use iced::advanced::renderer;
use iced::widget::{checkbox, container, pick_list, radio, scrollable, text_input};
use iced::{Background, Border, Color, Rectangle, Shadow};

use crate::palette;

/// Win2000 scrollbar: a light-gray (`COLOR_3DLIGHT`) track with a silver
/// (`COLOR_3DFACE`) thumb edged in shadow. iced can't draw the full 3D thumb
/// bevel (a rail scroller is one color + one border), so this is the closest
/// faithful approximation. Pass to `scrollable(...).style(mde_ui::scrollbar)`.
pub fn scrollbar(_theme: &iced::Theme, _status: scrollable::Status) -> scrollable::Style {
    // Flat eras (Carbon / Win10): a thin flat track with a gray thumb, no 3D edge
    // / arrow buttons.
    let rail = if palette::is_flat() {
        scrollable::Rail {
            background: Some(Background::Color(palette::color(palette::MENU))),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: palette::color(palette::BUTTON_SHADOW),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
            },
        }
    } else {
        scrollable::Rail {
            background: Some(Background::Color(palette::color(palette::BUTTON_LIGHT))),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: palette::color(palette::BUTTON_FACE),
                border: Border {
                    color: palette::color(palette::BUTTON_SHADOW),
                    width: 1.0,
                    radius: 0.0.into(),
                },
            },
        }
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

/// Corner radius for fields/controls under the active theme (flat eras
/// Carbon/Win10 = 2px, the 3D Win2000/BeOS = square).
fn ctl_radius() -> iced::border::Radius {
    if palette::is_flat() {
        2.0.into()
    } else {
        0.0.into()
    }
}

/// The Win2000 sunken-white dropdown (closed `pick_list` control): `COLOR_WINDOW`
/// fill, a recessed 1px edge, navy selection text. Pass to
/// `pick_list(...).style(mde_ui::sunken_picklist)`.
pub fn sunken_picklist(_theme: &iced::Theme, _status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: palette::color(palette::WINDOW_TEXT),
        placeholder_color: palette::color(palette::GRAY_TEXT),
        handle_color: palette::color(palette::WINDOW_TEXT),
        background: Background::Color(palette::color(palette::WINDOW)),
        border: Border {
            color: palette::color(palette::BUTTON_SHADOW),
            width: 1.0,
            radius: ctl_radius(),
        },
    }
}

/// The Win2000 sunken-white text field: `COLOR_WINDOW` fill with a recessed 1px
/// edge. Pass to `text_input(...).style(mde_ui::sunken_field)` so form fields
/// obey the rule for their kind instead of shipping the iced default.
pub fn sunken_field(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(palette::color(palette::WINDOW)),
        border: Border {
            color: palette::color(palette::BUTTON_SHADOW),
            width: 1.0,
            radius: ctl_radius(),
        },
        icon: palette::color(palette::WINDOW_TEXT),
        placeholder: palette::color(palette::GRAY_TEXT),
        value: palette::color(palette::WINDOW_TEXT),
        selection: palette::color(palette::HIGHLIGHT),
    }
}

/// The Win2000 check box: a sunken white box with a black check, label in
/// window text. Pass to `checkbox(label, checked).style(mde_ui::checkbox_style)`.
pub fn checkbox_style(_theme: &iced::Theme, _status: checkbox::Status) -> checkbox::Style {
    checkbox::Style {
        background: Background::Color(palette::color(palette::WINDOW)),
        icon_color: palette::color(palette::WINDOW_TEXT),
        border: Border {
            color: palette::color(palette::BUTTON_SHADOW),
            width: 1.0,
            radius: ctl_radius(),
        },
        text_color: Some(palette::color(palette::WINDOW_TEXT)),
    }
}

/// The Win2000 radio button: a sunken white circle with a black dot. Pass to
/// `radio(...).style(mde_ui::radio_style)`.
pub fn radio_style(_theme: &iced::Theme, _status: radio::Status) -> radio::Style {
    radio::Style {
        background: Background::Color(palette::color(palette::WINDOW)),
        dot_color: palette::color(palette::WINDOW_TEXT),
        border_width: 1.0,
        border_color: palette::color(palette::BUTTON_SHADOW),
        text_color: Some(palette::color(palette::WINDOW_TEXT)),
    }
}

/// Blend two colours by `t` (0 → `a`, 1 → `b`). Used to tint the accent for
/// hover/press states without naming a second hex (the colour stays derived from
/// the palette accent, so §2.1 holds).
fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// A **primary** (accent-filled) push button — the affirmative call-to-action on a
/// flat-era surface (Carbon / Win10). Accent fill + light label; hover lightens and
/// press darkens the accent. Pass to `button(...).style(mde_ui::button_primary)`.
/// (Under the 3D Win2000/BeOS eras forms use the bevelled [`Button`] instead; this
/// still renders sensibly — a square accent fill — if ever used there.)
pub fn button_primary(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use iced::widget::button::Status;
    let accent = palette::accent();
    let light = palette::color(palette::HIGHLIGHT_TEXT);
    let bg = match status {
        Status::Hovered => mix(accent, light, 0.14),
        Status::Pressed => mix(accent, palette::color(palette::WINDOW_FRAME), 0.20),
        Status::Disabled => mix(accent, palette::color(palette::WINDOW), 0.55),
        Status::Active => accent,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: light,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: ctl_radius(),
        },
        shadow: Shadow::default(),
    }
}

/// A **ghost** (text-only) push button — a low-emphasis / secondary action.
/// Transparent at rest with accent-coloured text; a faint accent wash on hover and
/// a stronger one on press. Pass to `button(...).style(mde_ui::button_ghost)`.
pub fn button_ghost(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    use iced::widget::button::Status;
    let accent = palette::accent();
    let wash = |alpha: f32| Background::Color(Color { a: alpha, ..accent });
    let background = match status {
        Status::Hovered => Some(wash(0.14)),
        Status::Pressed => Some(wash(0.22)),
        _ => None,
    };
    iced::widget::button::Style {
        background,
        text_color: match status {
            Status::Disabled => palette::color(palette::GRAY_TEXT),
            _ => accent,
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: ctl_radius(),
        },
        shadow: Shadow::default(),
    }
}

/// Fill an axis-aligned rectangle with a solid color (skips degenerate rects).
/// The one quad primitive every Win2000 edge is built from.
pub(crate) fn fill<R: renderer::Renderer>(r: &mut R, x: f32, y: f32, w: f32, h: f32, c: Color) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    r.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x,
                y,
                width: w,
                height: h,
            },
            border: Border::default(),
            shadow: Shadow::default(),
        },
        c,
    );
}

/// The Win2000 `DrawEdge`: optionally fill `face`, then lay the bevel's outer
/// (and, when `thickness >= 2`, inner) 1px lines around `rect`. This is the
/// single place a 1px edge can be wrong — [`Button`] and [`BevelFrame`] both
/// call it, so the raised/sunken/pressed look is identical everywhere.
pub(crate) fn draw_edge<R: renderer::Renderer>(
    r: &mut R,
    rect: Rectangle,
    bevel: Bevel,
    thickness: u16,
    face: Option<Color>,
) {
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    // Flat eras (Carbon / Win10): no 3D bevel. One flat fill + a single 1px subtle
    // border, 2px radius. Collapses raised/sunken/pressed into the same flat
    // surface; the face color (and accent on active states, chosen by the caller)
    // carries all the meaning. `bevel`/`thickness` are intentionally ignored here.
    if palette::is_flat() {
        let _ = (bevel, thickness);
        r.fill_quad(
            renderer::Quad {
                bounds: rect,
                border: Border {
                    color: palette::color(palette::WINDOW_FRAME),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                shadow: Shadow::default(),
            },
            face.unwrap_or(Color::TRANSPARENT),
        );
        return;
    }
    if let Some(face) = face {
        fill(r, x, y, w, h, face);
    }
    // Outer edge: top + left vs bottom + right.
    fill(r, x, y, w, 1.0, palette::color(bevel.outer_tl));
    fill(r, x, y, 1.0, h, palette::color(bevel.outer_tl));
    fill(r, x, y + h - 1.0, w, 1.0, palette::color(bevel.outer_br));
    fill(r, x + w - 1.0, y, 1.0, h, palette::color(bevel.outer_br));
    if thickness >= 2 {
        // Inner edge.
        fill(
            r,
            x + 1.0,
            y + 1.0,
            w - 2.0,
            1.0,
            palette::color(bevel.inner_tl),
        );
        fill(
            r,
            x + 1.0,
            y + 1.0,
            1.0,
            h - 2.0,
            palette::color(bevel.inner_tl),
        );
        fill(
            r,
            x + 1.0,
            y + h - 2.0,
            w - 2.0,
            1.0,
            palette::color(bevel.inner_br),
        );
        fill(
            r,
            x + w - 2.0,
            y + 1.0,
            1.0,
            h - 2.0,
            palette::color(bevel.inner_br),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::button::Status;

    #[test]
    fn primary_button_is_accent_filled() {
        palette::set_theme(palette::Theme::Carbon);
        let p = button_primary(&iced::Theme::Dark, Status::Active);
        assert_eq!(p.background, Some(Background::Color(palette::accent())));
        assert_eq!(p.text_color, palette::color(palette::HIGHLIGHT_TEXT));
        // Hover shifts the fill towards the light label — distinct from rest.
        let hov = button_primary(&iced::Theme::Dark, Status::Hovered);
        assert_ne!(hov.background, p.background);
    }

    #[test]
    fn ghost_button_is_transparent_with_accent_text() {
        palette::set_theme(palette::Theme::Carbon);
        let g = button_ghost(&iced::Theme::Dark, Status::Active);
        assert!(g.background.is_none(), "ghost is transparent at rest");
        assert_eq!(g.text_color, palette::accent());
        // Hover paints a faint accent wash.
        assert!(button_ghost(&iced::Theme::Dark, Status::Hovered)
            .background
            .is_some());
    }
}
