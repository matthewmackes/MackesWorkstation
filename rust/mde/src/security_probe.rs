//! Security-posture data layer for the Windows 10 Security dashboard (E14.3).
//!
//! Pure parsers + live readers for the five real posture checks — firewall, disk
//! encryption, antivirus, Secure Boot, and the TPM — mirroring `nm.rs`/`sysinfo.rs`:
//! no iced dependency, so the parsers are unit-tested directly on captured fixture
//! strings (no live-tool dependence in tests). The Security surface (E14.4) maps
//! [`Level`] onto the `palette::STATUS_*` roles (E14.2).
//!
//! Headless entry: `mde __security-probe` prints each tile's title/level/status.

use std::process::Command;

/// A tile's posture: maps to `palette::STATUS_OK`/`STATUS_WARN`/`STATUS_RISK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Risk,
}

/// One Security-dashboard status tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub title: String,
    pub status: String,
    pub level: Level,
}

impl Tile {
    fn new(title: &str, level: Level, status: impl Into<String>) -> Self {
        Tile {
            title: title.into(),
            status: status.into(),
            level,
        }
    }
}

/// The five probed posture checks (the dashboard adds advisory tiles around them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityStatus {
    pub firewall: Tile,
    pub encryption: Tile,
    pub antivirus: Tile,
    pub secureboot: Tile,
    pub tpm: Tile,
}

// --- pure parsers (unit-tested on fixtures) --------------------------------

/// `firewall-cmd --state` → is firewalld running? ("running" vs anything else).
pub fn parse_firewall_state(s: &str) -> bool {
    s.trim() == "running"
}

/// `firewall-cmd --get-active-zones` → the active zone names (the non-indented
/// lines; the indented `interfaces:`/`sources:` lines belong to the zone above).
pub fn parse_active_zones(s: &str) -> Vec<String> {
    s.lines()
        .filter(|l| !l.is_empty() && !l.starts_with(char::is_whitespace))
        .map(|l| l.trim().to_string())
        .collect()
}

/// `mokutil --sb-state` → `Some(true)` enabled, `Some(false)` disabled, `None`
/// when Secure Boot is unsupported / EFI vars absent.
pub fn parse_sb_state(s: &str) -> Option<bool> {
    let s = s.to_ascii_lowercase();
    if s.contains("secureboot enabled") {
        Some(true)
    } else if s.contains("secureboot disabled") {
        Some(false)
    } else {
        None
    }
}

/// `clamscan --version` (e.g. `ClamAV 1.0.1/27000/Mon ...`) → the engine version.
pub fn parse_clamav_version(s: &str) -> Option<String> {
    s.trim()
        .strip_prefix("ClamAV ")
        .map(|rest| rest.split('/').next().unwrap_or(rest).trim().to_string())
        .filter(|v| !v.is_empty())
}

/// `lsblk -f` (or any `lsblk` FSTYPE listing) → is any block device LUKS-encrypted?
pub fn parse_luks(lsblk_out: &str) -> bool {
    lsblk_out.contains("crypto_LUKS")
}

/// `/sys/class/tpm/tpm0/tpm_version_major` content → a human TPM version
/// ("2" → "2.0", "1" → "1.2"); `None` for empty/unknown.
pub fn parse_tpm_version(s: &str) -> Option<String> {
    match s.trim() {
        "2" => Some("2.0".into()),
        "1" => Some("1.2".into()),
        "" => None,
        other => Some(other.to_string()),
    }
}

// --- live readers ----------------------------------------------------------

fn out(bin: &str, args: &[&str]) -> Option<String> {
    let o = Command::new(bin).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&o.stdout).to_string())
}

/// firewalld detail for the Firewall page (E14.5): running, default zone, and the
/// active zones (mapped to Win10 network profiles in the view).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FirewallDetail {
    pub running: bool,
    pub default_zone: String,
    pub zones: Vec<String>,
}

/// Read the live firewalld detail (running / default zone / active zones).
pub fn firewall_detail() -> FirewallDetail {
    FirewallDetail {
        running: out("firewall-cmd", &["--state"])
            .map(|s| parse_firewall_state(&s))
            .unwrap_or(false),
        default_zone: out("firewall-cmd", &["--get-default-zone"])
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
        zones: out("firewall-cmd", &["--get-active-zones"])
            .map(|s| parse_active_zones(&s))
            .unwrap_or_default(),
    }
}

/// Map a firewalld zone name to the Windows 10 network-profile label it most
/// resembles (Domain / Private / Public).
pub fn win10_zone_label(zone: &str) -> &'static str {
    match zone {
        "home" | "internal" | "trusted" => "Private network",
        "work" | "dmz" => "Domain network",
        _ => "Public network",
    }
}

/// firewalld state + active zones.
pub fn firewall() -> Tile {
    match out("firewall-cmd", &["--state"]) {
        Some(s) if parse_firewall_state(&s) => {
            let zones = out("firewall-cmd", &["--get-active-zones"])
                .map(|z| parse_active_zones(&z))
                .unwrap_or_default();
            let detail = if zones.is_empty() {
                "Firewall is on.".to_string()
            } else {
                format!("Firewall is on — active zones: {}.", zones.join(", "))
            };
            Tile::new("Firewall & network protection", Level::Ok, detail)
        }
        Some(_) => Tile::new(
            "Firewall & network protection",
            Level::Risk,
            "Firewall is off.",
        ),
        None => Tile::new(
            "Firewall & network protection",
            Level::Warn,
            "firewalld is not installed.",
        ),
    }
}

/// Disk encryption (any LUKS device present).
pub fn encryption() -> Tile {
    match out("lsblk", &["-f"]) {
        Some(s) if parse_luks(&s) => Tile::new(
            "Device encryption",
            Level::Ok,
            "A LUKS-encrypted volume is present.",
        ),
        Some(_) => Tile::new(
            "Device encryption",
            Level::Warn,
            "No encrypted volume detected.",
        ),
        None => Tile::new("Device encryption", Level::Warn, "Could not read disks."),
    }
}

/// ClamAV antivirus presence + version.
pub fn antivirus() -> Tile {
    match out("clamscan", &["--version"]).and_then(|s| parse_clamav_version(&s)) {
        Some(v) => Tile::new(
            "Virus & threat protection",
            Level::Ok,
            format!("ClamAV {v} is installed."),
        ),
        None => Tile::new(
            "Virus & threat protection",
            Level::Warn,
            "No antivirus (ClamAV) installed.",
        ),
    }
}

/// Secure Boot state via `mokutil --sb-state`.
pub fn secureboot() -> Tile {
    match out("mokutil", &["--sb-state"]).and_then(|s| parse_sb_state(&s)) {
        Some(true) => Tile::new("Secure Boot", Level::Ok, "Secure Boot is on."),
        Some(false) => Tile::new("Secure Boot", Level::Warn, "Secure Boot is off."),
        None => Tile::new(
            "Secure Boot",
            Level::Warn,
            "Secure Boot is unavailable on this system.",
        ),
    }
}

/// TPM presence/version from sysfs (`/sys/class/tpm/tpm0`).
pub fn tpm() -> Tile {
    if !std::path::Path::new("/sys/class/tpm/tpm0").exists() {
        return Tile::new("Security processor (TPM)", Level::Warn, "No TPM detected.");
    }
    let ver = std::fs::read_to_string("/sys/class/tpm/tpm0/tpm_version_major")
        .ok()
        .and_then(|s| parse_tpm_version(&s));
    match ver {
        Some(v) => Tile::new(
            "Security processor (TPM)",
            Level::Ok,
            format!("TPM {v} is present."),
        ),
        None => Tile::new("Security processor (TPM)", Level::Ok, "A TPM is present."),
    }
}

/// Probe all five posture checks.
pub fn probe() -> SecurityStatus {
    SecurityStatus {
        firewall: firewall(),
        encryption: encryption(),
        antivirus: antivirus(),
        secureboot: secureboot(),
        tpm: tpm(),
    }
}

/// Headless dump for `mde __security-probe`.
pub fn debug_print() {
    let s = probe();
    for t in [s.firewall, s.encryption, s.antivirus, s.secureboot, s.tpm] {
        println!("[{:?}] {} — {}", t.level, t.title, t.status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firewall_state_running() {
        assert!(parse_firewall_state("running\n"));
        assert!(!parse_firewall_state("not running"));
        assert!(!parse_firewall_state(""));
    }

    #[test]
    fn active_zones_keep_only_zone_names() {
        let fixture =
            "public\n  interfaces: eth0 wlan0\n  sources: \nlibvirt\n  interfaces: virbr0\n";
        assert_eq!(parse_active_zones(fixture), vec!["public", "libvirt"]);
        assert!(parse_active_zones("").is_empty());
    }

    #[test]
    fn secure_boot_state() {
        assert_eq!(parse_sb_state("SecureBoot enabled\n"), Some(true));
        assert_eq!(parse_sb_state("SecureBoot disabled"), Some(false));
        assert_eq!(
            parse_sb_state("This system doesn't support Secure Boot"),
            None
        );
        assert_eq!(parse_sb_state("EFI variables are not supported"), None);
    }

    #[test]
    fn clamav_version_extracted() {
        assert_eq!(
            parse_clamav_version("ClamAV 1.0.1/27000/Mon Jan  1 00:00:00 2024\n").as_deref(),
            Some("1.0.1")
        );
        assert_eq!(
            parse_clamav_version("ClamAV 0.103.8").as_deref(),
            Some("0.103.8")
        );
        assert_eq!(parse_clamav_version("bash: clamscan: not found"), None);
    }

    #[test]
    fn luks_detected_in_lsblk() {
        let enc = "NAME   FSTYPE      \nsda                \nsda1   crypto_LUKS \n";
        assert!(parse_luks(enc));
        assert!(!parse_luks("NAME FSTYPE\nsda1 ext4\n"));
    }

    #[test]
    fn zone_to_win10_profile() {
        assert_eq!(win10_zone_label("home"), "Private network");
        assert_eq!(win10_zone_label("trusted"), "Private network");
        assert_eq!(win10_zone_label("work"), "Domain network");
        assert_eq!(win10_zone_label("public"), "Public network");
        assert_eq!(win10_zone_label("anything-else"), "Public network");
    }

    #[test]
    fn tpm_version_mapped() {
        assert_eq!(parse_tpm_version("2\n").as_deref(), Some("2.0"));
        assert_eq!(parse_tpm_version("1").as_deref(), Some("1.2"));
        assert_eq!(parse_tpm_version(""), None);
    }
}
