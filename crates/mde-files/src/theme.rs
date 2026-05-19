//! Visual tokens — Rust translation of the PatternFly v6 + Mackes warm-dark
//! palette declared at the top of the prototype's `:root { ... }` block.

use iced::Color;

const fn rgb_hex(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

const fn rgba_hex(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }
}

const fn white_alpha(a: f32) -> Color {
    Color { r: 1.0, g: 1.0, b: 1.0, a }
}

// ─── PatternFly v6 dark surface tokens ─────────────────────────────────────
pub const PF_BG_100: Color = rgb_hex(0x15, 0x15, 0x15);
pub const PF_BG_200: Color = rgb_hex(0x1b, 0x1d, 0x21);
pub const PF_BG_300: Color = rgb_hex(0x1f, 0x1f, 0x1f);
pub const PF_BG_400: Color = rgb_hex(0x29, 0x29, 0x29);
pub const PF_BORDER: Color = rgb_hex(0x44, 0x45, 0x48);

pub const PF_TEXT_100: Color = rgb_hex(0xf0, 0xf0, 0xf0);
pub const PF_TEXT_200: Color = rgb_hex(0xb8, 0xbb, 0xbe);
pub const PF_TEXT_300: Color = rgb_hex(0x8a, 0x8d, 0x90);

// ─── Mackes warm-dark accent ───────────────────────────────────────────────
pub const ACCENT: Color    = rgb_hex(0xf0, 0xab, 0x00);
pub const ACCENT_HI: Color = rgb_hex(0xff, 0xc1, 0x07);
pub const RUST: Color      = rgb_hex(0xe3, 0x6b, 0x3a);

// ─── PatternFly status colours ─────────────────────────────────────────────
pub const PF_INFO: Color    = rgb_hex(0x2b, 0x9a, 0xf3);
pub const PF_SUCCESS: Color = rgb_hex(0x3e, 0x86, 0x35);
pub const PF_DANGER: Color  = rgb_hex(0xc9, 0x19, 0x0b);

// ─── Common derived colours / surfaces ─────────────────────────────────────
pub const BG: Color             = PF_BG_100;
pub const FG: Color             = PF_TEXT_100;
pub const FG_DIM: Color         = PF_TEXT_200;
pub const FG_FAINT: Color       = PF_TEXT_300;
pub const WINDOW: Color         = PF_BG_300;
pub const WINDOW_TITLEBAR: Color = PF_BG_200;
pub const WINDOW_SIDE: Color    = rgb_hex(0x25, 0x25, 0x27);
pub const DIVIDER: Color        = white_alpha(0.08);

pub const ROW_HOVER: Color           = white_alpha(0.05);
pub const ROW_HOVER_FAINT: Color     = white_alpha(0.03);
pub const ACTIVE_RUST_BG: Color      = rgba_hex(0xe3, 0x6b, 0x3a, 0.16);
pub const ACTIVE_RUST_BORDER: Color  = RUST;
pub const PRIMARY_AMBER_BG: Color    = rgba_hex(0xf0, 0xab, 0x00, 0.06);
pub const PRIMARY_AMBER_BG_HOVER: Color = rgba_hex(0xf0, 0xab, 0x00, 0.12);
pub const PRIMARY_AMBER_BG_ACTIVE: Color = rgba_hex(0xf0, 0xab, 0x00, 0.18);
pub const PRIMARY_AMBER_BORDER: Color = rgba_hex(0xf0, 0xab, 0x00, 0.55);

pub const MESH_PILL_BG: Color     = rgba_hex(0xf0, 0xab, 0x00, 0.10);
pub const MESH_PILL_BORDER: Color = rgba_hex(0xf0, 0xab, 0x00, 0.25);
pub const LOCAL_PILL_BG: Color    = white_alpha(0.03);
pub const LOCAL_PILL_BORDER: Color = white_alpha(0.06);

pub const MESH_ROW_BG: Color       = rgba_hex(0xf0, 0xab, 0x00, 0.025);
pub const MESH_ROW_BG_HOVER: Color = rgba_hex(0xf0, 0xab, 0x00, 0.06);

pub const BANNER_BORDER: Color = rgba_hex(0xf0, 0xab, 0x00, 0.18);
pub const BANNER_TINT_A: Color = rgba_hex(0xf0, 0xab, 0x00, 0.10);

pub const ROW_DIVIDER: Color = white_alpha(0.03);

// ─── Dimensions ────────────────────────────────────────────────────────────
pub const WIN_W: f32 = 1480.0;
pub const WIN_H: f32 = 940.0;
pub const TITLEBAR_H: f32 = 32.0;
pub const SIDEBAR_W: f32 = 248.0;
pub const SIDE_ROW_PAD_Y: f32 = 5.0;
pub const SIDE_ROW_PAD_X: f32 = 14.0;
pub const SIDE_ROW_GAP: f32 = 10.0;

// ─── Font families ─────────────────────────────────────────────────────────
//
// Iced expects font *byte slices* registered up-front for custom fonts. The
// host system's Red Hat font installation is preferred (it ships with the MDE
// RPM); we expose the names so widgets can pick them by `iced::Font::with_name`.
pub const FONT_TEXT: &str    = "Red Hat Text";
pub const FONT_DISPLAY: &str = "Red Hat Display";
pub const FONT_MONO: &str    = "Red Hat Mono";

// ─── Status-dot colours ────────────────────────────────────────────────────
use crate::model::PeerStatus;

#[must_use]
pub fn peer_status_dot(status: PeerStatus) -> Color {
    match status {
        PeerStatus::Online  => PF_SUCCESS,
        PeerStatus::Idle    => ACCENT,
        PeerStatus::Offline => PF_BORDER,
        PeerStatus::Self_   => RUST,
    }
}

// ─── Iced theme ────────────────────────────────────────────────────────────

/// Build the project's Iced theme — a custom dark palette derived from the
/// PatternFly + warm-dark tokens above.
#[must_use]
pub fn theme() -> iced::Theme {
    iced::Theme::custom(
        "MDE Warm Dark".into(),
        iced::theme::Palette {
            background: WINDOW,
            text: FG,
            primary: ACCENT,
            success: PF_SUCCESS,
            danger: PF_DANGER,
        },
    )
}
