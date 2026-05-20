//! Apps → Sources & Repos panel — dnf repo enable/disable.
//!
//! CB-1.3 partial: ports the dnf-repolist slice of
//! `mackes/workbench/apps/sources.py`. The v1.x panel also
//! covered Flathub + RPM Fusion + fedora-workstation-repos
//! sections; those land as a separate CB-1.3 follow-up since
//! each needs a specific install workflow (flatpak
//! remote-add, dnf install of a release RPM, etc.).
//!
//! Reads via `dnf repolist --all --quiet`; writes via
//! `pkexec dnf config-manager setopt <id>.enabled=1|0` —
//! the `config-manager` plugin ships in dnf5's
//! `dnf5-plugins` package which is install-by-default on
//! Fedora workstation.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Padding, Task};
use tokio::process::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoRow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AppsSourcesPanel {
    pub repos: Vec<RepoRow>,
    pub filter: String,
    pub status: String,
    pub busy: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<RepoRow>),
    Error(String),
    FilterChanged(String),
    ToggleClicked { id: String, enable: bool },
    ToggleFinished { id: String, success: bool },
    RefreshClicked,
}

impl AppsSourcesPanel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Task<crate::Message> {
        Task::perform(
            async move {
                let raw = run_dnf_repolist().await;
                Message::Loaded(parse_dnf_repolist(&raw))
            },
            crate::Message::AppsSources,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<crate::Message> {
        match message {
            Message::Loaded(repos) => {
                self.repos = repos;
                self.status.clear();
                self.busy = false;
                Task::none()
            }
            Message::Error(msg) => {
                self.status = msg;
                self.busy = false;
                Task::none()
            }
            Message::FilterChanged(v) => {
                self.filter = v;
                Task::none()
            }
            Message::ToggleClicked { id, enable } => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.status = format!(
                    "{} {id} (polkit will prompt)…",
                    if enable { "Enabling" } else { "Disabling" },
                );
                Task::perform(
                    async move {
                        let success = run_dnf_config_manager(&id, enable).await;
                        Message::ToggleFinished { id, success }
                    },
                    crate::Message::AppsSources,
                )
            }
            Message::ToggleFinished { id, success } => {
                self.status = if success {
                    format!("Updated {id}.")
                } else {
                    format!("Updating {id} failed (see journalctl).")
                };
                self.busy = false;
                // Reload to reflect the new enabled state.
                Self::load()
            }
            Message::RefreshClicked => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.status = "Refreshing…".into();
                Self::load()
            }
        }
    }

    pub fn view(&self) -> Element<'_, crate::Message> {
        let filter_input = text_input("Filter…", &self.filter)
            .on_input(|v| crate::Message::AppsSources(Message::FilterChanged(v)));
        let refresh_btn = {
            let mut b = button(text("Refresh"));
            if !self.busy {
                b = b.on_press(crate::Message::AppsSources(Message::RefreshClicked));
            }
            b
        };

        let filtered: Vec<&RepoRow> = self
            .repos
            .iter()
            .filter(|r| matches_filter(&r.id, &r.name, &self.filter))
            .collect();

        let rows_view = filtered.iter().fold(column![], |col, r| {
            let id = r.id.clone();
            let next_enable = !r.enabled;
            let btn_label = if r.enabled { "Disable" } else { "Enable" };
            let toggle_btn = {
                let mut b = button(text(btn_label));
                if !self.busy {
                    b = b.on_press(crate::Message::AppsSources(Message::ToggleClicked {
                        id,
                        enable: next_enable,
                    }));
                }
                b
            };
            let state_label = if r.enabled { "enabled" } else { "disabled" };
            col.push(
                row![
                    text(&r.id).width(Length::Fixed(220.0)),
                    text(&r.name).width(Length::Fixed(280.0)),
                    text(state_label).width(Length::Fixed(80.0)),
                    toggle_btn,
                ]
                .spacing(12),
            )
        });

        column![
            row![filter_input, refresh_btn].spacing(12),
            scrollable(container(rows_view.spacing(4)).padding(Padding::new(0.0)))
                .height(Length::Fill),
            text(format!(
                "{} matching / {} total ({} enabled)",
                filtered.len(),
                self.repos.len(),
                self.repos.iter().filter(|r| r.enabled).count(),
            ))
            .size(13),
            text(&self.status).size(13),
        ]
        .spacing(12)
        .width(Length::Fill)
        .padding(Padding::new(0.0))
        .into()
    }
}

/// Case-insensitive substring match against either the repo
/// id or its display name. Empty filter matches all rows.
#[must_use]
pub fn matches_filter(id: &str, name: &str, filter: &str) -> bool {
    let f = filter.trim().to_lowercase();
    if f.is_empty() {
        return true;
    }
    id.to_lowercase().contains(&f) || name.to_lowercase().contains(&f)
}

/// Pure parser for `dnf repolist --all` output.
///
/// dnf5's `repolist --all` emits a 3-column table:
///   `repo id                  repo name           status`
/// The status column is the literal string `enabled` or
/// `disabled`. Header + blank lines are skipped. Repo names
/// can contain spaces, so we parse by stripping the
/// last-whitespace-separated column (status) and the
/// first-whitespace-separated column (id) — everything
/// between is the name.
#[must_use]
pub fn parse_dnf_repolist(raw: &str) -> Vec<RepoRow> {
    let mut rows: Vec<RepoRow> = raw.lines().filter_map(parse_repolist_line).collect();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows
}

fn parse_repolist_line(line: &str) -> Option<RepoRow> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("repo id") || trimmed.starts_with("Last metadata") {
        return None;
    }
    // Find the rightmost whitespace-separated word; if it's
    // not "enabled" or "disabled" the line isn't a repo row.
    let last_space = trimmed.rfind(char::is_whitespace)?;
    let status = trimmed[last_space + 1..].trim();
    let enabled = match status {
        "enabled" => true,
        "disabled" => false,
        _ => return None,
    };
    let rest = trimmed[..last_space].trim();
    // First word is the repo id.
    let first_space = rest.find(char::is_whitespace)?;
    let id = rest[..first_space].trim().to_string();
    let name = rest[first_space..].trim().to_string();
    if id.is_empty() {
        return None;
    }
    Some(RepoRow { id, name, enabled })
}

/// Shell out to `dnf repolist --all --quiet`. Returns stdout
/// on success; empty string on any failure.
pub async fn run_dnf_repolist() -> String {
    let Ok(output) = Command::new("dnf")
        .args(["repolist", "--all", "--quiet"])
        .output()
        .await
    else {
        return String::new();
    };
    if !output.status.success() {
        return String::new();
    }
    String::from_utf8(output.stdout).unwrap_or_default()
}

/// Shell out to `pkexec dnf config-manager setopt
/// <id>.enabled=0|1`. Returns `true` on a zero exit.
pub async fn run_dnf_config_manager(id: &str, enable: bool) -> bool {
    let setopt = format!("{id}.enabled={}", if enable { 1 } else { 0 });
    let Ok(output) = Command::new("pkexec")
        .args(["dnf", "config-manager", "setopt", &setopt])
        .output()
        .await
    else {
        return false;
    };
    output.status.success()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
repo id                          repo name                              status
fedora                           Fedora 44 - x86_64                     enabled
fedora-cisco-openh264            Fedora 44 openh264 (From Cisco)        disabled
google-chrome                    google-chrome                          disabled
updates                          Fedora 44 - x86_64 - Updates           enabled
";

    #[test]
    fn parse_dnf_repolist_extracts_id_name_and_status() {
        let rows = parse_dnf_repolist(SAMPLE);
        assert_eq!(rows.len(), 4);
        // Sorted by id.
        assert_eq!(rows[0].id, "fedora");
        assert_eq!(rows[0].name, "Fedora 44 - x86_64");
        assert!(rows[0].enabled);
        assert_eq!(rows[1].id, "fedora-cisco-openh264");
        assert!(!rows[1].enabled);
        assert_eq!(rows[3].id, "updates");
        assert!(rows[3].enabled);
    }

    #[test]
    fn parse_dnf_repolist_handles_repo_names_with_spaces() {
        let raw = "rpmfusion-free   RPM Fusion for Fedora 44 - Free     enabled\n";
        let rows = parse_dnf_repolist(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "rpmfusion-free");
        assert_eq!(rows[0].name, "RPM Fusion for Fedora 44 - Free");
        assert!(rows[0].enabled);
    }

    #[test]
    fn parse_dnf_repolist_skips_headers_and_blanks() {
        let raw = "\
repo id   repo name   status

Last metadata expiration check: now.
fedora    Fedora 44   enabled
";
        let rows = parse_dnf_repolist(raw);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "fedora");
    }

    #[test]
    fn parse_dnf_repolist_rejects_lines_without_status() {
        let raw = "fedora Fedora 44 maybe\n";
        assert!(parse_dnf_repolist(raw).is_empty());
    }

    #[test]
    fn parse_dnf_repolist_empty_on_empty_input() {
        assert!(parse_dnf_repolist("").is_empty());
    }

    #[test]
    fn matches_filter_searches_id_and_name() {
        assert!(matches_filter("rpmfusion-free", "RPM Fusion - Free", ""));
        assert!(matches_filter(
            "rpmfusion-free",
            "RPM Fusion - Free",
            "rpmfusion"
        ));
        // Name-side match.
        assert!(matches_filter(
            "repo123",
            "Fedora Workstation",
            "workstation"
        ));
        assert!(!matches_filter("fedora", "Fedora 44", "ubuntu"));
    }

    #[test]
    fn loaded_records_repos_and_clears_status() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        let rows = parse_dnf_repolist(SAMPLE);
        let _ = panel.update(Message::Loaded(rows.clone()));
        assert_eq!(panel.repos, rows);
        assert!(!panel.busy);
    }

    #[test]
    fn toggle_clicked_while_busy_is_noop() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        panel.status = "stale".into();
        let _ = panel.update(Message::ToggleClicked {
            id: "fedora".into(),
            enable: false,
        });
        assert_eq!(panel.status, "stale");
    }

    #[test]
    fn toggle_finished_success_reports_updated() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        let _ = panel.update(Message::ToggleFinished {
            id: "rpmfusion-free".into(),
            success: true,
        });
        assert!(!panel.busy);
        assert!(panel.status.contains("Updated rpmfusion-free"));
    }

    #[test]
    fn toggle_finished_failure_includes_failed_marker() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        let _ = panel.update(Message::ToggleFinished {
            id: "fedora".into(),
            success: false,
        });
        assert!(panel.status.contains("failed"));
        assert!(panel.status.contains("fedora"));
    }

    #[test]
    fn refresh_clicked_while_busy_is_noop() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        panel.status = "stale".into();
        let _ = panel.update(Message::RefreshClicked);
        assert_eq!(panel.status, "stale");
    }

    #[test]
    fn filter_changed_mutates_filter() {
        let mut panel = AppsSourcesPanel::new();
        let _ = panel.update(Message::FilterChanged("rpmfusion".into()));
        assert_eq!(panel.filter, "rpmfusion");
    }

    #[test]
    fn error_message_clears_busy_and_stores_msg() {
        let mut panel = AppsSourcesPanel::new();
        panel.busy = true;
        let _ = panel.update(Message::Error("dnf not on PATH".into()));
        assert_eq!(panel.status, "dnf not on PATH");
        assert!(!panel.busy);
    }
}
