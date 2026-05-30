//! Fedora system-tool wiring for the Win2000 Control Panel and the Start menu's
//! System Tools. Each entry maps a Windows 2000 name to a real Fedora tool, the
//! dnf package that provides it, and whether it runs in a terminal. Tools that
//! aren't installed can be installed on demand via `pkexec dnf` (a graphical
//! privilege prompt).

use std::process::{Command, Stdio};

#[allow(dead_code)]
pub struct Tool {
    /// Windows 2000 Control Panel / System Tools name.
    pub name: &'static str,
    /// Shell command that launches it.
    pub command: &'static str,
    /// Run inside a terminal (for CLI tools like `hostnamectl`).
    pub terminal: bool,
    /// dnf package that provides the binary (for install-if-missing).
    pub package: &'static str,
    /// Freedesktop icon-name candidates, best first.
    pub icons: &'static [&'static str],
}

/// The Control Panel / System Tools table (ported from control-panel.py).
pub const TOOLS: &[Tool] = &[
    Tool { name: "Add/Remove Programs", command: "dnfdragora", terminal: false, package: "dnfdragora", icons: &["system-software-install", "package-x-generic"] },
    Tool { name: "Automatic Updates", command: "dnfdragora-updater", terminal: false, package: "dnfdragora", icons: &["system-software-update", "software-update-available"] },
    Tool { name: "Windows Firewall", command: "firewall-config", terminal: false, package: "firewall-config", icons: &["security-high", "preferences-system-firewall"] },
    Tool { name: "Network and Dial-up Connections", command: "nm-connection-editor", terminal: false, package: "nm-connection-editor", icons: &["network-wired", "network-workgroup"] },
    Tool { name: "Sounds and Multimedia", command: "pavucontrol", terminal: false, package: "pavucontrol", icons: &["multimedia-volume-control", "audio-card"] },
    Tool { name: "Disk Management", command: "gnome-disks", terminal: false, package: "gnome-disk-utility", icons: &["drive-harddisk", "gnome-disks"] },
    Tool { name: "Partition Manager", command: "gparted", terminal: false, package: "gparted", icons: &["gparted", "drive-harddisk"] },
    Tool { name: "Storage Spaces", command: "blivet-gui", terminal: false, package: "blivet-gui", icons: &["drive-multidisk", "blivet-gui"] },
    Tool { name: "Users and Passwords", command: "seahorse", terminal: false, package: "seahorse", icons: &["system-users", "dialog-password"] },
    Tool { name: "Regional Options", command: "system-config-language", terminal: false, package: "system-config-language", icons: &["preferences-desktop-locale"] },
    Tool { name: "Keyboard", command: "im-chooser", terminal: false, package: "im-chooser", icons: &["input-keyboard", "preferences-desktop-keyboard"] },
    Tool { name: "Event Viewer", command: "gnome-abrt", terminal: false, package: "gnome-abrt", icons: &["logviewer", "utilities-system-monitor"] },
    Tool { name: "Security Center", command: "sealert -b", terminal: false, package: "setroubleshoot-server", icons: &["security-high", "preferences-system-privacy"] },
    Tool { name: "Create Installation Media", command: "mediawriter", terminal: false, package: "mediawriter", icons: &["media-optical", "media-removable"] },
    Tool { name: "System", command: "hostnamectl; echo; read -p 'Press Enter to close '", terminal: true, package: "systemd", icons: &["computer", "preferences-system"] },
    Tool { name: "Date and Time", command: "timedatectl; echo; read -p 'Press Enter to close '", terminal: true, package: "systemd", icons: &["preferences-system-time", "clock"] },
    // --- systemd core + service tooling (Win2000: Administrative Tools) -----
    Tool { name: "Printers", command: "system-config-printer", terminal: false, package: "system-config-printer", icons: &["printer", "preferences-system-printer"] },
    Tool { name: "Services", command: "systemctl --no-pager list-units --type=service; echo; read -p 'Press Enter to close '", terminal: true, package: "systemd", icons: &["preferences-system", "system-run"] },
    Tool { name: "Boot Performance", command: "systemd-analyze; echo; systemd-analyze blame | head -n 30; echo; read -p 'Press Enter to close '", terminal: true, package: "systemd", icons: &["utilities-system-monitor", "clock"] },
    Tool { name: "Name Resolution (DNS)", command: "resolvectl status; echo; read -p 'Press Enter to close '", terminal: true, package: "systemd-resolved", icons: &["network-wired", "preferences-system-network"] },
    Tool { name: "Temporary Files", command: "systemd-tmpfiles --cat-config | less", terminal: true, package: "systemd", icons: &["folder-temp", "user-trash"] },
];

/// The leading binary of a command (`sealert -b` -> `sealert`).
pub fn binary(command: &str) -> &str {
    command.split_whitespace().next().unwrap_or(command)
}

/// Whether the tool's binary is on `$PATH`.
pub fn is_installed(command: &str) -> bool {
    let bin = binary(command);
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Launch a tool (in `foot` if it wants a terminal).
pub fn launch(tool: &Tool) -> std::io::Result<()> {
    if tool.terminal {
        Command::new("foot").arg("sh").arg("-c").arg(tool.command).spawn()?;
    } else {
        Command::new("sh").arg("-c").arg(tool.command).spawn()?;
    }
    Ok(())
}

/// Tools whose binary is not currently installed.
pub fn missing() -> Vec<&'static Tool> {
    TOOLS.iter().filter(|t| !is_installed(t.command)).collect()
}

/// The unique dnf packages needed to satisfy all missing tools.
pub fn missing_packages() -> Vec<&'static str> {
    let mut pkgs: Vec<&str> = missing().iter().map(|t| t.package).collect();
    pkgs.sort_unstable();
    pkgs.dedup();
    pkgs
}

/// Install the given packages via a single graphical `pkexec dnf` prompt.
pub fn install(packages: &[&str]) -> std::io::Result<std::process::ExitStatus> {
    Command::new("pkexec")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .args(packages)
        .status()
}
