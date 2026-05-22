//! KDC2-2 discovery — UDP-broadcast announcements + mesh-shunt
//! synthetic-mDNS injection point.
//!
//! Stock KDE Connect uses UDP/1716 broadcasts on the local LAN
//! to announce a peer's identity. KDC2 keeps that exact behavior
//! for wire compatibility — phones discover MDE peers through
//! the upstream protocol — but layers a [`SyntheticAnnounce`]
//! injection point on top so peer A can tell peer B "phone X
//! exists, here's its identity envelope" through the MDE mesh
//! router, making X reachable from B without re-pairing.
//!
//! Networking + actual broadcast send/receive live in
//! `mde-kdc::discovery` (host integration, KDC2-3). This file
//! ships the **announce data model** + the synthetic-injection
//! seam.

use serde::{Deserialize, Serialize};

/// Identity announcement broadcast on UDP/1716 (or injected
/// through the mesh-shunt). Stays wire-compatible with the
/// upstream KDC identity packet's `body` shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announce {
    /// Stable per-device identifier (KDE Connect UUID).
    pub device_id: String,
    /// Display name. MDE peers append `[mde]` (see
    /// [`crate::MDE_DEVICE_NAME_SUFFIX`]).
    pub device_name: String,
    /// Coarse device type — drives the row icon glyph in the
    /// receiver's UI.
    pub device_type: DeviceType,
    /// Protocol version this peer speaks. Stock KDC currently
    /// emits `7`; KDC2 matches.
    pub protocol_version: u32,
    /// Plugin types this peer accepts (`kdeconnect.clipboard`,
    /// `kdeconnect.notification`, etc.). Upstream calls this
    /// `incomingCapabilities`.
    pub incoming_capabilities: Vec<String>,
    /// Plugin types this peer emits. Upstream calls this
    /// `outgoingCapabilities`.
    pub outgoing_capabilities: Vec<String>,
}

/// KDC's coarse device-type enumeration. Stays in lock-step with
/// the legacy v13.0 `mackes-kdc::DeviceKind` for serde token
/// compatibility (`phone`, `tablet`, `desktop`, `unknown`) — the
/// v2.1 KDC2 lock keeps these tokens stable so paired phones
/// don't re-classify on the v2.0 → v2.1 upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    /// Android handset.
    Phone,
    /// Tablet (Android / iOS).
    Tablet,
    /// Linux desktop (MDE peer OR a stock-KDC desktop client).
    Desktop,
    /// Anything else.
    Unknown,
}

/// Mesh-shunt: peer A injects "I see phone X" so peer B finds X
/// without a direct broadcast from X. The injection point is the
/// seam where KDC2-4 wires the mesh router into the discovery
/// layer.
///
/// KDC2-2.1 ships the data model + signature placeholder; the
/// actual SyntheticAnnounce verification + drop-if-stale logic
/// lands with the KDC2-4 mesh-shunt work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticAnnounce {
    /// The relayed identity announcement (verbatim from the
    /// originating peer's broadcast).
    pub announce: Announce,
    /// Identity of the MDE peer that's relaying. Receivers use
    /// this to filter (e.g. discard relays from a peer we don't
    /// trust).
    pub relayed_by: String,
    /// Monotonic timestamp of the relay (ms since Unix epoch).
    /// Used to drop stale announces — a peer that hasn't been
    /// re-announced in N minutes is treated as gone.
    pub relayed_at_ms: i64,
}

impl SyntheticAnnounce {
    /// True when this synthetic announce is recent enough to act
    /// on. KDC2-4 sets the staleness window from a config knob;
    /// this default (90 s) matches upstream KDC's own broadcast
    /// cadence.
    #[must_use]
    pub fn is_fresh(&self, now_ms: i64) -> bool {
        const STALE_WINDOW_MS: i64 = 90_000;
        now_ms.saturating_sub(self.relayed_at_ms) <= STALE_WINDOW_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announce_serializes_with_kdc_field_names() {
        // `deviceId`, `deviceName`, `incomingCapabilities`, etc. —
        // the upstream KDC identity packet uses camelCase. Our
        // serde rename_all is the wire lock.
        let a = Announce {
            device_id: "abc".to_string(),
            device_name: "lab-01 [mde]".to_string(),
            device_type: DeviceType::Desktop,
            protocol_version: 7,
            incoming_capabilities: vec!["kdeconnect.clipboard".into()],
            outgoing_capabilities: vec!["kdeconnect.notification".into()],
        };
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains(r#""deviceId":"abc""#));
        assert!(s.contains(r#""deviceName":"lab-01 [mde]""#));
        assert!(s.contains(r#""incomingCapabilities""#));
        assert!(s.contains(r#""outgoingCapabilities""#));
    }

    #[test]
    fn device_type_serializes_snake_case() {
        // Matches legacy `mackes-kdc::DeviceKind` for token
        // stability across the v2.0 → v2.1 upgrade.
        assert_eq!(serde_json::to_string(&DeviceType::Phone).unwrap(), r#""phone""#);
        assert_eq!(serde_json::to_string(&DeviceType::Tablet).unwrap(), r#""tablet""#);
        assert_eq!(
            serde_json::to_string(&DeviceType::Desktop).unwrap(),
            r#""desktop""#,
        );
        assert_eq!(
            serde_json::to_string(&DeviceType::Unknown).unwrap(),
            r#""unknown""#,
        );
    }

    #[test]
    fn synthetic_announce_is_fresh_within_90s_window() {
        let s = SyntheticAnnounce {
            announce: Announce {
                device_id: "abc".to_string(),
                device_name: "phone".to_string(),
                device_type: DeviceType::Phone,
                protocol_version: 7,
                incoming_capabilities: vec![],
                outgoing_capabilities: vec![],
            },
            relayed_by: "peer-A".to_string(),
            relayed_at_ms: 1_000_000,
        };
        // 50s after relay — fresh.
        assert!(s.is_fresh(1_050_000));
        // 90s after relay — still fresh (boundary inclusive).
        assert!(s.is_fresh(1_090_000));
        // 91s after relay — stale.
        assert!(!s.is_fresh(1_091_000));
        // 200s after relay — definitely stale.
        assert!(!s.is_fresh(1_200_000));
    }

    #[test]
    fn synthetic_announce_round_trips_through_json() {
        let s = SyntheticAnnounce {
            announce: Announce {
                device_id: "abc".to_string(),
                device_name: "phone".to_string(),
                device_type: DeviceType::Phone,
                protocol_version: 7,
                incoming_capabilities: vec!["kdeconnect.clipboard".into()],
                outgoing_capabilities: vec!["kdeconnect.notification".into()],
            },
            relayed_by: "peer-A".to_string(),
            relayed_at_ms: 1_700_000_000_000,
        };
        let raw = serde_json::to_string(&s).unwrap();
        let back: SyntheticAnnounce = serde_json::from_str(&raw).unwrap();
        assert_eq!(back, s);
    }
}
