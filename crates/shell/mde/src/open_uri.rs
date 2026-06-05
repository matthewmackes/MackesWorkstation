//! `mde open-uri <mde://…>` — the `x-scheme-handler/mde` URI dispatcher
//! (E4.20, migrated from the retired `mde-portal` `mde-open` + the
//! `action/shell` Bus responder).
//!
//! `xdg-open mde://…` routes here through `mde-open.desktop`. We parse the
//! URI ([`crate::uri`]) and run the matching **Win10** side effect directly
//! — the portal's Bus indirection isn't needed because every Win10 effect is
//! a stateless process spawn (open Explorer/Settings, launch an app, lock),
//! not a method on a single stateful GUI process.

use std::process::{Command, ExitCode};

use crate::uri::{parse_mde_uri, Action};

/// `mde open-uri <uri>` — parse + dispatch one `mde://` URI.
pub fn run(args: &[String]) -> ExitCode {
    let Some(uri) = args.first() else {
        eprintln!("usage: mde open-uri <mde://…>");
        return ExitCode::from(2);
    };
    let action = parse_mde_uri(uri);
    if matches!(action, Action::Unknown(_)) {
        eprintln!("mde open-uri: unrecognized URI: {uri}");
        return ExitCode::from(1);
    }
    dispatch(&action);
    ExitCode::SUCCESS
}

/// This `mde` binary's path, so the dispatch re-execs subcommands
/// (`mde lock`, `mde files`, …) as the *installed* shell, not whatever
/// `mde` happens to be on `$PATH`.
fn mde() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "mde".to_string())
}

fn spawn(cmd: &str, args: &[&str]) {
    let _ = Command::new(cmd).args(args).spawn();
}

/// Run the Win10 side effect for a parsed action. Every arm is a process
/// spawn (fire-and-forget); failures are swallowed (a broken handler must
/// never take down the caller).
fn dispatch(action: &Action) {
    let mde = mde();
    match action {
        Action::OpenApp(id) => spawn("gtk-launch", &[id]),
        Action::OpenFile(p) => spawn("xdg-open", &[&p.display().to_string()]),
        Action::Lock => spawn(&mde, &["lock"]),
        Action::Restart => spawn("systemctl", &["--user", "restart", "mde-session"]),
        // DnD / Focus assist lives in the Action Center (E4.6) — open it so the
        // toggle is one click away (there's no headless DnD flip in the Win10 era).
        Action::ToggleDnd => spawn(&mde, &["action-center"]),
        // labwc/wlr owns window focus; the portal's "raise the shell" is N/A.
        Action::Focus => {}
        Action::Goto { layer, sub, .. } => dispatch_goto(&mde, layer, sub.as_deref()),
        Action::Peer { .. } => {
            eprintln!("mde open-uri: cross-peer routing not wired (needs mesh RPC)");
        }
        Action::Unknown(raw) => eprintln!("mde open-uri: unknown: {raw}"),
    }
}

/// Map a portal-era `Goto` layer onto its Win10 surface.
fn dispatch_goto(mde: &str, layer: &str, sub: Option<&str>) {
    match layer {
        // The portal "library" layer ↦ Explorer; carry the sub-path when present.
        "library" => match sub {
            Some(s) => spawn(mde, &["files", s]),
            None => spawn(mde, &["files"]),
        },
        // "control"/"network" ↦ the modern Settings app.
        "control" | "network" => spawn(mde, &["settings"]),
        // "voip" ↦ the Your-Phone surface (closest Win10 idiom).
        "voip" => spawn(mde, &["phone"]),
        // "hub" ↦ the tiled Start overlay.
        "hub" => spawn(mde, &["start-win10"]),
        other => eprintln!("mde open-uri: unmapped layer: {other}"),
    }
}
