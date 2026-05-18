//! Top-bar widget construction.
//!
//! Phase 1.5–1.7: fills the left / center / right slots with the
//! initial visual widgets:
//!
//! - Left:   Mackes button (`apple_menu_button`)
//! - Center: HH:MM clock with a 60 s timer (`clock`)
//! - Right:  Six-glyph status cluster (`status_cluster`)
//!
//! Each widget is a stub with the right shape; behavior (menu dropdown,
//! drawer open, etc.) lands in later phases per `docs/PROJECT_WORKLIST.md`.

use std::process::Command;

use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;

use crate::{apple_menu, desktop_files, icons, recents, weather};

/// Default coords for the weather popover until `panel.toml` grows a
/// `[weather]` section (8.5.3 follow-up). London works as a sane non-zero
/// default — met.no returns a real forecast for it so the popover proves
/// the wiring on first launch.
const DEFAULT_WEATHER_LAT: f64 = 51.507;
const DEFAULT_WEATHER_LON: f64 = -0.128;

/// Glyph size shown in the 20 px top bar. 14 px lets the icon breathe
/// against the height without clipping baseline math.
const TOP_BAR_ICON_PX: i32 = 14;

/// Glyph used as the Mackes-menu button. Q23 hinted at a Carbon mark;
/// we use `applications-system-symbolic` as a stand-in until the real
/// brand glyph lands.
const MACKES_BUTTON_ICON: &str = "applications-system-symbolic";

/// Right-side status cluster, in render order (left-to-right). Per Q8.
const STATUS_ITEMS: &[(&str, &str)] = &[
    ("mesh", "network-wireless-symbolic"),
    ("clipboard", "edit-paste-symbolic"),
    ("volume", "audio-volume-high-symbolic"),
    ("battery", "battery-symbolic"),
    ("notifications", "mail-unread-symbolic"),
    ("user", "system-users-symbolic"),
];

/// Build the Mackes-menu button. Click → drops a `gtk::Menu` populated
/// by `apple_menu::build` of the live `.desktop` scan, plus the canonical
/// system-action items (Sleep / Restart / Shut Down / Lock / Sign Out).
#[must_use]
pub fn apple_menu_button() -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name("mackes-apple-menu-button");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);

    if let Some(pb) = icons::load(MACKES_BUTTON_ICON, TOP_BAR_ICON_PX) {
        button.set_image(Some(&gtk::Image::from_pixbuf(Some(&pb))));
        button.set_always_show_image(true);
    } else {
        // No Carbon theme available (dev tree); use a tiny text glyph
        // so the slot is at least visible.
        button.set_label("M");
    }

    let button_for_handler = button.clone();
    button.connect_clicked(move |_| {
        let menu = build_apple_menu();
        menu.show_all();
        menu.popup_at_widget(
            &button_for_handler,
            gdk::Gravity::SouthWest,
            gdk::Gravity::NorthWest,
            None,
        );
    });
    button
}

/// Construct the full Apple-menu `gtk::Menu`. Composition matches the
/// design lock's Q24 ordering:
///   About / ─ / Settings / Software Update / ─ / Recent → / Applications →
///   / ─ / Force Quit / ─ / Sleep / Restart / Shut Down / ─ / Lock / Sign Out
fn build_apple_menu() -> gtk::Menu {
    let menu = gtk::Menu::new();
    menu.set_widget_name("mackes-apple-menu");

    add_item(&menu, "About Mackes", || {
        launch("mackes", &["--about"]);
    });
    menu.append(&gtk::SeparatorMenuItem::new());
    add_item(&menu, "Settings…", || {
        launch("mackes", &[]);
    });
    add_item(&menu, "Software Update…", || {
        launch("mackes", &["--update"]);
    });
    menu.append(&gtk::SeparatorMenuItem::new());
    menu.append(&build_recents_submenu());
    menu.append(&build_applications_submenu());
    menu.append(&gtk::SeparatorMenuItem::new());
    add_item(&menu, "Force Quit…", || {
        eprintln!("mackes-panel: force-quit (stub)");
    });
    menu.append(&gtk::SeparatorMenuItem::new());
    add_item(&menu, "Sleep", || run_loginctl("suspend"));
    add_item(&menu, "Restart…", || run_loginctl("reboot"));
    add_item(&menu, "Shut Down…", || run_loginctl("poweroff"));
    menu.append(&gtk::SeparatorMenuItem::new());
    add_item(&menu, "Lock Screen", || run_loginctl("lock-session"));
    add_item(&menu, "Sign Out…", || {
        launch("xfce4-session-logout", &["--logout"]);
    });

    menu
}

fn build_recents_submenu() -> gtk::MenuItem {
    let parent = gtk::MenuItem::with_label("Recent Items");
    let submenu = gtk::Menu::new();
    submenu.set_widget_name("mackes-apple-menu-recents");

    let items = recents::load(10);
    if items.is_empty() {
        let placeholder = gtk::MenuItem::with_label("(no recent items)");
        placeholder.set_sensitive(false);
        submenu.append(&placeholder);
    } else {
        for item in items {
            let entry = gtk::MenuItem::with_label(&item.label);
            let uri = item.uri.clone();
            entry.connect_activate(move |_| {
                if let Err(e) = Command::new("xdg-open").arg(&uri).spawn() {
                    eprintln!("mackes-panel: xdg-open recent {uri} failed: {e}");
                }
            });
            submenu.append(&entry);
        }
    }

    parent.set_submenu(Some(&submenu));
    parent
}

fn build_applications_submenu() -> gtk::MenuItem {
    let parent = gtk::MenuItem::with_label("Applications");
    let submenu = gtk::Menu::new();
    submenu.set_widget_name("mackes-apple-menu-applications");

    for cat in apple_menu::build(&desktop_files::scan()) {
        let cat_item = gtk::MenuItem::with_label(cat.label);
        let cat_submenu = gtk::Menu::new();
        for entry in cat.entries {
            let item = gtk::MenuItem::with_label(&entry.name);
            let exec = entry.exec.clone();
            let terminal = entry.terminal;
            item.connect_activate(move |_| {
                launch_exec(&exec, terminal);
            });
            cat_submenu.append(&item);
        }
        cat_item.set_submenu(Some(&cat_submenu));
        submenu.append(&cat_item);
    }

    parent.set_submenu(Some(&submenu));
    parent
}

fn add_item<F>(menu: &gtk::Menu, label: &str, on_activate: F)
where
    F: Fn() + 'static,
{
    let item = gtk::MenuItem::with_label(label);
    item.connect_activate(move |_| on_activate());
    menu.append(&item);
}

/// `loginctl <verb>` (no `PolicyKit` prompt for `lock-session` and
/// `suspend`; `reboot`/`poweroff` will prompt via `PolicyKit`).
fn run_loginctl(verb: &str) {
    if let Err(e) = Command::new("loginctl").arg(verb).spawn() {
        eprintln!("mackes-panel: loginctl {verb} failed: {e}");
    }
}

fn launch(program: &str, args: &[&str]) {
    if let Err(e) = Command::new(program).args(args).spawn() {
        eprintln!("mackes-panel: launching {program} failed: {e}");
    }
}

/// Spawn an `Exec=` line from a `.desktop` file. We strip the
/// freedesktop field codes (`%U`, `%F`, etc.) and hand the rest to
/// `/bin/sh -c` so quoting works. Terminal apps get prefixed with the
/// user's preferred terminal.
///
/// Shared with `dock::AppModule` (Phase 5.1), so re-exported as `pub`.
pub fn launch_exec(exec: &str, terminal: bool) {
    let stripped = strip_field_codes(exec);
    let cmd = if terminal {
        format!("xfce4-terminal -e {stripped}")
    } else {
        stripped
    };
    if let Err(e) = Command::new("/bin/sh").arg("-c").arg(&cmd).spawn() {
        eprintln!("mackes-panel: spawn failed: {cmd}: {e}");
    }
}

fn strip_field_codes(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|tok| !is_field_code(tok))
        .collect::<Vec<&str>>()
        .join(" ")
}

fn is_field_code(token: &str) -> bool {
    matches!(
        token,
        "%f" | "%F" | "%u" | "%U" | "%d" | "%D" | "%n" | "%N" | "%i" | "%c" | "%k" | "%v" | "%m"
    )
}

/// Build the center clock widget. 12-hour format ("h:MM AM/PM") per the
/// 8.5.3 polish bundle. The label updates every 60 s and on startup, and
/// the whole thing is wrapped in a frameless `gtk::Button` whose click
/// pops up a weather panel patterned after xfce4-weather-plugin
/// (`crate::weather`).
///
/// `gtk::Label` is a reference-counted `GObject` handle, so cloning it
/// for the timer closure is just a refcount bump (no `Rc<RefCell<…>>`
/// needed).
#[must_use]
pub fn clock() -> gtk::Button {
    let label = gtk::Label::new(None);
    label.set_widget_name("mackes-top-clock-label");
    label.set_text(&current_clock_text());

    let button = gtk::Button::new();
    button.set_widget_name("mackes-top-clock");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);
    button.add(&label);

    let initial_delay_s = seconds_until_next_minute();
    glib::timeout_add_seconds_local(initial_delay_s, move || {
        label.set_text(&current_clock_text());
        let label_recurring = label.clone();
        glib::timeout_add_seconds_local(60, move || {
            label_recurring.set_text(&current_clock_text());
            glib::ControlFlow::Continue
        });
        glib::ControlFlow::Break
    });

    let button_for_click = button.clone();
    button.connect_clicked(move |_| {
        let popover = weather::build_popover(
            button_for_click.upcast_ref::<gtk::Widget>(),
            DEFAULT_WEATHER_LAT,
            DEFAULT_WEATHER_LON,
        );
        popover.show_all();
        popover.popup();
    });

    button
}

/// "h:MM AM/PM" in the system locale. We use `%l:%M %p` because `%l`
/// emits the hour without a leading zero (a leading space, actually —
/// per POSIX strftime — so we `trim_start` the result). `%H:%M` was the
/// 24-hour predecessor; we keep the fallback string the same length so
/// the slot doesn't visually jitter when the formatter fails.
fn current_clock_text() -> String {
    let now = glib::DateTime::now_local().expect("system clock");
    now.format("%l:%M %p").map_or_else(
        |_| "--:-- --".to_owned(),
        |g| g.as_str().trim_start().to_owned(),
    )
}

/// Seconds remaining until the next clock minute. `glib::timeout_add_seconds`
/// takes whole seconds, so we floor — losing at most a few hundred
/// milliseconds of accuracy on the first tick.
fn seconds_until_next_minute() -> u32 {
    let now = glib::DateTime::now_local().expect("system clock");
    let secs = now.second();
    if secs >= 60 {
        1
    } else {
        let remaining = 60 - secs;
        u32::try_from(remaining).unwrap_or(1)
    }
}

/// Build the right-side status cluster — six Carbon glyphs side by side.
/// Click anywhere in the cluster opens the Notification Drawer (Q28),
/// stubbed for now.
#[must_use]
pub fn status_cluster() -> gtk::Box {
    let cluster = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    cluster.set_widget_name("mackes-status-cluster");

    for (slug, icon_name) in STATUS_ITEMS {
        cluster.pack_start(&status_item(slug, icon_name), false, false, 0);
    }

    cluster
}

fn status_item(slug: &str, icon_name: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name(&format!("mackes-status-{slug}"));
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);

    let pb: Option<Pixbuf> = icons::load(icon_name, TOP_BAR_ICON_PX);
    if let Some(pb) = pb {
        button.set_image(Some(&gtk::Image::from_pixbuf(Some(&pb))));
        button.set_always_show_image(true);
    } else {
        // Dev fallback so the slot remains discoverable.
        button.set_label(&slug.chars().next().unwrap_or('?').to_string());
    }

    // 8.5.4 polish: clicking a status-cluster icon now pops up an in-
    // process review popover *immediately*. The popover then offers a
    // secondary button that hands off to the Python drawer subprocess
    // (`mackes --drawer --drawer-focus <slug>`, per Q28). This gives
    // the user visible feedback whether or not the drawer process is
    // up — addressing the "Unable to open the dropdown to review" bug.
    let slug_owned = slug.to_owned();
    let button_for_click = button.clone();
    button.connect_clicked(move |_| {
        let popover =
            build_status_popover(button_for_click.upcast_ref::<gtk::Widget>(), &slug_owned);
        popover.show_all();
        popover.popup();
    });

    button
}

/// Build the per-slug review popover shown when a status-cluster icon
/// is clicked. Lightweight: a heading naming the cluster, a single
/// short summary line, and an "Open in Drawer →" button that delegates
/// to the existing Python notification drawer.
fn build_status_popover(anchor: &gtk::Widget, slug: &str) -> gtk::Popover {
    let popover = gtk::Popover::new(Some(anchor));
    popover.set_widget_name(&format!("mackes-status-popover-{slug}"));
    popover.set_position(gtk::PositionType::Bottom);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_margin_start(16);
    column.set_margin_end(16);
    column.set_margin_top(12);
    column.set_margin_bottom(12);

    let title = gtk::Label::new(Some(status_popover_title(slug)));
    title.set_widget_name("mackes-status-popover-title");
    title.set_halign(gtk::Align::Start);
    column.pack_start(&title, false, false, 0);

    let summary = gtk::Label::new(Some(status_popover_summary(slug)));
    summary.set_widget_name("mackes-status-popover-summary");
    summary.set_halign(gtk::Align::Start);
    column.pack_start(&summary, false, false, 0);

    let drawer_btn = gtk::Button::with_label("Open in Drawer →");
    drawer_btn.set_widget_name("mackes-status-popover-drawer");
    let slug_owned = slug.to_owned();
    let popover_for_click = popover.clone();
    drawer_btn.connect_clicked(move |_| {
        if let Err(e) = Command::new("mackes")
            .args(["--drawer", "--drawer-focus", &slug_owned])
            .spawn()
        {
            eprintln!("mackes-panel: drawer launch failed ({slug_owned}): {e}");
        }
        popover_for_click.popdown();
    });
    column.pack_start(&drawer_btn, false, false, 0);

    popover.add(&column);
    popover
}

fn status_popover_title(slug: &str) -> &'static str {
    match slug {
        "mesh" => "Mesh",
        "clipboard" => "Clipboard",
        "volume" => "Volume",
        "battery" => "Battery",
        "notifications" => "Notifications",
        "user" => "User",
        _ => "Status",
    }
}

fn status_popover_summary(slug: &str) -> &'static str {
    match slug {
        "mesh" => "Peers, shares, services",
        "clipboard" => "Recent clipboard items",
        "volume" => "Output device & level",
        "battery" => "Power state & estimate",
        "notifications" => "Unread alerts",
        "user" => "Session & account",
        _ => "Status",
    }
}
