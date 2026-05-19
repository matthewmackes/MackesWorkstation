//! `org.mackes.Session` — session lifecycle. *Schema lives here;
//! server impl lives in `crates/mackes-session/`.*
//!
//! This module defines the canonical message shape so every consumer
//! (Workbench Python panels via DBus, the panel applets, mackesd's
//! Fleet service, etc.) imports the same interface name. The
//! `mackes-session` binary in Phase D constructs a real
//! `SessionService` struct that holds compositor lifecycle state.

#![cfg(feature = "async-services")]

use zbus::interface;

/// Placeholder service struct. `mackes-session` will replace this
/// with one that owns the running sway process handle + autostart
/// state. Phase A: just enough surface to compile + emit signals.
#[derive(Debug, Default, Clone)]
pub struct SessionService;

#[interface(name = "org.mackes.Session")]
impl SessionService {
    /// Request a clean logout (sway exits, graphical-session.target
    /// stops, the user is returned to the greeter).
    async fn logout(&self) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Session.Logout — implemented by mackes-session (Phase D)".into(),
        ))
    }

    /// Reboot the machine via `systemctl reboot`.
    async fn restart(&self) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Session.Restart — implemented by mackes-session (Phase D)".into(),
        ))
    }

    /// Power-off via `systemctl poweroff`.
    async fn shutdown(&self) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Session.Shutdown — implemented by mackes-session (Phase D)".into(),
        ))
    }

    /// Lock the session (invokes swaylock or the user-configured
    /// locker).
    async fn lock(&self) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Session.Lock — implemented by mackes-session (Phase D)".into(),
        ))
    }

    /// Persist the current sway window layout for restore-on-login.
    async fn save_layout(&self) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Session.SaveLayout — implemented by mackes-session (Phase D)".into(),
        ))
    }

    /// Signal: session about to end. Emitted by mackes-session before
    /// sway exits, so applets can save state.
    #[zbus(signal)]
    pub async fn ending(emitter: &zbus::object_server::SignalEmitter<'_>) -> zbus::Result<()>;
}
