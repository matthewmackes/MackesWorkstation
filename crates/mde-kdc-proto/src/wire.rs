//! KDC2-2 wire types — top-level packet envelope + the MDE
//! capability-negotiation header.
//!
//! The KDE Connect wire format is newline-delimited JSON. Every
//! packet is a single JSON object with three top-level fields:
//! `id` (millisecond timestamp), `type` (packet-type string,
//! e.g. `kdeconnect.identity`), and `body` (per-type payload).
//!
//! KDC2 adds one OPTIONAL top-level field: `mdeCaps`, a
//! [`CapabilitiesHeader`]. Stock KDE Connect clients ignore
//! unknown top-level fields, so adding this is wire-compatible
//! with every upstream client. Two MDE peers seeing each other's
//! `mdeCaps` light up MDE-only behaviors; phones (no `mdeCaps`)
//! get the stock-compatible subset.

use serde::{Deserialize, Serialize};

/// The MDE capability negotiation header. Optional; absent for
/// stock-client interop.
///
/// Reserved capability tokens (lock per v2.1 KDC2 survey):
///
///   * `mesh_relay` — peer agrees to relay messages through the
///     mesh for phones paired only with another MDE peer.
///   * `peer_card_probe_share` — peer agrees to share its
///     `PeerProbe` enrichment cache so [[project_v2_1_kdc2_native]]
///     two-MDE pairs avoid duplicate online enrichment lookups.
///   * `notification_dual_send_ack` — peer acks idempotent
///     dual-send so the sender can drop redundant copies once
///     the first acknowledgment lands.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapabilitiesHeader {
    /// MDE protocol-extension version. Independent of the KDC
    /// `PROTOCOL_VERSION` — this is just for negotiating MDE-
    /// internal feature bumps.
    #[serde(default)]
    pub mde_version: u32,
    /// Capability tokens this peer offers. Empty Vec is valid
    /// (acts like "no MDE-only behaviors enabled"). Receivers
    /// MUST treat unknown tokens as gracefully-degraded — never
    /// fail a handshake because of an unrecognized capability.
    #[serde(default)]
    pub offers: Vec<String>,
}

impl CapabilitiesHeader {
    /// Build a header offering exactly the three v2.1-locked
    /// capabilities. Used by host integration (KDC2-3) at
    /// handshake time.
    #[must_use]
    pub fn v2_1_lock() -> Self {
        Self {
            mde_version: 1,
            offers: vec![
                "mesh_relay".to_string(),
                "peer_card_probe_share".to_string(),
                "notification_dual_send_ack".to_string(),
            ],
        }
    }

    /// True when this header advertises support for the given
    /// capability token.
    #[must_use]
    pub fn offers(&self, capability: &str) -> bool {
        self.offers.iter().any(|c| c == capability)
    }
}

/// Top-level wire packet. Serializes to the KDE Connect newline-
/// delimited JSON shape: `{"id": …, "type": "…", "body": {…},
/// "mdeCaps": {…}}`.
///
/// `id` is a millisecond Unix timestamp — KDC uses it for de-
/// duplication on the receiver side (two packets with the same
/// `id` are treated as the same logical message). KDC2 keeps that
/// semantic; the mesh-shunt's dual-send relies on it.
///
/// `body` is left as `serde_json::Value` rather than a typed
/// enum-over-plugins because plugin types live in
/// [`crate::plugins`] and would create a circular dep if `wire`
/// owned every variant. KDC2-2.5 wires per-plugin downcast
/// helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    /// Millisecond Unix timestamp. Used as a deduplication key
    /// by the receiver — two `Packet`s with the same `id` from
    /// the same peer are the same logical message even if they
    /// arrive over different transports (dual-send semantics).
    pub id: i64,
    /// Packet-type identifier (e.g. `kdeconnect.identity`,
    /// `kdeconnect.notification`, `kdeconnect.clipboard`).
    /// Match upstream's tokens verbatim for wire compatibility.
    #[serde(rename = "type")]
    pub kind: String,
    /// Per-type payload. Plugins downcast this via the helpers
    /// in [`crate::plugins`].
    pub body: serde_json::Value,
    /// MDE-only capability header. `None` for handshakes with
    /// stock KDC clients; `Some` when both peers are MDE.
    #[serde(rename = "mdeCaps", default, skip_serializing_if = "Option::is_none")]
    pub mde_caps: Option<CapabilitiesHeader>,
}

impl Packet {
    /// True when this packet was emitted by an MDE peer (the
    /// `mdeCaps` header is present). False when from a stock
    /// KDC client.
    #[must_use]
    pub fn from_mde_peer(&self) -> bool {
        self.mde_caps.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_header_v2_1_lock_offers_all_three() {
        let h = CapabilitiesHeader::v2_1_lock();
        assert!(h.offers("mesh_relay"));
        assert!(h.offers("peer_card_probe_share"));
        assert!(h.offers("notification_dual_send_ack"));
        assert!(!h.offers("post_quantum")); // explicitly omitted per v2.1 lock
        assert_eq!(h.mde_version, 1);
    }

    #[test]
    fn capabilities_header_unknown_capability_is_false() {
        let h = CapabilitiesHeader::v2_1_lock();
        assert!(!h.offers("undefined_future_thing"));
    }

    #[test]
    fn packet_serializes_with_kdc_field_names() {
        // Wire compatibility lock: stock Android KDE Connect
        // clients deserialize against the upstream field names.
        // `type` is a reserved Rust keyword so we use #[serde(rename)]
        // — this test guards that the rename is in place.
        let p = Packet {
            id: 1_700_000_000_123,
            kind: "kdeconnect.identity".to_string(),
            body: serde_json::json!({"deviceName": "lab-01"}),
            mde_caps: None,
        };
        let s = serde_json::to_string(&p).unwrap();
        assert!(s.contains(r#""type":"kdeconnect.identity""#));
        assert!(s.contains(r#""id":1700000000123"#));
        assert!(s.contains(r#""body":{"deviceName":"lab-01"}"#));
        // mdeCaps absent — must NOT serialize (stock-client interop).
        assert!(!s.contains("mdeCaps"));
    }

    #[test]
    fn packet_with_mde_caps_serializes_under_kdc_camel_case() {
        let p = Packet {
            id: 1,
            kind: "kdeconnect.identity".to_string(),
            body: serde_json::Value::Null,
            mde_caps: Some(CapabilitiesHeader::v2_1_lock()),
        };
        let s = serde_json::to_string(&p).unwrap();
        // Field name lands as `mdeCaps` (camelCase) per KDE Connect's
        // upstream field-naming convention.
        assert!(s.contains(r#""mdeCaps":"#));
    }

    #[test]
    fn packet_round_trips_through_json() {
        let p = Packet {
            id: 42,
            kind: "kdeconnect.clipboard".to_string(),
            body: serde_json::json!({"content": "hello"}),
            mde_caps: Some(CapabilitiesHeader::v2_1_lock()),
        };
        let s = serde_json::to_string(&p).unwrap();
        let back: Packet = serde_json::from_str(&s).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn from_mde_peer_distinguishes_mde_vs_stock() {
        let stock = Packet {
            id: 1,
            kind: "kdeconnect.identity".to_string(),
            body: serde_json::Value::Null,
            mde_caps: None,
        };
        let mde = Packet {
            id: 1,
            kind: "kdeconnect.identity".to_string(),
            body: serde_json::Value::Null,
            mde_caps: Some(CapabilitiesHeader::default()),
        };
        assert!(!stock.from_mde_peer());
        assert!(mde.from_mde_peer());
    }

    #[test]
    fn deserializing_packet_with_unknown_extra_field_does_not_fail() {
        // Forward compatibility: when a future MDE release adds a
        // new top-level field, older MDE peers must keep parsing
        // packets cleanly (treating the unknown field as ignored).
        // serde's default `deny_unknown_fields = false` makes this
        // work; this test guards that nobody adds the attribute.
        let raw = r#"{"id":1,"type":"kdeconnect.identity","body":{},"futureField":42}"#;
        let p: Packet = serde_json::from_str(raw).unwrap();
        assert_eq!(p.kind, "kdeconnect.identity");
    }
}
