//! `mde-session` — v2.0.0 Phase D.1 (skeleton) + D.4 (lock) + D.7
//! (retires mackes-wm / mackes-enforce-session).
//!
//! User-session orchestrator. systemd's mde-session.service execs
//! this binary at login; it:
//!
//!   1. Reads the v1.x → v2.0.0 config-path migrator's marker (Phase
//!      0.5) — runs the migrator if it hasn't yet.
//!   2. Re-applies persisted settings sidecars (Phase C
//!      $XDG_CACHE_HOME/mde/*) via the matching applier modules.
//!   3. Registers the `dev.mackes.MDE.Session` zbus surface
//!      (Logout / Restart / Shutdown / Lock / SaveLayout).
//!   4. Execs the compositor (labwc) and waits.
//!   5. On SIGTERM / SIGINT cleanly tears down: signals the zbus
//!      server, waits for labwc to exit, exits 0.
//!
//! Iced + libcosmic for the logout / restart / shutdown dialog
//! (Phase D.2) is intentionally NOT pulled in here — that's a
//! separate process (`mde-logout-dialog`) the Session surface can
//! exec when needed. Keeping this binary Iced-free means the user-
//! session unit stays tiny (~250 LOC) + boots fast.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod autostart;
mod lock;
mod session;
mod theme_pump;

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::Context;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

/// MDE-shipped labwc config DIR. Two roles: (a) the `-C` fallback when the
/// operator has no per-user config at `~/.config/labwc/` at all, and (b) the
/// SEED source — `seed_user_labwc_config` copies any of its files MISSING from
/// the user dir into it before launch (see that fn for the regression it
/// fixes). labwc reads `{rc.xml,menu.xml,autostart,themerc}` from this
/// directory. This is the same tree the RPM ships as the skel (see
/// crates/shell/mde Cargo.toml generate-rpm assets).
const SYSTEM_LABWC_CONFIG_DIR: &str = "/usr/share/mde/skel/.config/labwc";

/// Compositor command. Defaults to `labwc` (wayland feature, the locked
/// compositor — plan §0 Q8) or `i3` (x11 feature); override via
/// `$MDE_COMPOSITOR` for development (e.g. `sway` for the nested-capture
/// harness).
fn compositor_cmd() -> String {
    mackesd_core::env_with_legacy_fallback("MDE_COMPOSITOR", "MACKES_COMPOSITOR")
        .unwrap_or_else(default_compositor)
}

#[cfg(not(feature = "x11"))]
fn default_compositor() -> String {
    "labwc".to_owned()
}

#[cfg(feature = "x11")]
fn default_compositor() -> String {
    "i3".to_owned()
}

fn user_labwc_config_dir() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    Some(base.join("labwc"))
}

/// Pure helper: pick the `-C <dir>` args labwc should be invoked with.
/// Empty vec = "let labwc resolve its config the default way" — returned
/// for non-labwc compositors, when the user already has `~/.config/labwc/`,
/// or when the system fallback dir is also absent.
fn labwc_config_args(
    compositor: &str,
    user_config_dir: Option<&Path>,
    system_config_dir: &Path,
) -> Vec<String> {
    if compositor != "labwc" {
        return Vec::new();
    }
    if let Some(p) = user_config_dir {
        if p.exists() {
            return Vec::new();
        }
    }
    if !system_config_dir.exists() {
        return Vec::new();
    }
    vec![
        "-C".to_string(),
        system_config_dir.to_string_lossy().into_owned(),
    ]
}

/// Recursively copy every file under `system_dir` that is ABSENT from
/// `user_dir` into `user_dir`, preserving the source's permission bits
/// (`std::fs::copy` carries the mode, so `autostart` + `scripts/*.sh` keep
/// their exec bit). Never overwrites an existing user file. Returns the count
/// copied.
///
/// This is the fix for the second-login black-desktop regression: the
/// `labwc -C <system skel>` fallback only fires when `~/.config/labwc/` is
/// **wholly** absent, but `mde panel` (`sync_root_menu` writes `menu.xml`),
/// `mouse.rs` (`rc.xml`) and `keyboard.rs` (`environment`) each create that dir
/// holding only THEIR one file. After any of them runs once, labwc reads a
/// PARTIAL user dir — missing the `autostart` that launches `mde panel` — so
/// the desktop comes up black with only labwc's stock menu. Seeding makes the
/// user dir self-complete, so labwc always finds the autostart regardless of
/// which tool touched the dir first. Pure over the two paths; unit-tested.
fn seed_missing_files(system_dir: &Path, user_dir: &Path) -> std::io::Result<usize> {
    if !system_dir.is_dir() {
        return Ok(0);
    }
    let mut copied = 0;
    for entry in std::fs::read_dir(system_dir)? {
        let entry = entry?;
        let src = entry.path();
        let dst = user_dir.join(entry.file_name());
        if src.is_dir() {
            copied += seed_missing_files(&src, &dst)?;
        } else if !dst.exists() {
            std::fs::create_dir_all(user_dir)?;
            std::fs::copy(&src, &dst)?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Seed the user's `~/.config/labwc/` from the system skel before launch (see
/// [`seed_missing_files`]). No-op for non-labwc compositors; never fatal — a
/// failure just leaves the `-C` fallback to do what it can.
fn seed_user_labwc_config(compositor: &str) {
    if compositor != "labwc" {
        return;
    }
    let Some(user_dir) = user_labwc_config_dir() else {
        return;
    };
    match seed_missing_files(Path::new(SYSTEM_LABWC_CONFIG_DIR), &user_dir) {
        Ok(0) => {}
        Ok(n) => tracing::info!(
            count = n,
            "mde-session: seeded {n} missing labwc skel file(s) into {}",
            user_dir.display()
        ),
        Err(e) => tracing::warn!("mde-session: seeding labwc skel into user config failed: {e}"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();
    tracing::info!("mde-session: starting");

    // 0. Portal-37 — apply the MDE-Dark + Intel One Mono settings to
    //    every GTK / Qt6 config file the pump owns before
    //    autostarted apps come up. Idempotent; never fatal.
    let changed = theme_pump::apply();
    if !changed.is_empty() {
        tracing::info!(count = changed.len(), "theme-pump: settings refreshed");
    }

    // 1. Read autostart entries the user has explicitly enabled and
    //    spawn each as a detached child. Hidden=true overlays
    //    suppress system-wide entries.
    autostart::launch_user_autostart().await;

    // 2. DBUS-1 — serve the session lifecycle verbs on the Bus
    //    (action/session/{logout,restart,shutdown,lock}), replacing the
    //    retired dev.mackes.MDE.Session D-Bus interface (Q96). Runs on a
    //    detached thread with its own current-thread runtime — `Persist`
    //    (rusqlite) isn't `Send`, so it can't live on this multi-thread
    //    async executor. The thread exits with the process on SIGTERM
    //    tear-down below.
    std::thread::Builder::new()
        .name("session-bus".into())
        .spawn(move || {
            let Some(bus_root) = mde_bus::default_data_dir() else {
                tracing::warn!("session responder: no Bus data dir; lifecycle verbs unavailable");
                return;
            };
            match mde_bus::persist::Persist::open(bus_root) {
                Ok(persist) => session::serve_bus(&persist, || false),
                Err(e) => tracing::warn!("session responder: opening Bus store: {e}"),
            }
        })
        .context("spawning the session Bus responder thread")?;

    // 3. Exec the compositor.
    let cmp = compositor_cmd();
    // Complete the user's ~/.config/labwc from the system skel first, so a
    // partial dir left by `mde panel`/`mouse`/`keyboard` (which would otherwise
    // mask the autostart) still boots a full desktop. See seed_missing_files.
    seed_user_labwc_config(&cmp);
    let user_cfg = user_labwc_config_dir();
    let extra_args = labwc_config_args(
        &cmp,
        user_cfg.as_deref(),
        Path::new(SYSTEM_LABWC_CONFIG_DIR),
    );
    if !extra_args.is_empty() {
        tracing::info!(
            "mde-session: no ~/.config/labwc — falling back to {SYSTEM_LABWC_CONFIG_DIR}",
        );
    }
    tracing::info!("mde-session: starting compositor {cmp}");
    let mut child = Command::new(&cmp)
        .args(&extra_args)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning {cmp}"))?;

    let mut sigterm = signal(SignalKind::terminate()).context("installing SIGTERM handler")?;
    let mut sigint = signal(SignalKind::interrupt()).context("installing SIGINT handler")?;

    tokio::select! {
        _ = sigterm.recv() => {
            tracing::info!("mde-session: SIGTERM received; killing compositor");
        }
        _ = sigint.recv() => {
            tracing::info!("mde-session: SIGINT received; killing compositor");
        }
        res = child.wait() => {
            match res {
                Ok(status) => tracing::info!("mde-session: compositor exited {status}"),
                Err(e)     => tracing::error!("mde-session: wait failed: {e}"),
            }
            return Ok(());
        }
    }
    let _ = child.start_kill();
    let _ = child.wait().await;
    tracing::info!("mde-session: exiting");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labwc_config_args_empty_for_non_labwc_compositor() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("sysdir");
        std::fs::create_dir(&system).unwrap();
        let user = tmp.path().join("user-missing");
        assert!(labwc_config_args("i3", Some(&user), &system).is_empty());
        assert!(labwc_config_args("sway", Some(&user), &system).is_empty());
    }

    #[test]
    fn labwc_config_args_empty_when_user_dir_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user-config");
        std::fs::create_dir(&user).unwrap();
        let system = tmp.path().join("system-config");
        std::fs::create_dir(&system).unwrap();
        assert!(labwc_config_args("labwc", Some(&user), &system).is_empty());
    }

    #[test]
    fn labwc_config_args_empty_when_system_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user-missing");
        let system = tmp.path().join("system-missing");
        assert!(labwc_config_args("labwc", Some(&user), &system).is_empty());
    }

    #[test]
    fn labwc_config_args_returns_big_c_flag_when_user_missing_and_system_present() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let user = tmp.path().join("user-missing");
        let system = tmp.path().join("system-config");
        std::fs::create_dir(&system).unwrap();
        let args = labwc_config_args("labwc", Some(&user), &system);
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-C");
        assert_eq!(args[1], system.to_string_lossy());
    }

    #[test]
    fn labwc_config_args_returns_big_c_flag_when_no_user_path_at_all() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("system-config");
        std::fs::create_dir(&system).unwrap();
        let args = labwc_config_args("labwc", None, &system);
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-C");
    }

    #[test]
    fn system_labwc_config_dir_constant_points_at_install_path() {
        assert_eq!(SYSTEM_LABWC_CONFIG_DIR, "/usr/share/mde/skel/.config/labwc");
    }

    /// The regression: a partial user dir (only `menu.xml`, as `mde panel`
    /// leaves it) gets `autostart` + nested `scripts/*` seeded in, while the
    /// pre-existing `menu.xml` is preserved verbatim.
    #[test]
    fn seed_completes_a_partial_user_dir_without_clobbering() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("system");
        std::fs::create_dir_all(system.join("scripts")).unwrap();
        std::fs::write(system.join("autostart"), b"mde panel &\n").unwrap();
        std::fs::write(system.join("menu.xml"), b"<system-menu/>").unwrap();
        std::fs::write(system.join("rc.xml"), b"<rc/>").unwrap();
        std::fs::write(system.join("scripts/brightness.sh"), b"#!/bin/sh\n").unwrap();

        // User has only their own customised menu.xml (the panel-generated file).
        let user = tmp.path().join("user");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(user.join("menu.xml"), b"<user-menu/>").unwrap();

        let copied = seed_missing_files(&system, &user).expect("seed");
        assert_eq!(copied, 3, "autostart + rc.xml + scripts/brightness.sh");
        assert_eq!(
            std::fs::read(user.join("menu.xml")).unwrap(),
            b"<user-menu/>",
            "the user's menu.xml must NOT be overwritten"
        );
        assert!(user.join("autostart").is_file());
        assert!(user.join("scripts/brightness.sh").is_file());

        // Second run is a clean no-op — everything is now present.
        assert_eq!(seed_missing_files(&system, &user).expect("re-seed"), 0);
    }

    /// A wholly-absent user dir is created and fully populated from the skel.
    #[test]
    fn seed_creates_user_dir_when_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("system");
        std::fs::create_dir_all(&system).unwrap();
        std::fs::write(system.join("autostart"), b"mde panel &\n").unwrap();
        let user = tmp.path().join("nonexistent/labwc");

        let copied = seed_missing_files(&system, &user).expect("seed");
        assert_eq!(copied, 1);
        assert!(user.join("autostart").is_file());
    }

    /// Missing system skel is a silent no-op (dev box with no RPM installed).
    #[test]
    fn seed_noop_when_system_skel_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("does-not-exist");
        let user = tmp.path().join("user");
        assert_eq!(seed_missing_files(&system, &user).expect("seed"), 0);
        assert!(
            !user.exists(),
            "no user dir conjured when there's nothing to seed"
        );
    }

    /// `std::fs::copy` carries the source mode, so a seeded `autostart` keeps
    /// its exec bit — labwc requires it to be runnable.
    #[cfg(unix)]
    #[test]
    fn seed_preserves_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().expect("tempdir");
        let system = tmp.path().join("system");
        std::fs::create_dir_all(&system).unwrap();
        let src = system.join("autostart");
        std::fs::write(&src, b"#!/bin/sh\nmde panel &\n").unwrap();
        std::fs::set_permissions(&src, std::fs::Permissions::from_mode(0o755)).unwrap();

        let user = tmp.path().join("user");
        seed_missing_files(&system, &user).expect("seed");
        let mode = std::fs::metadata(user.join("autostart"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "exec bits must survive the copy");
    }
}
