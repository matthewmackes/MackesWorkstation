//! Add/Remove Programs (B) — a themed, dnf-backed package manager that replaces
//! dnfdragora (which hangs on every launch and can't be killed).
//!
//! `mde add-remove` opens a window listing the curated software catalogue
//! ([`crate::catalogue`]) grouped by category, with each package's installed
//! state read from `rpm`. Install / Remove run `pkexec dnf` **off the UI thread**
//! (polkit handles the privilege prompt), so a slow download never freezes the
//! window; the row refreshes from `rpm` when the operation finishes. Mandatory
//! base-session packages are shown locked (Required), never removable.

use std::process::ExitCode;

use iced::widget::{button, container, scrollable, text, text_input, Column, Row};
use iced::{Element, Length, Padding, Task};

use mde_ui::{frame, metrics, palette};

use crate::catalogue;

pub fn run(_args: &[String]) -> ExitCode {
    match launch() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mde add-remove: {e}");
            ExitCode::FAILURE
        }
    }
}

/// One catalogue package + its live installed state.
struct Pkg {
    package: &'static str,
    category: &'static str,
    name: &'static str,
    /// Base-session package: always installed, never removable.
    mandatory: bool,
    installed: bool,
}

struct AddRemove {
    rows: Vec<Pkg>,
    /// The package an install/remove is currently running for (buttons disable
    /// while set, so only one dnf transaction runs at a time).
    busy: Option<String>,
    /// Last result line, shown in the status bar.
    status: Option<String>,
    /// Live filter (B.2a): name/package substring, case-insensitive. Empty = all.
    search: String,
}

#[derive(Debug, Clone)]
enum Message {
    /// Install (`true`) or remove (`false`) a package.
    Act(String, bool),
    /// The `pkexec dnf` transaction for a package finished.
    Done(String, bool),
    /// Edit the search/filter box (B.2a).
    SearchChanged(String),
}

fn load_rows() -> Vec<Pkg> {
    catalogue::catalogue()
        .into_iter()
        .map(|c| Pkg {
            package: c.package,
            category: c.category,
            name: c.name,
            mandatory: c.mandatory,
            installed: catalogue::is_installed(c.package),
        })
        .collect()
}

fn launch() -> iced::Result {
    iced::application(
        |_: &AddRemove| "Add/Remove Programs - mde".to_string(),
        update,
        view,
    )
    .theme(|_| palette::iced_theme())
    .font(mde_ui::font::REGULAR_BYTES)
    .font(mde_ui::font::BOLD_BYTES)
    .font(mde_ui::font::PLEX_REGULAR_BYTES)
    .font(mde_ui::font::PLEX_BOLD_BYTES)
    .default_font(mde_ui::font::ui())
    .run_with(|| {
        (
            AddRemove {
                rows: load_rows(),
                busy: None,
                status: None,
                search: String::new(),
            },
            Task::none(),
        )
    })
}

fn update(state: &mut AddRemove, message: Message) -> Task<Message> {
    match message {
        Message::Act(package, install) => {
            // Only one transaction at a time.
            if state.busy.is_none() {
                state.busy = Some(package.clone());
                let verb = if install { "Installing" } else { "Removing" };
                state.status = Some(format!("{verb} {package}…"));
                return act_task(package, install);
            }
        }
        Message::Done(package, ok) => {
            state.busy = None;
            // Re-read rpm — the source of truth (the user may have cancelled the
            // polkit prompt, or dnf may have refused a dependency).
            let now = catalogue::is_installed(&package);
            if let Some(p) = state.rows.iter_mut().find(|p| p.package == package) {
                p.installed = now;
            }
            state.status = Some(if ok {
                format!("Done: {package}.")
            } else {
                format!("'{package}' was not changed (cancelled or failed).")
            });
        }
        Message::SearchChanged(s) => state.search = s,
    }
    Task::none()
}

/// Run `pkexec dnf install|remove -y <package>` off the UI thread and report back.
fn act_task(package: String, install: bool) -> Task<Message> {
    Task::perform(
        async move {
            let pkg = package.clone();
            let ok = tokio::task::spawn_blocking(move || {
                let verb = if install { "install" } else { "remove" };
                std::process::Command::new("pkexec")
                    .args(["dnf", verb, "-y", &pkg])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            })
            .await
            .unwrap_or(false);
            (package, ok)
        },
        |(package, ok)| Message::Done(package, ok),
    )
}

fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding {
        top: t,
        right: r,
        bottom: b,
        left: l,
    }
}

fn section_header(label: &str) -> Element<'static, Message> {
    container(
        text(label.to_string())
            .size(metrics::UI_PX)
            .font(mde_ui::font::ui_bold()),
    )
    .padding(pad(8.0, 0.0, 2.0, 2.0))
    .into()
}

/// One package row: name + a right-aligned action (Install / Remove / Required),
/// disabled while any transaction is in flight.
fn pkg_row(p: &Pkg, busy: bool) -> Element<'static, Message> {
    let name = text(p.name.to_string())
        .size(metrics::UI_PX)
        .width(Length::FillPortion(5));
    let pkg = text(p.package.to_string())
        .size(metrics::UI_PX)
        .width(Length::FillPortion(3))
        .color(palette::color(palette::GRAY_TEXT));

    let action: Element<Message> = if p.mandatory {
        text("Required")
            .size(metrics::UI_PX)
            .color(palette::color(palette::GRAY_TEXT))
            .into()
    } else {
        let (label, install) = if p.installed {
            ("Remove", false)
        } else {
            ("Install", true)
        };
        let msg = (!busy).then(|| Message::Act(p.package.to_string(), install));
        button(text(label).size(metrics::UI_PX))
            .on_press_maybe(msg)
            .padding(pad(2.0, 10.0, 2.0, 10.0))
            .into()
    };

    Row::new()
        .spacing(8.0)
        .align_y(iced::Alignment::Center)
        .push(name)
        .push(pkg)
        .push(container(action).align_x(iced::alignment::Horizontal::Right))
        .padding(pad(2.0, 6.0, 2.0, 6.0))
        .into()
}

/// Case-insensitive substring match over a package's display name + its id
/// (B.2a). `q` must already be trimmed + lowercased; empty matches everything.
fn pkg_matches(p: &Pkg, q: &str) -> bool {
    q.is_empty() || p.name.to_lowercase().contains(q) || p.package.to_lowercase().contains(q)
}

fn view(state: &AddRemove) -> Element<'_, Message> {
    let busy = state.busy.is_some();
    // Live filter (B.2a): case-insensitive substring over the display name + the
    // package id; empty query matches everything.
    let q = state.search.trim().to_lowercase();
    let matches = |p: &Pkg| pkg_matches(p, &q);
    let shown = state.rows.iter().filter(|p| matches(p)).count();

    let mut list = Column::new().spacing(0.0).padding(pad(2.0, 8.0, 2.0, 8.0));
    for cat in catalogue::categories(
        &catalogue::catalogue(), // category order; cheap (static data)
    ) {
        let cat_rows: Vec<&Pkg> = state
            .rows
            .iter()
            .filter(|p| p.category == cat && matches(p))
            .collect();
        if cat_rows.is_empty() {
            continue; // hide a category with no match under the current filter
        }
        list = list.push(section_header(cat));
        for p in cat_rows {
            list = list.push(pkg_row(p, busy));
        }
    }

    let body = iced::widget::stack![
        frame::sunken().face(palette::color(palette::WINDOW)),
        container(scrollable(list).style(mde_ui::scrollbar))
            .width(Length::Fill)
            .height(Length::Fill),
    ];

    let status = text(state.status.clone().unwrap_or_else(|| {
        if q.is_empty() {
            format!("{} programs", state.rows.len())
        } else {
            format!("{} of {} programs", shown, state.rows.len())
        }
    }))
    .size(metrics::UI_PX)
    .color(palette::color(palette::WINDOW_TEXT));

    // Title on the left, a live search box on the right.
    let header = Row::new()
        .spacing(8.0)
        .align_y(iced::Alignment::Center)
        .push(
            text("Currently installed programs and available components")
                .size(metrics::UI_PX)
                .font(mde_ui::font::ui_bold())
                .width(Length::Fill),
        )
        .push(
            text_input("Search programs", &state.search)
                .on_input(Message::SearchChanged)
                .size(metrics::UI_PX)
                .width(Length::Fixed(220.0)),
        );

    container(
        Column::new()
            .spacing(6.0)
            .padding(8.0)
            .push(header)
            .push(container(body).height(Length::Fill))
            .push(status),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &'static str, package: &'static str) -> Pkg {
        Pkg {
            package,
            category: "Test",
            name,
            mandatory: false,
            installed: false,
        }
    }

    #[test]
    fn pkg_matches_name_and_id_case_insensitively() {
        let p = pkg("Web Browser (Firefox)", "firefox");
        assert!(pkg_matches(&p, "")); // empty matches all
        assert!(pkg_matches(&p, "browser")); // display name
        assert!(pkg_matches(&p, "firefox")); // package id
        assert!(pkg_matches(&p, "fire")); // substring of the id
        assert!(!pkg_matches(&p, "chrome"));
    }
}
