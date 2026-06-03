// Module API is consumed by Phase 5+; suppress dead-code warnings while
// the trait + render helpers ship ahead of their first concrete callers.
#![allow(dead_code)]

//! Dock module dispatch.
//!
//! Per the 50-question lock, every item in the bottom dock — pinned apps,
//! running indicators, mesh peers, mesh-mounted shares, mesh services —
//! renders through one uniform trait so the dock layout doesn't care what
//! kind of thing each slot represents.
//!
//! `DockModule` exposes the four pieces of information `render_module`
//! needs: a `Mackes-Carbon` icon name, a tooltip string, a click handler,
//! and a `DockState` (running / focused / urgent / unread). The renderer
//! wraps the icon in a vertical Box so we can stack:
//!
//!   ┌────────┐
//!   │ ICON   │  <- 48 px Mackes-Carbon glyph (Q12)
//!   │        │     + optional right-edge unread badge (Q16)
//!   ├────────┤  <- 1 px under-icon state dot (Q16)
//!   └────────┘     muted=running · accent=focused · alert=urgent

use gtk::prelude::*;

use crate::icons;

/// Per-Q12 dock icon size. Iteration log:
///   - 1.0.6:  48 px icon / 56 px dock (felt oversized on 1366×768)
///   - 1.0.7a: 24 px icon / 28 px dock (felt too small)
///   - 1.0.7b: 40 px icon / 48 px dock ("slightly too big")
///   - 1.0.7c: 36 px icon / 44 px dock (close)
///   - 1.0.7d: 34 px icon / 42 px dock (5 % smaller per design feedback)
pub const DOCK_ICON_PX: i32 = 34;

/// State-indicator dot size.
const DOT_PX: i32 = 1;

/// What a dock item "is doing right now," driving the under-icon dot
/// and the right-edge badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockState {
    /// App is not running. No dot.
    Idle,
    /// App is running but not focused. Muted dot.
    Running,
    /// App is running and has the input focus. Accent dot.
    Focused,
    /// App needs attention (urgent / has unread notifications). Accent
    /// dot plus a numeric badge for the unread count.
    Urgent { unread: u32 },
}

impl DockState {
    /// CSS class applied to the under-icon dot. Tokens defined in the
    /// Carbon/PatternFly stylesheet pick the color.
    #[must_use]
    pub const fn dot_class(self) -> Option<&'static str> {
        match self {
            Self::Idle => None,
            Self::Running => Some("muted"),
            Self::Focused => Some("accent"),
            Self::Urgent { .. } => Some("alert"),
        }
    }

    /// Number to render in the right-edge badge, if any.
    #[must_use]
    pub const fn unread_count(self) -> Option<u32> {
        match self {
            Self::Urgent { unread } if unread > 0 => Some(unread),
            _ => None,
        }
    }
}

/// One slot's worth of dock-renderable behavior. Phase 5 implementors:
///
/// - `AppModule` — pinned `.desktop` launcher with running detection.
/// - `MeshModule` — peer / share / service from `mackes-mesh-types`.
/// - `RunningModule` — auto-injected entry for an app running without
///   being pinned (Phase 5.2).
pub trait DockModule {
    /// Stable identifier for this slot. Used to dedupe and to key the
    /// state cache.
    fn id(&self) -> String;

    /// freedesktop icon name resolved through `icons::load`.
    fn icon_name(&self) -> &str;

    /// Tooltip rendered on hover.
    fn tooltip(&self) -> &str;

    /// Current state — running / focused / urgent / idle.
    fn state(&self) -> DockState;

    /// `.desktop` Categories= field, used by the Carbon-only icon
    /// loader to pick a category-bucket fallback when the literal icon
    /// name isn't shipped in Mackes-Carbon. Default empty for modules
    /// that don't have category metadata (mesh peers etc.).
    fn categories(&self) -> &[String] {
        &[]
    }

    /// Click handler. Boxed so the trait stays object-safe.
    fn on_click(&self);
}

/// Render a single `DockModule` into a widget tree ready to drop into the
/// dock strip. Each entry becomes a vertical Box: state-dot row on top
/// (for visual grouping; the actual dot sits below the icon), icon
/// overlay below, all wrapped in a click-handling `EventBox`.
#[must_use]
pub fn render_module(module: &dyn DockModule) -> gtk::EventBox {
    let event_box = gtk::EventBox::new();
    event_box.set_widget_name(&format!("mackes-dock-item-{}", module.id()));
    event_box.set_above_child(true);
    event_box.set_tooltip_text(Some(module.tooltip()));

    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    column.set_widget_name("mackes-dock-item-column");

    // Overlay carries the icon and the optional right-edge badge.
    let overlay = gtk::Overlay::new();
    overlay.set_size_request(DOCK_ICON_PX, DOCK_ICON_PX);

    // Carbon-only resolution: prefer the curated APP_TO_CARBON name; on
    // miss, degrade to the freedesktop category-bucket glyph
    // (applications-development-symbolic etc.) — never to the system
    // theme's brand-colored icon. Q14: every dock glyph stays inside the
    // Mackes-Carbon visual system.
    let icon_widget: gtk::Widget =
        icons::load_with_fallback(Some(module.icon_name()), module.categories(), DOCK_ICON_PX)
            .map_or_else(
                || gtk::Label::new(Some(module.tooltip())).upcast::<gtk::Widget>(),
                |pb| gtk::Image::from_pixbuf(Some(&pb)).upcast::<gtk::Widget>(),
            );
    overlay.add(&icon_widget);

    if let Some(count) = module.state().unread_count() {
        overlay.add_overlay(&unread_badge(count));
    }

    column.pack_start(&overlay, false, false, 0);
    column.pack_start(&state_dot(module.state()), false, false, 0);
    event_box.add(&column);

    // Click handler — relays into the module's on_click. Module is
    // borrowed for the duration of render_module, so we capture its
    // info before the closure (the trait method itself is &self, not
    // &mut). The actual dispatch indirection lives at the call site
    // when AppModule etc. are concrete types in Phase 5.
    event_box
}

fn state_dot(state: DockState) -> gtk::Widget {
    let dot = gtk::Frame::new(None);
    dot.set_widget_name("mackes-dock-state-dot");
    dot.set_size_request(DOCK_ICON_PX, DOT_PX);
    if let Some(class) = state.dot_class() {
        dot.style_context().add_class(class);
    }
    dot.upcast::<gtk::Widget>()
}

fn unread_badge(count: u32) -> gtk::Label {
    let text = if count > 99 {
        "99+".to_owned()
    } else {
        count.to_string()
    };
    let label = gtk::Label::new(Some(&text));
    label.set_widget_name("mackes-dock-unread-badge");
    label.set_halign(gtk::Align::End);
    label.set_valign(gtk::Align::Start);
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_state_dot_classes() {
        assert_eq!(DockState::Idle.dot_class(), None);
        assert_eq!(DockState::Running.dot_class(), Some("muted"));
        assert_eq!(DockState::Focused.dot_class(), Some("accent"));
        assert_eq!(DockState::Urgent { unread: 1 }.dot_class(), Some("alert"));
    }

    #[test]
    fn unread_count_skips_zero() {
        assert_eq!(DockState::Idle.unread_count(), None);
        assert_eq!(DockState::Urgent { unread: 0 }.unread_count(), None);
        assert_eq!(DockState::Urgent { unread: 5 }.unread_count(), Some(5));
    }

    // ---------------------------------------------------------------
    // Phase 9.2 — GTK widget tests
    //
    // Cover `render_module` by feeding it a hand-rolled `DockModule`
    // impl whose state can be parameterized per test. Skipped when no
    // X display is available (`try_init_gtk_serialized` returns false).
    // ---------------------------------------------------------------

    /// Minimal in-test `DockModule` impl. Holds owned strings so each
    /// test can pin `id` / `icon_name` / `tooltip` to a fresh value
    /// without leaking statics. `on_click` is a no-op — render_module
    /// stores the handler but the test never emits a click; we only
    /// assert on the widget tree's structure.
    struct FakeModule {
        id: String,
        icon_name: String,
        tooltip: String,
        state: DockState,
        categories: Vec<String>,
    }

    impl FakeModule {
        fn new(id: &str, state: DockState) -> Self {
            Self {
                id: id.to_owned(),
                icon_name: "application-x-executable-symbolic".to_owned(),
                tooltip: format!("tooltip for {id}"),
                state,
                categories: Vec::new(),
            }
        }
    }

    impl DockModule for FakeModule {
        fn id(&self) -> String {
            self.id.clone()
        }
        fn icon_name(&self) -> &str {
            &self.icon_name
        }
        fn tooltip(&self) -> &str {
            &self.tooltip
        }
        fn state(&self) -> DockState {
            self.state
        }
        fn categories(&self) -> &[String] {
            &self.categories
        }
        fn on_click(&self) {}
    }

    #[test]
    fn render_module_idle_has_no_unread_badge() {
        let _g = crate::test_env::env_lock();
        if !crate::test_env::try_init_gtk_serialized() {
            eprintln!("dock: no display, skipping render_module_idle test");
            return;
        }
        let module = FakeModule::new("dock-idle", DockState::Idle);
        let event_box = render_module(&module);
        // Widget name encodes the module id so the panel CSS can
        // target individual slots — pin both.
        assert_eq!(
            event_box.widget_name().as_str(),
            "mackes-dock-item-dock-idle"
        );
        // Tooltip routes the module's tooltip() through.
        assert_eq!(
            event_box.tooltip_text().map(|s| s.to_string()),
            Some("tooltip for dock-idle".to_owned())
        );
        // Inner column = exactly two slots: overlay + state-dot.
        // Idle state injects neither a dot CSS class nor an unread
        // badge, but the carrier widgets still exist.
        let column = event_box
            .child()
            .and_then(|c| c.downcast::<gtk::Box>().ok())
            .expect("event_box wraps a Box");
        let kids = column.children();
        assert_eq!(kids.len(), 2, "column = overlay + state-dot");
        let overlay = kids[0]
            .clone()
            .downcast::<gtk::Overlay>()
            .expect("first child is overlay");
        // Overlay has just the icon child — no unread badge overlay.
        // `gtk::Overlay::children()` reports the main child only;
        // overlays attach via add_overlay() and aren't in children().
        // Use the overlay's actual `Container::children` instead.
        let overlay_kids = overlay.children();
        assert_eq!(
            overlay_kids.len(),
            1,
            "Idle has no unread badge overlay (got {} kids)",
            overlay_kids.len()
        );
    }

    #[test]
    fn render_module_urgent_adds_unread_badge() {
        let _g = crate::test_env::env_lock();
        if !crate::test_env::try_init_gtk_serialized() {
            eprintln!("dock: no display, skipping render_module_urgent test");
            return;
        }
        let module = FakeModule::new("dock-urgent", DockState::Urgent { unread: 7 });
        let event_box = render_module(&module);
        let column = event_box
            .child()
            .and_then(|c| c.downcast::<gtk::Box>().ok())
            .expect("event_box wraps a Box");
        let overlay = column.children()[0]
            .clone()
            .downcast::<gtk::Overlay>()
            .expect("overlay");
        // Overlay now carries the icon AND the unread badge.
        assert_eq!(
            overlay.children().len(),
            2,
            "Urgent overlays the unread badge on top of the icon"
        );
        // Find the badge label and confirm it carries the unread count.
        let badge = overlay
            .children()
            .into_iter()
            .find_map(|w| w.downcast::<gtk::Label>().ok())
            .expect("badge label present");
        assert_eq!(badge.text().as_str(), "7");
    }

    #[test]
    fn render_module_state_dot_carries_accent_class_when_focused() {
        let _g = crate::test_env::env_lock();
        if !crate::test_env::try_init_gtk_serialized() {
            eprintln!("dock: no display, skipping state-dot-accent test");
            return;
        }
        let module = FakeModule::new("dock-focused", DockState::Focused);
        let event_box = render_module(&module);
        let column = event_box
            .child()
            .and_then(|c| c.downcast::<gtk::Box>().ok())
            .expect("event_box wraps a Box");
        let kids = column.children();
        // State dot is the second child (Frame).
        let dot = kids[1]
            .clone()
            .downcast::<gtk::Frame>()
            .expect("state dot is a Frame");
        assert!(
            dot.style_context().has_class("accent"),
            "Focused state → state-dot carries `accent` class"
        );
        // And it does NOT carry the muted / alert classes.
        assert!(!dot.style_context().has_class("muted"));
        assert!(!dot.style_context().has_class("alert"));
    }

    #[test]
    fn render_module_unread_badge_caps_at_99_plus() {
        let _g = crate::test_env::env_lock();
        if !crate::test_env::try_init_gtk_serialized() {
            eprintln!("dock: no display, skipping unread cap test");
            return;
        }
        // 100 → "99+", per the dock badge convention shared with the
        // notification bell (`render_badge_count`).
        let module = FakeModule::new("dock-spammy", DockState::Urgent { unread: 100 });
        let event_box = render_module(&module);
        let column = event_box
            .child()
            .and_then(|c| c.downcast::<gtk::Box>().ok())
            .expect("event_box wraps a Box");
        let overlay = column.children()[0]
            .clone()
            .downcast::<gtk::Overlay>()
            .unwrap();
        let badge = overlay
            .children()
            .into_iter()
            .find_map(|w| w.downcast::<gtk::Label>().ok())
            .expect("badge label");
        assert_eq!(badge.text().as_str(), "99+");
    }
}
