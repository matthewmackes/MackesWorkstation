//! NF-7.3 (v2.5) — Preview page.
//!
//! Shown after Apply finishes. Surfaces the post-enrollment
//! Nebula state so the operator can confirm the wizard
//! actually moved the peer into a working mesh before exiting:
//!
//!   * This peer's overlay IP (10.42.x.y form).
//!   * The lighthouse roster (one row per peer the
//!     `Nebula.Status.ListPeers` reply returns).
//!   * Active transport (nebula_direct / lighthouse_relay /
//!     https443) so the operator notices firewall-mode early.
//!   * Diagnostics banner if no peers show up within 30 s of
//!     landing on the page — per the Q11 lock the wizard tells
//!     the operator *why* the mesh might look empty (lighthouse
//!     unreachable / pre-enrollment / firewall blocking 4242)
//!     rather than silently rendering an empty list.
//!
//! Data layer (E0.3.1, EPIC-RETIRE-DBUS): the probe now rides the
//! mesh **Bus** action/reply pattern — `mde_bus::rpc::request`
//! against `action/nebula/self-node` + `action/nebula/list-peers`
//! — instead of the retired `dev.mackes.MDE.Nebula.Status` D-Bus
//! methods. mackesd serves these on its Bus responder thread
//! (`crates/mesh/mackesd/src/ipc/nebula.rs::serve_bus`). The reply
//! body is bare JSON (`SelfNodeSnapshot` / `Vec<PeerRow>`), so the
//! parsers deserialize it directly — no dbus-send `string "…"`
//! envelope to strip.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Diagnostics banner kicks in after this many seconds with
/// zero peers in the roster. Q11 lock — 30 s gives even slow
/// lighthouses (cellular, satellite) time to respond before
/// the operator gets a "something's wrong" hint.
pub const EMPTY_ROSTER_THRESHOLD_S: u64 = 30;

/// One row from the `Nebula.Status.SelfNode` reply.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfNodeView {
    /// Stable node-id (e.g. `peer:anvil`).
    pub node_id: String,
    /// Display hostname.
    pub host: String,
    /// `host` (lighthouse) or `peer`.
    pub role: String,
    /// Allocated overlay IP. Empty before enrollment completes.
    pub overlay_ip: String,
    /// Active CA epoch the local peer's cert was signed under.
    pub cert_epoch: i64,
    /// Mesh-id this peer belongs to. Empty when no CA exists.
    pub mesh_id: String,
}

/// One row from the `Nebula.Status.ListPeers` reply.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRowView {
    pub node_id: String,
    pub name: String,
    pub overlay_ip: String,
    pub reachable: String,
    /// "host" when the peer is a lighthouse; "" otherwise.
    /// Derived from the absence of an explicit "role" field
    /// in the existing PeerRow reply by cross-referencing
    /// node_id against the lighthouse list (when the daemon
    /// surfaces it); falls back to empty for v2.5 baseline.
    ///
    /// `#[serde(default)]` so the daemon's `PeerRow` JSON (which
    /// carries no `role_hint` field) deserializes cleanly — the
    /// field is wizard-local, not part of the Bus wire shape.
    #[serde(default)]
    pub role_hint: String,
}

/// Snapshot of the data the page renders. Held by the
/// wizard's update loop; refreshed via `probe()` whenever
/// the operator clicks Refresh or the page becomes active.
#[derive(Debug, Clone, Default)]
pub struct PreviewSnapshot {
    pub self_node: Option<SelfNodeView>,
    pub peers: Vec<PeerRowView>,
    /// Error message from the most recent probe (empty when
    /// the probe succeeded).
    pub error: String,
}

/// Decide whether to surface the diagnostics banner. Returns
/// `true` when more than `EMPTY_ROSTER_THRESHOLD_S` seconds
/// have elapsed since `landed_at` AND the peer list is empty
/// AND the self_node either has an overlay_ip (enrolled, mesh
/// just empty) or doesn't (pre-enrollment + lighthouse stuck).
/// Pulled out for direct testing without timing.
#[must_use]
pub fn should_show_diagnostics(snap: &PreviewSnapshot, elapsed_secs: u64) -> bool {
    elapsed_secs >= EMPTY_ROSTER_THRESHOLD_S && snap.peers.is_empty()
}

/// Human-readable diagnostic text the banner renders. Tuned to
/// give the operator a concrete next action.
#[must_use]
pub fn diagnostic_message(snap: &PreviewSnapshot) -> String {
    match snap.self_node.as_ref() {
        Some(s) if !s.overlay_ip.is_empty() => format!(
            "Enrolled at overlay IP {} but no peers visible yet. \
             Likely cause: the lighthouse rejected your join token \
             after signing, OR the lighthouse hasn't seen another \
             peer come online. Check `mackesd nebula peer-list` on \
             the lighthouse; run `mackesd nebula status` here to \
             confirm the active transport.",
            s.overlay_ip,
        ),
        Some(_) => {
            "Mesh probe returned a self-node row with no overlay IP. \
             Enrollment didn't complete — re-run `mackesd enroll \
             --token <…>` and check journalctl -u mackesd.service \
             for the signing error."
                .to_string()
        }
        None => {
            "No reply on `action/nebula/self-node` after 30s. \
             Is mackesd.service running? `systemctl status \
             mackesd.service` — if it's down, start it then click \
             Refresh."
                .to_string()
        }
    }
}

/// Per-verb timeout for the Bus round-trip. The wizard's probe is
/// interactive (operator is watching the Preview page), but the
/// 30 s empty-roster diagnostics threshold already covers the
/// "lighthouse is slow" case, so a short 2 s timeout per verb
/// keeps a hung/absent daemon from freezing the refresh — the
/// page falls back to its diagnostics banner.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Pure parser — deserialize the `action/nebula/self-node` reply
/// body (bare JSON `SelfNodeSnapshot`) into [`SelfNodeView`].
/// Returns `None` on any parse failure (or an `{"error": …}`
/// envelope); the page treats that as "probe failed" and shows
/// the banner.
#[must_use]
pub fn parse_self_node(raw: &str) -> Option<SelfNodeView> {
    if raw.trim().is_empty() {
        return None;
    }
    serde_json::from_str::<SelfNodeView>(raw).ok()
}

/// Pure parser — deserialize the `action/nebula/list-peers` reply
/// body (bare JSON `Vec<PeerRow>`) into `Vec<PeerRowView>`.
/// Returns `Vec::new()` on parse failure (treated as "zero peers"
/// so the page renders the diagnostics banner after the 30 s
/// threshold).
#[must_use]
pub fn parse_peer_list(raw: &str) -> Vec<PeerRowView> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<PeerRowView>>(raw).unwrap_or_default()
}

/// Probe the Nebula status surface over the mesh Bus
/// (`action/nebula/self-node` + `action/nebula/list-peers`) and
/// return a snapshot the page renders. Empty
/// (`Default::default()`) when the Bus data-dir is unreachable;
/// per-verb failures (timeout / no responder) leave that field at
/// its default so the page renders the diagnostics banner once the
/// timer fires.
///
/// Synchronous so the iced update loop can call it directly. The
/// `mde_bus::rpc::request` round-trip is async, so this builds a
/// short-lived current-thread runtime + `block_on`s the two
/// requests — the same sync-over-async shape `mesh_backend.rs`
/// used for its blocking D-Bus calls.
#[must_use]
pub fn probe() -> PreviewSnapshot {
    let Some(data_dir) = mde_bus::default_data_dir() else {
        return PreviewSnapshot {
            error: "no XDG data dir — cannot reach the mesh Bus".into(),
            ..PreviewSnapshot::default()
        };
    };
    let persist = match mde_bus::persist::Persist::open(data_dir) {
        Ok(p) => p,
        Err(e) => {
            return PreviewSnapshot {
                error: format!("mesh Bus unavailable: {e}"),
                ..PreviewSnapshot::default()
            };
        }
    };
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            return PreviewSnapshot {
                error: format!("probe runtime build failed: {e}"),
                ..PreviewSnapshot::default()
            };
        }
    };
    let (self_raw, peers_raw) = rt.block_on(async {
        let self_raw = request_verb(&persist, "self-node").await;
        let peers_raw = request_verb(&persist, "list-peers").await;
        (self_raw, peers_raw)
    });
    PreviewSnapshot {
        self_node: self_raw.as_deref().and_then(parse_self_node),
        peers: peers_raw.as_deref().map(parse_peer_list).unwrap_or_default(),
        error: String::new(),
    }
}

/// Publish one `action/nebula/<verb>` request + await the reply
/// body. `None` on timeout / no-responder / persist error — the
/// caller maps that to the page's empty/diagnostics rendering.
async fn request_verb(persist: &mde_bus::persist::Persist, verb: &str) -> Option<String> {
    let topic = format!("action/nebula/{verb}");
    match mde_bus::rpc::request(
        persist,
        &topic,
        mde_bus::hooks::config::Priority::Default,
        None,
        None,
        PROBE_TIMEOUT,
    )
    .await
    {
        Ok(reply) => reply.body,
        Err(e) => {
            tracing::debug!(topic = %topic, error = %e, "nebula probe: no reply");
            None
        }
    }
}

/// Compact human-readable summary line used in the wizard's
/// preview body. Format:
///   "Mesh: <mesh-id> · overlay: <ip> · <N> peers"
/// or appropriate fallbacks when fields are absent.
#[must_use]
pub fn summary_line(snap: &PreviewSnapshot) -> String {
    let mesh_id = snap
        .self_node
        .as_ref()
        .map(|s| {
            if s.mesh_id.is_empty() {
                "(no mesh)".to_string()
            } else {
                s.mesh_id.clone()
            }
        })
        .unwrap_or_else(|| "(no probe data)".to_string());
    let overlay = snap
        .self_node
        .as_ref()
        .map(|s| {
            if s.overlay_ip.is_empty() {
                "—".to_string()
            } else {
                s.overlay_ip.clone()
            }
        })
        .unwrap_or_else(|| "—".to_string());
    let peer_count = snap.peers.len();
    format!("Mesh: {mesh_id} · overlay: {overlay} · {peer_count} peers")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- parser coverage --------------------------------------

    #[test]
    fn parse_self_node_decodes_bare_bus_reply() {
        // E0.3.1 — the Bus reply body is bare JSON (no dbus-send
        // `string "…"` envelope). Mirrors the exact
        // SelfNodeSnapshot shape the responder serializes,
        // including the cert_expires_at field SelfNodeView ignores.
        let raw = r#"{"node_id":"peer:anvil","host":"anvil","role":"host","cert_epoch":2,"cert_expires_at":9999,"overlay_ip":"10.42.0.1","mesh_id":"mesh-anvil"}"#;
        let s = parse_self_node(raw).expect("decoded");
        assert_eq!(s.node_id, "peer:anvil");
        assert_eq!(s.role, "host");
        assert_eq!(s.overlay_ip, "10.42.0.1");
        assert_eq!(s.cert_epoch, 2);
        assert_eq!(s.mesh_id, "mesh-anvil");
    }

    #[test]
    fn parse_self_node_decodes_minimal_json() {
        let raw = r#"{"node_id":"peer:b","host":"b","role":"peer","cert_epoch":1,"overlay_ip":"10.42.0.5","mesh_id":"m"}"#;
        let s = parse_self_node(raw).expect("decoded");
        assert_eq!(s.role, "peer");
        assert_eq!(s.overlay_ip, "10.42.0.5");
    }

    #[test]
    fn parse_self_node_returns_none_for_empty() {
        assert!(parse_self_node("").is_none());
        assert!(parse_self_node("   ").is_none());
    }

    #[test]
    fn parse_self_node_returns_none_for_garbage() {
        assert!(parse_self_node("not json").is_none());
        // An {"error": …} envelope from the responder (unknown
        // verb / builder failure) doesn't decode to SelfNodeView.
        assert!(parse_self_node(r#"{"error":"unknown nebula verb"}"#).is_none());
    }

    #[test]
    fn parse_peer_list_decodes_bare_bus_reply() {
        // Mirrors the responder's Vec<PeerRow> wire shape: the
        // daemon emits no role_hint field (it's wizard-local with
        // #[serde(default)]) but does emit fingerprint/cert fields
        // PeerRowView ignores.
        let raw = r#"[{"node_id":"peer:b","name":"b","overlay_ip":"10.42.0.2","fingerprint":"abc","cert_epoch":3,"cert_expires_at":0,"reachable":"online"}]"#;
        let peers = parse_peer_list(raw);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, "peer:b");
        assert_eq!(peers[0].reachable, "online");
        assert_eq!(peers[0].role_hint, "", "role_hint defaults when absent");
    }

    #[test]
    fn parse_peer_list_returns_empty_for_garbage() {
        assert!(parse_peer_list("").is_empty());
        assert!(parse_peer_list("not json").is_empty());
    }

    #[test]
    fn parse_peer_list_decodes_empty_array() {
        assert!(parse_peer_list("[]").is_empty());
    }

    // ---- diagnostic banner gating -----------------------------

    #[test]
    fn diagnostics_dont_fire_before_threshold() {
        let snap = PreviewSnapshot::default();
        assert!(!should_show_diagnostics(&snap, 0));
        assert!(!should_show_diagnostics(&snap, EMPTY_ROSTER_THRESHOLD_S - 1));
    }

    #[test]
    fn diagnostics_fire_at_threshold_with_empty_roster() {
        let snap = PreviewSnapshot::default();
        assert!(should_show_diagnostics(&snap, EMPTY_ROSTER_THRESHOLD_S));
    }

    #[test]
    fn diagnostics_suppress_when_peers_present() {
        let snap = PreviewSnapshot {
            peers: vec![PeerRowView {
                node_id: "peer:b".into(),
                name: "b".into(),
                overlay_ip: "10.42.0.2".into(),
                reachable: "online".into(),
                role_hint: String::new(),
            }],
            ..PreviewSnapshot::default()
        };
        assert!(!should_show_diagnostics(&snap, EMPTY_ROSTER_THRESHOLD_S * 10));
    }

    // ---- diagnostic-message branches --------------------------

    #[test]
    fn diagnostic_message_when_enrolled_with_overlay_ip() {
        let snap = PreviewSnapshot {
            self_node: Some(SelfNodeView {
                node_id: "peer:a".into(),
                host: "a".into(),
                role: "peer".into(),
                overlay_ip: "10.42.0.5".into(),
                cert_epoch: 1,
                mesh_id: "m".into(),
            }),
            ..PreviewSnapshot::default()
        };
        let msg = diagnostic_message(&snap);
        assert!(msg.contains("10.42.0.5"));
        assert!(msg.contains("peer-list"));
    }

    #[test]
    fn diagnostic_message_when_enrolled_without_overlay_ip() {
        let snap = PreviewSnapshot {
            self_node: Some(SelfNodeView::default()),
            ..PreviewSnapshot::default()
        };
        let msg = diagnostic_message(&snap);
        assert!(msg.contains("Enrollment didn't complete"));
        assert!(msg.contains("mackesd enroll"));
    }

    #[test]
    fn diagnostic_message_when_no_probe_reply() {
        let snap = PreviewSnapshot::default();
        let msg = diagnostic_message(&snap);
        assert!(msg.contains("mackesd.service"));
        assert!(msg.contains("Refresh"));
    }

    // ---- summary_line --------------------------------------

    #[test]
    fn summary_line_with_full_snapshot() {
        let snap = PreviewSnapshot {
            self_node: Some(SelfNodeView {
                node_id: "peer:a".into(),
                host: "a".into(),
                role: "host".into(),
                overlay_ip: "10.42.0.1".into(),
                cert_epoch: 0,
                mesh_id: "mesh-anvil".into(),
            }),
            peers: vec![PeerRowView::default(), PeerRowView::default()],
            error: String::new(),
        };
        let s = summary_line(&snap);
        assert!(s.contains("Mesh: mesh-anvil"));
        assert!(s.contains("overlay: 10.42.0.1"));
        assert!(s.contains("2 peers"));
    }

    #[test]
    fn summary_line_pre_probe() {
        let snap = PreviewSnapshot::default();
        let s = summary_line(&snap);
        assert!(s.contains("(no probe data)"));
        assert!(s.contains("0 peers"));
    }

    #[test]
    fn summary_line_enrolled_but_zero_peers() {
        let snap = PreviewSnapshot {
            self_node: Some(SelfNodeView {
                node_id: "peer:a".into(),
                host: "a".into(),
                role: "host".into(),
                overlay_ip: "10.42.0.1".into(),
                cert_epoch: 0,
                mesh_id: "m".into(),
            }),
            ..PreviewSnapshot::default()
        };
        let s = summary_line(&snap);
        assert!(s.contains("Mesh: m"));
        assert!(s.contains("10.42.0.1"));
        assert!(s.contains("0 peers"));
    }
}
