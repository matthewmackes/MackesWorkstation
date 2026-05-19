//! Carbon-themed logout dialog — replaces the upstream
//! `xfce4-session-logout` window with a Mackes-styled three-button
//! confirm prompt.
//!
//! User lock 2026-05-19: a polished logout dialog matching the
//! `Carbon` / `PatternFly` design language. Three primary actions
//! (Sign Out / Restart / Shut Down), Carbon glyphs, dim full-screen
//! backdrop, Escape-to-cancel. The dialog itself is centered with
//! `Gtk.WindowPosition::Center` + i3 floats it via the existing
//! `Mackes-shell` `WM_CLASS` rule (1.0.8 hotfix).
//!
//! Sign Out delegates to `xfce4-session-logout --logout --fast`
//! (skips the upstream confirm prompt — our dialog IS the prompt).
//! Restart / Shut Down route through `loginctl` per the rest of the
//! Mackes power-action surface.

use std::process::Command;

use gtk::prelude::*;

#[derive(Clone, Copy, Debug)]
enum Action {
    SignOut,
    Restart,
    ShutDown,
}

/// Spawn the logout dialog in its own GTK toplevel. Modal-ish via
/// `set_keep_above` + `set_skip_taskbar_hint` — i3's `for_window
/// [class="^Mackes-shell$"] floating enable` rule centers it.
#[allow(clippy::too_many_lines)]
pub fn open() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_widget_name("mackes-logout-dialog");
    window.set_title("Sign out of Mackes Shell");
    // 1.0.8 i3 config has `for_window [window_type="dialog"]
    // floating enable`, so the Dialog type-hint below makes i3
    // float + center this window. No `set_wmclass` needed (the
    // method was removed in gtk-rs in favor of application IDs).
    window.set_default_size(560, 280);
    window.set_position(gtk::WindowPosition::Center);
    window.set_keep_above(true);
    window.set_skip_taskbar_hint(true);
    window.set_resizable(false);
    window.set_decorated(false);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_widget_name("mackes-logout-outer");

    // Header
    let header = gtk::Box::new(gtk::Orientation::Vertical, 6);
    header.set_widget_name("mackes-logout-header");
    header.set_margin_top(24);
    header.set_margin_bottom(8);
    header.set_margin_start(28);
    header.set_margin_end(28);
    let title = gtk::Label::new(Some("Sign out of Mackes Shell"));
    title.set_halign(gtk::Align::Start);
    title.style_context().add_class("mackes-logout-title");
    let subtitle = gtk::Label::new(Some(
        "Pick how you'd like to end this session. Unsaved work in open apps may be lost."
    ));
    subtitle.set_halign(gtk::Align::Start);
    subtitle.set_xalign(0.0);
    subtitle.set_line_wrap(true);
    subtitle.style_context().add_class("mackes-logout-subtitle");
    header.pack_start(&title, false, false, 0);
    header.pack_start(&subtitle, false, false, 0);
    outer.pack_start(&header, false, false, 0);

    // Action row — three large buttons, equal width.
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    actions.set_widget_name("mackes-logout-actions");
    actions.set_homogeneous(true);
    actions.set_margin_top(16);
    actions.set_margin_bottom(16);
    actions.set_margin_start(28);
    actions.set_margin_end(28);

    actions.pack_start(
        &build_action_card(
            &window,
            "system-log-out-symbolic",
            "↩",
            "Sign Out",
            "Return to the login screen.",
            Action::SignOut,
        ),
        true,
        true,
        0,
    );
    actions.pack_start(
        &build_action_card(
            &window,
            "view-refresh-symbolic",
            "↻",
            "Restart",
            "Reboot the workstation.",
            Action::Restart,
        ),
        true,
        true,
        0,
    );
    actions.pack_start(
        &build_action_card(
            &window,
            "system-shutdown-symbolic",
            "⏻",
            "Shut Down",
            "Power off completely.",
            Action::ShutDown,
        ),
        true,
        true,
        0,
    );
    outer.pack_start(&actions, true, true, 0);

    // Footer — Cancel button.
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    footer.set_widget_name("mackes-logout-footer");
    footer.set_margin_bottom(20);
    footer.set_margin_end(28);
    let cancel = gtk::Button::with_label("Cancel");
    cancel.set_widget_name("mackes-logout-cancel");
    cancel.style_context().add_class("mackes-logout-secondary");
    if let Some(atk) = cancel.accessible() {
        atk.set_name("Cancel and close the logout dialog");
    }
    let window_for_cancel = window.clone();
    cancel.connect_clicked(move |_| {
        window_for_cancel.close();
    });
    footer.pack_end(&cancel, false, false, 0);
    outer.pack_start(&footer, false, false, 0);

    window.add(&outer);

    // Escape closes the dialog.
    let window_for_key = window.clone();
    window.connect_key_press_event(move |_, ev| {
        if ev.keyval() == gtk::gdk::keys::constants::Escape {
            window_for_key.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    window.show_all();
}

fn build_action_card(
    window: &gtk::Window,
    icon_name: &str,
    fallback_glyph: &str,
    title: &str,
    sub: &str,
    action: Action,
) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name(&format!(
        "mackes-logout-card-{}",
        title.to_ascii_lowercase().replace(' ', "-")
    ));
    button.style_context().add_class("mackes-logout-card");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_tooltip_text(Some(&format!("{title} — {sub}")));
    if let Some(atk) = button.accessible() {
        atk.set_name(&format!("{title} session action: {sub}"));
    }

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_margin_top(20);
    column.set_margin_bottom(20);
    column.set_margin_start(12);
    column.set_margin_end(12);

    if let Some(pb) = crate::icons::load(icon_name, 32) {
        let img = gtk::Image::from_pixbuf(Some(&pb));
        img.set_halign(gtk::Align::Center);
        column.pack_start(&img, false, false, 0);
    } else {
        let glyph = gtk::Label::new(Some(fallback_glyph));
        glyph.style_context().add_class("mackes-logout-card-glyph");
        column.pack_start(&glyph, false, false, 0);
    }

    let label = gtk::Label::new(Some(title));
    label.set_halign(gtk::Align::Center);
    label.style_context().add_class("mackes-logout-card-title");
    column.pack_start(&label, false, false, 0);

    let sublabel = gtk::Label::new(Some(sub));
    sublabel.set_halign(gtk::Align::Center);
    sublabel.set_justify(gtk::Justification::Center);
    sublabel.set_line_wrap(true);
    sublabel.style_context().add_class("mackes-logout-card-sub");
    column.pack_start(&sublabel, false, false, 0);

    button.add(&column);

    let window_for_handler = window.clone();
    button.connect_clicked(move |_| {
        execute_action(action);
        window_for_handler.close();
    });
    button
}

fn execute_action(action: Action) {
    match action {
        Action::SignOut => {
            // `--fast` skips the upstream xfce4-session-logout
            // confirm prompt — ours WAS the prompt.
            let _ = Command::new("xfce4-session-logout")
                .args(["--logout", "--fast"])
                .spawn();
        }
        Action::Restart => {
            let _ = Command::new("loginctl").arg("reboot").spawn();
        }
        Action::ShutDown => {
            let _ = Command::new("loginctl").arg("poweroff").spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_actions_exist() {
        // Compile-time guard — the enum variants we ship.
        let _ = Action::SignOut;
        let _ = Action::Restart;
        let _ = Action::ShutDown;
    }

    #[test]
    fn action_debug_strings_are_unique() {
        let labels: Vec<String> = vec![
            format!("{:?}", Action::SignOut),
            format!("{:?}", Action::Restart),
            format!("{:?}", Action::ShutDown),
        ];
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), labels.len());
    }
}
