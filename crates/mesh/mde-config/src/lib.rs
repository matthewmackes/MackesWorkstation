//! Mackes Desktop Environment (MDE) — re-export facade over
//! `mackes_config`.
//!
//! Phase 0.2 transitional crate: new code can call
//! `use mde_config::…` during the v2.0.0 back-compat window.

#![forbid(unsafe_code)]

pub use mackes_config::*;
