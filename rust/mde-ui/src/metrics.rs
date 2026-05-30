//! Windows 2000 default UI metrics at 96 DPI.
//!
//! These are the target numbers the accuracy harness checks against (see
//! `rust/ACCURACY.md`). Values are the classic `SM_*` system metrics; adjust
//! only with a reference screenshot to back the change.

/// Title-bar height (SM_CYCAPTION), excluding the 3D frame.
pub const TITLE_BAR_HEIGHT: u16 = 18;
/// Sizing-frame thickness around a resizable window (SM_CXSIZEFRAME).
pub const SIZE_FRAME: u16 = 3;
/// Thin 3D frame thickness around a fixed/dialog window.
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

/// UI font family. Win2000 ships Tahoma; MDE-Retro aliases it to a humanist
/// sans where Tahoma is absent (see `fontconfig/fonts.conf`).
pub const UI_FONT: &str = "Tahoma";
/// UI font size in points (Tahoma 8pt).
pub const UI_FONT_PT: f32 = 8.0;
/// Title-bar font is Tahoma Bold at the same size.
pub const TITLE_FONT_BOLD: bool = true;
