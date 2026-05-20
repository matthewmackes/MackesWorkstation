//! Demo data — Rust port of FM_* const arrays in the prototype.
//!
//! These exist so the Iced app renders a complete UI before `mded` (Phase A of
//! v2.0.0) is wired in. When the daemon is ready, replace these by a `Backend`
//! trait implementation backed by zbus subscriptions.

use crate::model::{
    FileRow, LocalPin, Mime, Peer, PeerKind, PeerStatus, PinIcon, SelfNode, Transfer, TxDir,
};

pub const SELF_NODE: SelfNode = SelfNode {
    id: "yew",
    host: "yew.mesh",
    label: "this node",
    addr: "10.0.7.1",
    files: 1284,
    shared: 38,
};

pub const PEERS: &[Peer] = &[
    Peer {
        id: "pine",
        host: "pine.mesh",
        label: "matthew · workstation",
        kind: PeerKind::Desktop,
        addr: "10.0.7.14",
        status: PeerStatus::Online,
        latency: Some(14),
        files: 4912,
        shared: 211,
        last: "now",
        derp: "fra",
    },
    Peer {
        id: "birch",
        host: "birch.mesh",
        label: "home server · NAS",
        kind: PeerKind::Server,
        addr: "10.0.7.22",
        status: PeerStatus::Online,
        latency: Some(41),
        files: 18_403,
        shared: 1842,
        last: "12 s",
        derp: "ord",
    },
    Peer {
        id: "oak",
        host: "oak.mesh",
        label: "matt-phone",
        kind: PeerKind::Phone,
        addr: "10.0.7.41",
        status: PeerStatus::Idle,
        latency: Some(220),
        files: 612,
        shared: 4,
        last: "3 min",
        derp: "fra",
    },
    Peer {
        id: "cedar",
        host: "cedar.mesh",
        label: "CI · build runner",
        kind: PeerKind::Server,
        addr: "10.0.7.51",
        status: PeerStatus::Offline,
        latency: None,
        files: 0,
        shared: 0,
        last: "2 h ago",
        derp: "—",
    },
];

pub const RECENT_TRANSFERS: &[Transfer] = &[
    Transfer {
        dir: TxDir::In,
        name: "map2-release-v0.4.2.tar.zst",
        peer: "cedar.mesh",
        size: "14.2 MB",
        age: "12 s",
    },
    Transfer {
        dir: TxDir::Out,
        name: "design-notes.md",
        peer: "pine.mesh",
        size: "8 KB",
        age: "4 min",
    },
    Transfer {
        dir: TxDir::In,
        name: "kitchen-IMG_5611.jpg",
        peer: "oak.mesh",
        size: "3.8 MB",
        age: "14 min",
    },
    Transfer {
        dir: TxDir::In,
        name: "projector-warranty.pdf",
        peer: "birch.mesh",
        size: "210 KB",
        age: "1 h",
    },
    Transfer {
        dir: TxDir::Out,
        name: "screenshots/2026-05-19.zip",
        peer: "birch.mesh",
        size: "22.1 MB",
        age: "2 h",
    },
];

pub const INBOX: &[FileRow] = &[
    FileRow {
        name: "map2-release-v0.4.2.tar.zst",
        mime: Mime::Archive,
        size: "14.2 MB",
        age: "12 s",
        mesh: None,
        from: Some("cedar.mesh"),
    },
    FileRow {
        name: "meeting-notes-2026-05-18.md",
        mime: Mime::Doc,
        size: "4 KB",
        age: "6 min",
        mesh: None,
        from: Some("pine.mesh"),
    },
    FileRow {
        name: "kitchen-IMG_5611.jpg",
        mime: Mime::Image,
        size: "3.8 MB",
        age: "14 min",
        mesh: None,
        from: Some("oak.mesh"),
    },
    FileRow {
        name: "projector-warranty.pdf",
        mime: Mime::Pdf,
        size: "210 KB",
        age: "1 h",
        mesh: None,
        from: Some("birch.mesh"),
    },
    FileRow {
        name: "birch-photos-april/",
        mime: Mime::Folder,
        size: "— · 412 items",
        age: "3 h",
        mesh: None,
        from: Some("birch.mesh"),
    },
    FileRow {
        name: "pine-clipboard.txt",
        mime: Mime::Doc,
        size: "1 KB",
        age: "4 h",
        mesh: None,
        from: Some("pine.mesh"),
    },
];

pub const DOWNLOADS: &[FileRow] = &[
    FileRow {
        name: "cargo-1.87.0-x86_64-unknown-linux-gnu.tar.xz",
        mime: Mime::Archive,
        size: "38.4 MB",
        age: "5 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "map2-release-v0.4.2.tar.zst",
        mime: Mime::Archive,
        size: "14.2 MB",
        age: "12 s",
        mesh: Some("cedar.mesh"),
        from: None,
    },
    FileRow {
        name: "fedora-coreos-aarch64.qcow2",
        mime: Mime::Disk,
        size: "684 MB",
        age: "2 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "meeting-notes-2026-05-18.md",
        mime: Mime::Doc,
        size: "4 KB",
        age: "6 min",
        mesh: Some("pine.mesh"),
        from: None,
    },
    FileRow {
        name: "screenshot-2026-05-19-08-52-56.png",
        mime: Mime::Image,
        size: "218 KB",
        age: "3 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "kitchen-IMG_5611.jpg",
        mime: Mime::Image,
        size: "3.8 MB",
        age: "14 min",
        mesh: Some("oak.mesh"),
        from: None,
    },
    FileRow {
        name: "projector-warranty.pdf",
        mime: Mime::Pdf,
        size: "210 KB",
        age: "1 h",
        mesh: Some("birch.mesh"),
        from: None,
    },
    FileRow {
        name: "map2-panel-screenshot.png",
        mime: Mime::Image,
        size: "512 KB",
        age: "20 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "rust-1.87.0-src.tar.gz",
        mime: Mime::Archive,
        size: "186 MB",
        age: "1 d",
        mesh: None,
        from: None,
    },
];

pub const PINE_FILES: &[FileRow] = &[
    FileRow {
        name: "~mesh/",
        mime: Mime::Folder,
        size: "— · 38 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "screenshots/",
        mime: Mime::Folder,
        size: "— · 122 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "design-notes.md",
        mime: Mime::Doc,
        size: "8 KB",
        age: "4 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "meeting-notes-2026-05-18.md",
        mime: Mime::Doc,
        size: "4 KB",
        age: "6 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "map2-panel-mockup.fig",
        mime: Mime::Doc,
        size: "1.4 MB",
        age: "1 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "pine-clipboard.txt",
        mime: Mime::Doc,
        size: "1 KB",
        age: "4 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "desktop.jpg",
        mime: Mime::Image,
        size: "2.2 MB",
        age: "1 d",
        mesh: None,
        from: None,
    },
];

pub const BIRCH_FILES: &[FileRow] = &[
    FileRow {
        name: "~mesh/",
        mime: Mime::Folder,
        size: "— · 1842 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "family-photos/",
        mime: Mime::Folder,
        size: "— · 14.2k items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "media/",
        mime: Mime::Folder,
        size: "— · 612 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "backups/",
        mime: Mime::Folder,
        size: "— · 211 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "projector-warranty.pdf",
        mime: Mime::Pdf,
        size: "210 KB",
        age: "1 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "fedora-coreos-aarch64.qcow2",
        mime: Mime::Disk,
        size: "684 MB",
        age: "2 h",
        mesh: None,
        from: None,
    },
];

pub const OAK_FILES: &[FileRow] = &[
    FileRow {
        name: "Camera/",
        mime: Mime::Folder,
        size: "— · 412 items",
        age: "—",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "kitchen-IMG_5611.jpg",
        mime: Mime::Image,
        size: "3.8 MB",
        age: "14 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "voice-memo-2026-05-19.m4a",
        mime: Mime::Doc,
        size: "420 KB",
        age: "20 min",
        mesh: None,
        from: None,
    },
];

pub const LOCAL_PINS: &[LocalPin] = &[
    LocalPin {
        id: "home",
        name: "Home",
        path: "~/",
        icon: PinIcon::Home,
    },
    LocalPin {
        id: "docs",
        name: "Documents",
        path: "~/Documents",
        icon: PinIcon::Doc2,
    },
    LocalPin {
        id: "pics",
        name: "Pictures",
        path: "~/Pictures",
        icon: PinIcon::Image,
    },
    LocalPin {
        id: "music",
        name: "Music",
        path: "~/Music",
        icon: PinIcon::Doc,
    },
    LocalPin {
        id: "videos",
        name: "Videos",
        path: "~/Videos",
        icon: PinIcon::Player,
    },
    LocalPin {
        id: "code",
        name: "Code",
        path: "~/code",
        icon: PinIcon::Rust,
    },
    LocalPin {
        id: "root",
        name: "Filesystem",
        path: "/",
        icon: PinIcon::Hdd,
    },
    LocalPin {
        id: "trash",
        name: "Trash",
        path: "empty",
        icon: PinIcon::Trash,
    },
];

pub const LOCAL_RECENT: &[FileRow] = &[
    FileRow {
        name: ".bashrc",
        mime: Mime::Doc,
        size: "3 KB",
        age: "2 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "Documents/journal.md",
        mime: Mime::Doc,
        size: "14 KB",
        age: "5 h",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "code/map2/",
        mime: Mime::Folder,
        size: "— · 312 items",
        age: "12 min",
        mesh: None,
        from: None,
    },
    FileRow {
        name: "Pictures/wallpapers/",
        mime: Mime::Folder,
        size: "— · 28 items",
        age: "1 d",
        mesh: None,
        from: None,
    },
];

/// Files shared by a peer, looked up by `peer.id`. Empty slice for unknown ids.
#[must_use]
pub fn peer_files(id: &str) -> &'static [FileRow] {
    match id {
        "pine" => PINE_FILES,
        "birch" => BIRCH_FILES,
        "oak" => OAK_FILES,
        _ => &[],
    }
}

/// How many peers are online right now (used in banners + sidebar header).
#[must_use]
pub fn online_count() -> usize {
    PEERS
        .iter()
        .filter(|p| p.status == PeerStatus::Online)
        .count()
}

/// Sum of `shared` across self + all peers — the "Shared" stat in the banner.
#[must_use]
pub fn total_shared() -> u64 {
    u64::from(SELF_NODE.shared) + PEERS.iter().map(|p| u64::from(p.shared)).sum::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_peers_three_active() {
        assert_eq!(PEERS.len(), 4);
        let active = PEERS
            .iter()
            .filter(|p| !matches!(p.status, PeerStatus::Offline))
            .count();
        assert_eq!(active, 3);
    }

    #[test]
    fn online_count_matches_prototype() {
        assert_eq!(online_count(), 2);
    }

    #[test]
    fn total_shared_matches_prototype() {
        assert_eq!(total_shared(), 38 + 211 + 1842 + 4 + 0);
    }

    #[test]
    fn downloads_mesh_count_matches_prototype() {
        let mesh_arrived = DOWNLOADS.iter().filter(|d| d.mesh.is_some()).count();
        assert_eq!(mesh_arrived, 4);
    }

    #[test]
    fn peer_files_lookup_returns_known_peer_files() {
        assert_eq!(peer_files("pine").len(), PINE_FILES.len());
        assert_eq!(peer_files("birch").len(), BIRCH_FILES.len());
        assert_eq!(peer_files("oak").len(), OAK_FILES.len());
        assert!(peer_files("cedar").is_empty());
        assert!(peer_files("nonexistent").is_empty());
    }
}
