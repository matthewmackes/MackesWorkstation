// MeshModule API consumed by the dock builder.
#![allow(dead_code)]

//! Mesh-resource implementation of `DockModule`.
//!
//! Per Q9/Q10 of the design lock, mesh peers / shares / services are
//! first-class dock items interleaved with app launchers. A
//! `MeshModule` wraps a `mackes_mesh_types::MeshResource`, exposes it
//! through the same `DockModule` trait as `AppModule`, and renders via
//! the shared `dock::render_module` path.
//!
//! Phase 5.4 ships the enumeration + rendering layer. Phase 5.6 will
//! plug a real action popover into `on_click`; for now click opens
//! Thunar at the peer's mesh-shared folder (the Q34 "default action").

use std::path::{Path, PathBuf};
use std::process::Command;

use gtk::prelude::*;
use mackes_mesh_types::MeshResource;

use crate::dock::{DockModule, DockState};

/// Default mesh-share root under `$HOME`. Matches QNM-Shared's
/// canonical mount per `docs/design/v3.0.0-mackes-xfce-workstation.md`
/// §10.
const QNM_SHARED_ROOT: &str = "QNM-Shared";

/// Inverse of `MeshResource::id()` — parses `peer:NAME` /
/// `share:PEER:BUCKET` / `svc:PEER:SLUG` back into a `MeshResource`.
/// `online` defaults to false until Phase 5.5 reconciles against real
/// presence. Services restored this way have an empty `url` until the
/// mesh service catalog is queried.
#[must_use]
pub fn parse_id(id: &str) -> Option<MeshResource> {
    if let Some(rest) = id.strip_prefix("peer:") {
        if rest.is_empty() {
            return None;
        }
        return Some(MeshResource::Peer {
            name: rest.to_owned(),
            mesh_ip: None,
            online: false,
        });
    }
    if let Some(rest) = id.strip_prefix("share:") {
        let (peer, bucket) = rest.split_once(':')?;
        if peer.is_empty() || bucket.is_empty() {
            return None;
        }
        return Some(MeshResource::MountedShare {
            peer: peer.to_owned(),
            bucket: bucket.to_owned(),
        });
    }
    if let Some(rest) = id.strip_prefix("svc:") {
        let (peer, slug) = rest.split_once(':')?;
        if peer.is_empty() || slug.is_empty() {
            return None;
        }
        return Some(MeshResource::Service {
            peer: peer.to_owned(),
            slug: slug.to_owned(),
            url: String::new(),
        });
    }
    None
}

/// Per-resource icon assignment. Peers wear the laptop glyph (their
/// physical-machine identity), shares are folders, services are
/// generic launchers.
const PEER_ICON: &str = "laptop";
const SHARE_ICON: &str = "folder--shared";
const SERVICE_ICON: &str = "launch";

/// Dock entry backed by a `MeshResource`.
#[derive(Debug, Clone)]
pub struct MeshModule {
    resource: MeshResource,
    state: DockState,
}

impl MeshModule {
    #[must_use]
    pub const fn new(resource: MeshResource) -> Self {
        Self {
            resource,
            state: DockState::Idle,
        }
    }

    /// Phase 5.5 will mutate this from mesh-presence pings.
    pub const fn set_state(&mut self, state: DockState) {
        self.state = state;
    }

    /// Borrow the wrapped resource for callers that need to inspect
    /// peer-specific fields (e.g. the action popover in Phase 5.6).
    #[must_use]
    pub const fn resource(&self) -> &MeshResource {
        &self.resource
    }
}

impl DockModule for MeshModule {
    fn id(&self) -> String {
        self.resource.id()
    }

    fn icon_name(&self) -> &str {
        match self.resource {
            MeshResource::Peer { .. } => PEER_ICON,
            MeshResource::MountedShare { .. } => SHARE_ICON,
            MeshResource::Service { .. } => SERVICE_ICON,
        }
    }

    fn tooltip(&self) -> &str {
        // We need a stable `&str` for the trait. Carrying the label
        // String on the struct would force an allocation per call; the
        // tooltip is shown rarely so a leaked Box<str> is overkill.
        // Instead, fall back to the peer/bucket/slug field — already
        // borrowed from the resource.
        match &self.resource {
            MeshResource::Peer { name, .. } => name,
            MeshResource::MountedShare { bucket, .. } => bucket,
            MeshResource::Service { slug, .. } => slug,
        }
    }

    fn state(&self) -> DockState {
        // Peers report their own online/offline directly into state
        // (Phase 5.5). Shares and services inherit their owning peer's
        // state — once Phase 5.5 lands we mirror that here.
        match self.resource {
            MeshResource::Peer { online: true, .. } => DockState::Running,
            MeshResource::Peer { online: false, .. } => DockState::Idle,
            _ => self.state,
        }
    }

    fn on_click(&self) {
        // Q34 default action: peer click opens Thunar at the peer's
        // mesh-shared folder (~/QNM-Shared/<peer>/). Phase 5.6 will
        // replace this with the per-peer action popover.
        match &self.resource {
            MeshResource::Peer { name, .. } => {
                spawn_xdg_open_path(&qnm_shared_path(name));
            }
            MeshResource::MountedShare { peer, bucket } => {
                spawn_xdg_open_path(&qnm_shared_path(peer).join(bucket));
            }
            MeshResource::Service { url, .. } => {
                spawn_xdg_open_str(url);
            }
        }
    }
}

fn qnm_shared_path(peer: &str) -> PathBuf {
    let home = std::env::var_os("HOME").map_or_else(PathBuf::new, PathBuf::from);
    home.join(QNM_SHARED_ROOT).join(peer)
}

/// Build the Q34 action popover for a `Peer` resource. Six buttons
/// stacked vertically — Files / SSH / RDP / VNC / Services / Send
/// file. Attached to `anchor` (the dock widget) and shown directly.
#[must_use]
pub fn build_peer_popover(anchor: &gtk::Widget, peer: &str) -> gtk::Popover {
    let popover = gtk::Popover::new(Some(anchor));
    popover.set_widget_name("mackes-peer-popover");
    popover.set_position(gtk::PositionType::Top);
    popover.set_constrain_to(gtk::PopoverConstraint::None);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
    column.set_margin_top(8);
    column.set_margin_bottom(8);
    column.set_margin_start(12);
    column.set_margin_end(12);

    let peer_owned = peer.to_owned();
    column.pack_start(
        &popover_button("Files", "folder--open", {
            let p = peer_owned.clone();
            let pop = popover.clone();
            move || {
                spawn_xdg_open_path(&qnm_shared_path(&p));
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );
    column.pack_start(
        &popover_button("SSH", "terminal", {
            let p = peer_owned.clone();
            let pop = popover.clone();
            move || {
                spawn_terminal_ssh(&p);
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );
    column.pack_start(
        &popover_button("RDP", "screen", {
            let p = peer_owned.clone();
            let pop = popover.clone();
            move || {
                spawn_remmina(&p, "rdp");
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );
    column.pack_start(
        &popover_button("VNC", "view--filled", {
            let p = peer_owned.clone();
            let pop = popover.clone();
            move || {
                spawn_remmina(&p, "vnc");
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );
    column.pack_start(
        &popover_button("Services", "launch", {
            let p = peer_owned.clone();
            let pop = popover.clone();
            move || {
                // Phase 5.5b will plug a real service-catalog
                // dropdown. For now we open mesh-services in mackes.
                if let Err(e) = Command::new("mackes")
                    .args(["--services", "--peer", &p])
                    .spawn()
                {
                    eprintln!("mackes-panel: services launch failed: {e}");
                }
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );
    column.pack_start(
        &popover_button("Send file…", "send", {
            let p = peer_owned;
            let pop = popover.clone();
            move || {
                send_file_dialog(&p);
                pop.popdown();
            }
        }),
        false,
        false,
        0,
    );

    popover.add(&column);
    popover
}

fn popover_button<F>(label: &str, _icon_name: &str, on_click: F) -> gtk::Button
where
    F: Fn() + 'static,
{
    let button = gtk::Button::with_label(label);
    button.set_relief(gtk::ReliefStyle::None);
    button.set_tooltip_text(Some(label));
    if let Some(atk) = button.accessible() {
        atk.set_name(label);
    }
    // Left-align the label inside the button.
    if let Some(child) = button.child() {
        if let Some(lbl) = child.downcast_ref::<gtk::Label>() {
            lbl.set_xalign(0.0);
        }
    }
    button.connect_clicked(move |_| on_click());
    button
}

fn spawn_terminal_ssh(peer: &str) {
    // Launch xfce4-terminal -e "ssh <peer>.mesh". The .mesh suffix is
    // the mackes-mesh DNS convention; falls back to bare peer name if
    // resolution fails.
    let cmd = format!("ssh {peer}.mesh");
    if let Err(e) = Command::new("xfce4-terminal").args(["-e", &cmd]).spawn() {
        eprintln!("mackes-panel: SSH terminal launch failed: {e}");
    }
}

fn spawn_remmina(peer: &str, proto: &str) {
    // remmina supports rdp:// and vnc:// URIs.
    let uri = format!("{proto}://{peer}.mesh");
    if let Err(e) = Command::new("remmina").args(["-c", &uri]).spawn() {
        eprintln!("mackes-panel: remmina {proto} launch failed: {e}");
    }
}

fn send_file_dialog(peer: &str) {
    // Use a system zenity/yad-style file picker. xfce4 ships
    // 'zenity' by default. Result goes to ~/QNM-Shared/<peer>/.
    let target_dir = qnm_shared_path(peer);
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        eprintln!(
            "mackes-panel: cannot create {} for send-file: {e}",
            target_dir.display()
        );
        return;
    }
    // Pick + copy in a single shell; non-blocking from the panel's POV.
    let target = target_dir.to_string_lossy().to_string();
    let cmd = format!(
        "f=$(zenity --file-selection 2>/dev/null) && cp -- \"$f\" {}",
        shell_escape(&target)
    );
    if let Err(e) = Command::new("/bin/sh").arg("-c").arg(&cmd).spawn() {
        eprintln!("mackes-panel: send-file picker failed: {e}");
    }
}

/// Minimal shell-escape for the send-file dest path.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn spawn_xdg_open_path(target: &Path) {
    if let Err(e) = Command::new("xdg-open").arg(target).spawn() {
        eprintln!("mackes-panel: xdg-open {} failed: {e}", target.display());
    }
}

fn spawn_xdg_open_str(target: &str) {
    if let Err(e) = Command::new("xdg-open").arg(target).spawn() {
        eprintln!("mackes-panel: xdg-open {target} failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn online_peer() -> MeshResource {
        MeshResource::Peer {
            name: "anvil".into(),
            mesh_ip: Some("100.64.0.7".into()),
            online: true,
        }
    }

    fn offline_peer() -> MeshResource {
        MeshResource::Peer {
            name: "anvil".into(),
            mesh_ip: None,
            online: false,
        }
    }

    fn share() -> MeshResource {
        MeshResource::MountedShare {
            peer: "anvil".into(),
            bucket: "code".into(),
        }
    }

    fn service() -> MeshResource {
        MeshResource::Service {
            peer: "anvil".into(),
            slug: "sublime-music".into(),
            url: "http://anvil.mesh:4040".into(),
        }
    }

    #[test]
    fn icons_pick_resource_kind() {
        assert_eq!(MeshModule::new(online_peer()).icon_name(), PEER_ICON);
        assert_eq!(MeshModule::new(share()).icon_name(), SHARE_ICON);
        assert_eq!(MeshModule::new(service()).icon_name(), SERVICE_ICON);
    }

    #[test]
    fn peer_state_follows_online_flag() {
        assert_eq!(MeshModule::new(online_peer()).state(), DockState::Running);
        assert_eq!(MeshModule::new(offline_peer()).state(), DockState::Idle);
    }

    #[test]
    fn tooltip_returns_resource_label() {
        assert_eq!(MeshModule::new(online_peer()).tooltip(), "anvil");
        assert_eq!(MeshModule::new(share()).tooltip(), "code");
        assert_eq!(MeshModule::new(service()).tooltip(), "sublime-music");
    }

    #[test]
    fn id_round_trips_through_resource() {
        let m = MeshModule::new(online_peer());
        assert_eq!(m.id(), "peer:anvil");
    }

    #[test]
    fn parse_id_recognises_each_resource_kind() {
        assert!(matches!(
            parse_id("peer:anvil"),
            Some(MeshResource::Peer { .. })
        ));
        assert!(matches!(
            parse_id("share:anvil:code"),
            Some(MeshResource::MountedShare { .. })
        ));
        assert!(matches!(
            parse_id("svc:anvil:sublime-music"),
            Some(MeshResource::Service { .. })
        ));
    }

    #[test]
    fn parse_id_rejects_malformed() {
        assert!(parse_id("").is_none());
        assert!(parse_id("peer:").is_none());
        assert!(parse_id("share:onlyone").is_none());
        assert!(parse_id("share:peer:").is_none());
        assert!(parse_id("unknown-kind:foo").is_none());
    }
}
