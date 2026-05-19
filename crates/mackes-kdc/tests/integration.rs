//! Integration tests — Phase 13.6.
//!
//! These tests exercise the file-format contract the
//! `mackesd-kdc-bridge` daemon (Phase 13.2.1, deferred) and the
//! Workbench Connect panels (Phase 13.3.x, deferred) both depend on.
//! No Avahi binding is required — the bridge's wire surface is a
//! per-peer JSONL file under `~/QNM-Shared/<peer>/kdc/announce.jsonl`,
//! so a temp directory + the `mackes_kdc` value types are enough to
//! pin the format end-to-end.
//!
//! When 13.2.1 lands its daemon implementation will be unit-tested
//! against the *same* fixtures so the bridge can't drift from the
//! schema captured here.

use mackes_kdc::{paired_device_ids, Device, DeviceKind, MirroredNotification};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Build a JSONL announce file the way the mDNS bridge would —
/// one JSON record per line, no trailing comma, terminated with a
/// final newline. Returns the absolute path written.
fn write_announce_jsonl(root: &Path, peer: &str, devices: &[Device]) -> PathBuf {
    let dir = root.join(peer).join("kdc");
    fs::create_dir_all(&dir).expect("mkdir announce dir");
    let path = dir.join("announce.jsonl");
    let mut f = fs::File::create(&path).expect("create announce.jsonl");
    for d in devices {
        let line = serde_json::to_string(d).expect("serialize Device");
        writeln!(f, "{line}").expect("write announce line");
    }
    path
}

/// Parse a JSONL announce file back into a `Vec<Device>` — mirrors
/// what the mDNS bridge will do on its read side. Skips blank lines
/// so an empty file (peer offline) yields an empty vec instead of an
/// error.
fn read_announce_jsonl(path: &Path) -> Vec<Device> {
    let raw = fs::read_to_string(path).expect("read announce.jsonl");
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse Device line"))
        .collect()
}

// ---------------------------------------------------------------- //
// Bridge file-format round-trip tests.
// ---------------------------------------------------------------- //

#[test]
fn bridge_announce_jsonl_round_trips_a_single_phone() {
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();

    let pixel = Device {
        id: "a1b2c3d4e5f6a1b2".into(),
        name: "Pixel 8".into(),
        kind: DeviceKind::Phone,
        reachable: true,
        battery_pct: Some(73),
        last_seen_s: 1_700_000_000,
    };

    let path = write_announce_jsonl(&qnm_shared, "peer-anvil", std::slice::from_ref(&pixel));
    let back = read_announce_jsonl(&path);

    assert_eq!(back.len(), 1, "one announce line in, one out");
    assert_eq!(back[0], pixel, "device round-trips verbatim through JSONL");
}

#[test]
fn bridge_announce_jsonl_handles_a_mixed_fleet_per_peer() {
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();

    let fleet = vec![
        Device {
            id: "0123456789abcdef0123456789abcdef".into(),
            name: "Pixel 8".into(),
            kind: DeviceKind::Phone,
            reachable: true,
            battery_pct: Some(73),
            last_seen_s: 1_700_000_000,
        },
        Device {
            id: "deadbeefcafebabe1234".into(),
            name: "iPad mini".into(),
            kind: DeviceKind::Tablet,
            reachable: false,
            battery_pct: Some(12),
            last_seen_s: 1_699_900_000,
        },
        Device {
            id: "fedcba9876543210fedcba9876543210".into(),
            name: "Frame.work 13".into(),
            kind: DeviceKind::Desktop,
            reachable: true,
            battery_pct: None,
            last_seen_s: 1_700_000_500,
        },
        Device {
            id: "0fdb8a73c2e14c7d9b6a".into(),
            name: "Garmin Fenix".into(),
            kind: DeviceKind::Unknown,
            reachable: false,
            battery_pct: Some(54),
            last_seen_s: 1_699_800_000,
        },
    ];

    let path = write_announce_jsonl(&qnm_shared, "peer-anvil", &fleet);
    let back = read_announce_jsonl(&path);

    assert_eq!(back.len(), fleet.len(), "every record round-trips");
    for (orig, parsed) in fleet.iter().zip(back.iter()) {
        assert_eq!(orig, parsed, "device {} survived the JSONL trip", orig.id);
    }

    // Verify the on-disk file has exactly one line per device — the
    // bridge reads line-by-line and a missing newline would silently
    // drop the last entry.
    let raw = fs::read_to_string(&path).expect("read raw");
    let line_count = raw.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(line_count, fleet.len());
}

#[test]
fn bridge_announce_directory_enumerates_every_peer() {
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();

    // Three peers each announcing one phone.
    let peers = ["peer-anvil", "peer-forge", "peer-quench"];
    for (idx, peer) in peers.iter().enumerate() {
        let id = format!("a1b2c3d4e5f6{idx:04x}");
        let d = Device {
            id: id.clone(),
            name: format!("Phone {peer}"),
            kind: DeviceKind::Phone,
            reachable: true,
            battery_pct: Some(80),
            last_seen_s: 1_700_000_000 + i64::try_from(idx).expect("peer idx fits i64"),
        };
        write_announce_jsonl(&qnm_shared, peer, std::slice::from_ref(&d));
    }

    // Walk QNM-Shared/<peer>/kdc/announce.jsonl the way the bridge
    // does — every directory that contains a populated announce file
    // is a candidate for re-announcement on the local LAN.
    let mut announced: Vec<(String, Device)> = Vec::new();
    for entry in fs::read_dir(&qnm_shared).expect("read qnm-shared") {
        let entry = entry.expect("dir entry");
        let peer_name = entry.file_name().to_string_lossy().to_string();
        let announce = entry.path().join("kdc").join("announce.jsonl");
        if announce.is_file() {
            for d in read_announce_jsonl(&announce) {
                announced.push((peer_name.clone(), d));
            }
        }
    }
    announced.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(announced.len(), peers.len(), "every peer surfaced");
    let observed_peers: Vec<&str> = announced.iter().map(|(p, _)| p.as_str()).collect();
    for p in peers {
        assert!(observed_peers.contains(&p), "peer {p} present");
    }
}

#[test]
fn bridge_empty_announce_file_is_handled_as_peer_offline() {
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();

    // Peer wrote an empty file (no devices currently mirrored) —
    // either because it just came up or because all its phones are
    // unreachable. The bridge must treat this as "zero announcements"
    // not as a parse error.
    let dir = qnm_shared.join("peer-anvil").join("kdc");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("announce.jsonl");
    fs::write(&path, "").expect("write empty");

    let back = read_announce_jsonl(&path);
    assert!(back.is_empty(), "empty file ⇒ empty fleet, not error");
}

#[test]
fn bridge_announce_skips_blank_lines_between_records() {
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();

    // Compose a JSONL file with deliberate blank lines — appenders
    // sometimes leave them in. The reader must skip cleanly.
    let dir = qnm_shared.join("peer-anvil").join("kdc");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("announce.jsonl");

    let one = Device {
        id: "a1b2c3d4e5f6a1b2".into(),
        name: "Pixel".into(),
        kind: DeviceKind::Phone,
        reachable: true,
        battery_pct: Some(50),
        last_seen_s: 1_700_000_000,
    };
    let two = Device {
        id: "1111111111111111".into(),
        name: "Older Pixel".into(),
        kind: DeviceKind::Phone,
        reachable: false,
        battery_pct: None,
        last_seen_s: 1_699_900_000,
    };

    let body = format!(
        "{}\n\n{}\n",
        serde_json::to_string(&one).expect("serialize one"),
        serde_json::to_string(&two).expect("serialize two"),
    );
    fs::write(&path, body).expect("write");

    let back = read_announce_jsonl(&path);
    assert_eq!(back.len(), 2);
    assert_eq!(back[0], one);
    assert_eq!(back[1], two);
}

// ---------------------------------------------------------------- //
// First-launch import flow — `paired_device_ids` against a temp
// `~/.config/kdeconnect/` tree.
// ---------------------------------------------------------------- //

#[test]
fn paired_device_ids_imports_uuid_dirs_skipping_state_dirs() {
    let tmp = TempDir::new().expect("tempdir");
    let fake_home = tmp.path().to_path_buf();
    let kdc = fake_home.join(".config").join("kdeconnect");
    fs::create_dir_all(&kdc).expect("mkdir kdc");

    // Three real pairing dirs (UUID-shaped) — should all import.
    let real_uuids = [
        "a1b2c3d4e5f6a1b2",
        "0123456789abcdef0123456789abcdef",
        "deadbeefcafebabe1234",
    ];
    for u in real_uuids {
        fs::create_dir(kdc.join(u)).expect("mkdir uuid");
    }

    // KDE Connect's own state dirs — must NOT show up in the
    // imported pairing list.
    for state in ["config", "cache", "log", "trusted_devices"] {
        fs::create_dir(kdc.join(state)).expect("mkdir state");
    }

    // Plus a stray file (not a dir) — must be ignored.
    fs::write(kdc.join("daemon.lock"), "pid:1234").expect("touch file");

    // Point HOME at our fake tree, run the importer, then restore.
    let prior = std::env::var_os("HOME");
    std::env::set_var("HOME", &fake_home);
    let ids = paired_device_ids();
    match prior {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }

    // Order is filesystem-dependent — compare as sorted sets.
    let mut got = ids;
    got.sort();
    let mut want: Vec<String> = real_uuids.iter().map(|s| (*s).to_string()).collect();
    want.sort();
    assert_eq!(
        got, want,
        "importer must surface every UUID-shaped pairing dir and only those",
    );
}

#[test]
fn mirrored_notification_announce_jsonl_round_trips() {
    // The bridge mirrors notifications via the same JSONL pattern as
    // device announcements — separate file
    // (`~/QNM-Shared/<peer>/kdc/notifications.jsonl`) but identical
    // codec contract.
    let tmp = TempDir::new().expect("tempdir");
    let qnm_shared = tmp.path().to_path_buf();
    let dir = qnm_shared.join("peer-anvil").join("kdc");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("notifications.jsonl");

    let notifications = vec![
        MirroredNotification {
            device_id: "a1b2c3d4e5f6a1b2".into(),
            notification_id: "notif-1".into(),
            app: "org.signal.Signal".into(),
            title: "Alice".into(),
            text: "On my way home — 10 min out.".into(),
            at_s: 1_700_000_010,
        },
        MirroredNotification {
            device_id: "a1b2c3d4e5f6a1b2".into(),
            notification_id: "notif-2".into(),
            app: "com.google.android.gm".into(),
            title: "GitHub".into(),
            text: "[matthewmackes/MAP2-RELEASES] PR #42 ready for review".into(),
            at_s: 1_700_000_020,
        },
    ];

    let mut body = String::new();
    for n in &notifications {
        body.push_str(&serde_json::to_string(n).expect("serialize"));
        body.push('\n');
    }
    fs::write(&path, &body).expect("write");

    // Read back line-by-line, parse each, and verify equality.
    let raw = fs::read_to_string(&path).expect("read");
    let parsed: Vec<MirroredNotification> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse"))
        .collect();

    assert_eq!(parsed.len(), notifications.len());
    for (orig, p) in notifications.iter().zip(parsed.iter()) {
        assert_eq!(orig, p, "notification {} round-trips", orig.notification_id);
    }
}
