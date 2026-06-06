//! UI metrics at 96 DPI.
//!
//! Two layers live here: the **Carbon design tokens** (the 8px spacing scale +
//! the Carbon type scale — the single source for new/converted surfaces, E9.2),
//! and the legacy `SM_*`-derived chrome metrics the dense shell still uses as
//! documented pragmatic exceptions (the panel/tray/title-bar were sized to the
//! classic system metrics; re-sizing them to the Carbon scale is per-surface E9.3
//! work, not a blanket re-base). Adjust a Carbon token only with a Carbon-spec
//! reference + the matching `checklist.rs` pin in the same commit (§2.2/§2.3).

// --- Carbon 8px spacing scale (E9.2) ---------------------------------------
// The IBM Carbon v11 `$spacing-01..13` step scale (carbondesignsystem.com/
// elements/spacing). The one source for layout gaps/padding on Carbon surfaces,
// so a `.spacing(..)`/`.padding(..)` is a named step, not a raw float. Dense
// shell chrome that predates the grid keeps its `SM_*` value as a documented
// exception until its surface converts (E9.3).
pub const SPACING_01: f32 = 2.0;
pub const SPACING_02: f32 = 4.0;
pub const SPACING_03: f32 = 8.0;
pub const SPACING_04: f32 = 12.0;
pub const SPACING_05: f32 = 16.0;
pub const SPACING_06: f32 = 24.0;
pub const SPACING_07: f32 = 32.0;
pub const SPACING_08: f32 = 40.0;
pub const SPACING_09: f32 = 48.0;
pub const SPACING_10: f32 = 64.0;
pub const SPACING_11: f32 = 80.0;
pub const SPACING_12: f32 = 96.0;
pub const SPACING_13: f32 = 160.0;

// --- Carbon type scale (E9.2) ----------------------------------------------
// The IBM Carbon v11 type tokens (carbondesignsystem.com/elements/typography/
// type-sets), in device px at 96 DPI. Named here so converted surfaces size text
// from a Carbon token. The dense shell body text keeps `UI_PX` (8pt → 11px) as a
// documented pragmatic exception (a Carbon `body-01` 14px shell would be too
// large for the Win2000-derived chrome density); these tokens are for headings
// and converted surfaces.
pub const TYPE_LABEL_01: f32 = 12.0; // caption / label / helper-text
pub const TYPE_BODY_01: f32 = 14.0;
pub const TYPE_BODY_02: f32 = 16.0;
pub const TYPE_HEADING_03: f32 = 20.0;
pub const TYPE_HEADING_04: f32 = 28.0;
pub const TYPE_HEADING_05: f32 = 32.0;
pub const TYPE_HEADING_06: f32 = 42.0;
pub const TYPE_HEADING_07: f32 = 54.0;

// --- Legacy SM_*-derived chrome metrics (pragmatic exceptions) -------------
// The classic `SM_*` system metrics the dense shell chrome still uses (panel,
// tray, title bar, scrollbar). These are the target numbers the accuracy harness
// checks against (see `ACCURACY.md`); adjust only with a reference to back it.

/// Title-bar height (SM_CYCAPTION), excluding the 3D frame.
pub const TITLE_BAR_HEIGHT: u16 = 18;
/// Sizing-frame thickness around a resizable window (SM_CXSIZEFRAME).
/// labwc-owned today: labwc draws the window frame, so this is transcribed for
/// completeness, not applied by mde (see ACCURACY.md §0).
pub const SIZE_FRAME: u16 = 3;
/// Thin 3D frame thickness around a fixed/dialog window. labwc-owned (as above).
pub const FIXED_FRAME: u16 = 1;
/// Each bevel is two 1px lines.
pub const BEVEL_LINE: u16 = 1;
/// Scrollbar thickness (SM_CXVSCROLL / SM_CYHSCROLL).
pub const SCROLLBAR: u16 = 16;
/// Menu-bar item height (SM_CYMENU).
pub const MENU_HEIGHT: u16 = 18;
/// The taskbar height (one row of 28px-ish buttons + bevel).
pub const TASKBAR_HEIGHT: u16 = 28;
/// Default min width of a taskbar window button before it elides.
pub const TASKBAR_BUTTON_MIN: u16 = 160;

/// The Win2000 UI font — the ground-truth TARGET. Tahoma is not freely
/// distributable, so the shell renders a substitute (`mde_ui::font::FAMILY`);
/// this records the original so the gap is named, not hidden. The renderer
/// never requests this string — see `font.rs` for what actually ships.
pub const UI_FONT_TARGET: &str = "Tahoma";
/// The Windows 10 era's UI font TARGET — Segoe UI, likewise not redistributable.
/// Per §2.4 the gap is named, not laundered: the Win10 era ships the already-
/// bundled IBM Plex Sans (`font::PLEX_FAMILY`) as its humanist-sans substitute,
/// so no new TTF/licence is added. The renderer never requests this string.
pub const UI_FONT_TARGET_WIN10: &str = "Segoe UI";
/// UI font size in points (Tahoma 8pt) — the transcribed system value.
pub const UI_FONT_PT: f32 = 8.0;
/// `UI_FONT_PT` in device pixels at 96 DPI (8pt → 10.67px, rounded to 11): the
/// ONE size every UI `.size(...)` call must use, so the "8pt everywhere" rule
/// has a single source of truth instead of scattered literals.
pub const UI_PX: f32 = 11.0;
/// The web-view info-band folder title — the one larger display size in the
/// shell (Win2000 drew this caption well above body text). Single source so the
/// band title isn't a scattered literal either.
pub const INFO_TITLE_PX: f32 = 16.0;
/// Setup-wizard heading size (the "Choose Components" step title). Named so the
/// installer doesn't carry a scattered literal (§2.3).
pub const WIZARD_HEADING_PX: f32 = 15.0;
/// Setup-wizard status-bar caption size (smaller than UI text).
pub const WIZARD_STATUS_PX: f32 = 10.0;
/// The big monitor-number overlay drawn by Display ▸ Identify.
pub const IDENTIFY_PX: f32 = 48.0;
/// The large clock on the Windows 10 lock screen (`mde lock`, E10.8).
pub const LOCK_CLOCK_PX: f32 = 72.0;
/// Title-bar font is the UI font, bold, at the same size.
pub const TITLE_FONT_BOLD: bool = true;
/// The Windows 10 Task View window-tile size (px) — a square-ish window card.
pub const TASKVIEW_TILE: f32 = 200.0;
/// The Windows 10 Security dashboard status-tile size (px), E14.2 — a square
/// tile (icon + title + status line) in the 6-up home grid.
pub const SECURITY_TILE: f32 = 150.0;

// --- Nerd-glyph / badge sizes (§2.3) ----------------------------------------
// The shell's chrome glyphs are sized larger than body text. Named here so each
// is a single source like `UI_PX`, not a literal scattered across panel.rs /
// action_center.rs. Pinned in `checklist.rs::ui_size_is_one_source_of_truth`.

/// Standard taskbar / tray / notification-area Nerd glyph (the ≡ Start switcher,
/// SNI tray glyphs, the Win10 search magnifier).
pub const PANEL_GLYPH_PX: f32 = 15.0;
/// Slightly larger Nerd glyph for the Win10 bar's named buttons (Task View,
/// Action Center) and the Action Center's inline affordance glyphs (brightness
/// sun, "All settings" gear).
pub const BUTTON_GLYPH_PX: f32 = 16.0;
/// The prominent Win10 Start-button logo glyph (larger than the other bar icons).
pub const START_GLYPH_PX: f32 = 18.0;
/// The large glyph centered in an Action Center quick-action square tile.
pub const TILE_GLYPH_PX: f32 = 20.0;
/// The tiny unread-count chip drawn on the Win10 Action Center button.
pub const BADGE_PX: f32 = 9.0;
