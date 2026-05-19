//! Autostart applier — Phase A stub.
//!
//! Phase C reads `~/.config/autostart/*.desktop` plus the system-wide
//! `/etc/xdg/autostart/`, applies `Hidden=true` to the items in
//! `autostart.hidden`, and writes new `.desktop` entries for the
//! items in `autostart.extra`. `mackes-session` (Phase D) honors
//! these on each login.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply an `autostart.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("autostart: {UNIMPLEMENTED}");
}

/// Read the current `autostart.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("autostart: {UNIMPLEMENTED}");
}
