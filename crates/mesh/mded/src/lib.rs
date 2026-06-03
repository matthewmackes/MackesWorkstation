//! Mackes Desktop Environment (MDE) meta-daemon — re-export
//! facade over `mackesd_core`.
//!
//! Phase 0.2 transitional crate: new code can call `use mded::…`
//! during the v2.0.0 back-compat window. Everything in
//! `mackesd_core` is re-exported here; type identity is
//! preserved (`mded::Worker` IS `mackesd_core::Worker`, same
//! struct), so the two paths are interchangeable.
//!
//! The actual implementation lives in `crates/mackesd/`. The
//! directory rename + drop of this alias lands at the v2.0.0
//! cut commit per CB-3.1.

#![forbid(unsafe_code)]

// Re-export everything mackesd_core surfaces. `*` because the
// crate's exported set is the authoritative API; this facade is
// strictly pass-through.
pub use mackesd_core::*;
