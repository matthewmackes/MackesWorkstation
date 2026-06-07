//! Carbon widgets for iced.
//!
//! Flat custom `Widget`/style wiring (the flat button, sunken field, title bar,
//! menubar, tree, column list). (The Win2000 3D-bevel model was retired in the
//! Carbon-only collapse, E9.7 — every control renders flat now.)

pub mod button;
pub mod frame;
pub mod groupbox;
pub mod infoband;
pub mod tabs;

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
    // Carbon: a thin flat track with a gray thumb, no 3D edge / arrow buttons.
    let rail = scrollable::Rail {
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
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

/// Corner radius for fields/controls — the flat Carbon 2px radius (the 3D Win2000
/// square edge was retired in the Carbon-only collapse, E9.7).
fn ctl_radius() -> iced::border::Radius {
    2.0.into()
}

/// The Win2000 sunken-white dropdown (closed `pick_list` control): `COLOR_WINDOW`
/// fill, a recessed 1px edge, navy selection text. Pass to
/// `pick_list(...).style(mde_ui::sunken_picklist)`.
pub fn sunken_picklist(_theme: &iced::Theme, status: pick_list::Status) -> pick_list::Style {
    // Carbon interaction state (E9.4): the open (focused) dropdown shows the 2px
    // `$focus` ring; closed/hovered keep the 1px border-strong edge.
    pick_list::Style {
        text_color: palette::color(palette::WINDOW_TEXT),
        placeholder_color: palette::color(palette::GRAY_TEXT),
        handle_color: palette::color(palette::WINDOW_TEXT),
        background: Background::Color(palette::color(palette::WINDOW)),
        border: focus_border(matches!(status, pick_list::Status::Opened)),
    }
}

/// The Win2000 sunken-white text field: `COLOR_WINDOW` fill with a recessed 1px
/// edge. Pass to `text_input(...).style(mde_ui::sunken_field)` so form fields
/// obey the rule for their kind instead of shipping the iced default.
pub fn sunken_field(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    // Carbon interaction state (E9.4): a focused field shows the strict 2px `$focus`
    // ring (the accent); every other state keeps the 1px border-strong edge.
    let border = focus_border(matches!(status, text_input::Status::Focused));
    text_input::Style {
        background: Background::Color(palette::color(palette::WINDOW)),
        border,
        icon: palette::color(palette::WINDOW_TEXT),
        placeholder: palette::color(palette::GRAY_TEXT),
        value: palette::color(palette::WINDOW_TEXT),
        selection: palette::color(palette::HIGHLIGHT),
    }
}

/// The shared control edge for the active interaction state (E9.4): the strict 2px
/// Carbon `$focus` ring (accent) when focused, else the 1px border-strong edge. One
/// source so every field/dropdown's focus ring is identical and can't drift.
fn focus_border(focused: bool) -> Border {
    if focused {
        Border {
            color: palette::accent(),
            width: 2.0,
            radius: ctl_radius(),
        }
    } else {
        Border {
            color: palette::color(palette::BUTTON_SHADOW),
            width: 1.0,
            radius: ctl_radius(),
        }
    }
}

/// The Carbon check box (E9.4 interaction states): a **checked** box fills with the
/// accent and shows a light check; an **unchecked** box is the field surface with a
/// 1px border-strong edge; a **disabled** box mutes its label. Pass to
/// `checkbox(label, checked).style(mde_ui::checkbox_style)`.
pub fn checkbox_style(_theme: &iced::Theme, status: checkbox::Status) -> checkbox::Style {
    let is_checked = matches!(
        status,
        checkbox::Status::Active { is_checked: true }
            | checkbox::Status::Hovered { is_checked: true }
            | checkbox::Status::Disabled { is_checked: true }
    );
    let disabled = matches!(status, checkbox::Status::Disabled { .. });
    let (background, border_color) = if is_checked {
        (palette::accent(), palette::accent())
    } else {
        (
            palette::color(palette::WINDOW),
            palette::color(palette::BUTTON_SHADOW),
        )
    };
    checkbox::Style {
        background: Background::Color(background),
        icon_color: palette::color(palette::HIGHLIGHT_TEXT), // light check on the accent fill
        border: Border {
            color: border_color,
            width: 1.0,
            radius: ctl_radius(),
        },
        text_color: Some(palette::color(if disabled {
            palette::GRAY_TEXT
        } else {
            palette::WINDOW_TEXT
        })),
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

/// Draw a flat Carbon control edge: one flat `face` fill + a single 1px subtle
/// border at a 2px radius. The single place a control edge is defined — [`Button`]
/// and [`BevelFrame`] both call it, so the flat surface is identical everywhere.
/// (The Win2000 raised/sunken/pressed 3D bevel was retired in the Carbon-only
/// collapse, E9.7; the face color + the caller's accent on active states carry all
/// the meaning now.)
pub(crate) fn draw_edge<R: renderer::Renderer>(r: &mut R, rect: Rectangle, face: Option<Color>) {
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
    fn chrome_accent_honours_the_toggle() {
        // The panel chrome's accent tint follows "show accent on Start & taskbar":
        // on → the UI accent, off → a neutral grey. (Independent of theme/accent.)
        palette::set_accent_on_chrome(true);
        assert_eq!(palette::chrome_accent(), palette::accent());
        palette::set_accent_on_chrome(false);
        assert_eq!(
            palette::chrome_accent(),
            palette::color(palette::BUTTON_SHADOW)
        );
        palette::set_accent_on_chrome(true); // restore default
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

    /// E9.4 — a focused field / open dropdown shows the strict 2px Carbon `$focus`
    /// ring (the accent); the resting state keeps the 1px border-strong edge.
    #[test]
    fn focus_ring_is_2px_accent_on_fields() {
        use iced::widget::{pick_list, text_input};
        palette::set_theme(palette::Theme::Carbon);

        let focused = super::sunken_field(&iced::Theme::Dark, text_input::Status::Focused);
        assert_eq!(focused.border.width, 2.0);
        assert_eq!(focused.border.color, palette::accent());

        let resting = super::sunken_field(&iced::Theme::Dark, text_input::Status::Active);
        assert_eq!(resting.border.width, 1.0);
        assert_eq!(resting.border.color, palette::color(palette::BUTTON_SHADOW));

        let open = super::sunken_picklist(&iced::Theme::Dark, pick_list::Status::Opened);
        assert_eq!(open.border.width, 2.0);
        assert_eq!(open.border.color, palette::accent());
    }

    /// E9.4 — a checked checkbox fills with the accent (light check); unchecked is
    /// the field surface + border-strong edge; disabled mutes the label.
    #[test]
    fn checkbox_checked_is_accent_filled() {
        use iced::widget::checkbox::Status as Cb;
        palette::set_theme(palette::Theme::Carbon);

        let checked = super::checkbox_style(&iced::Theme::Dark, Cb::Active { is_checked: true });
        assert_eq!(
            checked.background,
            Background::Color(palette::accent()),
            "checked box fills with the accent"
        );
        assert_eq!(checked.icon_color, palette::color(palette::HIGHLIGHT_TEXT));

        let unchecked = super::checkbox_style(&iced::Theme::Dark, Cb::Active { is_checked: false });
        assert_eq!(
            unchecked.background,
            Background::Color(palette::color(palette::WINDOW))
        );
        assert_eq!(
            unchecked.border.color,
            palette::color(palette::BUTTON_SHADOW)
        );

        let disabled =
            super::checkbox_style(&iced::Theme::Dark, Cb::Disabled { is_checked: false });
        assert_eq!(
            disabled.text_color,
            Some(palette::color(palette::GRAY_TEXT))
        );
    }
}
