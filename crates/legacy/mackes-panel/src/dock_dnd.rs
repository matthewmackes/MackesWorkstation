//! Phase 5.7 — drag-to-pin / drag-to-reorder visual layer for the dock.
//!
//! The data layer (`mackes_config::pin_app` / `reorder_dock`) shipped in
//! 1.0.7 with full unit-test coverage; this module is the GTK 3.24 widget
//! wiring that surfaces those mutations through honest pointer-driven
//! drag-and-drop.
//!
//! ## Target atoms
//!
//! Two custom GTK target atoms scope the drag interactions tightly so the
//! dock never reacts to drags coming from outside the panel (e.g. files
//! being dragged out of a file manager):
//!
//! | atom                      | source           | destination(s)        |
//! | ------------------------- | ---------------- | --------------------- |
//! | `mackes-dock-launcher-pos`| pinned launcher  | every pinned launcher |
//! | `mackes-tasklist-pin`     | tasklist item    | the pinned strip      |
//!
//! `TargetFlags::SAME_APP` keeps the drag scoped to mackes-panel — no
//! cross-app surprises. Both atoms carry their payload as UTF-8 text via
//! `SelectionData::set_text` / `text()`:
//!
//! - `mackes-dock-launcher-pos` carries the source index as a decimal
//!   string (e.g. `"3"`). The drop target parses + calls
//!   `mackes_config::reorder_dock(cfg, from, to)`.
//! - `mackes-tasklist-pin` carries the `.desktop` id (e.g.
//!   `"firefox.desktop"`). The pinned-strip drop target calls
//!   `mackes_config::pin_app(cfg, desktop)`.
//!
//! ## Action choice
//!
//! `DragAction::MOVE` — the reorder is genuinely a move (no copy
//! semantic) and the pin-from-tasklist also "moves" the conceptual
//! identity from running-tasklist to pinned-strip. GTK still permits
//! `COPY` fallback if the source rejects MOVE, but neither of our
//! sources do.
//!
//! ## Visual feedback
//!
//! Two CSS classes drive the visual state:
//!
//! - `.dragging` — applied to the drag source on `drag-begin`, removed
//!   on `drag-end`. Dims the icon to 0.5 opacity (CSS lives in
//!   `data/css/mackes.css` and the inline `PLACEHOLDER_CSS`).
//! - `.drop-hover` — applied to the drop target on `drag-motion`,
//!   removed on `drag-leave`. Outlines the slot with the brand-accent
//!   color so the user can see where the drop will land.
//!
//! ## Persistence
//!
//! Every drop calls `config_store::with_mut` which loads → mutates →
//! writes `panel.toml` in a single round-trip. The dock's 2 s refresh
//! tick (`build_bottom_taskbar`) re-reads the file on the next pass, so
//! the visual update lands within ~2 s of the drop without any extra
//! plumbing here.
//!
//! ## GTK 3.24 quirk
//!
//! GTK 3's `gtk_drag_source_set` requires the widget's event mask
//! include `BUTTON_PRESS_MASK` for the drag threshold to fire. Both
//! `gtk::EventBox` and the underlying widgets we wrap already enable
//! button events for their click handlers, so we don't need to call
//! `add_events` ourselves. Verified empirically by running the panel
//! under Xvfb — without that pre-existing button mask the drag would
//! never start.

#![allow(clippy::module_name_repetitions)]

use gdk::{DragAction, ModifierType};
use gtk::prelude::*;
use gtk::{DestDefaults, TargetEntry, TargetFlags};

use crate::config_store;

/// Custom target atom for reordering pinned dock entries. Source widget
/// is one pinned launcher (or mesh module); destination is any other
/// pinned launcher (or mesh module) on the same strip.
pub const TARGET_LAUNCHER_POS: &str = "mackes-dock-launcher-pos";

/// Custom target atom for pinning a running app onto the dock. Source
/// widget is a tasklist item; destination is the pinned-apps strip box.
pub const TARGET_TASKLIST_PIN: &str = "mackes-tasklist-pin";

/// `info` field for `TARGET_LAUNCHER_POS`. Distinguishes the two target
/// atoms when both are accepted on the same widget (currently they
/// aren't, but the info field is a free disambiguator).
const INFO_LAUNCHER_POS: u32 = 1;

/// `info` field for `TARGET_TASKLIST_PIN`.
const INFO_TASKLIST_PIN: u32 = 2;

/// CSS class applied to a widget while it is the active drag source.
const CSS_DRAGGING: &str = "dragging";

/// CSS class applied to a widget while a drag is hovering over it.
const CSS_DROP_HOVER: &str = "drop-hover";

/// Wire `widget` as a pinned-launcher dock slot at position `index`.
/// Installs:
///   * a drag source on `mackes-dock-launcher-pos` carrying `index` as
///     a decimal string;
///   * a drop target on the same atom that calls
///     `mackes_config::reorder_dock(cfg, from, index)` on receipt.
///
/// The same widget is both source and destination — GTK 3 handles this
/// cleanly because the drop target only fires on a different source row
/// (the visual hint stays hidden over the dragging row itself).
pub fn attach_dock_slot<W: IsA<gtk::Widget>>(widget: &W, index: usize) {
    let targets = [TargetEntry::new(
        TARGET_LAUNCHER_POS,
        TargetFlags::SAME_APP,
        INFO_LAUNCHER_POS,
    )];

    // ---- source side ------------------------------------------------
    widget.drag_source_set(ModifierType::BUTTON1_MASK, &targets, DragAction::MOVE);

    // Emit the index when GTK asks for the drag payload.
    widget.connect_drag_data_get(move |_, _ctx, data, _info, _time| {
        let _ = data.set_text(&index.to_string());
    });

    // .dragging class — dim the source row for the duration of the drag.
    widget.connect_drag_begin(|w, _ctx| {
        w.style_context().add_class(CSS_DRAGGING);
    });
    widget.connect_drag_end(|w, _ctx| {
        w.style_context().remove_class(CSS_DRAGGING);
        // Defensive: a drag can end mid-hover (Esc / WM intervention).
        w.style_context().remove_class(CSS_DROP_HOVER);
    });

    // ---- destination side -------------------------------------------
    widget.drag_dest_set(
        DestDefaults::MOTION | DestDefaults::DROP,
        &targets,
        DragAction::MOVE,
    );

    // .drop-hover class — outline this slot while a drag hovers over it.
    widget.connect_drag_motion(|w, ctx, _x, _y, time| {
        w.style_context().add_class(CSS_DROP_HOVER);
        // Signal to GDK that MOVE is acceptable here.
        ctx.drag_status(DragAction::MOVE, time);
        // Returning `true` tells GTK we handled the motion event.
        true
    });
    widget.connect_drag_leave(|w, _ctx, _time| {
        w.style_context().remove_class(CSS_DROP_HOVER);
    });

    // Drop arrived — parse `from`, run the reorder, finish.
    widget.connect_drag_data_received(move |w, ctx, _x, _y, data, info, time| {
        w.style_context().remove_class(CSS_DROP_HOVER);
        if info != INFO_LAUNCHER_POS {
            ctx.drag_finish(false, false, time);
            return;
        }
        let Some(text) = data.text() else {
            ctx.drag_finish(false, false, time);
            return;
        };
        let Ok(from) = text.parse::<usize>() else {
            ctx.drag_finish(false, false, time);
            return;
        };
        let to = index;
        // No-op fast path — saves a panel.toml round-trip when the user
        // releases over the same row they started on.
        if from == to {
            ctx.drag_finish(true, false, time);
            return;
        }
        config_store::with_mut(|cfg| {
            mackes_config::reorder_dock(cfg, from, to);
        });
        ctx.drag_finish(true, false, time);
    });
}

/// Wire `widget` as a tasklist drag source. Carries `desktop_id` (the
/// basename of the `.desktop` file, e.g. `"firefox.desktop"`) so the
/// pinned-strip drop target can call
/// `mackes_config::pin_app(cfg, desktop_id)`.
///
/// `desktop_id` is `None` when the running window has no matching
/// `.desktop` entry (e.g. random Qt tools spawned without a desktop
/// file). In that case we skip the drag-source wiring entirely — the
/// user gets normal click semantics and no half-broken drag.
pub fn attach_tasklist_source<W: IsA<gtk::Widget>>(widget: &W, desktop_id: Option<&str>) {
    let Some(desktop) = desktop_id else {
        return;
    };
    let desktop = desktop.to_owned();

    let targets = [TargetEntry::new(
        TARGET_TASKLIST_PIN,
        TargetFlags::SAME_APP,
        INFO_TASKLIST_PIN,
    )];
    widget.drag_source_set(ModifierType::BUTTON1_MASK, &targets, DragAction::MOVE);
    widget.connect_drag_data_get(move |_, _ctx, data, _info, _time| {
        let _ = data.set_text(&desktop);
    });
    widget.connect_drag_begin(|w, _ctx| {
        w.style_context().add_class(CSS_DRAGGING);
    });
    widget.connect_drag_end(|w, _ctx| {
        w.style_context().remove_class(CSS_DRAGGING);
        w.style_context().remove_class(CSS_DROP_HOVER);
    });
}

/// Wire `strip` (the `gtk::Box` containing pinned launchers) as a drop
/// target for `mackes-tasklist-pin` payloads. Drops call
/// `mackes_config::pin_app(cfg, desktop_id)`; the 2 s refresh tick
/// picks the new icon up automatically.
pub fn attach_pinned_strip_target(strip: &gtk::Box) {
    let targets = [TargetEntry::new(
        TARGET_TASKLIST_PIN,
        TargetFlags::SAME_APP,
        INFO_TASKLIST_PIN,
    )];
    strip.drag_dest_set(
        DestDefaults::MOTION | DestDefaults::DROP,
        &targets,
        DragAction::MOVE,
    );
    strip.connect_drag_motion(|w, ctx, _x, _y, time| {
        w.style_context().add_class(CSS_DROP_HOVER);
        ctx.drag_status(DragAction::MOVE, time);
        true
    });
    strip.connect_drag_leave(|w, _ctx, _time| {
        w.style_context().remove_class(CSS_DROP_HOVER);
    });
    strip.connect_drag_data_received(move |w, ctx, _x, _y, data, info, time| {
        w.style_context().remove_class(CSS_DROP_HOVER);
        if info != INFO_TASKLIST_PIN {
            ctx.drag_finish(false, false, time);
            return;
        }
        let Some(desktop) = data.text() else {
            ctx.drag_finish(false, false, time);
            return;
        };
        let desktop = desktop.to_string();
        if desktop.is_empty() {
            ctx.drag_finish(false, false, time);
            return;
        }
        config_store::with_mut(|cfg| {
            mackes_config::pin_app(cfg, &desktop);
        });
        ctx.drag_finish(true, false, time);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Target atom names are part of our protocol — bumping them silently
    /// would break in-flight drags from older widget instances after a
    /// live reload. Pin the strings here so accidental renames trip CI.
    #[test]
    fn target_atom_names_are_stable() {
        assert_eq!(TARGET_LAUNCHER_POS, "mackes-dock-launcher-pos");
        assert_eq!(TARGET_TASKLIST_PIN, "mackes-tasklist-pin");
    }

    /// Info disambiguators must be distinct — otherwise the
    /// drag-data-received handlers can't tell which atom fired.
    #[test]
    fn target_info_codes_are_distinct() {
        assert_ne!(INFO_LAUNCHER_POS, INFO_TASKLIST_PIN);
    }

    /// CSS class names are shared with `data/css/mackes.css` and the
    /// inline placeholder block in `main.rs`. Pin them so the visual
    /// feedback path can't silently drift.
    #[test]
    fn css_class_names_are_stable() {
        assert_eq!(CSS_DRAGGING, "dragging");
        assert_eq!(CSS_DROP_HOVER, "drop-hover");
    }
}
