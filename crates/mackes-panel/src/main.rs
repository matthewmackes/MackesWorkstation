//! mackes-panel — top status bar + bottom dock for Mackes XFCE Workstation.
//!
//! Phase 0.4: first-boot top bar — a 20 px strut-anchored window at the top
//! edge of the primary monitor. No content yet (Phase 1.x lands appmenu /
//! clock / status cluster). Exits cleanly on SIGINT / SIGTERM so systemd
//! user units can manage the lifecycle.

#![forbid(unsafe_code)]

use gdk::prelude::*;
use gtk::prelude::*;

const TOP_BAR_HEIGHT_PX: i32 = 20;
const APP_ID: &str = "shell.mackes.Panel";

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_top_bar);

    // Quit cleanly on SIGTERM / SIGINT. We use unix_signal_add_local so
    // the handler runs on the GTK main thread (gtk::Application is !Send).
    // The default GTK behavior is unclean exit on signal; for a long-
    // running panel we want orderly shutdown so systemd's TimeoutStopSec
    // doesn't SIGKILL us.
    let app_for_sigterm = app.clone();
    glib::unix_signal_add_local(libc::SIGTERM, move || {
        app_for_sigterm.quit();
        glib::ControlFlow::Break
    });
    let app_for_sigint = app.clone();
    glib::unix_signal_add_local(libc::SIGINT, move || {
        app_for_sigint.quit();
        glib::ControlFlow::Break
    });

    app.run()
}

fn build_top_bar(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("mackes-panel-top")
        .decorated(false)
        .skip_taskbar_hint(true)
        .skip_pager_hint(true)
        .resizable(false)
        .type_hint(gdk::WindowTypeHint::Dock)
        .build();

    let screen_width = primary_monitor_width().unwrap_or(1920);
    window.set_default_size(screen_width, TOP_BAR_HEIGHT_PX);
    window.move_(0, 0);

    // Visual placeholder until Phase 1.x — solid PatternFly dark
    // surface (#151515) so the strut is visible during development.
    let style = window.style_context();
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(b"window { background-color: #151515; }")
        .expect("inline css must parse");
    style.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    window.show_all();
}

/// Primary monitor's width in CSS pixels. Returns `None` if no display
/// (CI / headless), so callers can fall back to a sensible default.
fn primary_monitor_width() -> Option<i32> {
    let display = gdk::Display::default()?;
    let monitor = display.primary_monitor()?;
    Some(monitor.geometry().width())
}
