//! Notification applier — Phase A stub.
//!
//! Phase C wires DND + location + default-expire by calling into
//! `crate::workers::notifications_server` (mackesd is the
//! `org.freedesktop.Notifications` daemon — see Phase B.10). The
//! applier is just a thin shim that updates the worker's runtime
//! config; storage is the `settings` table.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `notification.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("notification: {UNIMPLEMENTED}");
}

/// Read the current `notification.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("notification: {UNIMPLEMENTED}");
}
