//! CUPS data layer for Settings ▸ Devices ▸ Printers & scanners (E12.4).
//!
//! Shells the standard CUPS command-line tools — `lpstat` (list + default),
//! `lpinfo` (discover devices for "+ Add a printer"), `lpadmin` (add/remove a
//! queue), `lpoptions` (set the user default), and `lp` (print the test page).
//! No iced dependency: the iced page (`settings.rs`) calls these off the UI
//! thread, exactly like the BlueZ layer ([`crate::bluez`]). Headless entry:
//! `mde __cups-list`.
//!
//! Privilege split mirrors the tools themselves: **set-default** (`lpoptions -d`,
//! writes `~/.cups/lpoptions`) and **test page** (`lp`, submits a job) are
//! per-user and run directly; **add**/**remove** (`lpadmin`) mutate the system
//! queue config and run through `pkexec` (the same path Accounts uses for
//! `useradd`/`userdel`).

use std::process::Command;

/// CUPS ships this PostScript test page; falls back to a generated file only if
/// it is somehow absent (it is part of the `cups` package).
const TESTPRINT: &str = "/usr/share/cups/data/testprint";

/// What `lpstat` reports a queue is doing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PrinterState {
    #[default]
    Idle,
    Printing,
    Stopped,
}

impl PrinterState {
    pub fn label(self) -> &'static str {
        match self {
            PrinterState::Idle => "Ready",
            PrinterState::Printing => "Printing",
            PrinterState::Stopped => "Paused",
        }
    }
}

/// One installed print queue.
#[derive(Debug, Clone, Default)]
pub struct Printer {
    pub name: String,
    pub info: String, // human description (lpstat -l "Description:"); falls back to name
    pub state: PrinterState,
    pub is_default: bool,
    pub is_pdf: bool, // the cups-pdf virtual queue (Win10 "Print to PDF"), E12.5
}

/// A discovered-but-not-yet-added device (the "+ Add a printer" scan).
#[derive(Debug, Clone, Default)]
pub struct Device {
    pub uri: String,
    pub info: String, // make-and-model / info from `lpinfo -l`
}

/// Snapshot for the page: whether CUPS is installed, the queue list, and any
/// devices surfaced by the most recent discovery scan.
#[derive(Debug, Clone, Default)]
pub struct CupsState {
    pub present: bool,
    pub printers: Vec<Printer>,
    pub discovered: Vec<Device>,
}

fn run(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Env seam for deterministic gallery captures (same idea as `MDE_PASSWD_PATH` in
/// [`crate::sysinfo`] / `MDE_SYSUPGRADE_STATE` in [`crate::packages`]): when
/// `MDE_CUPS_FIXTURE` points to a file, its contents stand in for `lpstat -l -p`
/// and `MDE_CUPS_DEFAULT` names the default queue — so the page can render a
/// populated, default-marked list headlessly without a real CUPS queue.
fn fixture() -> Option<(String, Option<String>)> {
    let body = std::fs::read_to_string(std::env::var("MDE_CUPS_FIXTURE").ok()?).ok()?;
    let default = std::env::var("MDE_CUPS_DEFAULT")
        .ok()
        .filter(|s| !s.is_empty());
    Some((body, default))
}

/// True if the `lpstat` binary is runnable at all (CUPS installed). Distinguishes
/// "no printers" (present, empty list) from "CUPS absent" (page shows an advisory).
fn cups_present() -> bool {
    std::env::var_os("MDE_CUPS_FIXTURE").is_some()
        || Command::new("lpstat").arg("-r").output().is_ok()
}

/// Parse `lpstat -d` ("system default destination: NAME" / "no system default…").
pub fn parse_default(out: &str) -> Option<String> {
    out.lines().find_map(|l| {
        l.split_once("destination:")
            .map(|(_, n)| n.trim().to_string())
            .filter(|n| !n.is_empty())
    })
}

/// Parse `lpstat -l -p` into the queue list. Each queue opens with a
/// `printer NAME ...` line (state read from its wording); the indented
/// `Description:` line that follows, if any, becomes `info`.
pub fn parse_printers(out: &str, default: Option<&str>) -> Vec<Printer> {
    let mut printers: Vec<Printer> = Vec::new();
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("printer ") {
            let name = rest.split_whitespace().next().unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let lower = rest.to_lowercase();
            // "now printing" → Printing; "disabled"/"stopped" → Stopped; else Idle.
            let state = if lower.contains("printing") {
                PrinterState::Printing
            } else if lower.contains("disabled") || lower.contains("stopped") {
                PrinterState::Stopped
            } else {
                PrinterState::Idle
            };
            let is_default = default == Some(name.as_str());
            printers.push(Printer {
                info: name.clone(),
                name,
                state,
                is_default,
                is_pdf: false,
            });
        } else if let Some(desc) = line.trim().strip_prefix("Description:") {
            // Attach the description to the queue currently being built.
            if let Some(p) = printers.last_mut() {
                let d = desc.trim();
                if !d.is_empty() {
                    p.info = d.to_string();
                }
            }
        }
    }
    printers
}

/// Parse `lpstat -v` ("device for NAME: cups-pdf:/…") for the name of the cups-pdf
/// virtual queue — the Win10 "Print to PDF" equivalent (E12.5). The `cups-pdf:`
/// backend is the definitive marker (a queue merely *named* "pdf" doesn't count).
pub fn parse_pdf_queue(out: &str) -> Option<String> {
    out.lines().find_map(|l| {
        let body = l.trim().strip_prefix("device for ")?;
        let (name, uri) = body.split_once(':')?;
        uri.trim()
            .starts_with("cups-pdf")
            .then(|| name.trim().to_string())
    })
}

/// The cups-pdf queue name, if one is installed. Fixture-aware (`MDE_CUPS_PDF`).
fn pdf_queue_name() -> Option<String> {
    if std::env::var_os("MDE_CUPS_FIXTURE").is_some() {
        return std::env::var("MDE_CUPS_PDF").ok().filter(|s| !s.is_empty());
    }
    parse_pdf_queue(&run("lpstat", &["-v"])?)
}

/// Read the installed queues + the system default, then flag the cups-pdf queue.
pub fn printers() -> Vec<Printer> {
    let pdf = pdf_queue_name();
    let mut list = if let Some((body, default)) = fixture() {
        parse_printers(&body, default.as_deref())
    } else {
        let default = run("lpstat", &["-d"]).and_then(|o| parse_default(&o));
        let body = run("lpstat", &["-l", "-p"]).unwrap_or_default();
        parse_printers(&body, default.as_deref())
    };
    if let Some(pn) = &pdf {
        for p in &mut list {
            if &p.name == pn {
                p.is_pdf = true;
            }
        }
    }
    list
}

/// Whether the cups-pdf "Print to PDF" queue is currently installed.
pub fn pdf_installed() -> bool {
    pdf_queue_name().is_some()
}

/// One-shot "set up Print to PDF": install the `cups-pdf` package, then create the
/// virtual queue if its post-install step didn't (some setups auto-register it).
/// One `pkexec` prompt for the whole thing. Returned argv runs under pkexec.
pub fn ensure_pdf_cmd() -> Vec<String> {
    vec![
        "sh".into(),
        "-c".into(),
        "dnf install -y cups-pdf && \
         (lpstat -v 2>/dev/null | grep -qi 'cups-pdf' || \
          lpadmin -p Print-to-PDF -E -v cups-pdf:/ -m everywhere)"
            .into(),
    ]
}

/// The page snapshot (no discovery — that is a separate, slower scan).
pub fn state() -> CupsState {
    if !cups_present() {
        return CupsState::default();
    }
    CupsState {
        present: true,
        printers: printers(),
        discovered: Vec::new(),
    }
}

/// A `lpinfo -v` line names a *real* device only when its URI has an authority
/// (`scheme://host…`). The bare scheme rows (`network http`, `network lpd`,
/// `file cups-brf:/`, `serial serial:/dev/…`) are connection *kinds*, not devices.
fn is_real_device_uri(uri: &str) -> bool {
    match uri.split_once("://") {
        // The `file` backend (e.g. `file:///dev/null`) is a CUPS debug sink, not a
        // printer you would add — drop it along with the authority-less kinds.
        Some((scheme, rest)) => !rest.is_empty() && scheme != "file",
        None => false,
    }
}

/// Parse `lpinfo -l -v` (long form) into discoverable devices. The long form
/// emits a block per device: `Device: uri = …`, then indented `class = …`,
/// `info = …`, `make-and-model = …`. We keep real devices and prefer the
/// make-and-model (then info) as the label.
pub fn parse_devices(out: &str) -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    let mut uri = String::new();
    let mut info = String::new();
    let mut make = String::new();
    let flush = |devices: &mut Vec<Device>, uri: &str, info: &str, make: &str| {
        if !uri.is_empty() && is_real_device_uri(uri) {
            let label = if !make.is_empty() {
                make
            } else if !info.is_empty() {
                info
            } else {
                uri
            };
            devices.push(Device {
                uri: uri.to_string(),
                info: label.to_string(),
            });
        }
    };
    for line in out.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("Device:") {
            // Start of a new block — flush the previous one.
            flush(&mut devices, &uri, &info, &make);
            uri.clear();
            info.clear();
            make.clear();
            // `Device: uri = foo` sometimes carries the uri inline.
            if let Some((_, u)) = v.split_once("uri = ") {
                uri = u.trim().to_string();
            }
        } else if let Some((k, val)) = t.split_once('=') {
            let (k, val) = (k.trim(), val.trim());
            match k {
                "uri" => uri = val.to_string(),
                "info" => info = val.to_string(),
                "make-and-model" => make = val.to_string(),
                _ => {}
            }
        }
    }
    flush(&mut devices, &uri, &info, &make);
    devices
}

/// Discover addable devices for "+ Add a printer". Slow (probes the network), so
/// the page runs it off-thread. Empty on a host with no reachable printers.
pub fn discover() -> Vec<Device> {
    let out = run("lpinfo", &["-l", "-v"]).unwrap_or_default();
    parse_devices(&out)
}

/// CUPS queue names allow only `[A-Za-z0-9_-]` and no spaces/slashes. Derive one
/// from a device label; collapse runs of disallowed chars to a single `_`.
pub fn sanitize_queue_name(label: &str) -> String {
    let mut out = String::new();
    let mut last_us = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
            last_us = false;
        } else if !last_us {
            out.push('_');
            last_us = true;
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "Printer".to_string()
    } else {
        trimmed
    }
}

/// `lpoptions -d NAME` — set the *user* default (no root; writes ~/.cups/lpoptions).
pub fn set_default(name: &str) {
    let _ = Command::new("lpoptions").args(["-d", name]).status();
}

/// `lp -d NAME TESTPRINT` — submit the CUPS test page (no root; just queues a job).
pub fn print_test_page(name: &str) {
    let path = if std::path::Path::new(TESTPRINT).exists() {
        TESTPRINT.to_string()
    } else {
        // Last-resort: a one-line text job so the action is never a silent no-op.
        let tmp = std::env::temp_dir().join("mde-testpage.txt");
        let _ = std::fs::write(&tmp, "MackesDE printer test page\n");
        tmp.to_string_lossy().into_owned()
    };
    let _ = Command::new("lp").args(["-d", name, &path]).status();
}

/// `lpadmin -p NAME -E -v URI -m everywhere` — add a driverless (IPP Everywhere)
/// queue. Returns the argv for `pkexec` (system config change → root).
pub fn add_cmd(name: &str, uri: &str) -> Vec<String> {
    vec![
        "lpadmin".into(),
        "-p".into(),
        name.into(),
        "-E".into(),
        "-v".into(),
        uri.into(),
        "-m".into(),
        "everywhere".into(),
    ]
}

/// `lpadmin -x NAME` — delete a queue. Returns the argv for `pkexec`.
pub fn remove_cmd(name: &str) -> Vec<String> {
    vec!["lpadmin".into(), "-x".into(), name.into()]
}

/// Headless dump for `mde __cups-list`.
pub fn debug_list() {
    let st = state();
    println!("cups present={} printers={}", st.present, st.printers.len());
    for p in &st.printers {
        println!(
            "  {} [{}] {}{}{}",
            p.name,
            p.state.label(),
            p.info,
            if p.is_default { " (default)" } else { "" },
            if p.is_pdf { " (pdf)" } else { "" }
        );
    }
    println!("print-to-pdf installed={}", pdf_installed());
    let devs = discover();
    println!("discoverable devices={}", devs.len());
    for d in devs {
        println!("  {} <- {}", d.info, d.uri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parsed() {
        assert_eq!(
            parse_default("system default destination: Office\n").as_deref(),
            Some("Office")
        );
        assert_eq!(parse_default("no system default destination\n"), None);
    }

    #[test]
    fn pdf_queue_detected_by_backend() {
        let out = "\
device for Office: ipp://host/printer
device for Cups-PDF: cups-pdf:/
device for NotPdf: socket://10.0.0.5
";
        assert_eq!(parse_pdf_queue(out).as_deref(), Some("Cups-PDF"));
        // A queue merely *named* pdf, on a real backend, is not the cups-pdf queue.
        assert_eq!(parse_pdf_queue("device for pdfish: socket://h\n"), None);
        assert_eq!(parse_pdf_queue("no devices\n"), None);
    }

    #[test]
    fn printers_states_and_default() {
        let out = "\
printer Office is idle.  enabled since Mon
\tDescription: HP LaserJet in Room 2
\tLocation: Room 2
printer Draft now printing job 5.  enabled since Tue
printer Old disabled since Wed - paused
";
        let ps = parse_printers(out, Some("Office"));
        assert_eq!(ps.len(), 3);
        assert_eq!(ps[0].name, "Office");
        assert_eq!(ps[0].info, "HP LaserJet in Room 2");
        assert_eq!(ps[0].state, PrinterState::Idle);
        assert!(ps[0].is_default);
        assert_eq!(ps[1].state, PrinterState::Printing);
        assert!(!ps[1].is_default);
        // No Description line → info falls back to the name.
        assert_eq!(ps[1].info, "Draft");
        assert_eq!(ps[2].state, PrinterState::Stopped);
    }

    #[test]
    fn devices_filtered_and_labelled() {
        // Bare schemes (no authority) are kinds, not devices, and must drop out;
        // a real usb/dnssd device with a make-and-model survives and is labelled.
        let out = "\
Device: uri = network
        class = network
        info = Internet Printing Protocol
Device: uri = usb://HP/LaserJet?serial=42
        class = direct
        info = HP LaserJet (USB)
        make-and-model = HP LaserJet Pro
Device: uri = file:///dev/null
        class = file
        info = bogus
";
        let ds = parse_devices(out);
        assert_eq!(ds.len(), 1, "only the usb device is real");
        assert_eq!(ds[0].uri, "usb://HP/LaserJet?serial=42");
        assert_eq!(ds[0].info, "HP LaserJet Pro");
    }

    #[test]
    fn queue_names_sanitized() {
        assert_eq!(sanitize_queue_name("HP LaserJet Pro"), "HP_LaserJet_Pro");
        assert_eq!(sanitize_queue_name("Brother (Wi-Fi)"), "Brother_Wi-Fi");
        assert_eq!(sanitize_queue_name("///"), "Printer");
        assert_eq!(sanitize_queue_name("Office"), "Office");
    }

    #[test]
    fn commands_shaped() {
        assert_eq!(
            add_cmd("Office", "ipp://host/printer"),
            vec![
                "lpadmin",
                "-p",
                "Office",
                "-E",
                "-v",
                "ipp://host/printer",
                "-m",
                "everywhere"
            ]
        );
        assert_eq!(remove_cmd("Office"), vec!["lpadmin", "-x", "Office"]);
    }
}
