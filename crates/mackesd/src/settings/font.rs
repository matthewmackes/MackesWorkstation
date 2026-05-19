//! Font applier — Phase A stub.
//!
//! Phase C wires font name + hinting + antialias via:
//!   - `gsettings set org.gnome.desktop.interface font-name <…>`
//!   - `~/.config/fontconfig/fonts.conf` rewriting + `fc-cache -r`
//!   - cosmic-config `com.mackes.Theme` font keys (libcosmic apps).

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `font.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("font: {UNIMPLEMENTED}");
}

/// Read the current `font.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("font: {UNIMPLEMENTED}");
}
