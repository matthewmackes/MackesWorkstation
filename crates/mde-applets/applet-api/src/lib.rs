//! `mde-applet-api` — the shared `Applet` trait + state
//! types every Phase E1 applet binary implements.
//!
//! Phase E1.1: split the in-process mackes-panel monolith
//! into separate Iced binaries (one per applet) that the
//! panel host discovers via a manifest at
//! `/usr/share/mde/applets/*.json`. This crate holds the
//! cross-binary contract — types both the host and the
//! applet bins agree on.
//!
//! Why "API" as a separate crate: the panel host doesn't
//! link the applets (they're separate processes); both
//! sides just share these structs through stable JSON
//! manifests on disk + a small message-vocab over wl_data
//! when the host hands events to an applet.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Stable identifier for an applet, used as the basename of
/// its install manifest at
/// `/usr/share/mde/applets/{id}.json`. Lowercase ASCII +
/// hyphens — matches the panel-host's `applets.<id>`
/// settings keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppletId(pub String);

impl AppletId {
    /// Construct an `AppletId` from a string slice without
    /// validating. Use [`AppletId::parse`] when the input
    /// comes from user data.
    #[must_use]
    pub fn from_static(s: &'static str) -> Self {
        Self(s.to_string())
    }

    /// Parse + validate an applet id. Returns `Err` when the
    /// string contains anything outside `[a-z0-9-]` or is
    /// empty / overlong.
    ///
    /// # Errors
    ///
    /// Returns `Err(&'static str)` with a human-readable
    /// reason on invalid input.
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        if s.is_empty() {
            return Err("applet id cannot be empty");
        }
        if s.len() > 64 {
            return Err("applet id must be ≤ 64 characters");
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err("applet id may only contain lowercase ASCII, digits, and `-`");
        }
        Ok(Self(s.to_string()))
    }

    /// Bare-string view, useful for filesystem joins.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One row in the applet-manifest JSON the panel host
/// reads at startup. Format:
///
/// ```json
/// {
///   "id": "clock",
///   "binary": "mde-applet-clock",
///   "slot": "top-bar-right",
///   "summary": "Clock + date pill",
///   "version": "2.0.0"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppletManifest {
    /// Stable applet identifier — `lowercase-kebab`.
    pub id: AppletId,
    /// Binary basename the panel host execs.
    pub binary: String,
    /// Which panel slot this applet occupies.
    pub slot: AppletSlot,
    /// One-line human-readable description for the
    /// settings UI.
    pub summary: String,
    /// SemVer string the host can compare against its
    /// minimum-supported manifest version.
    pub version: String,
}

/// Panel slot an applet wants to occupy. The host enforces
/// at most one applet per slot per output; conflicting
/// manifests fall back to alphabetical id ordering with the
/// later id wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppletSlot {
    /// Top bar, leftmost cluster.
    TopBarLeft,
    /// Top bar, middle cluster (date/clock by default).
    TopBarCenter,
    /// Top bar, right cluster (status icons, notifications).
    TopBarRight,
    /// Bottom dock — taskbar.
    Dock,
    /// Layer-shell overlay (wallpaper, popovers).
    Overlay,
}

impl AppletSlot {
    /// Stable string form used in manifests + settings keys.
    /// Mirrors the `kebab-case` rename rule above; kept as a
    /// `pub const` so tests can pin it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopBarLeft => "top-bar-left",
            Self::TopBarCenter => "top-bar-center",
            Self::TopBarRight => "top-bar-right",
            Self::Dock => "dock",
            Self::Overlay => "overlay",
        }
    }
}

/// Per-applet runtime state passed to every reducer. Owned
/// by the applet binary; the host only sees rendered output
/// via the standard Iced `view() -> Element` contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppletState {
    /// Whether the applet is currently active (visible in
    /// its slot). The host toggles this via a settings change
    /// or a workspace-switch event.
    pub active: bool,
    /// Active accent color the applet should paint with.
    /// `#RRGGBB` hex — the host pushes this on theme change.
    pub accent: String,
}

impl AppletState {
    /// Construct an active state with a given accent.
    #[must_use]
    pub fn active_with_accent(accent: impl Into<String>) -> Self {
        Self {
            active: true,
            accent: accent.into(),
        }
    }
}

/// Inbound message vocabulary the panel host pushes to
/// running applet binaries via a stable JSON-line protocol
/// on stdin. Applets respond with rendered events on stdout
/// (decoded by the host into Iced sub-messages).
///
/// Today the messages are minimal — the host MVP only sends
/// `Accent` + `Visibility` events. Phase E1.3 fills in the
/// click-through routing, the wakeup pulse for poll-driven
/// applets (network, audio), and the suspend signal for the
/// laptop-lid path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostMessage {
    /// Theme change — accent color updates.
    Accent {
        /// `#RRGGBB` accent color string.
        color: String,
    },
    /// Slot visibility changed (active workspace, panel
    /// hide-when-fullscreen, etc.).
    Visibility {
        /// `true` if the applet should now render; `false`
        /// to pause rendering (state is preserved).
        active: bool,
    },
    /// Shutdown signal — applet should flush state + exit 0.
    Shutdown,
}

/// Every applet's `main()` lands an `Applet` impl. The
/// public methods match Iced's stateful-component pattern
/// (init, view, update, subscription) plus the
/// host-protocol hooks.
pub trait Applet: Sized + 'static {
    /// The applet's Iced message type.
    type Message: 'static;

    /// Stable identifier — matches the manifest `id`.
    fn id(&self) -> AppletId;

    /// Apply a [`HostMessage`] (theme change, visibility,
    /// shutdown). Returns `true` when the applet should
    /// re-render its view in response.
    fn handle_host(&mut self, msg: HostMessage) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applet_id_parse_accepts_canonical_form() {
        assert!(AppletId::parse("clock").is_ok());
        assert!(AppletId::parse("notification-bell").is_ok());
        assert!(AppletId::parse("mesh-status-v2").is_ok());
        assert_eq!(AppletId::parse("clock").unwrap().as_str(), "clock");
    }

    #[test]
    fn applet_id_parse_rejects_invalid_inputs() {
        assert!(AppletId::parse("").is_err());
        assert!(AppletId::parse("Clock").is_err());
        assert!(AppletId::parse("with space").is_err());
        assert!(AppletId::parse("under_score").is_err());
        assert!(AppletId::parse(&"x".repeat(65)).is_err());
    }

    #[test]
    fn slot_as_str_matches_kebab_case_rename() {
        assert_eq!(AppletSlot::TopBarLeft.as_str(), "top-bar-left");
        assert_eq!(AppletSlot::TopBarCenter.as_str(), "top-bar-center");
        assert_eq!(AppletSlot::TopBarRight.as_str(), "top-bar-right");
        assert_eq!(AppletSlot::Dock.as_str(), "dock");
        assert_eq!(AppletSlot::Overlay.as_str(), "overlay");
    }

    #[test]
    fn slot_serde_round_trip_uses_kebab_case() {
        let s = serde_json::to_string(&AppletSlot::TopBarRight).unwrap();
        assert_eq!(s, "\"top-bar-right\"");
        let back: AppletSlot = serde_json::from_str(&s).unwrap();
        assert_eq!(back, AppletSlot::TopBarRight);
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let m = AppletManifest {
            id: AppletId::from_static("clock"),
            binary: "mde-applet-clock".into(),
            slot: AppletSlot::TopBarCenter,
            summary: "Clock + date pill".into(),
            version: "2.0.0".into(),
        };
        let s = serde_json::to_string(&m).unwrap();
        let back: AppletManifest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn applet_state_active_with_accent_is_active() {
        let s = AppletState::active_with_accent("#6366f1");
        assert!(s.active);
        assert_eq!(s.accent, "#6366f1");
    }

    #[test]
    fn host_message_serde_round_trip_uses_snake_case_tag() {
        let m = HostMessage::Accent {
            color: "#ff0000".into(),
        };
        let s = serde_json::to_string(&m).unwrap();
        assert!(s.contains("\"kind\":\"accent\""), "got: {s}");
        let back: HostMessage = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);

        let v = HostMessage::Visibility { active: false };
        let s = serde_json::to_string(&v).unwrap();
        assert!(s.contains("\"kind\":\"visibility\""), "got: {s}");

        let q = HostMessage::Shutdown;
        let s = serde_json::to_string(&q).unwrap();
        assert!(s.contains("\"kind\":\"shutdown\""), "got: {s}");
    }
}
