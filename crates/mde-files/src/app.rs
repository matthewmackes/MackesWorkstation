//! Iced application — top-level State, Message, update, view.

use iced::widget::{column, container, row, scrollable};
use iced::{Background, Border, Color, Element, Length, Padding, Size, Task, Theme};

use crate::demo_data as data;
use crate::model::{Layout, View};
use crate::theme as t;
use crate::views;

#[derive(Debug, Clone)]
pub enum Message {
    SelectView(View),
    ToggleLocal,
    SetLayout(Layout),
    SearchChanged(String),
    Refresh,
    TitlebarMinimize,
    TitlebarMaximize,
    TitlebarClose,
    PeerCardBrowse(&'static str),
    PeerCardSend(&'static str),
    PrimaryAction,
    /// No-op message used by buttons that don't have a wired behaviour yet
    /// (e.g. the sidebar's panel-toggle, the peer card's `…` button).
    Noop,
}

/// Breadcrumb segment used by the toolbar.
#[derive(Debug, Clone)]
pub struct Crumb {
    pub label: String,
    /// True if this crumb belongs to a mesh path. Affects colour + the trailing tag chip.
    pub mesh: bool,
}

#[derive(Debug, Default)]
pub struct MdeFiles {
    pub view: View,
    pub local_open: bool,
    pub layout: Layout,
    pub search: String,
}

impl MdeFiles {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the Iced application.
    ///
    /// Builds a fresh `MdeFiles` state, registers the warm-dark theme, opens a
    /// 1480×940 window, and dispatches updates from `Message`.
    pub fn run() -> iced::Result {
        iced::application(Self::title, Self::update, Self::view)
            .theme(Self::theme)
            .window_size(Size::new(t::WIN_W, t::WIN_H))
            .run()
    }

    fn title(&self) -> String {
        "Artifact Manager".into()
    }

    fn theme(&self) -> Theme {
        t::theme()
    }

    /// Update reducer — every interaction in the UI flows through this single
    /// function. No async work happens here yet (the demo backend is in-memory);
    /// once `mded` is wired, several variants will return real `Task`s.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectView(v) => {
                self.view = v;
                if !matches!(v, View::Local) {
                    self.local_open = false;
                }
            }
            Message::ToggleLocal => {
                self.local_open = !self.local_open;
                if self.local_open && !matches!(self.view, View::Local) {
                    self.view = View::Local;
                } else if !self.local_open && matches!(self.view, View::Local) {
                    self.view = View::default();
                }
            }
            Message::SetLayout(l) => self.layout = l,
            Message::SearchChanged(s) => self.search = s,
            Message::PeerCardBrowse(id) => self.view = View::Peer(id),
            Message::Refresh
            | Message::TitlebarMinimize
            | Message::TitlebarMaximize
            | Message::TitlebarClose
            | Message::PeerCardSend(_)
            | Message::PrimaryAction
            | Message::Noop => {
                // Demo backend has no side effects to run; leave these as routing hooks
                // for the future `Backend::DBus` impl.
            }
        }
        Task::none()
    }

    /// Top-level view tree.
    pub fn view(&self) -> Element<'_, Message> {
        let crumbs = breadcrumbs_for(self.view);

        let main_body: Element<'_, Message> = match self.view {
            View::MeshOverview => views::mesh_overview(&data::SELF_NODE),
            View::Inbox        => views::inbox(&data::SELF_NODE),
            View::Peer(id) => {
                if let Some(peer) = data::PEERS.iter().find(|p| p.id == id) {
                    views::peer_folder(peer, &data::SELF_NODE)
                } else {
                    empty_state("no peer").into()
                }
            }
            View::Downloads => views::downloads(),
            View::Local     => views::local_veil(&data::SELF_NODE),
        };

        let content = container(scrollable(
            container(main_body).padding(Padding { top: 18.0, right: 22.0, bottom: 28.0, left: 22.0 }),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(t::PF_BG_300)),
            border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() },
            ..container::Style::default()
        });

        let main = column![
            views::toolbar(self.view, self.layout, &self.search, crumbs),
            content,
        ]
        .spacing(0);

        let body = row![
            views::sidebar(self.view, self.local_open, &data::SELF_NODE),
            container(main).width(Length::Fill).height(Length::Fill),
        ]
        .height(Length::Fill);

        let online = data::online_count();
        let total = data::PEERS.len();

        container(
            column![views::titlebar(online, total), body].spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(t::WINDOW)),
            border: Border { color: Color { a: 0.08, ..Color::WHITE }, width: 1.0, radius: 0.0.into() },
            ..container::Style::default()
        })
        .into()
    }
}

fn breadcrumbs_for(view: View) -> Vec<Crumb> {
    match view {
        View::MeshOverview => vec![
            Crumb { label: "Mesh".into(),     mesh: true },
            Crumb { label: "Overview".into(), mesh: false },
        ],
        View::Inbox => vec![
            Crumb { label: "Mesh".into(),  mesh: true },
            Crumb { label: "Inbox".into(), mesh: false },
        ],
        View::Peer(id) => {
            let host = data::PEERS.iter().find(|p| p.id == id).map_or("?", |p| p.host);
            vec![
                Crumb { label: "Mesh".into(),     mesh: true },
                Crumb { label: host.to_string(), mesh: false },
            ]
        }
        View::Downloads => vec![
            Crumb { label: "~".into(),         mesh: false },
            Crumb { label: "Downloads".into(), mesh: false },
        ],
        View::Local => vec![
            Crumb { label: "Local".into(),                     mesh: false },
            Crumb { label: data::SELF_NODE.host.to_string(),   mesh: false },
            Crumb { label: "/".into(),                          mesh: false },
        ],
    }
}

fn empty_state(label: &str) -> Element<'static, Message> {
    container(iced::widget::text(label.to_string()).size(12).color(t::FG_FAINT))
        .padding(Padding::new(56.0))
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border { color: Color { a: 0.10, ..Color::WHITE }, width: 1.0, radius: 0.0.into() },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_view_is_mesh_overview() {
        let s = MdeFiles::default();
        assert_eq!(s.view, View::MeshOverview);
        assert!(!s.local_open);
        assert_eq!(s.layout, Layout::List);
        assert!(s.search.is_empty());
    }

    #[test]
    fn toggle_local_opens_local_view() {
        let mut s = MdeFiles::default();
        let _ = s.update(Message::ToggleLocal);
        assert!(s.local_open);
        assert_eq!(s.view, View::Local);
        let _ = s.update(Message::ToggleLocal);
        assert!(!s.local_open);
        assert_eq!(s.view, View::MeshOverview);
    }

    #[test]
    fn selecting_non_local_view_closes_local_disclosure() {
        let mut s = MdeFiles::default();
        s.local_open = true;
        s.view = View::Local;
        let _ = s.update(Message::SelectView(View::Inbox));
        assert_eq!(s.view, View::Inbox);
        assert!(!s.local_open, "local disclosure must close when leaving Local view");
    }

    #[test]
    fn peer_card_browse_routes_to_peer_view() {
        let mut s = MdeFiles::default();
        let _ = s.update(Message::PeerCardBrowse("pine"));
        assert_eq!(s.view, View::Peer("pine"));
    }

    #[test]
    fn breadcrumbs_match_view() {
        let c = breadcrumbs_for(View::MeshOverview);
        assert_eq!(c.len(), 2);
        assert!(c[0].mesh);
        assert_eq!(c[0].label, "Mesh");
        assert_eq!(c[1].label, "Overview");

        let c = breadcrumbs_for(View::Peer("birch"));
        assert_eq!(c[1].label, "birch.mesh");

        let c = breadcrumbs_for(View::Local);
        assert_eq!(c.len(), 3);
        assert!(!c.iter().any(|x| x.mesh));
    }
}
