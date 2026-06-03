//! mde-drawer — Iced quick-actions overlay.
//!
//! Phase E.8.1 + E.8.2: side-drawer that slides in from the
//! right edge and hosts:
//!
//! - **Quick Actions** (E.8.2 lock): DND / caffeine toggles
//!   backed by the flag-file path (Phase C.4 / C.5).
//! - **Brightness + Volume** sliders (E.6.1 / E.6.2 sliders
//!   crate, consumed from mde-panel as a path-dep).
//! - **Notifications list** (inline variant of the standalone
//!   notification-center applet — reads the same JSON cache).
//! - **Battery + Hardware** chip (upower over zbus).
//!
//! 2026 design language:
//! - 360px wide, full-height layer-shell surface anchored
//!   `Right + Top + Bottom`.
//! - Translucent glass background with 18px corner radius on the
//!   left edge only.
//! - 280ms slide-in tween (Cubic ease-out).
//! - Dismissable via Esc, click-outside, or the `mde-panel
//!   --drawer` re-invocation.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Width of the drawer surface in logical pixels (Phase 1.1.0
/// secondary-overlay lock).
pub const DRAWER_WIDTH_PX: u16 = 360;

/// Slide-in tween duration.
pub const SLIDE_DURATION_MS: u64 = 280;

// ──────────────────────────────────────────────────────────────
// Section model
// ──────────────────────────────────────────────────────────────

/// One drawer section. Each section is independently expandable
/// + retains its own state via cosmic-config (Phase C.7-style
/// sidecars under `$XDG_CONFIG_HOME/mde/drawer.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawerSection {
    QuickActions,
    Sliders,
    Notifications,
    Hardware,
}

impl DrawerSection {
    /// Display label for the section header.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            DrawerSection::QuickActions => "Quick actions",
            DrawerSection::Sliders => "Brightness & volume",
            DrawerSection::Notifications => "Notifications",
            DrawerSection::Hardware => "Hardware",
        }
    }

    /// Locked section order — top to bottom in the drawer.
    #[must_use]
    pub const fn ordered() -> [DrawerSection; 4] {
        [
            DrawerSection::QuickActions,
            DrawerSection::Sliders,
            DrawerSection::Notifications,
            DrawerSection::Hardware,
        ]
    }
}

// ──────────────────────────────────────────────────────────────
// Quick-actions section
// ──────────────────────────────────────────────────────────────

/// Quick-action toggles (Phase E.8.2 lock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuickToggle {
    /// Do-not-disturb (suppresses notification popups).
    DoNotDisturb,
    /// Caffeine (suppresses idle / lock-on-idle).
    Caffeine,
    /// Night-light (warm screen color temperature).
    NightLight,
    /// Airplane mode (rfkill block all wireless).
    Airplane,
}

impl QuickToggle {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            QuickToggle::DoNotDisturb => "Do not disturb",
            QuickToggle::Caffeine => "Caffeine",
            QuickToggle::NightLight => "Night light",
            QuickToggle::Airplane => "Airplane mode",
        }
    }

    /// Filesystem path of the flag file backing this toggle.
    /// `Some(path)` means the toggle is on when the file exists,
    /// off otherwise.
    #[must_use]
    pub fn flag_path(&self, cache_root: &Path) -> PathBuf {
        let stem = match self {
            QuickToggle::DoNotDisturb => "notifications-dnd",
            QuickToggle::Caffeine => "caffeine",
            QuickToggle::NightLight => "night-light",
            QuickToggle::Airplane => "airplane",
        };
        cache_root.join("mde").join(stem)
    }

    /// Probe whether the toggle is currently on.
    #[must_use]
    pub fn is_on(&self, cache_root: &Path) -> bool {
        self.flag_path(cache_root).exists()
    }

    /// Flip the toggle by creating or removing the flag file.
    pub fn set(&self, cache_root: &Path, on: bool) -> std::io::Result<()> {
        let path = self.flag_path(cache_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if on {
            std::fs::write(&path, "")
        } else {
            match std::fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
        }
    }

    /// All quick-action toggles in their locked display order.
    #[must_use]
    pub const fn ordered() -> [QuickToggle; 4] {
        [
            QuickToggle::DoNotDisturb,
            QuickToggle::Caffeine,
            QuickToggle::NightLight,
            QuickToggle::Airplane,
        ]
    }
}

// ──────────────────────────────────────────────────────────────
// Notifications section (drawer-inline)
// ──────────────────────────────────────────────────────────────

/// Persistence record for one notification in the inline list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationRow {
    pub id: u64,
    pub app: String,
    pub title: String,
    pub body: String,
    pub urgency: u8,
    pub created_at_ms: u64,
    pub dismissed: bool,
}

/// Pure helper — parse the same ~/.cache/mackes/notifications.json
/// cache the standalone notifications applet consumes.
#[must_use]
pub fn parse_notifications(json: &str) -> Vec<NotificationRow> {
    serde_json::from_str(json).unwrap_or_default()
}

/// Filter dismissed entries — drawer only shows unread.
#[must_use]
pub fn unread_only(rows: Vec<NotificationRow>) -> Vec<NotificationRow> {
    rows.into_iter().filter(|r| !r.dismissed).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn drawer_width_locked_at_360px() {
        assert_eq!(DRAWER_WIDTH_PX, 360);
    }

    #[test]
    fn slide_duration_locked_at_280ms() {
        assert_eq!(SLIDE_DURATION_MS, 280);
    }

    #[test]
    fn section_order_lists_four_distinct() {
        let order = DrawerSection::ordered();
        assert_eq!(order.len(), 4);
        let unique: std::collections::HashSet<_> = order.iter().collect();
        assert_eq!(unique.len(), 4);
    }

    #[test]
    fn section_labels_match_lock() {
        assert_eq!(DrawerSection::QuickActions.label(), "Quick actions");
        assert_eq!(DrawerSection::Sliders.label(), "Brightness & volume");
        assert_eq!(DrawerSection::Notifications.label(), "Notifications");
        assert_eq!(DrawerSection::Hardware.label(), "Hardware");
    }

    #[test]
    fn quick_toggles_have_four_locked_entries() {
        assert_eq!(QuickToggle::ordered().len(), 4);
    }

    #[test]
    fn quick_toggle_labels_match_lock() {
        assert_eq!(QuickToggle::DoNotDisturb.label(), "Do not disturb");
        assert_eq!(QuickToggle::Caffeine.label(), "Caffeine");
        assert_eq!(QuickToggle::NightLight.label(), "Night light");
        assert_eq!(QuickToggle::Airplane.label(), "Airplane mode");
    }

    #[test]
    fn quick_toggle_flag_paths_are_in_mde_subdir() {
        let cache = PathBuf::from("/test/cache");
        let path = QuickToggle::DoNotDisturb.flag_path(&cache);
        assert!(path.starts_with("/test/cache/mde/"));
        assert!(path.ends_with("notifications-dnd"));
    }

    #[test]
    fn quick_toggle_set_on_then_off_round_trips() {
        let tmp = tempdir().unwrap();
        let toggle = QuickToggle::Caffeine;
        assert!(!toggle.is_on(tmp.path()));
        toggle.set(tmp.path(), true).unwrap();
        assert!(toggle.is_on(tmp.path()));
        toggle.set(tmp.path(), false).unwrap();
        assert!(!toggle.is_on(tmp.path()));
    }

    #[test]
    fn quick_toggle_set_off_when_already_off_is_noop() {
        let tmp = tempdir().unwrap();
        let toggle = QuickToggle::NightLight;
        // Already off — set(off) should not error.
        toggle.set(tmp.path(), false).unwrap();
        assert!(!toggle.is_on(tmp.path()));
    }

    #[test]
    fn parse_notifications_handles_empty() {
        assert!(parse_notifications("").is_empty());
    }

    #[test]
    fn parse_notifications_round_trips() {
        let row = NotificationRow {
            id: 1,
            app: "mde-workbench".into(),
            title: "Saved".into(),
            body: "Snapshot created".into(),
            urgency: 1,
            created_at_ms: 1_700_000_000_000,
            dismissed: false,
        };
        let json = serde_json::to_string(&vec![row.clone()]).unwrap();
        let parsed = parse_notifications(&json);
        assert_eq!(parsed, vec![row]);
    }

    #[test]
    fn unread_only_filters_dismissed() {
        let rows = vec![
            NotificationRow {
                id: 1,
                app: "a".into(),
                title: "t".into(),
                body: "b".into(),
                urgency: 1,
                created_at_ms: 0,
                dismissed: false,
            },
            NotificationRow {
                id: 2,
                app: "a".into(),
                title: "t".into(),
                body: "b".into(),
                urgency: 1,
                created_at_ms: 0,
                dismissed: true,
            },
        ];
        let unread = unread_only(rows);
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].id, 1);
    }
}
