//! `mde://` URI scheme (E4.20 — migrated verbatim from the retired
//! `mde-portal` `uri.rs`, Portal-35).
//!
//! Single source of truth for shell deep linking. External apps emit
//! `mde://...` URIs via `xdg-open`, routed through the
//! `x-scheme-handler/mde` desktop file; the [`mde open-uri`](crate::open_uri)
//! subcommand parses the URI here and runs the matching Win10 side effect.
//! (The portal's Bus indirection is gone — Win10's dispatch is stateless
//! process spawns, so the handler dispatches directly.)
//!
//! Grammar:
//!
//! ```text
//!   mde://<verb>[/<path>][?<query>]
//! ```
//!
//! Verbs:
//!   • `hub`              → Goto("hub")
//!   • `library[/<path>]` → Goto("library")           (path piece reserved)
//!   • `control[/<panel>]`→ Goto("control")           (sub-panel reserved)
//!   • `voip`             → Goto("voip")
//!   • `network`          → Goto("network")
//!   • `lock`             → Lock
//!   • `focus`            → Focus
//!   • `dnd-toggle`       → ToggleDND
//!   • `restart`          → Restart
//!   • `peer/<host>/<sub>`→ Peer(host, parse(sub))   (cross-peer)
//!   • `app/<id>`         → OpenApp(id)              (gtk-launch)
//!   • `file/<path>`      → OpenFile(path)           (xdg-open passthrough)
//!
//! Unknown verbs return `Action::Unknown(uri)` so callers can log
//! without panicking.

use std::path::PathBuf;

/// Parsed `mde://` action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Navigate to a named shell surface/layer.
    Goto {
        layer: String,
        /// Optional sub-path (`/Downloads` in `mde://library/Downloads`).
        sub: Option<String>,
        /// Optional query string (`?filter=foo` in `mde://library?filter=foo`).
        query: Option<String>,
    },
    /// Activate the lock screen.
    Lock,
    /// Bring the shell to the foreground (no-op under labwc — wlr owns focus).
    Focus,
    /// Flip the do-not-disturb state.
    ToggleDnd,
    /// Soft-restart the session.
    Restart,
    /// Cross-peer routing: deliver the nested action on `<host>`.
    Peer { host: String, inner: Box<Action> },
    /// Launch a desktop app by its `.desktop` id (without the `.desktop` suffix).
    OpenApp(String),
    /// Open a file path via `xdg-open`.
    OpenFile(PathBuf),
    /// Unrecognized URI — preserved verbatim so callers can log it.
    Unknown(String),
}

const SCHEME: &str = "mde://";

/// Parse a `mde://` URI into an [`Action`].
///
/// Returns `Action::Unknown(input)` when the URI:
///   • does not begin with `mde://`,
///   • is missing a verb after the scheme,
///   • or uses a verb the parser does not recognize.
///
/// The parser does not perform percent-decoding — callers that need
/// it should decode the path/query themselves with a library like
/// `percent-encoding`.  This keeps the parser dependency-free.
#[must_use]
pub fn parse_mde_uri(input: &str) -> Action {
    let Some(rest) = input.strip_prefix(SCHEME) else {
        return Action::Unknown(input.to_string());
    };
    // Split path + query at the first `?`.
    let (path, query) = match rest.find('?') {
        Some(i) => (&rest[..i], Some(rest[i + 1..].to_string())),
        None => (rest, None),
    };
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return Action::Unknown(input.to_string());
    }
    // Split verb from sub-path on the first `/`.
    let (verb, sub) = match path.find('/') {
        Some(i) => (&path[..i], Some(path[i + 1..].to_string())),
        None => (path, None),
    };
    match verb {
        "hub" | "library" | "control" | "voip" | "network" => Action::Goto {
            layer: verb.to_string(),
            sub,
            query,
        },
        "lock" => Action::Lock,
        "focus" => Action::Focus,
        "dnd-toggle" => Action::ToggleDnd,
        "restart" => Action::Restart,
        "peer" => parse_peer(sub.as_deref().unwrap_or(""), input),
        "app" => match sub {
            Some(id) if !id.is_empty() => Action::OpenApp(id),
            _ => Action::Unknown(input.to_string()),
        },
        "file" => match sub {
            Some(p) if !p.is_empty() => {
                // Restore leading slash for absolute paths so
                // `mde://file//home/x/y.txt` ↦ `/home/x/y.txt`.
                let path = if p.starts_with('/') {
                    p
                } else {
                    format!("/{p}")
                };
                Action::OpenFile(PathBuf::from(path))
            }
            _ => Action::Unknown(input.to_string()),
        },
        _ => Action::Unknown(input.to_string()),
    }
}

fn parse_peer(rest: &str, original: &str) -> Action {
    // `<host>/<verb>...`
    let Some(slash) = rest.find('/') else {
        return Action::Unknown(original.to_string());
    };
    let host = rest[..slash].to_string();
    if host.is_empty() {
        return Action::Unknown(original.to_string());
    }
    let inner = format!("{SCHEME}{}", &rest[slash + 1..]);
    let inner_action = parse_mde_uri(&inner);
    if matches!(inner_action, Action::Unknown(_)) {
        return Action::Unknown(original.to_string());
    }
    Action::Peer {
        host,
        inner: Box::new(inner_action),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn goto(layer: &str) -> Action {
        Action::Goto {
            layer: layer.to_string(),
            sub: None,
            query: None,
        }
    }

    #[test]
    fn parse_bare_verbs() {
        assert_eq!(parse_mde_uri("mde://hub"), goto("hub"));
        assert_eq!(parse_mde_uri("mde://library"), goto("library"));
        assert_eq!(parse_mde_uri("mde://control"), goto("control"));
        assert_eq!(parse_mde_uri("mde://voip"), goto("voip"));
        assert_eq!(parse_mde_uri("mde://network"), goto("network"));
        assert_eq!(parse_mde_uri("mde://lock"), Action::Lock);
        assert_eq!(parse_mde_uri("mde://focus"), Action::Focus);
        assert_eq!(parse_mde_uri("mde://dnd-toggle"), Action::ToggleDnd);
        assert_eq!(parse_mde_uri("mde://restart"), Action::Restart);
    }

    #[test]
    fn parse_library_with_subpath() {
        assert_eq!(
            parse_mde_uri("mde://library/Downloads"),
            Action::Goto {
                layer: "library".into(),
                sub: Some("Downloads".into()),
                query: None,
            }
        );
    }

    #[test]
    fn parse_control_with_subpanel() {
        assert_eq!(
            parse_mde_uri("mde://control/network"),
            Action::Goto {
                layer: "control".into(),
                sub: Some("network".into()),
                query: None,
            }
        );
    }

    #[test]
    fn parse_sub_and_query() {
        assert_eq!(
            parse_mde_uri("mde://library/Downloads?sort=mtime"),
            Action::Goto {
                layer: "library".into(),
                sub: Some("Downloads".into()),
                query: Some("sort=mtime".into()),
            }
        );
    }

    #[test]
    fn parse_open_app() {
        assert_eq!(
            parse_mde_uri("mde://app/org.gnome.TextEditor"),
            Action::OpenApp("org.gnome.TextEditor".into())
        );
    }

    #[test]
    fn parse_open_file_absolute() {
        assert_eq!(
            parse_mde_uri("mde://file//home/mm/notes.md"),
            Action::OpenFile(PathBuf::from("/home/mm/notes.md"))
        );
    }

    #[test]
    fn parse_open_file_relative_becomes_absolute() {
        assert_eq!(
            parse_mde_uri("mde://file/home/mm/notes.md"),
            Action::OpenFile(PathBuf::from("/home/mm/notes.md"))
        );
    }

    #[test]
    fn parse_peer_inner_goto() {
        assert_eq!(
            parse_mde_uri("mde://peer/host2/library/Downloads"),
            Action::Peer {
                host: "host2".into(),
                inner: Box::new(Action::Goto {
                    layer: "library".into(),
                    sub: Some("Downloads".into()),
                    query: None,
                }),
            }
        );
    }

    #[test]
    fn parse_peer_missing_host_is_unknown() {
        assert!(matches!(
            parse_mde_uri("mde://peer//lock"),
            Action::Unknown(_)
        ));
    }

    #[test]
    fn parse_wrong_scheme_is_unknown() {
        assert!(matches!(
            parse_mde_uri("https://example.com"),
            Action::Unknown(_)
        ));
        assert!(matches!(parse_mde_uri(""), Action::Unknown(_)));
        assert!(matches!(parse_mde_uri("mde://"), Action::Unknown(_)));
    }

    #[test]
    fn parse_unknown_verb_is_unknown() {
        assert_eq!(
            parse_mde_uri("mde://flubber"),
            Action::Unknown("mde://flubber".into())
        );
    }

    #[test]
    fn parse_trailing_slash_tolerated() {
        assert_eq!(parse_mde_uri("mde://hub/"), goto("hub"));
        assert_eq!(parse_mde_uri("mde://lock/"), Action::Lock);
    }
}
