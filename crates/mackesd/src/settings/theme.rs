//! Theme applier — Phase A stub.
//!
//! Phase C wires GTK + libcosmic theme + accent + icon-theme by:
//!   1. Writing the matching `gsettings set org.gnome.desktop.interface`
//!      keys (libadwaita-aware apps).
//!   2. Updating `cosmic-config` under `com.mackes.Theme` (libcosmic
//!      apps in the panel + applets).
//!   3. Re-deriving the `cosmic-theme::Theme` token bundle via the
//!      new `crates/mackes-theme/` adapter at process startup; live
//!      apps reload via the cosmic-config file-watcher.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `theme.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("theme: {UNIMPLEMENTED}");
}

/// Read the current `theme.*` setting from the live system. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("theme: {UNIMPLEMENTED}");
}
