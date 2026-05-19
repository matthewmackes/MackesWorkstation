//! Display applier — Phase A stub.
//!
//! Phase C wires resolution / scale / brightness / night-light via:
//!   - `wlr-output-management-unstable-v1` (smithay-client-toolkit).
//!   - `brightnessctl set N%` (DRM kernel API; Wayland-portable).
//!   - `swaymsg output <name> …` for compositor-level adjustments.
//!   - `gammastep` (or built-in night-light) for the temperature
//!     overlay.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `display.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("display: {UNIMPLEMENTED}");
}

/// Read the current `display.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("display: {UNIMPLEMENTED}");
}
