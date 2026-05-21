//! Phase E.4.4 — exposé grid (F3).
//!
//! Reads `swaymsg -t get_tree` via `swayipc-async` (when wired
//! at Phase E.3), flattens to `window_type=="normal"` leaves, and
//! renders an overlay grid with one card per window. Click sends
//! `swaymsg [con_id=<N>] focus` and dismisses.
//!
//! For Phase E.4.4 the pure-fn helpers (grid_columns, card_layout,
//! truncate_title) port verbatim from the GTK version's
//! tests — they're trivial geometry math that doesn't depend on
//! the rendering backend.

/// Maximum columns per row (caps at 6 even when there are
/// hundreds of windows).
pub const MAX_COLUMNS: usize = 6;

/// Card aspect ratio (width / height).
pub const CARD_ASPECT: f32 = 16.0 / 9.0;

/// Minimum card width in logical pixels.
pub const MIN_CARD_WIDTH: f32 = 220.0;

/// Compute grid column count for N windows. Capped at
/// [`MAX_COLUMNS`].
#[must_use]
pub fn grid_columns(window_count: usize) -> usize {
    if window_count == 0 {
        return 1;
    }
    let sqrt = (window_count as f64).sqrt().ceil() as usize;
    sqrt.min(MAX_COLUMNS).max(1)
}

/// Per-card width given the available exposé surface and the
/// expected card count.
#[must_use]
pub fn card_layout(surface_width: f32, surface_height: f32, count: usize) -> (f32, f32) {
    let cols = grid_columns(count) as f32;
    let mut card_w = (surface_width / cols).max(MIN_CARD_WIDTH);
    let mut card_h = card_w / CARD_ASPECT;
    // Cap by surface height — fall back to height-driven sizing.
    let rows_needed = (count as f32 / cols).ceil().max(1.0);
    let max_card_h = surface_height / rows_needed;
    if card_h > max_card_h {
        card_h = max_card_h;
        card_w = card_h * CARD_ASPECT;
    }
    (card_w, card_h)
}

/// Truncate a window title with `…` once it exceeds `max` chars.
/// Multi-byte safe.
#[must_use]
pub fn truncate_title(title: &str, max: usize) -> String {
    let count = title.chars().count();
    if count <= max {
        return title.to_string();
    }
    let mut out: String = title.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// One exposé card.
#[derive(Debug, Clone)]
pub struct ExposeCard {
    pub con_id: u64,
    pub title: String,
    pub app_id: String,
}

/// Filter + flatten a list of windows into exposé cards.
/// `window_type == "normal"` only.
#[must_use]
pub fn cards_from_windows<I>(windows: I) -> Vec<ExposeCard>
where
    I: IntoIterator<Item = SwayWindow>,
{
    windows
        .into_iter()
        .filter(|w| w.window_type == "normal")
        .map(|w| ExposeCard {
            con_id: w.con_id,
            title: w.title,
            app_id: w.app_id,
        })
        .collect()
}

/// Swayipc-shaped window record. Defined here so the cards
/// helper is testable without a live sway connection.
#[derive(Debug, Clone)]
pub struct SwayWindow {
    pub con_id: u64,
    pub title: String,
    pub app_id: String,
    /// Sway's `window_type` enum — "normal" / "dialog" / "scratchpad" etc.
    pub window_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_columns_minimum_one() {
        assert_eq!(grid_columns(0), 1);
        assert_eq!(grid_columns(1), 1);
    }

    #[test]
    fn grid_columns_caps_at_six() {
        assert_eq!(grid_columns(100), 6);
        assert_eq!(grid_columns(36), 6);
    }

    #[test]
    fn grid_columns_uses_ceil_sqrt() {
        assert_eq!(grid_columns(2), 2);
        assert_eq!(grid_columns(4), 2);
        assert_eq!(grid_columns(5), 3);
        assert_eq!(grid_columns(9), 3);
    }

    #[test]
    fn card_layout_respects_min_width() {
        let (w, _) = card_layout(800.0, 600.0, 10);
        assert!(w >= MIN_CARD_WIDTH);
    }

    #[test]
    fn card_layout_caps_by_surface_height() {
        let (_, h) = card_layout(2000.0, 200.0, 4);
        assert!(h <= 200.0);
    }

    #[test]
    fn card_layout_aspect_preserved_when_height_caps() {
        let (w, h) = card_layout(2000.0, 200.0, 4);
        let ratio = w / h;
        assert!((ratio - CARD_ASPECT).abs() < 0.01);
    }

    #[test]
    fn truncate_title_passes_short_strings() {
        assert_eq!(truncate_title("hi", 10), "hi");
    }

    #[test]
    fn truncate_title_ellipsizes_long_strings() {
        let out = truncate_title("0123456789abcdef", 8);
        assert_eq!(out.chars().count(), 8);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn cards_from_windows_filters_non_normal() {
        let windows = vec![
            SwayWindow {
                con_id: 1,
                title: "A".into(),
                app_id: "a".into(),
                window_type: "normal".into(),
            },
            SwayWindow {
                con_id: 2,
                title: "B".into(),
                app_id: "b".into(),
                window_type: "dialog".into(),
            },
            SwayWindow {
                con_id: 3,
                title: "C".into(),
                app_id: "c".into(),
                window_type: "scratchpad".into(),
            },
        ];
        let cards = cards_from_windows(windows);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].con_id, 1);
    }

    #[test]
    fn card_layout_one_window_fills_surface() {
        let (w, h) = card_layout(1920.0, 1080.0, 1);
        // 1 window, 1 column → card_w = surface_w
        // but capped by height-aspect math
        assert!(w > 0.0);
        assert!(h > 0.0);
    }
}
