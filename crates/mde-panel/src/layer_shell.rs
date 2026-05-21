//! Phase E.2 — wlr-layer-shell-v1 anchor + strut configuration.
//!
//! The actual SCTK integration is gated behind the `wayland`
//! feature; this module ships the configuration data model +
//! pure-fn helpers that an eventual `iced_layershell` (or
//! direct-SCTK) integration consumes.
//!
//! The 1.1.0 Win10 layout lock dictates:
//! - bottom-edge anchor
//! - 40px height
//! - exclusive-zone enabled (other surfaces resize around it)
//! - Layer::Top (above normal windows, below overlay popups)
//! - Keyboard interactivity: OnDemand (popovers can grab keys)

use crate::top_bar::TOP_BAR_HEIGHT_PX;

/// Edge the panel anchors to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Z-level the panel sits at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// Keyboard-interactivity policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}

/// Configured anchor for the panel layer-shell surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorConfig {
    pub edge: Edge,
    pub layer: Layer,
    pub height_px: u16,
    pub exclusive_zone: bool,
    pub keyboard: KeyboardInteractivity,
    pub namespace: String,
}

impl Default for AnchorConfig {
    fn default() -> Self {
        Self::bottom_panel()
    }
}

impl AnchorConfig {
    /// Locked configuration for the main bottom-edge panel.
    #[must_use]
    pub fn bottom_panel() -> Self {
        Self {
            edge: Edge::Bottom,
            layer: Layer::Top,
            height_px: TOP_BAR_HEIGHT_PX,
            exclusive_zone: true,
            keyboard: KeyboardInteractivity::OnDemand,
            namespace: "mde-panel".into(),
        }
    }

    /// Watermark surface — sits BELOW normal windows (background)
    /// so it doesn't steal clicks from the wallpaper.
    #[must_use]
    pub fn watermark() -> Self {
        Self {
            edge: Edge::Bottom,
            layer: Layer::Background,
            height_px: 24,
            exclusive_zone: false,
            keyboard: KeyboardInteractivity::None,
            namespace: "mde-watermark".into(),
        }
    }

    /// Drawer surface — anchored to the right edge, full height.
    #[must_use]
    pub fn drawer() -> Self {
        Self {
            edge: Edge::Right,
            layer: Layer::Top,
            height_px: 0, // height ignored when anchored to a vertical edge
            exclusive_zone: false,
            keyboard: KeyboardInteractivity::OnDemand,
            namespace: "mde-drawer".into(),
        }
    }
}

/// Strut calculation — how many pixels of screen real estate the
/// panel claims. Other surfaces resize around this when
/// `exclusive_zone` is enabled.
#[must_use]
pub fn exclusive_zone_px(cfg: &AnchorConfig) -> i32 {
    if cfg.exclusive_zone {
        i32::from(cfg.height_px)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_panel_anchors_bottom_at_locked_height() {
        let cfg = AnchorConfig::bottom_panel();
        assert_eq!(cfg.edge, Edge::Bottom);
        assert_eq!(cfg.layer, Layer::Top);
        assert_eq!(cfg.height_px, TOP_BAR_HEIGHT_PX);
        assert!(cfg.exclusive_zone);
        assert_eq!(cfg.keyboard, KeyboardInteractivity::OnDemand);
        assert_eq!(cfg.namespace, "mde-panel");
    }

    #[test]
    fn watermark_sits_below_at_background_layer() {
        let cfg = AnchorConfig::watermark();
        assert_eq!(cfg.layer, Layer::Background);
        assert!(!cfg.exclusive_zone);
        assert_eq!(cfg.keyboard, KeyboardInteractivity::None);
    }

    #[test]
    fn drawer_anchors_right_edge() {
        let cfg = AnchorConfig::drawer();
        assert_eq!(cfg.edge, Edge::Right);
        assert_eq!(cfg.layer, Layer::Top);
        assert!(!cfg.exclusive_zone);
    }

    #[test]
    fn exclusive_zone_returns_height_when_enabled() {
        let cfg = AnchorConfig::bottom_panel();
        assert_eq!(exclusive_zone_px(&cfg), i32::from(TOP_BAR_HEIGHT_PX));
    }

    #[test]
    fn exclusive_zone_returns_zero_when_disabled() {
        let cfg = AnchorConfig::watermark();
        assert_eq!(exclusive_zone_px(&cfg), 0);
    }

    #[test]
    fn default_anchor_is_the_bottom_panel() {
        let cfg = AnchorConfig::default();
        assert_eq!(cfg.namespace, "mde-panel");
    }

    #[test]
    fn namespaces_are_distinct_across_surfaces() {
        let ns: std::collections::HashSet<_> = [
            AnchorConfig::bottom_panel().namespace,
            AnchorConfig::watermark().namespace,
            AnchorConfig::drawer().namespace,
        ]
        .into_iter()
        .collect();
        assert_eq!(ns.len(), 3);
    }
}
