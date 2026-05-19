//! Notification bell — tray button right of the status cluster,
//! before the clock.
//!
//! Locked 2026-05-19 via the `Rust Desktop.zip` handoff bundle:
//!
//! - Permanent bell + unread badge.
//! - Continuous slow pulse (~1.6 s) while unread > 0 AND modal closed.
//! - Click → opens `crate::notification_center::open()`.
//!
//! Reads unread state from `~/.cache/mackes/notifications.json` on a
//! 2 s poll cadence — same rhythm as the status cluster, cheap, no
//! event subscription needed.

use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::notification_center;

/// Build the tray button. Returned widget is intended to be packed
/// into the taskbar's right slot, immediately before the clock.
#[must_use]
pub fn build() -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name("mackes-notification-bell");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);

    let stack = gtk::Overlay::new();
    let bell_icon = gtk::Image::new();
    if let Some(pb) = crate::icons::load("mail-unread-symbolic", 16) {
        bell_icon.set_from_pixbuf(Some(&pb));
    } else {
        bell_icon.set_from_icon_name(Some("mail-unread-symbolic"), gtk::IconSize::Menu);
    }
    stack.add(&bell_icon);

    let badge = gtk::Label::new(None);
    badge.set_widget_name("mackes-notification-bell-badge");
    badge.style_context().add_class("mackes-notification-bell-badge");
    badge.set_halign(gtk::Align::End);
    badge.set_valign(gtk::Align::Start);
    badge.set_visible(false);
    stack.add_overlay(&badge);

    button.add(&stack);
    button.set_tooltip_text(Some("Notifications"));
    if let Some(atk) = button.accessible() {
        atk.set_name("Notifications");
        atk.set_description("Mesh-synced notification center. Click to open.");
    }

    // Track "modal open" state shared with the click handler so the
    // pulse stops the moment the user opens the center.
    let modal_open = Rc::new(std::cell::Cell::new(false));

    let modal_open_for_click = Rc::clone(&modal_open);
    button.connect_clicked(move |btn| {
        modal_open_for_click.set(true);
        btn.style_context().remove_class("pulsing");
        notification_center::open();
        // The center is modal but non-blocking — we don't know when
        // the user closes it. Clear the flag after a short delay so
        // pulse can resume if the user keeps the modal open + new
        // unreads arrive (~10 s is plenty of UX deadband).
        let modal_open_inner = Rc::clone(&modal_open_for_click);
        glib::timeout_add_local_once(std::time::Duration::from_secs(10), move || {
            modal_open_inner.set(false);
        });
    });

    // 2 s poll — refresh badge + pulse from disk-backed cache.
    let button_for_timer = button.clone();
    let badge_for_timer = badge.clone();
    let modal_for_timer = Rc::clone(&modal_open);
    glib::timeout_add_seconds_local(2, move || {
        let unread = notification_center::unread_count(&notification_center::load());
        apply_state(&button_for_timer, &badge_for_timer, unread, modal_for_timer.get());
        glib::ControlFlow::Continue
    });

    // Initial paint.
    let unread = notification_center::unread_count(&notification_center::load());
    apply_state(&button, &badge, unread, false);

    button
}

fn apply_state(button: &gtk::Button, badge: &gtk::Label, unread: usize, modal_open: bool) {
    let ctx = button.style_context();
    if unread > 0 {
        ctx.add_class("has-unread");
        badge.set_text(&render_badge_count(unread));
        badge.set_visible(true);
        if modal_open {
            ctx.remove_class("pulsing");
        } else {
            ctx.add_class("pulsing");
        }
    } else {
        ctx.remove_class("has-unread");
        ctx.remove_class("pulsing");
        badge.set_visible(false);
    }
}

/// Cap badge label at "99+" per the handoff lock.
#[must_use]
pub fn render_badge_count(unread: usize) -> String {
    if unread > 99 {
        "99+".to_owned()
    } else {
        unread.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_count_capped_at_99_plus() {
        assert_eq!(render_badge_count(0), "0");
        assert_eq!(render_badge_count(1), "1");
        assert_eq!(render_badge_count(99), "99");
        assert_eq!(render_badge_count(100), "99+");
        assert_eq!(render_badge_count(1_000_000), "99+");
    }
}
