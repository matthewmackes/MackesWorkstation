//! Windows 2000 "Classic" system palette.
//!
//! These values are the ground truth transcribed from
//! `assets/reference/win2000-classic-colors.ini`. They are kept as plain
//! `(u8, u8, u8)` tuples so this module has no dependency on any GUI toolkit;
//! use [`color`] to convert to an `iced::Color` at the edges.

/// An sRGB 8-bit-per-channel color, `(r, g, b)`.
pub type Rgb = (u8, u8, u8);

// --- Core Win2000 Classic colors (COLOR_* / GetSysColor defaults) ----------
pub const BACKGROUND: Rgb = (0x3a, 0x6e, 0xa5); // desktop
pub const ACTIVE_TITLE: Rgb = (0x0a, 0x24, 0x6a); // focused title bar / Highlight
pub const ACTIVE_TITLE_GRADIENT: Rgb = (0xa6, 0xca, 0xf0); // title gradient end
pub const INACTIVE_TITLE: Rgb = (0x80, 0x80, 0x80);
pub const TITLE_TEXT: Rgb = (0xff, 0xff, 0xff);
pub const INACTIVE_TITLE_TEXT: Rgb = (0xd4, 0xd0, 0xc8);

pub const MENU: Rgb = (0xd4, 0xd0, 0xc8);
pub const MENU_TEXT: Rgb = (0x00, 0x00, 0x00);
pub const WINDOW: Rgb = (0xff, 0xff, 0xff);
pub const WINDOW_TEXT: Rgb = (0x00, 0x00, 0x00);
pub const WINDOW_FRAME: Rgb = (0x00, 0x00, 0x00);

// 3D button/face bevel ramp (light -> dark).
pub const BUTTON_FACE: Rgb = (0xd4, 0xd0, 0xc8);
pub const BUTTON_HILIGHT: Rgb = (0xff, 0xff, 0xff); // brightest bevel
pub const BUTTON_LIGHT: Rgb = (0xdf, 0xdf, 0xdf);
pub const BUTTON_SHADOW: Rgb = (0x80, 0x80, 0x80);
pub const BUTTON_DK_SHADOW: Rgb = (0x40, 0x40, 0x40); // darkest bevel
pub const BUTTON_TEXT: Rgb = (0x00, 0x00, 0x00);

pub const HIGHLIGHT: Rgb = (0x0a, 0x24, 0x6a); // selection
pub const HIGHLIGHT_TEXT: Rgb = (0xff, 0xff, 0xff);
pub const GRAY_TEXT: Rgb = (0x80, 0x80, 0x80); // disabled

pub const INFO_TEXT: Rgb = (0x00, 0x00, 0x00); // tooltip
pub const INFO_WINDOW: Rgb = (0xff, 0xff, 0xe1);
pub const URGENT: Rgb = (0x80, 0x00, 0x00); // MDE-Retro: urgent window (maroon)

/// Convert a palette [`Rgb`] into an `iced::Color`.
pub fn color(rgb: Rgb) -> iced::Color {
    iced::Color::from_rgb8(rgb.0, rgb.1, rgb.2)
}
