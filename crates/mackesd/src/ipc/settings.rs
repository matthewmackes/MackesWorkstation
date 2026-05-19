//! `dev.mackes.MDE.Settings` — DBus surface for the settings store.
//!
//! Phase A ships the interface decoration and a service struct that
//! holds no state yet. Phase C wires it through to
//! `crate::settings::{apply, current}` + the SQLite `settings` table.
//!
//! v2.0.0 Phase 0.4 rebrand — interface name moved from
//! `org.mackes.Settings`. Backward-compat alias .service file ships
//! under the old name for one release; see `data/dbus-1/services/`.

#![cfg(feature = "async-services")]

use zbus::interface;

/// Object exposed at `/dev/mackes/MDE/Settings` on the session bus.
#[derive(Debug, Default, Clone)]
pub struct SettingsService;

/// Stable D-Bus name used by Phase 0.4-onward callers.
pub const SERVICE_NAME: &str = "dev.mackes.MDE.Settings";

/// Object-path under [`SERVICE_NAME`].
pub const OBJECT_PATH: &str = "/dev/mackes/MDE/Settings";

#[interface(name = "dev.mackes.MDE.Settings")]
impl SettingsService {
    /// Read a setting by dot-notated key. Returns the JSON-encoded
    /// value as a string. Phase A: always returns the Phase A
    /// "unimplemented" sentinel.
    async fn get(&self, key: &str) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::Failed(format!(
            "Settings.Get({key}) — {}",
            crate::settings::UNIMPLEMENTED
        )))
    }

    /// Write a setting by dot-notated key. `value_json` is the
    /// JSON-encoded payload. Phase A: stub.
    async fn set(&self, key: &str, _value_json: &str) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::Failed(format!(
            "Settings.Set({key}) — {}",
            crate::settings::UNIMPLEMENTED
        )))
    }

    /// Enumerate every known setting key (dot-notated string form).
    /// Returns immediately — no DB hit, no I/O.
    async fn list_keys(&self) -> Vec<String> {
        crate::settings::SettingKey::all()
            .iter()
            .map(|k| k.as_str().to_string())
            .collect()
    }

    /// Snapshot every current value. Phase A: stub.
    async fn snapshot(&self) -> zbus::fdo::Result<String> {
        Err(zbus::fdo::Error::Failed(format!(
            "Settings.Snapshot — {}",
            crate::settings::UNIMPLEMENTED
        )))
    }

    /// Restore from a snapshot JSON. Phase A: stub.
    async fn restore(&self, _snapshot_json: &str) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::Failed(format!(
            "Settings.Restore — {}",
            crate::settings::UNIMPLEMENTED
        )))
    }

    /// Signal: a setting changed. `key` is the dot-notated key.
    /// Emitted by the applier path after a successful Set or
    /// reconcile push.
    #[zbus(signal)]
    pub async fn changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        key: &str,
    ) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_keys_returns_every_setting_key() {
        let svc = SettingsService;
        let keys = svc.list_keys().await;
        assert_eq!(keys.len(), crate::settings::SettingKey::all().len());
        assert!(keys.iter().any(|k| k == "theme.accent"));
        assert!(keys.iter().any(|k| k == "power.profile"));
    }

    #[tokio::test]
    async fn get_returns_unimplemented_in_phase_a() {
        let svc = SettingsService;
        let err = svc.get("theme.name").await.unwrap_err();
        assert!(format!("{err}").to_lowercase().contains("phase c")
            || format!("{err}").to_lowercase().contains("not implemented"));
    }
}
