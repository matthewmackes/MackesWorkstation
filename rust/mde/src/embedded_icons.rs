//! Carbon (Mackes-Carbon / IBM Carbon) icon art embedded directly into the
//! `mde` binary, replacing the platform's Windows 2000 icon set. One SVG per
//! freedesktop icon name the shell renders; chosen + verified by the
//! win2k->carbon mapping survey. Consulted first by `icons::icon_any`.
//!
//! GENERATED — do not edit by hand. Source: github.com/matthewmackes/MDE
//! (data/icons/Mackes-Carbon). Regenerate via the icon-mapping survey.

/// (freedesktop name -> SVG bytes) for every platform icon, sorted by name.
pub const ICONS: &[(&str, &[u8])] = &[
    ("application-pdf", include_bytes!("embedded_icons/application-pdf.svg")), // = document--pdf
    ("application-x-archive", include_bytes!("embedded_icons/application-x-archive.svg")), // = archive
    ("application-x-executable", include_bytes!("embedded_icons/application-x-executable.svg")), // = application-x-executable
    ("application-x-generic", include_bytes!("embedded_icons/application-x-generic.svg")), // = document--unknown
    ("applications-all", include_bytes!("embedded_icons/applications-all.svg")), // = apps
    ("applications-internet", include_bytes!("embedded_icons/applications-internet.svg")), // = applications-internet
    ("applications-other", include_bytes!("embedded_icons/applications-other.svg")), // = apps
    ("applications-system", include_bytes!("embedded_icons/applications-system.svg")), // = gears
    ("audio-x-generic", include_bytes!("embedded_icons/audio-x-generic.svg")), // = audio-x-generic
    ("battery", include_bytes!("embedded_icons/battery.svg")), // = battery
    ("computer", include_bytes!("embedded_icons/computer.svg")), // = user-desktop
    ("document-open-recent", include_bytes!("embedded_icons/document-open-recent.svg")), // = document-open-recent
    ("document-save", include_bytes!("embedded_icons/document-save.svg")), // = document-save
    ("drive-harddisk", include_bytes!("embedded_icons/drive-harddisk.svg")), // = drive-harddisk
    ("drive-multidisk", include_bytes!("embedded_icons/drive-multidisk.svg")), // = storage-pool
    ("edit-find", include_bytes!("embedded_icons/edit-find.svg")), // = edit-find
    ("firefox", include_bytes!("embedded_icons/firefox.svg")), // = earth
    ("firefox-esr", include_bytes!("embedded_icons/firefox-esr.svg")), // = applications-internet
    ("folder", include_bytes!("embedded_icons/folder.svg")), // = folder
    ("folder-applications", include_bytes!("embedded_icons/folder-applications.svg")), // = folder
    ("folder-documents", include_bytes!("embedded_icons/folder-documents.svg")), // = folder--details
    ("folder-home", include_bytes!("embedded_icons/folder-home.svg")), // = user-home
    ("folder-new", include_bytes!("embedded_icons/folder-new.svg")), // = folder-new
    ("folder-new-symbolic", include_bytes!("embedded_icons/folder-new-symbolic.svg")), // = folder-new-symbolic
    ("folder-temp", include_bytes!("embedded_icons/folder-temp.svg")), // = hourglass
    ("gparted", include_bytes!("embedded_icons/gparted.svg")), // = drive-harddisk
    ("help-browser", include_bytes!("embedded_icons/help-browser.svg")), // = help-browser
    ("help-contents", include_bytes!("embedded_icons/help-contents.svg")), // = table-of-contents
    ("image-x-generic", include_bytes!("embedded_icons/image-x-generic.svg")), // = image-x-generic
    ("input-keyboard", include_bytes!("embedded_icons/input-keyboard.svg")), // = input-keyboard
    ("internet-web-browser", include_bytes!("embedded_icons/internet-web-browser.svg")), // = applications-internet
    ("logviewer", include_bytes!("embedded_icons/logviewer.svg")), // = report
    ("media-optical", include_bytes!("embedded_icons/media-optical.svg")), // = media-record
    ("multimedia-volume-control", include_bytes!("embedded_icons/multimedia-volume-control.svg")), // = multimedia-volume-control
    ("network-server", include_bytes!("embedded_icons/network-server.svg")), // = bare-metal-server
    ("network-wired", include_bytes!("embedded_icons/network-wired.svg")), // = network--3
    ("network-workgroup", include_bytes!("embedded_icons/network-workgroup.svg")), // = network-workgroup
    ("package-x-generic", include_bytes!("embedded_icons/package-x-generic.svg")), // = box
    ("preferences-desktop-display", include_bytes!("embedded_icons/preferences-desktop-display.svg")), // = video-display
    ("preferences-desktop-locale", include_bytes!("embedded_icons/preferences-desktop-locale.svg")), // = preferences-desktop-locale
    ("preferences-system", include_bytes!("embedded_icons/preferences-system.svg")), // = settings
    ("preferences-system-time", include_bytes!("embedded_icons/preferences-system-time.svg")), // = time
    ("printer", include_bytes!("embedded_icons/printer.svg")), // = printer
    ("security-high", include_bytes!("embedded_icons/security-high.svg")), // = security-high
    ("stacer", include_bytes!("embedded_icons/stacer.svg")), // = meter
    ("system-file-manager", include_bytes!("embedded_icons/system-file-manager.svg")), // = folder
    ("system-help", include_bytes!("embedded_icons/system-help.svg")), // = help
    ("system-log-out", include_bytes!("embedded_icons/system-log-out.svg")), // = system-log-out
    ("system-run", include_bytes!("embedded_icons/system-run.svg")), // = run
    ("system-search", include_bytes!("embedded_icons/system-search.svg")), // = system-search
    ("system-shutdown", include_bytes!("embedded_icons/system-shutdown.svg")), // = system-shutdown
    ("system-software-install", include_bytes!("embedded_icons/system-software-install.svg")), // = system-software-install
    ("system-software-update", include_bytes!("embedded_icons/system-software-update.svg")), // = system-software-update
    ("system-users", include_bytes!("embedded_icons/system-users.svg")), // = system-users
    ("temperature", include_bytes!("embedded_icons/temperature.svg")), // = temperature
    ("terminal", include_bytes!("embedded_icons/terminal.svg")), // = terminal
    ("text-html", include_bytes!("embedded_icons/text-html.svg")), // = code
    ("text-plain", include_bytes!("embedded_icons/text-plain.svg")), // = text-x-generic
    ("text-x-generic", include_bytes!("embedded_icons/text-x-generic.svg")), // = text-x-generic
    ("text-x-script", include_bytes!("embedded_icons/text-x-script.svg")), // = text-x-script
    ("user-desktop", include_bytes!("embedded_icons/user-desktop.svg")), // = user-desktop
    ("user-home", include_bytes!("embedded_icons/user-home.svg")), // = user-home
    ("utilities-system-monitor", include_bytes!("embedded_icons/utilities-system-monitor.svg")), // = utilities-system-monitor
    ("utilities-terminal", include_bytes!("embedded_icons/utilities-terminal.svg")), // = utilities-terminal
    ("video-display", include_bytes!("embedded_icons/video-display.svg")), // = video-display
    ("video-x-generic", include_bytes!("embedded_icons/video-x-generic.svg")), // = video-x-generic
    ("web-browser", include_bytes!("embedded_icons/web-browser.svg")), // = applications-internet
    ("wireshark", include_bytes!("embedded_icons/wireshark.svg")), // = chart--network
];
