//! `org.freedesktop.Notifications` — the desktop notification spec,
//! implemented by mackesd (Phase B.10).
//!
//! Phase A ships the interface shape every spec-compliant client
//! expects. Phase B wires it through to the `notifications` SQLite
//! table + the Iced applet overlay at
//! `crates/mackes-applets/notifications/`.
//!
//! By matching the spec object path + bus name in Phase B, every
//! libnotify / notify-send / GTK app reaches mackesd transparently,
//! retiring mako/fnott/xfce4-notifyd in one stroke.

#![cfg(feature = "async-services")]

use std::collections::HashMap;

use zbus::interface;
use zbus::zvariant::Value;

/// Object exposed at `/org/freedesktop/Notifications`. Phase A: shell.
#[derive(Debug, Default, Clone)]
pub struct NotificationsService;

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationsService {
    /// Notify a user. Returns the notification id (which the spec
    /// requires to be a u32 ≥ 1). Phase A: returns a synthetic id 1
    /// without persisting anything.
    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &self,
        _app_name: &str,
        replaces_id: u32,
        _app_icon: &str,
        _summary: &str,
        _body: &str,
        _actions: Vec<&str>,
        _hints: HashMap<&str, Value<'_>>,
        _expire_timeout: i32,
    ) -> u32 {
        // Phase B: persist to the notifications table + emit
        // NotificationClosed/ActionInvoked signals as the user
        // interacts with the Iced overlay.
        if replaces_id == 0 {
            1
        } else {
            replaces_id
        }
    }

    /// Close a previously-sent notification.
    async fn close_notification(&self, _id: u32) {
        // Phase B: stamp dismissed_at in the notifications table +
        // emit NotificationClosed(id, reason=3).
    }

    /// Server capabilities the spec requires us to advertise.
    /// Pinned at Phase A; we'll add `"actions"`, `"action-icons"`,
    /// `"body-markup"` in Phase B once the applet supports them.
    async fn get_capabilities(&self) -> Vec<&'static str> {
        vec!["body", "persistence", "icon-static"]
    }

    /// Server identity. Spec requires (name, vendor, version, spec_version).
    async fn get_server_information(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        ("mackesd", "mackes-shell", env!("CARGO_PKG_VERSION"), "1.2")
    }

    /// Signal: a notification was closed (by the user, by timeout,
    /// or by the server). `reason` follows the spec:
    /// 1 = expired, 2 = dismissed by user, 3 = closed by call,
    /// 4 = undefined / reserved.
    #[zbus(signal)]
    pub async fn notification_closed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    /// Signal: the user invoked an action.
    #[zbus(signal)]
    pub async fn action_invoked(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capabilities_include_body_and_persistence() {
        let svc = NotificationsService;
        let caps = svc.get_capabilities().await;
        assert!(caps.contains(&"body"));
        assert!(caps.contains(&"persistence"));
    }

    #[tokio::test]
    async fn server_info_reports_mackesd() {
        let svc = NotificationsService;
        let (name, vendor, _version, spec) = svc.get_server_information().await;
        assert_eq!(name, "mackesd");
        assert_eq!(vendor, "mackes-shell");
        assert_eq!(spec, "1.2", "must match freedesktop spec version");
    }

    #[tokio::test]
    async fn notify_returns_replaces_id_when_nonzero() {
        let svc = NotificationsService;
        let id = svc
            .notify(
                "test-app",
                42,
                "",
                "summary",
                "body",
                vec![],
                HashMap::new(),
                -1,
            )
            .await;
        assert_eq!(id, 42, "non-zero replaces_id must be honored per spec");
    }
}
