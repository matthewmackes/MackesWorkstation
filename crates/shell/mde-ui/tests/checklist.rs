//! Static accuracy checklist (layer 1 of `ACCURACY.md`).
//!
//! These tests pin the palette's role-constant ground truth (the exact tuple
//! values, which are the `carbon()` remap's exact-match lookup keys until the
//! deeper re-root, E9.7 slice 3c) plus the Carbon tokens and the UI metrics. They
//! have no Wayland or GUI dependency, so they gate every build: any accidental
//! drift in a color or metric fails CI immediately. (The Win2000 + BeOS themes
//! were retired in the Carbon-only collapse, E9.7; metrics/font re-base to the
//! Carbon type-scale + 8px grid is the E9.2 follow-on this unblocks.)
//!
//! The dynamic screenshot spot-check (layer 2) lives in the `mde` crate and
//! validates that the *rendered* output actually paints these values.

use mde_ui::metrics;
use mde_ui::palette::{self, Rgb};
use std::sync::Mutex;

// The active theme/mode live in process-global atomics. cargo runs tests in
// parallel threads of one process, so any test that switches the theme and any
// test that reads `color()` expecting a specific theme must serialize through
// this guard, and switchers must restore the Carbon default before releasing.
static THEME_GUARD: Mutex<()> = Mutex::new(());

/// An `iced::Color` channel back to its 0-255 byte, for remap assertions.
fn ch(v: f32) -> u8 {
    (v * 255.0).round() as u8
}

// --- Palette ---------------------------------------------------------------

/// The role constants are the `carbon()` remap's exact-match lookup keys (still the
/// Win2000 ground-truth values until the deeper re-root, E9.7 slice 3c), so pinning
/// them by value keeps `carbon()` from silently missing a role if one ever drifts.
#[test]
fn background_lookup_key_is_pinned() {
    assert_eq!(palette::BACKGROUND, (0x3a, 0x6e, 0xa5));
}

#[test]
fn active_title_is_navy_with_blue_gradient_end() {
    assert_eq!(palette::ACTIVE_TITLE, (0x0a, 0x24, 0x6a));
    assert_eq!(palette::ACTIVE_TITLE_GRADIENT, (0xa6, 0xca, 0xf0));
}

#[test]
fn inactive_title_is_gray() {
    assert_eq!(palette::INACTIVE_TITLE, (0x80, 0x80, 0x80));
}

#[test]
fn selection_highlight_is_navy_on_white() {
    assert_eq!(palette::HIGHLIGHT, (0x0a, 0x24, 0x6a));
    // Sentinel white (0xff,0xff,0xfe) — renders pure white in Win2000; the 1-LSB
    // marker lets the Carbon dark remap keep selection text light. See palette.rs.
    assert_eq!(palette::HIGHLIGHT_TEXT, (0xff, 0xff, 0xfe));
}

#[test]
fn window_and_frame_silver() {
    assert_eq!(palette::WINDOW, (0xff, 0xff, 0xff));
    assert_eq!(palette::MENU, (0xd4, 0xd0, 0xc8));
    assert_eq!(palette::BUTTON_FACE, (0xd4, 0xd0, 0xc8));
}

/// The Carbon sentinels are load-bearing (§2.2): a "fix" to pure white/black would
/// break the dark-mode text/surface split. Pin both by name so they can't drift,
/// plus the SHELL_HEADER role's silver identity (distinct key from BUTTON_FACE).
#[test]
fn carbon_sentinels_and_header_are_pinned() {
    assert_eq!(palette::TITLE_TEXT, (0xff, 0xff, 0xfe)); // white text, distinct from WINDOW surface
    assert_eq!(palette::HIGHLIGHT_TEXT, (0xff, 0xff, 0xfe));
    assert_eq!(palette::WINDOW_FRAME, (0x00, 0x00, 0x01)); // frame, distinct from black TEXT
    assert_eq!(palette::SHELL_HEADER, (0xd4, 0xd0, 0xc7)); // ≠ BUTTON_FACE (…c8)
}

/// The 3D ramp must run strictly light -> dark so bevels read correctly.
#[test]
fn bevel_ramp_is_monotonically_darker() {
    let lum = |c: Rgb| c.0 as u32 + c.1 as u32 + c.2 as u32;
    assert!(lum(palette::BUTTON_HILIGHT) > lum(palette::BUTTON_LIGHT));
    assert!(lum(palette::BUTTON_LIGHT) > lum(palette::BUTTON_FACE));
    assert!(lum(palette::BUTTON_FACE) > lum(palette::BUTTON_SHADOW));
    assert!(lum(palette::BUTTON_SHADOW) > lum(palette::BUTTON_DK_SHADOW));
}

#[test]
fn bevel_endpoints_match_checklist() {
    // raised = white/#dfdfdf (TL) over #808080/#404040 (BR)
    assert_eq!(palette::BUTTON_HILIGHT, (0xff, 0xff, 0xff));
    assert_eq!(palette::BUTTON_LIGHT, (0xdf, 0xdf, 0xdf));
    assert_eq!(palette::BUTTON_SHADOW, (0x80, 0x80, 0x80));
    assert_eq!(palette::BUTTON_DK_SHADOW, (0x40, 0x40, 0x40));
}

/// `color()` must round-trip an 8-bit channel exactly (no gamma surprises). Since
/// the Win2000 identity theme was retired (E9.7), `color()` always applies the
/// `carbon()` remap; pin Carbon-dark `BACKGROUND` → Gray 100 (0x16,0x16,0x16) and
/// check the bytes survive the `f32` round-trip exactly.
#[test]
fn color_conversion_is_exact_8bit() {
    use mde_ui::palette::Theme;
    let _g = THEME_GUARD.lock().unwrap();
    palette::set_theme(Theme::Carbon);
    palette::set_dark(true);
    let c = palette::color(palette::BACKGROUND);
    assert_eq!((ch(c.r), ch(c.g), ch(c.b)), (0x16, 0x16, 0x16));
}

/// MackesDE rebrand (§2.2): the Windows 10 era now uses **Carbon's coloring
/// verbatim** — its former blue accent + per-user accent picker were retired, so
/// the era is a Carbon-skinned modern *layout*, not a distinct palette. Pin that
/// `color()` under `Theme::Windows10` equals `Theme::Carbon` for a representative
/// role set in BOTH modes, so a future re-divergence fails CI. The atomics are
/// process-global, so this holds THEME_GUARD and restores the default before
/// releasing. (Supersedes the old `windows10_remap_pins`, which pinned the
/// now-removed Win10 blue.)
#[test]
fn windows10_uses_carbon_coloring() {
    use mde_ui::palette::Theme;
    let _g = THEME_GUARD.lock().unwrap();

    let roles = [
        palette::HIGHLIGHT,
        palette::ACTIVE_TITLE,
        palette::WINDOW,
        palette::WINDOW_TEXT,
        palette::MENU,
        palette::BACKGROUND,
        palette::TITLE_TEXT,
        palette::WINDOW_FRAME,
        palette::INFO_BAND,
    ];
    let rgb = |c: iced::Color| (ch(c.r), ch(c.g), ch(c.b));
    for dark in [false, true] {
        palette::set_theme(Theme::Carbon);
        palette::set_dark(dark);
        let carbon: Vec<_> = roles.iter().map(|&r| rgb(palette::color(r))).collect();

        palette::set_theme(Theme::Windows10);
        palette::set_dark(dark);
        for (i, &r) in roles.iter().enumerate() {
            assert_eq!(
                rgb(palette::color(r)),
                carbon[i],
                "Win10 role #{i} diverges from Carbon (dark={dark})"
            );
        }
    }

    // The accent is now Carbon Blue, never the retired Win10 blue (0x0078d4).
    palette::set_theme(Theme::Windows10);
    palette::set_dark(true);
    assert_eq!(
        rgb(palette::color(palette::HIGHLIGHT)),
        palette::carbon_accent()
    );

    palette::set_theme(Theme::Carbon); // restore the default
    palette::set_dark(true);
}

/// E14.2: pin the security-status roles (OK / WARN / RISK) + the dashboard tile
/// metric. Identity values are the classic-era green / amber / red; Carbon (and
/// Win10, the Security era) repaint them to the IBM Carbon support palette. Holds
/// THEME_GUARD and restores the Carbon default before releasing.
#[test]
fn security_status_palette_pins() {
    use mde_ui::palette::Theme;
    assert_eq!(palette::STATUS_OK, (0x00, 0x80, 0x00));
    assert_eq!(palette::STATUS_WARN, (0xc0, 0x60, 0x00));
    assert_eq!(palette::STATUS_RISK, (0xc0, 0x00, 0x00));
    assert_eq!(metrics::SECURITY_TILE, 150.0);

    let _g = THEME_GUARD.lock().unwrap();
    let rgb = |c: iced::Color| (ch(c.r), ch(c.g), ch(c.b));
    palette::set_theme(Theme::Carbon);
    palette::set_dark(true);
    assert_eq!(rgb(palette::color(palette::STATUS_OK)), (0x42, 0xbe, 0x65));
    assert_eq!(
        rgb(palette::color(palette::STATUS_WARN)),
        (0xf1, 0xc2, 0x1b)
    );
    assert_eq!(
        rgb(palette::color(palette::STATUS_RISK)),
        (0xfa, 0x4d, 0x56)
    );
    // Win10 shares Carbon's coloring → identical support colors.
    palette::set_theme(Theme::Windows10);
    palette::set_dark(true);
    assert_eq!(rgb(palette::color(palette::STATUS_OK)), (0x42, 0xbe, 0x65));

    palette::set_theme(Theme::Carbon); // restore the default
    palette::set_dark(true);
}

/// E15.12: pin the palette roles the Windows 10 network surfaces paint with — the
/// **accent** (flyout toggle pills, Wi-Fi signal bars, data-usage bars, selection)
/// and the page **surface / caption neutrals** — so the Networking look can't drift
/// under `Theme::Windows10`. They equal Carbon (the rebrand). Holds THEME_GUARD and
/// restores the default at the end.
#[test]
fn windows10_network_palette_pins() {
    use mde_ui::palette::Theme;
    let _g = THEME_GUARD.lock().unwrap();
    let rgb = |c: iced::Color| (ch(c.r), ch(c.g), ch(c.b));

    // The roles the net surfaces use; capture Carbon dark as the reference.
    let roles = [
        palette::HIGHLIGHT,
        palette::WINDOW,
        palette::MENU,
        palette::GRAY_TEXT,
    ];
    palette::set_theme(Theme::Carbon);
    palette::set_dark(true);
    let carbon: Vec<_> = roles.iter().map(|&r| rgb(palette::color(r))).collect();

    palette::set_theme(Theme::Windows10);
    palette::set_dark(true);
    for (i, &r) in roles.iter().enumerate() {
        assert_eq!(
            rgb(palette::color(r)),
            carbon[i],
            "Win10 net role #{i} drifted from Carbon"
        );
    }
    // The accent (pills / bars / selection) is specifically Carbon Blue.
    assert_eq!(
        rgb(palette::color(palette::HIGHLIGHT)),
        palette::carbon_accent()
    );

    palette::set_theme(Theme::Carbon); // restore the default
    palette::set_dark(true);
}

/// App-chrome colors live in the palette too, so nothing outside it names a
/// raw hex; pin them so a future hand-tuned literal fails here instead.
#[test]
fn app_chrome_colors_are_pinned() {
    assert_eq!(palette::INFO_BAND, (0x1d, 0x5c, 0xa8));
    assert_eq!(palette::SETUP_GRADIENT_TOP, (0x1c, 0x4a, 0x8f));
    assert_eq!(palette::SETUP_GRADIENT_BOTTOM, (0x08, 0x16, 0x40));
    assert_eq!(palette::SETUP_PROGRESS, (0x16, 0x3a, 0xa8));
    // Start-menu logo banner brand art (fixed, emitted via hex_fixed).
    assert_eq!(palette::LOGO_BANNER_BG, (0x00, 0x00, 0x00));
    assert_eq!(palette::LOGO_BANNER_GLOW, (0x3a, 0x6a, 0xd0));
    assert_eq!(palette::LOGO_BANNER_GLOW_FADE, (0x0a, 0x1a, 0x40));
    assert_eq!(palette::LOGO_TEXT, (0xff, 0xff, 0xff));
    assert_eq!(palette::LOGO_TEXT_ACCENT, (0x6f, 0x9f, 0xe0));
    // Critical/danger role (wired into the critical-toast tint, E3).
    assert_eq!(palette::URGENT, (0x80, 0x00, 0x00));
}

// --- Metrics ---------------------------------------------------------------

#[test]
fn title_bar_is_18px() {
    assert_eq!(metrics::TITLE_BAR_HEIGHT, 18);
}

#[test]
fn frames_match_win2000() {
    assert_eq!(metrics::SIZE_FRAME, 3);
    assert_eq!(metrics::FIXED_FRAME, 1);
    assert_eq!(metrics::BEVEL_LINE, 1);
}

#[test]
fn taskbar_is_28px() {
    assert_eq!(metrics::TASKBAR_HEIGHT, 28);
}

#[test]
fn scrollbar_and_menu_rows() {
    assert_eq!(metrics::SCROLLBAR, 16);
    assert_eq!(metrics::MENU_HEIGHT, 18);
}

/// Pin what the renderer ACTUALLY ships, not the unattainable target. Win2000's
/// Tahoma isn't freely distributable, so the shell renders Droid Sans; a green
/// "accuracy" test must never launder that approximation by asserting "Tahoma".
/// The target is recorded separately so the gap stays named.
#[test]
fn ui_font_is_the_shipped_substitute() {
    assert_eq!(mde_ui::font::FAMILY, "Droid Sans"); // the family every renderer loads
    assert_eq!(metrics::UI_FONT_TARGET, "Tahoma"); // the documented ground truth
    const { assert!(metrics::TITLE_FONT_BOLD) }; // title bars are bold (compile-time pin)
}

/// 8pt at 96 DPI is 10.67px → 11; UI_PX is the single size the renderer uses,
/// so "8pt everywhere" is one derived constant rather than 38 magic literals.
#[test]
fn ui_size_is_one_source_of_truth() {
    assert_eq!(metrics::UI_FONT_PT, 8.0);
    assert_eq!(metrics::UI_PX, (metrics::UI_FONT_PT * 96.0 / 72.0).round());
    // INFO_TITLE_PX is §2.3's one larger UI size (info-band/about/control-panel
    // headings); pin it so a silent drift fails CI like UI_PX does.
    assert_eq!(metrics::INFO_TITLE_PX, 16.0);
    // The remaining named UI sizes (§2.3): the Identify overlay and the two
    // Setup-wizard sizes. Pinned so they stay a single source, not literals.
    assert_eq!(metrics::IDENTIFY_PX, 48.0);
    assert_eq!(metrics::WIZARD_HEADING_PX, 15.0);
    assert_eq!(metrics::WIZARD_STATUS_PX, 10.0);
    // The taskbar window-button minimum width (SM_*-style layout metric).
    assert_eq!(metrics::TASKBAR_BUTTON_MIN, 160);
    // The Nerd-glyph / badge chrome sizes (§2.3): named so panel.rs and
    // action_center.rs carry no scattered glyph `.size()` literals.
    assert_eq!(metrics::PANEL_GLYPH_PX, 15.0);
    assert_eq!(metrics::BUTTON_GLYPH_PX, 16.0);
    assert_eq!(metrics::START_GLYPH_PX, 18.0);
    assert_eq!(metrics::TILE_GLYPH_PX, 20.0);
    assert_eq!(metrics::BADGE_PX, 9.0);
}

/// E9.2 — pin the IBM Carbon design-token substrate: the `$spacing-01..13` 8px
/// step scale and the Carbon type scale, in device px at 96 DPI. Per §2.3 change a
/// token only with a Carbon-spec reference + this pin in the same commit. (These
/// are the single source converted surfaces size from; the dense shell chrome
/// keeps its `SM_*` metrics as documented pragmatic exceptions until it converts.)
#[test]
fn carbon_spacing_and_type_tokens_pinned() {
    // 8px spacing scale.
    assert_eq!(metrics::SPACING_01, 2.0);
    assert_eq!(metrics::SPACING_02, 4.0);
    assert_eq!(metrics::SPACING_03, 8.0);
    assert_eq!(metrics::SPACING_04, 12.0);
    assert_eq!(metrics::SPACING_05, 16.0);
    assert_eq!(metrics::SPACING_06, 24.0);
    assert_eq!(metrics::SPACING_07, 32.0);
    assert_eq!(metrics::SPACING_08, 40.0);
    assert_eq!(metrics::SPACING_09, 48.0);
    assert_eq!(metrics::SPACING_10, 64.0);
    assert_eq!(metrics::SPACING_11, 80.0);
    assert_eq!(metrics::SPACING_12, 96.0);
    assert_eq!(metrics::SPACING_13, 160.0);
    // Type scale.
    assert_eq!(metrics::TYPE_LABEL_01, 12.0);
    assert_eq!(metrics::TYPE_BODY_01, 14.0);
    assert_eq!(metrics::TYPE_BODY_02, 16.0);
    assert_eq!(metrics::TYPE_HEADING_03, 20.0);
    assert_eq!(metrics::TYPE_HEADING_04, 28.0);
    assert_eq!(metrics::TYPE_HEADING_05, 32.0);
    assert_eq!(metrics::TYPE_HEADING_06, 42.0);
    assert_eq!(metrics::TYPE_HEADING_07, 54.0);
    // Carbon button heights (E9.3): small 32 / medium 40 / large 48.
    assert_eq!(metrics::BUTTON_SM, 32.0);
    assert_eq!(metrics::BUTTON_MD, 40.0);
    assert_eq!(metrics::BUTTON_LG, 48.0);
}
