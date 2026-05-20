//! Mackes Desktop Environment (MDE) — re-export facade over
//! `mackes_theme`.
//!
//! Phase 0.2 transitional crate. The directory name
//! `mde-theme-alias` exists to keep clear of the eventual
//! `mackes-theme` → `mde-theme` rename at the v2.0.0 cut. The
//! lib name `mde_theme` is what callers import.

#![forbid(unsafe_code)]

pub use mackes_theme::*;
