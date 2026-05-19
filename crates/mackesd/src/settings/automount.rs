//! Automount applier — Phase A stub.
//!
//! Phase C wires removable-media policies via udisks2 DBus
//! (`org.freedesktop.UDisks2`). Replaces the `thunar-volman` xfconf
//! channel entirely. Each key maps to a udisks2 setting:
//!   - `automount.on_insert`     → `org.freedesktop.UDisks2.Filesystem.Mount` filter
//!   - `automount.open_on_mount` → file-manager spawn helper inside the daemon
//!   - `automount.autorun`       → explicit allow/deny (default deny)

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply an `automount.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("automount: {UNIMPLEMENTED}");
}

/// Read the current `automount.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("automount: {UNIMPLEMENTED}");
}
