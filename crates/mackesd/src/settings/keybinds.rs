//! Keybinds applier — Phase A stub.
//!
//! Phase C renders the binding map into
//! `~/.config/sway/config.d/mackes-bindings.conf` and runs
//! `swaymsg reload`. The map is a JSON dict of `{ "Mod+key": "exec
//! command", ... }`. Defaults live in
//! `data/sway/config.d/mackes-defaults.conf`; user overrides win
//! lexicographically because the include directive sorts.

use super::{SettingKey, SettingValue, UNIMPLEMENTED};

/// Apply a `keybinds.*` setting. Stub until Phase C.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn apply(_key: SettingKey, _value: &SettingValue) -> anyhow::Result<()> {
    anyhow::bail!("keybinds: {UNIMPLEMENTED}");
}

/// Read the current `keybinds.*` setting. Stub.
///
/// # Errors
///
/// Always returns the Phase A "unimplemented" sentinel.
pub fn current(_key: SettingKey) -> anyhow::Result<SettingValue> {
    anyhow::bail!("keybinds: {UNIMPLEMENTED}");
}
