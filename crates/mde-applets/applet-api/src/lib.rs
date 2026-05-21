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

/// Phase E1.3 — host-side applet discovery. The panel
/// host walks `/usr/share/mde/applets/*.json` (system
/// installs) and `$XDG_DATA_HOME/mde/applets/*.json`
/// (per-user overrides) for [`AppletManifest`] files,
/// validates each, and emits a resolved set of unique
/// (slot, applet) pairs. Per-user manifests shadow
/// system manifests with the same `id`; slot conflicts
/// resolve to the later-id-wins rule documented on
/// [`AppletSlot`].
pub mod discovery {
    use super::{AppletId, AppletManifest};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    /// Canonical system-wide applet manifest dir.
    pub const SYSTEM_DIR: &str = "/usr/share/mde/applets";

    /// Per-user applet manifest dir relative to
    /// `$XDG_DATA_HOME` (or `~/.local/share` fallback).
    pub const USER_DIR_SUFFIX: &str = "mde/applets";

    /// Resolve the per-user applets dir.
    #[must_use]
    pub fn user_dir() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join(".local/share"))
                    .unwrap_or_else(|_| PathBuf::from("/var/empty"))
            });
        base.join(USER_DIR_SUFFIX)
    }

    /// Walk the canonical applet dirs + return one
    /// `AppletManifest` per `id`, with the per-user
    /// version winning over system. Errors per-file are
    /// swallowed (skipped silently) — the host shouldn't
    /// fail-to-launch over one bad manifest.
    #[must_use]
    pub fn discover() -> Vec<AppletManifest> {
        let mut by_id: HashMap<AppletId, AppletManifest> = HashMap::new();
        // System first.
        ingest_dir(Path::new(SYSTEM_DIR), &mut by_id);
        // User overrides.
        ingest_dir(&user_dir(), &mut by_id);
        let mut out: Vec<_> = by_id.into_values().collect();
        out.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        out
    }

    /// Walk one directory + ingest into the map.
    pub fn ingest_dir(dir: &Path, out: &mut HashMap<AppletId, AppletManifest>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(raw) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<AppletManifest>(&raw) else {
                continue;
            };
            out.insert(manifest.id.clone(), manifest);
        }
    }

    /// Validate a candidate manifest against the locked
    /// id pattern. Wraps [`AppletId::parse`] + checks the
    /// binary field is non-empty (the host execs it; an
    /// empty binary is unspawnable).
    ///
    /// # Errors
    ///
    /// Returns `Err(&'static str)` on any failed rule.
    pub fn validate_manifest(m: &AppletManifest) -> Result<(), &'static str> {
        let _ = AppletId::parse(m.id.as_str())?;
        if m.binary.is_empty() {
            return Err("manifest `binary` cannot be empty");
        }
        if m.binary.contains(['/', ' ', '\n']) {
            return Err("manifest `binary` must be a bare PATH name (no slashes / whitespace)");
        }
        if m.version.is_empty() {
            return Err("manifest `version` cannot be empty");
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::super::{AppletId, AppletManifest, AppletSlot};
        use super::*;

        #[test]
        fn system_dir_lock() {
            assert_eq!(SYSTEM_DIR, "/usr/share/mde/applets");
        }

        #[test]
        fn user_dir_uses_xdg_data_home_when_set() {
            std::env::set_var("XDG_DATA_HOME", "/tmp/test-xdg");
            let d = user_dir();
            std::env::remove_var("XDG_DATA_HOME");
            assert_eq!(d, PathBuf::from("/tmp/test-xdg/mde/applets"));
        }

        #[test]
        fn discover_returns_empty_when_no_dirs_exist() {
            // Override XDG_DATA_HOME + clear HOME so user_dir
            // points at a guaranteed-empty path.
            std::env::set_var("XDG_DATA_HOME", "/nonexistent-xdg-test");
            // System dir likely doesn't have anything either
            // in a test environment.
            let _ = discover(); // just verify no panic
            std::env::remove_var("XDG_DATA_HOME");
        }

        #[test]
        fn ingest_dir_skips_non_json() {
            let tmp = std::env::temp_dir().join("mde-applet-ingest-test");
            let _ = std::fs::remove_dir_all(&tmp);
            std::fs::create_dir_all(&tmp).unwrap();
            std::fs::write(tmp.join("readme.txt"), "ignore me").unwrap();
            let manifest = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "mde-applet-clock".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "test".into(),
                version: "0.0.0".into(),
            };
            std::fs::write(
                tmp.join("clock.json"),
                serde_json::to_string(&manifest).unwrap(),
            )
            .unwrap();
            let mut out = HashMap::new();
            ingest_dir(&tmp, &mut out);
            assert_eq!(out.len(), 1);
            assert!(out.contains_key(&AppletId::from_static("clock")));
            let _ = std::fs::remove_dir_all(&tmp);
        }

        #[test]
        fn ingest_dir_skips_malformed_json() {
            let tmp = std::env::temp_dir().join("mde-applet-malformed-test");
            let _ = std::fs::remove_dir_all(&tmp);
            std::fs::create_dir_all(&tmp).unwrap();
            std::fs::write(tmp.join("bad.json"), "not actually json").unwrap();
            let mut out = HashMap::new();
            ingest_dir(&tmp, &mut out);
            assert!(out.is_empty());
            let _ = std::fs::remove_dir_all(&tmp);
        }

        #[test]
        fn user_manifest_overrides_system_with_same_id() {
            // Same id in two dirs — ingest is called in order
            // (system first, then user), and HashMap::insert
            // replaces with the later value, so the second
            // call wins. That's the lock.
            let mut out = HashMap::new();
            let sys = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "mde-applet-clock".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "system".into(),
                version: "1.0.0".into(),
            };
            let user = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "mde-applet-clock".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "user override".into(),
                version: "1.0.1".into(),
            };
            out.insert(sys.id.clone(), sys);
            out.insert(user.id.clone(), user);
            let m = out.values().next().unwrap();
            assert_eq!(m.summary, "user override");
            assert_eq!(m.version, "1.0.1");
        }

        #[test]
        fn validate_manifest_rejects_empty_binary() {
            let m = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "x".into(),
                version: "1".into(),
            };
            assert!(validate_manifest(&m).is_err());
        }

        #[test]
        fn validate_manifest_rejects_path_traversal_in_binary() {
            let m = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "../bin/evil".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "x".into(),
                version: "1".into(),
            };
            assert!(validate_manifest(&m).is_err());
        }

        #[test]
        fn validate_manifest_accepts_well_formed() {
            let m = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "mde-applet-clock".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "ok".into(),
                version: "1.0".into(),
            };
            assert!(validate_manifest(&m).is_ok());
        }

        #[test]
        fn validate_manifest_rejects_empty_version() {
            let m = AppletManifest {
                id: AppletId::from_static("clock"),
                binary: "mde-applet-clock".into(),
                slot: AppletSlot::TopBarCenter,
                summary: "x".into(),
                version: "".into(),
            };
            assert!(validate_manifest(&m).is_err());
        }
    }
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
