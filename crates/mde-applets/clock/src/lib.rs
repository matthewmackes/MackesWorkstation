//! Clock + date pill applet — top-bar-center slot.
//!
//! Phase E1.2.1: smallest of the Phase E1 applets.
//! Renders the current local-time string at a fixed
//! `YYYY-MM-DD HH:MM` format. The applet binary loops on
//! a 30 s wakeup tick (no need for second-resolution on
//! the panel surface).

#![forbid(unsafe_code)]

use mde_applet_api::{AppletId, AppletSlot, HostMessage};

/// Build the canonical manifest the panel host picks up
/// from `/usr/share/mde/applets/clock.json`.
#[must_use]
pub fn manifest() -> mde_applet_api::AppletManifest {
    mde_applet_api::AppletManifest {
        id: AppletId::from_static("clock"),
        binary: "mde-applet-clock".into(),
        slot: AppletSlot::TopBarCenter,
        summary: "Clock + date pill".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    }
}

/// Format a Unix-epoch-seconds timestamp as
/// `YYYY-MM-DD HH:MM` (UTC). Returns `"--:--"` for
/// non-positive timestamps — used by the applet's loading
/// state.
#[must_use]
pub fn format_clock(secs: i64) -> String {
    if secs <= 0 {
        return "--:--".to_string();
    }
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}")
}

/// Howard Hinnant civil-from-days. Shared with the run-
/// history + mesh-history panels in mde-workbench; copied
/// here so this applet stays a standalone binary with
/// minimal deps.
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if mo <= 2 { y + 1 } else { y };
    (year as i32, mo as u32, d as u32)
}

/// Decide whether a host-pushed message means the applet
/// should re-render its view. Accent changes + visibility
/// flips trigger a render; Shutdown short-circuits to false
/// (the caller is expected to flush + exit).
#[must_use]
pub fn handle_host(msg: &HostMessage) -> bool {
    !matches!(msg, HostMessage::Shutdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_lands_in_top_bar_center_slot() {
        let m = manifest();
        assert_eq!(m.id.as_str(), "clock");
        assert_eq!(m.slot, AppletSlot::TopBarCenter);
        assert_eq!(m.binary, "mde-applet-clock");
        assert!(!m.summary.is_empty());
    }

    #[test]
    fn format_clock_renders_known_timestamps() {
        // 1_715_000_000 secs since epoch -> 2024-05-06 12:53 UTC.
        let s = format_clock(1_715_000_000);
        assert!(s.starts_with("2024-05-06"), "got: {s}");
        assert_eq!(s, "2024-05-06 12:53");
    }

    #[test]
    fn format_clock_dashes_non_positive_timestamps() {
        assert_eq!(format_clock(0), "--:--");
        assert_eq!(format_clock(-1), "--:--");
    }

    #[test]
    fn handle_host_short_circuits_shutdown() {
        assert!(!handle_host(&HostMessage::Shutdown));
        assert!(handle_host(&HostMessage::Accent {
            color: "#000".into()
        }));
        assert!(handle_host(&HostMessage::Visibility { active: true }));
    }

    #[test]
    fn days_to_ymd_anchor_dates() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        assert_eq!(days_to_ymd(19_723), (2024, 1, 1));
    }
}
