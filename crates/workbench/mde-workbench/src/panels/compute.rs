//! Compute group root — local + fleet VM / pod instance list (E6.10).
//!
//! Rebuilds the legacy `crates/legacy/mde-virtual` instance enumeration
//! onto the workbench: lists local KVM domains (`virsh list --all`) +
//! Podman containers (`podman ps --all --format json`) with per-instance
//! state. The pure parsers (`parse_virsh_list_state`, `parse_podman_ps`,
//! `state_is_running`/`state_is_paused`) are ported 1:1 from
//! `mde-virtual::app` (VIRT-13/18) so this surface and the retired tool
//! agree byte-for-byte on how libvirt/podman output reads.
//!
//! This slice (E6.10 #1) is the Compute foundation: the bespoke group
//! root that enumerates instances + their state. Live lifecycle ops
//! (start/stop/bulk actions), the 4-step VM wizard, per-instance
//! sparklines, cold migration, and the virt-viewer console land in the
//! later E6.10 slices. The list degrades gracefully when neither
//! hypervisor tool is installed (empty list + a "no hypervisor" status,
//! never a panic) — the standalone-first cross-cutting rule.

use iced::widget::{column, row, text, Space};
use iced::{Element, Length, Task};
use mde_theme::{spacing, FontSize, Palette, TypeRole};

use crate::controls::{variant_button, ButtonVariant};

/// Whether an enumerated instance is a libvirt VM or a Podman container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceKind {
    Vm,
    Container,
}

impl InstanceKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Vm => "VM",
            Self::Container => "Container",
        }
    }
}

/// One enumerated compute instance: name + kind + the raw lifecycle
/// state string libvirt / podman reported (`running`, `shut off`,
/// `paused`, `exited`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    pub kind: InstanceKind,
    pub state: String,
}

/// The result of one enumeration pass. `sources` names the hypervisor
/// tools that actually responded (so an empty `instances` list can tell
/// "no instances" apart from "no hypervisor installed").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Enumeration {
    pub instances: Vec<Instance>,
    pub sources: Vec<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct ComputePanel {
    instances: Vec<Instance>,
    status: String,
    loaded: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Enumeration),
    RefreshClicked,
}

impl ComputePanel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only accessor for the enumerated instances (test/inspection).
    #[must_use]
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    /// Status line shown under the header.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Kick off a `virsh` + `podman` enumeration on the iced executor.
    pub fn load() -> Task<crate::Message> {
        Task::perform(
            async move { Message::Loaded(enumerate().await) },
            crate::Message::Compute,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<crate::Message> {
        match message {
            Message::Loaded(e) => {
                self.status = status_line(&e);
                self.instances = e.instances;
                self.loaded = true;
                Task::none()
            }
            Message::RefreshClicked => Self::load(),
        }
    }

    pub fn view(&self) -> Element<'_, crate::Message> {
        let palette = Palette::dark();
        // Carbon type scale + 8px spacing grid via mde-theme tokens (the
        // workbench's design-token source — it's on iced 0.14, so it can't
        // consume mde-ui's iced-0.13 metrics module; mde-theme is the
        // shared, version-decoupled token crate every panel reads, E9.6).
        let sizes = FontSize::defaults();
        let title = text("Compute")
            .size(TypeRole::Display.size_in(sizes))
            .color(palette.text.into_iced_color());
        let subtitle = text("Local and fleet VMs and containers")
            .size(TypeRole::Body.size_in(sizes))
            .color(palette.text_muted.into_iced_color());
        let refresh = variant_button(
            "Refresh",
            ButtonVariant::Ghost,
            Some(crate::Message::Compute(Message::RefreshClicked)),
            palette,
        );
        let header = row![
            column![title, subtitle]
                .spacing(f32::from(spacing::BASE[0]))
                .width(Length::Fill),
            refresh,
        ]
        .align_y(iced::alignment::Vertical::Center);

        let body: Element<'_, crate::Message> = if self.instances.is_empty() {
            // Honest empty-state: distinguish "nothing running" from
            // "no hypervisor" via the status line set at load time.
            let msg = if self.loaded {
                self.status.clone()
            } else {
                "Loading instances…".to_string()
            };
            column![text(msg)
                .size(TypeRole::Body.size_in(sizes))
                .color(palette.text_muted.into_iced_color())]
            .into()
        } else {
            let mut rows: Vec<Element<'_, crate::Message>> = vec![instance_header_row(palette)];
            for inst in &self.instances {
                rows.push(instance_row(inst, palette));
            }
            column(rows).spacing(f32::from(spacing::BASE[1])).into()
        };

        column![
            header,
            Space::new().height(Length::Fixed(f32::from(spacing::BASE[4]))),
            body,
            Space::new().height(Length::Fixed(f32::from(spacing::BASE[2]))),
            text(&self.status)
                .size(TypeRole::Caption.size_in(sizes))
                .color(palette.text_muted.into_iced_color()),
        ]
        .padding(f32::from(spacing::BASE[2]))
        .width(Length::Fill)
        .into()
    }
}

/// The instance-table header row (Name / Kind / State).
fn instance_header_row<'a>(palette: Palette) -> Element<'a, crate::Message> {
    let muted = palette.text_muted.into_iced_color();
    let cap = TypeRole::Caption.size_in(FontSize::defaults());
    row![
        text("Name")
            .size(cap)
            .color(muted)
            .width(Length::FillPortion(3)),
        text("Kind")
            .size(cap)
            .color(muted)
            .width(Length::FillPortion(1)),
        text("State")
            .size(cap)
            .color(muted)
            .width(Length::FillPortion(2)),
    ]
    .spacing(f32::from(spacing::BASE[3]))
    .into()
}

/// One instance row, the state coloured by liveness.
fn instance_row<'a>(inst: &Instance, palette: Palette) -> Element<'a, crate::Message> {
    let body = TypeRole::Body.size_in(FontSize::defaults());
    let state_color = if state_is_running(&inst.state) {
        palette.success
    } else if state_is_paused(&inst.state) {
        palette.warning
    } else {
        palette.text_muted
    };
    row![
        text(inst.name.clone())
            .size(body)
            .color(palette.text.into_iced_color())
            .width(Length::FillPortion(3)),
        text(inst.kind.label())
            .size(body)
            .color(palette.text_muted.into_iced_color())
            .width(Length::FillPortion(1)),
        text(inst.state.clone())
            .size(body)
            .color(state_color.into_iced_color())
            .width(Length::FillPortion(2)),
    ]
    .spacing(f32::from(spacing::BASE[3]))
    .into()
}

/// Human status line for an enumeration result.
#[must_use]
pub fn status_line(e: &Enumeration) -> String {
    if e.sources.is_empty() {
        return "No local hypervisor found — install libvirt (virsh) or podman to manage compute."
            .to_string();
    }
    let n = e.instances.len();
    let noun = if n == 1 { "instance" } else { "instances" };
    format!("{n} {noun} via {}.", e.sources.join(" + "))
}

/// True when a libvirt/podman state string reads as actively running.
/// Ported from `mde-virtual::app::state_is_running`.
#[must_use]
pub fn state_is_running(state: &str) -> bool {
    state.to_ascii_lowercase().contains("running")
}

/// True when a state string reads as paused/suspended.
/// Ported from `mde-virtual::app::state_is_paused`.
#[must_use]
pub fn state_is_paused(state: &str) -> bool {
    let s = state.to_ascii_lowercase();
    s.contains("paused") || s.contains("suspended")
}

/// Parse `virsh list --all` table output into `(name, state)` pairs.
/// Ported 1:1 from `mde-virtual::app::parse_virsh_list_state`.
#[must_use]
pub fn parse_virsh_list_state(stdout: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("---") {
            continue;
        }
        let cols: Vec<&str> = t.split_whitespace().collect();
        if cols.first().copied() == Some("Id") {
            continue; // header row
        }
        if cols.len() < 3 {
            continue;
        }
        let name = cols[1].to_string();
        let state = cols[2..].join(" ");
        out.push((name, state));
    }
    out
}

/// Parse `podman ps --all --format json` into `(name, state)` pairs.
/// Adapted from `mde-virtual::app::parse_podman_ps_local` (which carried
/// extra fields this list view doesn't need yet). Garbage → empty.
#[must_use]
pub fn parse_podman_ps(stdout: &str) -> Vec<(String, String)> {
    let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(stdout) else {
        return vec![];
    };
    rows.into_iter()
        .filter_map(|row| {
            let name = row
                .get("Names")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                return None;
            }
            let state = row
                .get("State")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some((name, state))
        })
        .collect()
}

/// Run a hypervisor query command, returning its stdout on success.
/// `None` when the binary is absent or the command fails — the caller
/// treats that as "this hypervisor isn't available here".
async fn run_query(program: &str, args: &[&str]) -> Option<String> {
    let output = tokio::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Enumerate local KVM domains + Podman containers in one pass. Each
/// source is queried independently so a missing tool degrades to "skip
/// that source" rather than failing the whole list.
async fn enumerate() -> Enumeration {
    let mut instances = Vec::new();
    let mut sources = Vec::new();

    if let Some(stdout) = run_query("virsh", &["-c", "qemu:///system", "list", "--all"]).await {
        sources.push("virsh");
        for (name, state) in parse_virsh_list_state(&stdout) {
            instances.push(Instance {
                name,
                kind: InstanceKind::Vm,
                state,
            });
        }
    }
    if let Some(stdout) = run_query("podman", &["ps", "--all", "--format", "json"]).await {
        sources.push("podman");
        for (name, state) in parse_podman_ps(&stdout) {
            instances.push(Instance {
                name,
                kind: InstanceKind::Container,
                state,
            });
        }
    }

    Enumeration { instances, sources }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_virsh_list_table() {
        let out = " Id   Name        State\n\
                    -------------------------------\n\
                     1    fedora-vm   running\n\
                     -    build-box   shut off\n";
        let got = parse_virsh_list_state(out);
        assert_eq!(
            got,
            vec![
                ("fedora-vm".to_string(), "running".to_string()),
                ("build-box".to_string(), "shut off".to_string()),
            ]
        );
    }

    #[test]
    fn virsh_parser_skips_header_and_rules() {
        // Header ("Id …") + the dashed rule must not become rows.
        let out = " Id   Name   State\n----\n";
        assert!(parse_virsh_list_state(out).is_empty());
    }

    #[test]
    fn parses_podman_ps_json_names_and_state() {
        let out = r#"[{"Names":["web"],"State":"running"},{"Names":["db"],"State":"exited"}]"#;
        let got = parse_podman_ps(out);
        assert_eq!(
            got,
            vec![
                ("web".to_string(), "running".to_string()),
                ("db".to_string(), "exited".to_string()),
            ]
        );
    }

    #[test]
    fn podman_parser_returns_empty_on_garbage() {
        assert!(parse_podman_ps("not json").is_empty());
        assert!(parse_podman_ps("").is_empty());
    }

    #[test]
    fn running_and_paused_state_detection() {
        assert!(state_is_running("running"));
        assert!(state_is_running("RUNNING"));
        assert!(!state_is_running("shut off"));
        assert!(state_is_paused("paused"));
        assert!(state_is_paused("suspended"));
        assert!(!state_is_paused("running"));
    }

    #[test]
    fn status_line_distinguishes_no_hypervisor_from_empty() {
        let none = Enumeration::default();
        assert!(status_line(&none).contains("No local hypervisor"));

        let empty_but_present = Enumeration {
            instances: vec![],
            sources: vec!["virsh"],
        };
        assert!(status_line(&empty_but_present).contains("0 instances"));

        let one = Enumeration {
            instances: vec![Instance {
                name: "vm".into(),
                kind: InstanceKind::Vm,
                state: "running".into(),
            }],
            sources: vec!["virsh", "podman"],
        };
        let s = status_line(&one);
        assert!(s.contains("1 instance"), "{s}");
        assert!(s.contains("virsh + podman"), "{s}");
    }

    #[test]
    fn loaded_message_populates_and_marks_loaded() {
        let mut panel = ComputePanel::new();
        let _ = panel.update(Message::Loaded(Enumeration {
            instances: vec![Instance {
                name: "fedora-vm".into(),
                kind: InstanceKind::Vm,
                state: "running".into(),
            }],
            sources: vec!["virsh"],
        }));
        assert_eq!(panel.instances().len(), 1);
        assert!(panel.status().contains("1 instance"));
    }

    #[test]
    fn view_constructs_for_empty_and_populated() {
        // Empty (pre-load) and populated states both render without panic.
        let empty = ComputePanel::new();
        let _: Element<'_, crate::Message> = empty.view();

        let mut populated = ComputePanel::new();
        let _ = populated.update(Message::Loaded(Enumeration {
            instances: vec![Instance {
                name: "web".into(),
                kind: InstanceKind::Container,
                state: "running".into(),
            }],
            sources: vec!["podman"],
        }));
        let _: Element<'_, crate::Message> = populated.view();
    }
}
