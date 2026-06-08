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
                                                  // Recorded ground truth, but NOT rendered by mde: labwc draws title bars as a
                                                  // flat `window.active.title.bg` color, so the navy→blue gradient caption is the
                                                  // known casualty of the mde↔labwc boundary (see ACCURACY.md §0). Kept so the
                                                  // value is transcribed; it only returns if mde ever draws client-side title rows.
pub const ACTIVE_TITLE_GRADIENT: Rgb = (0xa6, 0xca, 0xf0); // title gradient end (labwc-owned)
pub const INACTIVE_TITLE: Rgb = (0x80, 0x80, 0x80);
// (0xff,0xff,0xfe) is a SENTINEL — visually pure white in Win2000/BeOS, but a
// distinct key so the Carbon remap can tell "white text on a colored/dark
// surface" (must stay light) apart from WINDOW (a white *surface* that must
// darken in dark mode). Win2000 ground truth is still white. See `carbon()`.
pub const TITLE_TEXT: Rgb = (0xff, 0xff, 0xfe);
pub const INACTIVE_TITLE_TEXT: Rgb = (0xd4, 0xd0, 0xc8);

pub const MENU: Rgb = (0xd4, 0xd0, 0xc8);
pub const MENU_TEXT: Rgb = (0x00, 0x00, 0x00);
pub const WINDOW: Rgb = (0xff, 0xff, 0xff);
pub const WINDOW_TEXT: Rgb = (0x00, 0x00, 0x00);
// (0x00,0x00,0x01) is a SENTINEL — visually pure black in Win2000/BeOS, but a
// distinct key so the Carbon remap can tell a window/border FRAME apart from
// black TEXT (WINDOW_TEXT etc.), which must lighten in dark mode while frames
// become a subtle border gray. Win2000 ground truth is still black.
pub const WINDOW_FRAME: Rgb = (0x00, 0x00, 0x01);

// 3D button/face bevel ramp (light -> dark).
pub const BUTTON_FACE: Rgb = (0xd4, 0xd0, 0xc8);
pub const BUTTON_HILIGHT: Rgb = (0xff, 0xff, 0xff); // brightest bevel
pub const BUTTON_LIGHT: Rgb = (0xdf, 0xdf, 0xdf);
pub const BUTTON_SHADOW: Rgb = (0x80, 0x80, 0x80);
pub const BUTTON_DK_SHADOW: Rgb = (0x40, 0x40, 0x40); // darkest bevel
pub const BUTTON_TEXT: Rgb = (0x00, 0x00, 0x00);

pub const HIGHLIGHT: Rgb = (0x0a, 0x24, 0x6a); // selection
                                               // SENTINEL white text (see TITLE_TEXT) — selection text stays white on the
                                               // accent fill in both Carbon light and dark.
pub const HIGHLIGHT_TEXT: Rgb = (0xff, 0xff, 0xfe);
pub const GRAY_TEXT: Rgb = (0x80, 0x80, 0x80); // disabled

pub const INFO_TEXT: Rgb = (0x00, 0x00, 0x00); // tooltip
pub const INFO_WINDOW: Rgb = (0xff, 0xff, 0xe1);
/// Critical/danger accent (Win2000 maroon; `carbon()` remaps it to a
/// Carbon Red 60 danger red). Drives the critical-notification toast tint (E3).
pub const URGENT: Rgb = (0x80, 0x00, 0x00);

// --- MDE-Retro app chrome (NOT GetSysColor) --------------------------------
// Colors for surfaces Windows 2000 drew with bespoke art rather than a system
// color: the Explorer / Control-Panel "web view" info band and the Setup
// wizard's blue. They live here, separated from the system table above, so that
// NOTHING outside this module names a raw hex value.
/// The Explorer / Control Panel web-view info band (left blue pane).
pub const INFO_BAND: Rgb = (0x1d, 0x5c, 0xa8);
/// GUI Setup background gradient (top → bottom).
pub const SETUP_GRADIENT_TOP: Rgb = (0x1c, 0x4a, 0x8f);
pub const SETUP_GRADIENT_BOTTOM: Rgb = (0x08, 0x16, 0x40);
/// GUI Setup progress-bar fill, and the dimmed (pending/subtitle) text on it.
pub const SETUP_PROGRESS: Rgb = (0x16, 0x3a, 0xa8);
pub const SETUP_SUBTITLE: Rgb = (0x9e, 0xb2, 0xdb);
/// The Start-menu side "logo banner" brand art (Win2000/Me classic): a black
/// strip, a blue glow rising from the foot, and the rotated product name in
/// white + light blue. These are FIXED brand colors — emitted via [`hex_fixed`]
/// (NOT theme-remapped), since a logo reads identically in every era. §2.1 names
/// `LOGO_*` as belonging here in `palette.rs`.
pub const LOGO_BANNER_BG: Rgb = (0x00, 0x00, 0x00);
pub const LOGO_BANNER_GLOW: Rgb = (0x3a, 0x6a, 0xd0);
pub const LOGO_BANNER_GLOW_FADE: Rgb = (0x0a, 0x1a, 0x40);
pub const LOGO_TEXT: Rgb = (0xff, 0xff, 0xff);
pub const LOGO_TEXT_ACCENT: Rgb = (0x6f, 0x9f, 0xe0);

/// The shell bar / UI Shell header surface. Identity value is the Win2000 silver
/// taskbar face (a distinct key from `BUTTON_FACE` so the Carbon remap can paint
/// the header its own Gray 100 / white). Under Carbon it becomes the flat header
/// strip; under Win2000/BeOS it reads as the classic silver bar.
pub const SHELL_HEADER: Rgb = (0xd4, 0xd0, 0xc7);

/// Security status semantics (E14.2), consumed by the Windows 10 Security
/// dashboard (E14.4): OK / WARN / RISK. Identity values are classic-era
/// green / amber / red; the Carbon (and Win10) remap repaints them to the IBM
/// Carbon support palette (Green 50 / Yellow 30 / Red 60).
pub const STATUS_OK: Rgb = (0x00, 0x80, 0x00);
pub const STATUS_WARN: Rgb = (0xc0, 0x60, 0x00);
pub const STATUS_RISK: Rgb = (0xc0, 0x00, 0x00);

// --- Runtime theme switch --------------------------------------------------
// The palette constants above are the canonical Win2000 role keys. Alternate
// themes are applied by remapping those role colors at the `color()` edge — so
// no call site changes and every surface retints together. The active shell
// binary selects the theme at startup from persisted state (see mde state.rs /
// main.rs). Four themes exist: Windows 2000 (identity), BeOS, IBM Carbon, and
// Windows 10 (Carbon and Windows 10 share a light/dark mode + accent hue).
use std::sync::atomic::{AtomicU8, Ordering};

/// The active shell theme. (The Win2000 "Classic" theme was retired in the
/// Carbon-only collapse, E9.7 slice 3; `Windows10` remains as a Carbon-skinned
/// layout until its files.rs branches are rewritten by E10.)
/// E9.7 — Carbon is the only look (the Windows10 / Win2000 / BeOS variants were
/// retired in the Carbon-only collapse). A single-variant enum is kept so the
/// theme API (`set_theme`/`theme()`/`is_carbon`) stays stable for callers; the
/// deeper "remove the theme machinery entirely" is a follow-on cleanup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Carbon,
}

// Carbon mode/accent atomics (read on the draw hot path, so lock-free).
static DARK: AtomicU8 = AtomicU8::new(1); // Carbon default mode = dark
static ACCENT: AtomicU8 = AtomicU8::new(0); // 0=blue 1=orange 2=red 3=neutral (icon accent)

/// Select the active theme. Carbon-only — accepts `Theme` for API stability.
pub fn set_theme(_theme: Theme) {}

/// The active theme (always Carbon under the Carbon-only collapse).
pub fn theme() -> Theme {
    Theme::Carbon
}

/// Set Carbon mode: dark (true) or light (false). No effect outside Carbon.
pub fn set_dark(on: bool) {
    DARK.store(on as u8, Ordering::Relaxed);
}

/// Whether Carbon is in dark mode.
pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed) != 0
}

/// Set the icon accent hue (0=blue 1=orange 2=red 3=neutral). Consumed by the
/// shell's icon tinting; the UI accent itself is always Carbon Blue 60.
pub fn set_accent(idx: u8) {
    ACCENT.store(idx, Ordering::Relaxed);
}

/// The icon accent index (0=blue 1=orange 2=red 3=neutral).
pub fn accent_idx() -> u8 {
    ACCENT.load(Ordering::Relaxed)
}

// Windows 10 "show accent color on Start & taskbar" (E7.5a). Default on.
static ACCENT_ON_CHROME: AtomicU8 = AtomicU8::new(1);

/// Set whether the Windows 10 taskbar/Start chrome tints with the accent. When
/// off, chrome highlights fall back to a neutral grey. (main.rs sets this from
/// `state.win10_accent_on_taskbar` at startup.)
pub fn set_accent_on_chrome(on: bool) {
    ACCENT_ON_CHROME.store(on as u8, Ordering::Relaxed);
}

/// The accent for Windows 10 taskbar/Start **chrome** — the UI accent when the
/// "show accent on Start & taskbar" toggle is on, else a neutral grey. Content
/// surfaces keep using [`accent`]; only the panel chrome honours the toggle.
pub fn chrome_accent() -> iced::Color {
    if ACCENT_ON_CHROME.load(Ordering::Relaxed) != 0 {
        accent()
    } else {
        color(BUTTON_SHADOW)
    }
}

/// Whether the Carbon theme is active (always true under the Carbon-only
/// collapse; retained for call-site stability).
pub fn is_carbon() -> bool {
    theme() == Theme::Carbon
}

// ───────────────────────────────────────────────────────────────────────────
// IBM Carbon v11 design tokens (E9.2, 2026-06-06). The canonical Carbon palette
// the Carbon theme remaps onto — the named source for the Gray 10 / Gray 90 /
// Gray 100 themes the platform is collapsing to (E9, Carbon-only). Values are
// the published Carbon tokens (carbondesignsystem.com/elements/color/tokens);
// per §2.2 change one only with a spec reference + update `carbon_tokens_pinned`
// in the same commit. These name the values the `carbon()` remap used inline.
/// Carbon gray ramp.
pub const GRAY_10: Rgb = (0xf4, 0xf4, 0xf4);
pub const GRAY_50: Rgb = (0x8d, 0x8d, 0x8d);
pub const GRAY_60: Rgb = (0x6f, 0x6f, 0x6f);
pub const GRAY_70: Rgb = (0x52, 0x52, 0x52);
pub const GRAY_80: Rgb = (0x39, 0x39, 0x39);
pub const GRAY_90: Rgb = (0x26, 0x26, 0x26);
pub const GRAY_100: Rgb = (0x16, 0x16, 0x16);
pub const WHITE: Rgb = (0xff, 0xff, 0xff);
/// Layer-hover tokens (the subtle lift on hover over a layer surface).
pub const GRAY_80_HOVER: Rgb = (0x47, 0x47, 0x47); // dark layer-hover
pub const GRAY_10_HOVER: Rgb = (0xe8, 0xe8, 0xe8); // light layer-hover
/// Interactive — Carbon Blue ramp.
pub const BLUE_30: Rgb = (0xa6, 0xc8, 0xff);
pub const BLUE_40: Rgb = (0x78, 0xa9, 0xff);
pub const BLUE_50: Rgb = (0x45, 0x89, 0xff);
pub const BLUE_60: Rgb = (0x0f, 0x62, 0xfe);
pub const BLUE_70: Rgb = (0x00, 0x43, 0xce);
pub const BLUE_80: Rgb = (0x00, 0x2d, 0x9c);
pub const BLUE_100: Rgb = (0x00, 0x11, 0x41);
/// Support — status colors.
pub const RED_50: Rgb = (0xfa, 0x4d, 0x56);
pub const RED_60: Rgb = (0xda, 0x1e, 0x28);
pub const GREEN_40: Rgb = (0x42, 0xbe, 0x65);
pub const GREEN_50: Rgb = (0x24, 0xa1, 0x48);
pub const YELLOW_30: Rgb = (0xf1, 0xc2, 0x1b);
/// Icon accent — Carbon Orange ramp.
pub const ORANGE_40: Rgb = (0xff, 0x83, 0x2b);
pub const ORANGE_70: Rgb = (0xba, 0x4e, 0x00);

/// The Carbon Blue 60 interactive accent for the active mode (UI accent — drives
/// selection, focus, primary buttons, links). Always blue regardless of the
/// separate *icon* accent hue.
pub fn carbon_accent() -> Rgb {
    if is_dark() {
        BLUE_50 // on dark
    } else {
        BLUE_60 // on light
    }
}

/// The icon tint for an accent hue (0=blue 1=orange 2=red 3=neutral) in the
/// given mode. The four Carbon icon accents live here, not at the icon call site
/// (§2.1: no raw hex outside palette.rs). Returns the Carbon token `Rgb`.
pub fn icon_accent(idx: u8, dark: bool) -> Rgb {
    match idx {
        0 => {
            if dark {
                BLUE_40
            } else {
                BLUE_60
            }
        } // blue
        1 => {
            if dark {
                ORANGE_40
            } else {
                ORANGE_70
            }
        } // orange
        2 => {
            if dark {
                RED_50
            } else {
                RED_60
            }
        } // red
        _ => {
            if dark {
                GRAY_10
            } else {
                GRAY_100
            }
        } // neutral
    }
}

/// Map a Win2000 role color to its IBM Carbon token, per the active light/dark
/// mode. Tuple keys are the canonical Win2000 role values (note the white/black
/// text vs surface SENTINELS above, which let text stay legible after surfaces
/// invert in dark mode). Tokens follow Carbon Gray 10 (light) / Gray 90 (dark).
fn carbon(rgb: Rgb) -> Rgb {
    let dark = is_dark();
    let accent = carbon_accent();
    match rgb {
        // Selection / title / accent roles -> Carbon Blue 60.
        (0x0a, 0x24, 0x6a) => accent, // HIGHLIGHT + ACTIVE_TITLE
        (0xa6, 0xca, 0xf0) => accent, // ACTIVE_TITLE_GRADIENT
        (0x1d, 0x5c, 0xa8) => accent, // INFO_BAND (web-view accent/links)
        // White TEXT on a colored/dark surface (sentinel) -> stays white.
        (0xff, 0xff, 0xfe) => (0xff, 0xff, 0xff), // TITLE_TEXT + HIGHLIGHT_TEXT
        // Window-frame / border (sentinel black) -> subtle border gray.
        (0x00, 0x00, 0x01) => {
            if dark {
                GRAY_70
            } else {
                GRAY_50
            }
        }
        // Black text roles -> text-primary (invert in dark).
        (0x00, 0x00, 0x00) => {
            if dark {
                GRAY_10
            } else {
                GRAY_100
            }
        }
        // White surfaces (WINDOW / BUTTON_HILIGHT) -> field / layer-01.
        (0xff, 0xff, 0xff) => {
            if dark {
                GRAY_80
            } else {
                WHITE
            }
        }
        // Silver panel / menu / button face / inactive title text -> layer.
        (0xd4, 0xd0, 0xc8) => {
            if dark {
                GRAY_80
            } else {
                GRAY_10
            }
        }
        // Shell/UI-Shell header -> Gray 100 (dark) / white (light).
        (0xd4, 0xd0, 0xc7) => {
            if dark {
                GRAY_100
            } else {
                WHITE
            }
        }
        // Inner bevel light -> hover layer (mostly unused once flattened).
        (0xdf, 0xdf, 0xdf) => {
            if dark {
                GRAY_80_HOVER
            } else {
                GRAY_10_HOVER
            }
        }
        // Bevel shadow / disabled / inactive -> text-secondary / border-strong.
        (0x80, 0x80, 0x80) => {
            if dark {
                GRAY_60
            } else {
                GRAY_50
            }
        }
        // Dark bevel -> border-subtle.
        (0x40, 0x40, 0x40) => {
            if dark {
                GRAY_70
            } else {
                GRAY_60
            }
        }
        // Desktop background -> deepest gray (dark) / light gray (light).
        // (light value is a custom mid-gray, not a Carbon ramp token.)
        (0x3a, 0x6e, 0xa5) => {
            if dark {
                GRAY_100
            } else {
                (0xd0, 0xd0, 0xd0)
            }
        }
        // Tooltip background -> layer.
        (0xff, 0xff, 0xe1) => {
            if dark {
                GRAY_80
            } else {
                WHITE
            }
        }
        // Urgent / error -> Carbon danger red.
        (0x80, 0x00, 0x00) => {
            if dark {
                RED_50
            } else {
                RED_60
            }
        }
        // Setup-wizard blues -> accent family.
        (0x1c, 0x4a, 0x8f) => {
            if dark {
                BLUE_70
            } else {
                BLUE_60
            }
        }
        (0x08, 0x16, 0x40) => {
            if dark {
                BLUE_100
            } else {
                BLUE_80
            }
        }
        (0x16, 0x3a, 0xa8) => accent,
        (0x9e, 0xb2, 0xdb) => {
            if dark {
                BLUE_30
            } else {
                GRAY_70
            }
        }
        // Security status (E14.2) -> IBM Carbon support palette.
        (0x00, 0x80, 0x00) => {
            // STATUS_OK -> support-success (Green 40 dark / Green 50 light).
            if dark {
                GREEN_40
            } else {
                GREEN_50
            }
        }
        (0xc0, 0x60, 0x00) => {
            // STATUS_WARN -> support-warning (Yellow 30 dark / darker amber light).
            // (light value is a custom darker amber, not a Carbon ramp token.)
            if dark {
                YELLOW_30
            } else {
                (0xb2, 0x8a, 0x00)
            }
        }
        (0xc0, 0x00, 0x00) => {
            // STATUS_RISK -> support-error (Red 50 dark / Red 60 light).
            if dark {
                RED_50
            } else {
                RED_60
            }
        }
        // Brand flag art and anything else -> unchanged.
        other => other,
    }
}

/// Convert a palette [`Rgb`] into an `iced::Color`, applying the active theme. Both
/// surviving themes (Carbon + the Carbon-skinned Windows 10 layout) remap the
/// Win2000-keyed role constants through `carbon()` — the role tuples remain the
/// `carbon()` lookup keys until the deeper constant re-root (E9.7 slice 3c).
pub fn color(rgb: Rgb) -> iced::Color {
    let rgb = carbon(rgb);
    iced::Color::from_rgb8(rgb.0, rgb.1, rgb.2)
}

/// The theme-remapped role color as a `#rrggbb` string — for passing a palette
/// color to an external tool that wants hex (e.g. `swaybg -c`), keeping the hex
/// formatting on the palette edge (§2.1).
pub fn hex(rgb: Rgb) -> String {
    let rgb = carbon(rgb);
    format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
}

/// A FIXED brand color as a `#rrggbb` string, deliberately NOT theme-remapped —
/// for logo/brand art (the Start-menu side banner) that must read identically in
/// every era. Keeps the hex formatting on the palette edge (§2.1) like [`hex`],
/// but skips the per-theme remap so a logo never recolors with the active theme.
pub fn hex_fixed(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
}

/// The UI accent as an `iced::Color` (Carbon Blue 60 under Carbon; the Win2000
/// navy HIGHLIGHT otherwise). Convenience for accent underlines/focus rings.
pub fn accent() -> iced::Color {
    color(HIGHLIGHT)
}

/// The iced [`Theme`](iced::Theme) for the **active palette mode**. iced derives a
/// widget's DEFAULT styling — most importantly the color of any `text()` that
/// doesn't set `.color()` — from this theme's base colors. Surfaces previously
/// hardcoded `iced::Theme::Light`, so under the dark Carbon/Win10 palette every
/// un-colored label fell back to iced's near-black default — black text on a dark
/// surface. Building the theme from the live palette (dark surface → light default
/// text) makes those defaults contrast the real background. Every app surface
/// should use `.theme(|_| mde_ui::palette::iced_theme())`.
pub fn iced_theme() -> iced::Theme {
    iced::Theme::custom(
        "MDE".to_string(),
        iced::theme::Palette {
            background: color(WINDOW),
            text: color(WINDOW_TEXT),
            primary: accent(),
            success: accent(),
            danger: color(URGENT),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E9.2 / §2.2 — pin the IBM Carbon v11 design tokens to their published
    /// values (carbondesignsystem.com/elements/color/tokens). Change a token
    /// only with a spec reference and update this assertion in the same commit.
    #[test]
    fn carbon_tokens_pinned() {
        // Gray ramp.
        assert_eq!(GRAY_10, (0xf4, 0xf4, 0xf4));
        assert_eq!(GRAY_50, (0x8d, 0x8d, 0x8d));
        assert_eq!(GRAY_60, (0x6f, 0x6f, 0x6f));
        assert_eq!(GRAY_70, (0x52, 0x52, 0x52));
        assert_eq!(GRAY_80, (0x39, 0x39, 0x39));
        assert_eq!(GRAY_90, (0x26, 0x26, 0x26));
        assert_eq!(GRAY_100, (0x16, 0x16, 0x16));
        assert_eq!(WHITE, (0xff, 0xff, 0xff));
        // Blue ramp (interactive).
        assert_eq!(BLUE_30, (0xa6, 0xc8, 0xff));
        assert_eq!(BLUE_40, (0x78, 0xa9, 0xff));
        assert_eq!(BLUE_50, (0x45, 0x89, 0xff));
        assert_eq!(BLUE_60, (0x0f, 0x62, 0xfe));
        assert_eq!(BLUE_70, (0x00, 0x43, 0xce));
        assert_eq!(BLUE_80, (0x00, 0x2d, 0x9c));
        assert_eq!(BLUE_100, (0x00, 0x11, 0x41));
        // Support.
        assert_eq!(RED_50, (0xfa, 0x4d, 0x56));
        assert_eq!(RED_60, (0xda, 0x1e, 0x28));
        assert_eq!(GREEN_40, (0x42, 0xbe, 0x65));
        assert_eq!(GREEN_50, (0x24, 0xa1, 0x48));
        assert_eq!(YELLOW_30, (0xf1, 0xc2, 0x1b));
        // Orange (icon accent).
        assert_eq!(ORANGE_40, (0xff, 0x83, 0x2b));
        assert_eq!(ORANGE_70, (0xba, 0x4e, 0x00));
    }

    /// The Carbon dark theme resolves its core roles onto the Gray 100 token set
    /// (E9.2 — guards the remap against drift now that the values are named).
    #[test]
    fn carbon_dark_maps_core_roles_to_gray_100_tokens() {
        set_theme(Theme::Carbon);
        set_dark(true);
        assert_eq!(carbon(WINDOW_TEXT), GRAY_10, "primary text -> Gray 10");
        assert_eq!(carbon(WINDOW), GRAY_80, "field/layer -> Gray 80");
        assert_eq!(carbon_accent(), BLUE_50, "dark accent -> Blue 50");
    }
}
