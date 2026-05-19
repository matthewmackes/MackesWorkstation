//! Wallpaper applier — Phase A stub.
//!
//! Phase C writes per-output wallpaper config that
//! `crates/mackes-applets/bg/` (the cosmic-bg-style layer-shell
//! surface) reads via cosmic-config. The bg applet is its own process
//! anchored at `Layer::Background`; on config change it reloads via
//! the cosmic-config file-watcher.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `wallpaper.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("wallpaper: {UNIMPLEMENTED}");
}

/// Read the current `wallpaper.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("wallpaper: {UNIMPLEMENTED}");
}
