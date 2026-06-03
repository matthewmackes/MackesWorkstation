//! Phase E.4.2 — focused-app hero widget.
//!
//! The hero is the panel's largest single piece of content: a
//! single-line display of the currently-focused window's title
//! + app icon, anchored to the left of the layout cluster. It
//! slides on focus change with a 280 ms ease-out tween.
//!
//! State writers (Phase E.3 foreign-toplevel listener, when it
//! lands) call `Hero::set_focused(title, app_id)` whenever the
//! sway focus signal fires. The widget's `view()` renders the
//! current title in whatever pane the panel orchestrator slots
//! it into.

use std::time::{Duration, Instant};

/// Default tween length per the 1.1.0 Win10 layout lock.
pub const SLIDE_DURATION_MS: u64 = 280;

/// Maximum displayed-title length before ellipsizing.
pub const MAX_TITLE_CHARS: usize = 64;

#[derive(Debug, Clone, Default)]
pub struct Hero {
    /// Currently displayed (post-tween) title.
    current: Option<HeroEntry>,
    /// New title arriving — fades in over [`SLIDE_DURATION_MS`].
    incoming: Option<HeroEntry>,
    /// Timestamp at which the incoming entry started its slide.
    incoming_started_at: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroEntry {
    pub title: String,
    pub app_id: String,
}

impl Hero {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the focused window. If it's the same as the current
    /// entry, this is a no-op. Otherwise the previous entry
    /// slides out + the new one slides in.
    pub fn set_focused(&mut self, title: String, app_id: String) {
        let new_entry = HeroEntry { title, app_id };
        if self.current.as_ref() == Some(&new_entry) {
            return;
        }
        self.incoming = Some(new_entry);
        self.incoming_started_at = Some(Instant::now());
    }

    /// Advance the slide. Once the duration elapses, the incoming
    /// entry becomes the current entry.
    pub fn tick(&mut self, now: Instant) {
        if let (Some(_incoming), Some(start)) = (&self.incoming, self.incoming_started_at) {
            if now.duration_since(start) >= Duration::from_millis(SLIDE_DURATION_MS) {
                self.current = self.incoming.take();
                self.incoming_started_at = None;
            }
        }
    }

    /// What to display right now (post-tween entry, or the
    /// incoming entry if no current).
    #[must_use]
    pub fn display(&self) -> Option<&HeroEntry> {
        self.incoming.as_ref().or(self.current.as_ref())
    }

    /// Ellipsized version of the display title.
    #[must_use]
    pub fn display_title(&self) -> Option<String> {
        self.display().map(|e| ellipsize(&e.title, MAX_TITLE_CHARS))
    }

    /// Slide progress 0.0 → 1.0 — 1.0 once the tween completes.
    /// Used by the renderer to drive opacity / transform.
    #[must_use]
    pub fn progress_at(&self, now: Instant) -> f32 {
        let Some(start) = self.incoming_started_at else {
            return 1.0;
        };
        let elapsed = now.duration_since(start).as_millis() as f32;
        let full = SLIDE_DURATION_MS as f32;
        (elapsed / full).clamp(0.0, 1.0)
    }
}

/// Truncate `s` to at most `max` characters, appending `…` when
/// truncation happens. Counts in characters, not bytes, so the
/// result is multi-byte-safe.
#[must_use]
pub fn ellipsize(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_duration_is_280ms_per_lock() {
        assert_eq!(SLIDE_DURATION_MS, 280);
    }

    #[test]
    fn max_title_is_64_chars() {
        assert_eq!(MAX_TITLE_CHARS, 64);
    }

    #[test]
    fn empty_hero_displays_nothing() {
        let hero = Hero::new();
        assert!(hero.display().is_none());
        assert!(hero.display_title().is_none());
    }

    #[test]
    fn set_focused_makes_entry_displayable() {
        let mut hero = Hero::new();
        hero.set_focused("Workbench".into(), "mde-workbench".into());
        let entry = hero.display().unwrap();
        assert_eq!(entry.title, "Workbench");
        assert_eq!(entry.app_id, "mde-workbench");
    }

    #[test]
    fn same_focused_is_noop() {
        let mut hero = Hero::new();
        hero.set_focused("A".into(), "a".into());
        hero.tick(Instant::now() + Duration::from_millis(500));
        let started = hero.incoming_started_at;
        hero.set_focused("A".into(), "a".into());
        assert_eq!(hero.incoming_started_at, started);
    }

    #[test]
    fn tick_promotes_incoming_to_current() {
        let mut hero = Hero::new();
        hero.set_focused("First".into(), "one".into());
        let later = Instant::now() + Duration::from_millis(SLIDE_DURATION_MS + 50);
        hero.tick(later);
        assert!(hero.incoming.is_none());
        assert_eq!(hero.current.as_ref().unwrap().title, "First");
    }

    #[test]
    fn ellipsize_no_truncation_when_short() {
        assert_eq!(ellipsize("hello", 10), "hello");
    }

    #[test]
    fn ellipsize_truncates_with_ellipsis() {
        let s = "this is a long title that goes on and on";
        let out = ellipsize(s, 10);
        assert_eq!(out.chars().count(), 10);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn ellipsize_is_unicode_safe() {
        let s = "한국어를 잘 못해요";
        let out = ellipsize(s, 5);
        assert_eq!(out.chars().count(), 5);
    }

    #[test]
    fn progress_is_zero_when_no_incoming() {
        let hero = Hero::new();
        assert_eq!(hero.progress_at(Instant::now()), 1.0);
    }

    #[test]
    fn progress_grows_from_zero_to_one() {
        let mut hero = Hero::new();
        hero.set_focused("x".into(), "y".into());
        let start = hero.incoming_started_at.unwrap();
        assert!((hero.progress_at(start) - 0.0).abs() < 0.01);
        let mid = start + Duration::from_millis(SLIDE_DURATION_MS / 2);
        let p = hero.progress_at(mid);
        assert!(p > 0.4 && p < 0.6);
        let done = start + Duration::from_millis(SLIDE_DURATION_MS + 100);
        assert_eq!(hero.progress_at(done), 1.0);
    }

    #[test]
    fn display_title_ellipsizes() {
        let mut hero = Hero::new();
        let long_title = "a".repeat(100);
        hero.set_focused(long_title, "x".into());
        let displayed = hero.display_title().unwrap();
        assert_eq!(displayed.chars().count(), MAX_TITLE_CHARS);
    }
}
