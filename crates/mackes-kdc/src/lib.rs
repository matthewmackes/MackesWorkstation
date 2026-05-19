//! `mackes_kdc` — KDE Connect client crate (Phase 13 Option A,
//! locked 2026-05-19).
//!
//! Surfaces the typed value model the Mackes Workbench Connect
//! panels (13.3.1–.6) consume. Today ships the schema + a small
//! pure-function layer for parsing config; the live `zbus` wiring
//! against `org.kde.kdeconnect.*` lands alongside the panel
//! implementations.
//!
//! Why a separate crate (rather than a module inside mackes-panel)?
//!
//! - The mesh-mDNS bridge daemon (`mackesd-kdc-bridge`, Phase 13.2)
//!   needs the same `DBus` surface but runs independently of the panel.
//! - The Workbench panels are Python (today) — having the Rust types
//!   in their own crate makes the eventual `PyO3` bridge a clean
//!   import target.
//! - Lets us version-skew KDE Connect's protocol independently of
//!   the panel's release cadence.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Stable identifier for one paired device.
pub type DeviceId = String;

/// Device shape — surfaced verbatim in the Workbench Devices panel
/// (13.3.1) row list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    /// Stable identifier (KDE Connect UUID).
    pub id: DeviceId,
    /// Human display name (phone model or KDE setting).
    pub name: String,
    /// Device type — drives the row icon glyph.
    pub kind: DeviceKind,
    /// `true` when reachable now (on-LAN OR mesh-bridge re-announced).
    pub reachable: bool,
    /// Most recent battery percentage (0..=100) if reported.
    pub battery_pct: Option<u8>,
    /// Last-seen Unix epoch seconds.
    pub last_seen_s: i64,
}

/// Coarse device types KDE Connect distinguishes between. Each is
/// rendered with a Carbon glyph in the Devices panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    /// Android handset.
    Phone,
    /// Tablet (Android / iOS).
    Tablet,
    /// Linux desktop with KDE Connect installed.
    Desktop,
    /// Anything else (smartwatch, smart TV, etc).
    Unknown,
}

/// Notification mirrored from a remote device. Surfaced in the
/// Mackes Drawer's Notifications section with a phone-glyph badge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MirroredNotification {
    /// Origin device.
    pub device_id: DeviceId,
    /// Stable notification ID on the source device.
    pub notification_id: String,
    /// App that emitted the notification.
    pub app: String,
    /// Notification title.
    pub title: String,
    /// Notification body.
    pub text: String,
    /// Unix epoch seconds.
    pub at_s: i64,
}

/// Default file-transfer destination root. Per 13.1.1 lock:
/// configurable in `panel.toml:[kdeconnect.destinations]`, defaults
/// to `~/Downloads/<device>`.
#[must_use]
pub fn default_download_root() -> PathBuf {
    std::env::var_os("HOME")
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join("Downloads")
}

/// Scan `~/.config/kdeconnect/` and surface every paired device id
/// the upstream daemon knows about. Used by the first-launch
/// detection routine (13.1.3) to seed the Mackes-side
/// `kdeconnect.toml`.
#[must_use]
pub fn paired_device_ids() -> Vec<DeviceId> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let cfg_root = PathBuf::from(home).join(".config/kdeconnect");
    let Ok(entries) = std::fs::read_dir(&cfg_root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // KDE Connect's pairing dirs are UUID-shaped (32-char
            // hex). Filter strictly so we don't import the daemon's
            // own state directory (`config`, `cache`, etc.).
            if looks_like_kdc_uuid(name) {
                out.push(name.to_owned());
            }
        }
    }
    out
}

fn looks_like_kdc_uuid(s: &str) -> bool {
    s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '_')
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests panic on serde failure, no recovery needed")]
mod tests {
    use super::*;

    #[test]
    fn device_round_trips_through_json() {
        let d = Device {
            id: "abc123def456".into(),
            name: "Pixel 8".into(),
            kind: DeviceKind::Phone,
            reachable: true,
            battery_pct: Some(73),
            last_seen_s: 1_700_000_000,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn device_kind_serializes_snake_case() {
        let json = serde_json::to_string(&DeviceKind::Tablet).unwrap();
        assert_eq!(json, r#""tablet""#);
    }

    #[test]
    fn looks_like_uuid_accepts_kdc_shapes() {
        assert!(looks_like_kdc_uuid("a1b2c3d4e5f6a1b2"));
        assert!(looks_like_kdc_uuid("a1b2c3d4_e5f6a1b2"));
        assert!(!looks_like_kdc_uuid(""));
        assert!(!looks_like_kdc_uuid("short"));
        assert!(!looks_like_kdc_uuid("config")); // KDC's own state dir
    }

    #[test]
    fn default_download_root_under_home() {
        let p = default_download_root();
        assert!(p.ends_with("Downloads"));
    }

    #[test]
    fn paired_device_ids_returns_empty_on_missing_dir() {
        std::env::set_var("HOME", "/nonexistent/zzz/yyy");
        let v = paired_device_ids();
        assert!(v.is_empty());
        std::env::remove_var("HOME");
    }

    // ----------------------------------------------------------------
    // Phase 13.6 — extended test coverage. Each kind round-trips JSON,
    // boundary battery values stay clean, and the mDNS-bridge uuid
    // filter rejects every KDE Connect state-directory name we'd
    // accidentally import otherwise.
    // ----------------------------------------------------------------

    #[test]
    fn device_kind_phone_round_trips_as_snake_case() {
        let json = serde_json::to_string(&DeviceKind::Phone).unwrap();
        assert_eq!(json, r#""phone""#);
        let back: DeviceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DeviceKind::Phone);
    }

    #[test]
    fn device_kind_tablet_round_trips_as_snake_case() {
        let json = serde_json::to_string(&DeviceKind::Tablet).unwrap();
        assert_eq!(json, r#""tablet""#);
        let back: DeviceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DeviceKind::Tablet);
    }

    #[test]
    fn device_kind_desktop_round_trips_as_snake_case() {
        let json = serde_json::to_string(&DeviceKind::Desktop).unwrap();
        assert_eq!(json, r#""desktop""#);
        let back: DeviceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DeviceKind::Desktop);
    }

    #[test]
    fn device_kind_unknown_round_trips_as_snake_case() {
        let json = serde_json::to_string(&DeviceKind::Unknown).unwrap();
        assert_eq!(json, r#""unknown""#);
        let back: DeviceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DeviceKind::Unknown);
    }

    #[test]
    fn mirrored_notification_round_trips_through_json() {
        let n = MirroredNotification {
            device_id: "a1b2c3d4e5f6a1b2".into(),
            notification_id: "msg-42".into(),
            app: "org.thunderbird.Thunderbird".into(),
            title: "Inbox — 1 new".into(),
            text: "Pull request review requested by alice".into(),
            at_s: 1_700_000_000,
        };
        let json = serde_json::to_string(&n).unwrap();
        // Field names land verbatim (no rename_all on the struct).
        assert!(json.contains(r#""device_id":"a1b2c3d4e5f6a1b2""#));
        assert!(json.contains(r#""notification_id":"msg-42""#));
        let back: MirroredNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, n);
    }

    #[test]
    fn looks_like_kdc_uuid_rejects_every_kdc_state_dir() {
        // KDE Connect's own state directories live alongside the
        // per-device pairing folders under ~/.config/kdeconnect/.
        // The bridge must skip every one of them.
        for state_dir in &["config", "cache", "log", "trusted_devices", ""] {
            assert!(
                !looks_like_kdc_uuid(state_dir),
                "uuid filter must reject KDE state dir {state_dir:?}",
            );
        }
    }

    #[test]
    fn looks_like_kdc_uuid_accepts_realistic_pairing_dir_names() {
        // Real-world KDE Connect pairing-dir basenames are 16 to 32
        // hex chars, sometimes with `_` separators. Sample a few from
        // the upstream daemon's output.
        for uuid in &[
            "a1b2c3d4e5f6a1b2",                  // 16 hex chars
            "a1b2c3d4_e5f6a1b2",                 // hex + separator
            "0123456789abcdef0123456789abcdef",  // 32-char UUID
            "deadbeefcafebabe1234",              // 20 hex chars
        ] {
            assert!(
                looks_like_kdc_uuid(uuid),
                "uuid filter must accept pairing dir {uuid:?}",
            );
        }
    }

    #[test]
    fn battery_pct_boundary_values_serialize_cleanly() {
        // 0 — phone at the bottom of the gauge.
        let mut d = Device {
            id: "a1b2c3d4e5f6a1b2".into(),
            name: "Pixel".into(),
            kind: DeviceKind::Phone,
            reachable: true,
            battery_pct: Some(0),
            last_seen_s: 1,
        };
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains(r#""battery_pct":0"#));
        let back: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(back.battery_pct, Some(0));

        // 100 — phone fully charged.
        d.battery_pct = Some(100);
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains(r#""battery_pct":100"#));
        let back: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(back.battery_pct, Some(100));

        // None — phone never reported battery (e.g. tablet without
        // battery sensor, or first-tick before mirroring catches up).
        d.battery_pct = None;
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains(r#""battery_pct":null"#));
        let back: Device = serde_json::from_str(&json).unwrap();
        assert_eq!(back.battery_pct, None);
    }
}
