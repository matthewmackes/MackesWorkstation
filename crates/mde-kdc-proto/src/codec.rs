//! KDC2-2 codec — newline-delimited JSON frame encoding.
//!
//! Skeleton landed here in KDC2-2.1. Full encode/decode plus a
//! libFuzzer corpus arrive in KDC2-2.2. The functions below ship as
//! the smallest correct implementation (single-packet encode + a
//! line-split decoder) so the wire test in KDC2-2.3's loopback
//! harness has something to round-trip against.

use crate::wire::Packet;

/// Encode a single packet to its KDC wire form (one line of JSON,
/// terminated by `\n`).
///
/// Errors propagate from `serde_json` — a `Packet` that holds a
/// `serde_json::Value` body which can't be serialized (numeric
/// `NaN` / `Infinity`) returns the underlying error so the caller
/// can decide whether to log + drop or panic.
pub fn encode_frame(packet: &Packet) -> Result<String, serde_json::Error> {
    let mut out = serde_json::to_string(packet)?;
    out.push('\n');
    Ok(out)
}

/// Decode a single frame from a newline-terminated byte slice.
///
/// The KDC protocol's framing is "one JSON object per line." This
/// helper accepts either a single frame (with or without trailing
/// `\n`) or a leading frame from a stream (anything after the
/// first newline is ignored — the caller is responsible for
/// re-feeding the remainder). KDC2-2.2 ships a stream-aware
/// `FrameDecoder` that holds partial buffers.
pub fn decode_frame(raw: &[u8]) -> Result<Packet, serde_json::Error> {
    // Stop at the first newline; KDC wire is line-delimited.
    let line: &[u8] = raw.split(|&b| b == b'\n').next().unwrap_or(raw);
    serde_json::from_slice(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::CapabilitiesHeader;

    #[test]
    fn encode_frame_is_newline_terminated() {
        let p = Packet {
            id: 1,
            kind: "kdeconnect.identity".to_string(),
            body: serde_json::Value::Null,
            mde_caps: None,
        };
        let s = encode_frame(&p).unwrap();
        assert!(s.ends_with('\n'));
        // Exactly one newline — the frame contains no internal
        // line break.
        assert_eq!(s.matches('\n').count(), 1);
    }

    #[test]
    fn encode_then_decode_round_trips() {
        let p = Packet {
            id: 42,
            kind: "kdeconnect.clipboard".to_string(),
            body: serde_json::json!({"content": "hello"}),
            mde_caps: Some(CapabilitiesHeader::v2_1_lock()),
        };
        let encoded = encode_frame(&p).unwrap();
        let decoded = decode_frame(encoded.as_bytes()).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn decode_frame_ignores_trailing_stream_data() {
        // Caller fed us two concatenated frames — we return the
        // first and let them handle the remainder.
        let two_frames = b"{\"id\":1,\"type\":\"kdeconnect.identity\",\"body\":{}}\n{\"id\":2,\"type\":\"kdeconnect.clipboard\",\"body\":{}}\n";
        let p = decode_frame(two_frames).unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.kind, "kdeconnect.identity");
    }

    #[test]
    fn decode_frame_rejects_garbage() {
        let raw = b"not valid JSON\n";
        assert!(decode_frame(raw).is_err());
    }
}
