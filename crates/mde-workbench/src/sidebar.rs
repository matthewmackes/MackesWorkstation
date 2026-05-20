//! Iced sidebar widget.
//!
//! CB-1.2 lock: collapsible per-group rows. Pure-data
//! [`SidebarState`] (which groups are expanded, which row is
//! focused for keyboard navigation) lives here so the reducer
//! tests can stay Iced-free; the actual `view()` builder pulls
//! in Iced widgets.

use iced::widget::{button, column, container, row, text, Column};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::model::{nav_model, Group, NavEntry, View};

/// Per-group expand/collapse + focus state. The active group
/// (matching the current [`View`]) is always expanded
/// automatically — additional groups can be toggled by the user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SidebarState {
    /// Groups the user explicitly toggled open in addition to
    /// the active one. The active group never appears here —
    /// it's implicitly expanded.
    user_expanded: Vec<Group>,
}

impl SidebarState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Is `group` currently expanded? True when it's the active
    /// group or the user toggled it open.
    #[must_use]
    pub fn is_expanded(&self, group: Group, active: Group) -> bool {
        group == active || self.user_expanded.contains(&group)
    }

    /// Toggle the user-expanded state of `group`. No-op when
    /// `group` is the active one (which is always expanded).
    pub fn toggle(&mut self, group: Group, active: Group) {
        if group == active {
            return;
        }
        if let Some(idx) = self.user_expanded.iter().position(|g| *g == group) {
            self.user_expanded.remove(idx);
        } else {
            self.user_expanded.push(group);
        }
    }
}

/// Build the sidebar tree for an [`App`](crate::App).
///
/// The builder consumes the live [`SidebarState`] and the
/// current [`View`] so the active group is highlighted and
/// auto-expanded. Click events are emitted as the supplied
/// [`Message`](crate::Message) variants.
pub fn view<'a>(
    state: &'a SidebarState,
    view: View,
    on_group_click: impl Fn(Group) -> crate::Message + 'a,
    on_panel_click: impl Fn(Group, &'static str) -> crate::Message + 'a,
) -> Element<'a, crate::Message> {
    let active = view.group();
    let mut col: Column<'a, crate::Message> = column![].spacing(2).padding(8);

    for entry in nav_model() {
        col = col.push(group_header(entry.group, active, &on_group_click));
        if state.is_expanded(entry.group, active) {
            col = col.push(group_panels(&entry, view, &on_panel_click));
        }
    }

    container(col)
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.10, 0.10, 0.12))),
            border: Border {
                color: Color::from_rgb(0.18, 0.18, 0.22),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn group_header<'a>(
    group: Group,
    active: Group,
    on_click: &(impl Fn(Group) -> crate::Message + 'a),
) -> Element<'a, crate::Message> {
    let label = if group == active {
        format!("• {}", group.label())
    } else {
        group.label().to_string()
    };
    button(text(label).size(15))
        .width(Length::Fill)
        .on_press(on_click(group))
        .into()
}

fn group_panels<'a>(
    entry: &NavEntry,
    view: View,
    on_panel_click: &(impl Fn(Group, &'static str) -> crate::Message + 'a),
) -> Element<'a, crate::Message> {
    let mut col: Column<'a, crate::Message> = column![].spacing(1).padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 18.0,
    });
    for panel in &entry.panels {
        let is_active = matches!(
            view,
            View::Panel { group, panel: slug }
                if group == entry.group && slug == panel.slug()
        );
        let label = if is_active {
            format!("→ {}", panel.label())
        } else {
            panel.label().to_string()
        };
        col = col.push(
            button(text(label).size(13))
                .width(Length::Fill)
                .on_press(on_panel_click(entry.group, panel.slug())),
        );
    }
    row![col].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_no_user_expansions() {
        let state = SidebarState::new();
        assert!(!state.is_expanded(Group::Apps, Group::Dashboard));
    }

    #[test]
    fn active_group_is_always_expanded() {
        let state = SidebarState::new();
        for active in Group::all() {
            assert!(
                state.is_expanded(active, active),
                "active group {active:?} should always be expanded"
            );
        }
    }

    #[test]
    fn toggle_expands_then_collapses_inactive_group() {
        let mut state = SidebarState::new();
        let active = Group::Dashboard;
        assert!(!state.is_expanded(Group::Network, active));
        state.toggle(Group::Network, active);
        assert!(state.is_expanded(Group::Network, active));
        state.toggle(Group::Network, active);
        assert!(!state.is_expanded(Group::Network, active));
    }

    #[test]
    fn toggle_on_active_group_is_noop() {
        let mut state = SidebarState::new();
        state.toggle(Group::Dashboard, Group::Dashboard);
        // The internal storage should stay empty — the active
        // group is implicitly expanded, never explicitly.
        assert!(state.user_expanded.is_empty());
        assert!(state.is_expanded(Group::Dashboard, Group::Dashboard));
    }

    #[test]
    fn multiple_groups_can_be_user_expanded_simultaneously() {
        let mut state = SidebarState::new();
        let active = Group::Dashboard;
        state.toggle(Group::Apps, active);
        state.toggle(Group::Network, active);
        assert!(state.is_expanded(Group::Apps, active));
        assert!(state.is_expanded(Group::Network, active));
        assert!(!state.is_expanded(Group::Fleet, active));
    }
}
