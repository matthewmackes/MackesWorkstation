//! Power applier — Phase A stub.
//!
//! Phase C wires lid / suspend / profile / caffeine via:
//!   - `org.freedesktop.login1` (logind DBus): lid handler, suspend.
//!   - `power-profiles-daemon` DBus for `power-saver` /
//!     `balanced` / `performance`.
//!   - `swayidle` config rewriting for idle timeouts.
//!   - `systemd-inhibit` for presentation-mode "caffeine" lock.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `power.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("power: {UNIMPLEMENTED}");
}

/// Read the current `power.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("power: {UNIMPLEMENTED}");
}
