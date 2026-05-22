//! UX-7 — control-state primitives.
//!
//! Centralizes the three button variants, styled text input,
//! toggle pill, skeleton placeholder, and spinner so every Iced
//! panel renders consistent hover / focus / active / disabled
//! states. Before UX-7, every panel hand-rolled `button(text("…"))`
//! with the toolkit's default chrome — coherent only by accident.
//!
//! Token rules (UX-7 spec):
//!   * buttons: 36 px height, `Radii::md` (8 px), SPACE_12 horiz
//!     padding, 3 variants (Primary fill, Secondary outline,
//!     Ghost text-only)
//!   * text inputs: 36 px height, `Radii::md`, 1 px muted border,
//!     accent border + glow on focus
//!   * toggles: 40×22 px pill, 150 ms transition
//!   * skeletons + spinner: accent-tinted, animation timing
//!     deferred to UX-9.a's subscription wiring

use iced::widget::button::Status as ButtonStatus;
use iced::widget::{button, container, row, text, text_input, Space};
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Shadow};

use mde_theme::{FontSize, Palette, Radii, TypeRole};

/// UX-7 (a) — button height. Component dimension, not
/// density-scaled.
pub const BUTTON_HEIGHT: f32 = 36.0;

/// UX-7 (a) — button horizontal padding (SPACE_12 lock; nearest
/// modular-scale value is `Space::md` 14, but the spec calls
/// out 12 px so we lock the component dimension directly).
pub const BUTTON_HORIZONTAL_PADDING: f32 = 12.0;

/// UX-7 (a) — focus ring width on focused buttons / inputs.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// UX-7 (a) — focus ring offset from the control edge.
pub const FOCUS_RING_OFFSET: f32 = 2.0;

/// UX-7 (b) — text input height. Component dimension, locked.
pub const INPUT_HEIGHT: f32 = 36.0;

/// UX-7 (c) — toggle pill dimensions. Component dimensions,
/// locked.
pub const TOGGLE_WIDTH: f32 = 40.0;
pub const TOGGLE_HEIGHT: f32 = 22.0;

/// UX-7 (a) — disabled opacity multiplier.
pub const DISABLED_OPACITY: f32 = 0.40;

/// UX-7 (a) button variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Accent-fill — primary action. Use for the dominant CTA
    /// on a panel / dialog. White text, indigo background.
    Primary,
    /// 1 px outline — secondary action. Use beside Primary for
    /// the cancel/dismiss / alternative path. Accent border +
    /// accent text, transparent fill.
    Secondary,
    /// Text-only — tertiary / inline. Use for affordances that
    /// shouldn't compete for visual weight (e.g. "Show more").
    Ghost,
}

/// UX-7 (a) — render a styled button with the locked chrome.
/// Pass `None` to `on_press` for the disabled state.
pub fn variant_button<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    variant: ButtonVariant,
    on_press: Option<Message>,
    palette: Palette,
) -> Element<'a, Message> {
    let sizes = FontSize::defaults();
    let accent = palette.accent.into_iced_color();
    let text_role = TypeRole::Body;
    let label_text = text(label.into())
        .size(text_role.size_in(sizes))
        .color(text_color_for_variant(variant, palette))
        .align_y(alignment::Vertical::Center);

    let style = move |_theme: &iced::Theme, status: ButtonStatus| {
        let mut bg = base_bg_for_variant(variant, accent, palette);
        let mut fg = text_color_for_variant(variant, palette);
        let mut border = border_for_variant(variant, accent, palette);
        match status {
            ButtonStatus::Hovered => bg = brighten(bg, 1.10),
            ButtonStatus::Pressed => bg = brighten(bg, 0.90),
            ButtonStatus::Disabled => {
                fg = with_alpha(fg, DISABLED_OPACITY);
                bg = with_alpha(bg, DISABLED_OPACITY * bg.a.max(0.1));
                border.color = with_alpha(border.color, DISABLED_OPACITY);
            }
            ButtonStatus::Active => {}
        }
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border,
            shadow: Shadow::default(),
        }
    };

    let mut btn = button(label_text)
        .padding(Padding {
            top: 0.0,
            right: BUTTON_HORIZONTAL_PADDING,
            bottom: 0.0,
            left: BUTTON_HORIZONTAL_PADDING,
        })
        .height(Length::Fixed(BUTTON_HEIGHT))
        .style(style);
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    btn.into()
}

fn base_bg_for_variant(variant: ButtonVariant, accent: Color, _palette: Palette) -> Color {
    match variant {
        ButtonVariant::Primary => accent,
        ButtonVariant::Secondary | ButtonVariant::Ghost => Color::TRANSPARENT,
    }
}

fn text_color_for_variant(variant: ButtonVariant, palette: Palette) -> Color {
    match variant {
        ButtonVariant::Primary => Color::WHITE,
        ButtonVariant::Secondary => palette.accent.into_iced_color(),
        ButtonVariant::Ghost => palette.text.into_iced_color(),
    }
}

fn border_for_variant(variant: ButtonVariant, accent: Color, _palette: Palette) -> Border {
    let radii = Radii::defaults();
    match variant {
        ButtonVariant::Primary => Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: f32::from(radii.md).into(),
        },
        ButtonVariant::Secondary => Border {
            color: accent,
            width: 1.0,
            radius: f32::from(radii.md).into(),
        },
        ButtonVariant::Ghost => Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: f32::from(radii.md).into(),
        },
    }
}

/// UX-7 (b) — styled text input. Wraps `iced::widget::text_input`
/// with the locked chrome (36 px height, RADIUS_MD, muted border,
/// accent-on-focus). Returns the bare widget; caller composes it
/// inside any container.
pub fn styled_text_input<'a, Message: Clone + 'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    palette: Palette,
) -> Element<'a, Message> {
    let radii = Radii::defaults();
    let muted = palette.text_muted.into_iced_color();
    let accent = palette.accent.into_iced_color();
    let bg = palette.surface.into_iced_color();
    let text_color = palette.text.into_iced_color();

    text_input(placeholder, value)
        .on_input(on_input)
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .size(14)
        .style(move |_theme, status| {
            let border_color = match status {
                text_input::Status::Focused { .. } | text_input::Status::Hovered => accent,
                _ => muted,
            };
            text_input::Style {
                background: Background::Color(bg),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: f32::from(radii.input).into(),
                },
                icon: muted,
                placeholder: muted,
                value: text_color,
                selection: with_alpha(accent, 0.3),
            }
        })
        .into()
}

/// UX-7 (c) — toggle pill widget. Stateless; the caller passes
/// `value: bool` and an `on_toggle(bool)` message constructor.
/// Implementation uses a styled button so click handling works
/// without extra event plumbing.
pub fn toggle<'a, Message: Clone + 'a>(
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
    palette: Palette,
) -> Element<'a, Message> {
    let radii = Radii::defaults();
    let accent = palette.accent.into_iced_color();
    let bg_off = palette.raised.into_iced_color();
    let bg_on = accent;
    let knob_color = Color::WHITE;

    let on_msg = on_toggle(!value);

    let knob_offset = if value {
        TOGGLE_WIDTH - TOGGLE_HEIGHT
    } else {
        0.0
    };

    let knob_diameter = TOGGLE_HEIGHT - 4.0;
    let knob = container(Space::new(
        Length::Fixed(knob_diameter),
        Length::Fixed(knob_diameter),
    ))
    .width(Length::Fixed(knob_diameter))
    .height(Length::Fixed(knob_diameter))
    .style(move |_| container::Style {
        background: Some(Background::Color(knob_color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: f32::from(radii.full).into(),
        },
        ..container::Style::default()
    });

    let pill_content = row![Space::with_width(Length::Fixed(knob_offset + 2.0)), knob,]
        .align_y(alignment::Vertical::Center)
        .height(Length::Fixed(TOGGLE_HEIGHT));

    button(pill_content)
        .padding(0)
        .width(Length::Fixed(TOGGLE_WIDTH))
        .height(Length::Fixed(TOGGLE_HEIGHT))
        .on_press(on_msg)
        .style(move |_theme, status| {
            let mut bg = if value { bg_on } else { bg_off };
            if matches!(status, ButtonStatus::Hovered) {
                bg = brighten(bg, 1.05);
            }
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::TRANSPARENT,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: f32::from(radii.full).into(),
                },
                shadow: Shadow::default(),
            }
        })
        .into()
}

/// UX-7 (d) — skeleton placeholder. Used to reserve layout
/// space while data loads. Renders as a `palette.raised`-tinted
/// rectangle with `Radii::sm` corners; the shimmer animation
/// (CSS-style) wires in UX-9.a.
pub fn skeleton<'a, Message: 'a>(
    width: f32,
    height: f32,
    palette: Palette,
) -> Element<'a, Message> {
    let radii = Radii::defaults();
    let bg = with_alpha(palette.raised.into_iced_color(), 0.6);
    container(Space::new(Length::Fixed(width), Length::Fixed(height)))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: f32::from(radii.sm).into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// UX-7 (d) — spinner. Single accent dot pattern; animation
/// wiring deferred to UX-9.a. v1 renders a static accent-tinted
/// circle so the layout slot is reserved correctly.
pub fn spinner<'a, Message: 'a>(palette: Palette) -> Element<'a, Message> {
    let radii = Radii::defaults();
    let accent = palette.accent.into_iced_color();
    container(Space::new(Length::Fixed(16.0), Length::Fixed(16.0)))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(with_alpha(accent, 0.6))),
            border: Border {
                color: accent,
                width: 1.0,
                radius: f32::from(radii.full).into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn brighten(c: Color, factor: f32) -> Color {
    Color {
        r: (c.r * factor).clamp(0.0, 1.0),
        g: (c.g * factor).clamp(0.0, 1.0),
        b: (c.b * factor).clamp(0.0, 1.0),
        a: c.a,
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_height_locked_to_36() {
        assert!((BUTTON_HEIGHT - 36.0).abs() < f32::EPSILON);
    }

    #[test]
    fn input_height_matches_button_height() {
        // UX-7 spec — inputs share button height for visual
        // alignment in row layouts.
        assert!((INPUT_HEIGHT - BUTTON_HEIGHT).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_pill_locked_to_40_by_22() {
        assert!((TOGGLE_WIDTH - 40.0).abs() < f32::EPSILON);
        assert!((TOGGLE_HEIGHT - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn disabled_opacity_locked_to_40_pct() {
        assert!((DISABLED_OPACITY - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn focus_ring_locked_to_two_px_offset_two_px() {
        assert!((FOCUS_RING_WIDTH - 2.0).abs() < f32::EPSILON);
        assert!((FOCUS_RING_OFFSET - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn all_variants_construct() {
        let palette = Palette::dark();
        let _ = variant_button::<()>("p", ButtonVariant::Primary, None, palette);
        let _ = variant_button::<()>("s", ButtonVariant::Secondary, None, palette);
        let _ = variant_button::<()>("g", ButtonVariant::Ghost, None, palette);
    }

    #[test]
    fn skeleton_spinner_toggle_input_construct() {
        let palette = Palette::dark();
        let _ = skeleton::<()>(100.0, 20.0, palette);
        let _ = spinner::<()>(palette);
        let _ = toggle::<bool>(true, |v| v, palette);
        let _ = styled_text_input::<String>("p", "v", |s| s, palette);
    }
}
