//! `NetworkManager` tray icon — live interface state + full controls
//! popover, backed by `nmcli`.
//!
//! User lock 2026-05-19: a Mackes-styled tray icon with live state
//! that surfaces every `NetworkManager` option without requiring the
//! upstream nm-applet (which assumes a system tray that Mackes
//! doesn't ship). The button itself lives in the taskbar to the left
//! of the 6-probe status cluster.
//!
//! The icon's glyph + label reflect the most-prominent active
//! connection: wired link if any is up, wireless SSID if not, "off"
//! when both are down. Polls every 5 s (cheap — `nmcli -t -f` is
//! ~10 ms even on a busy host).
//!
//! Popover sections:
//!
//! - **Connections**: every saved profile from `nmcli -t -f
//!   NAME,UUID,TYPE,DEVICE connection show`. Active rows carry a
//!   green dot; click toggles up/down via `nmcli connection
//!   {up,down}`.
//! - **Wi-Fi networks**: `nmcli -t -f SSID,SIGNAL,SECURITY device
//!   wifi list` — click to connect.
//! - **Controls**: airplane-mode toggle (`nmcli networking off/on`),
//!   Wi-Fi scan (`nmcli device wifi rescan`), edit current profile
//!   (`nm-connection-editor` if installed).
//!
//! Tooltips on each row carry the literal `nmcli` command so the
//! popover doubles as a cheatsheet.

use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

/// Snapshot of the live NM state, captured once per refresh tick.
#[derive(Debug, Default, Clone)]
struct NmState {
    /// Active wired connection's device name (e.g. `eno1`), if any.
    wired_device: Option<String>,
    /// Active wireless SSID + device, if any.
    wifi: Option<(String, String)>,
    /// `true` when `nmcli networking` reports `disabled`.
    airplane: bool,
}

impl NmState {
    fn probe() -> Self {
        let mut out = Self::default();
        if let Ok(o) = Command::new("nmcli")
            .args(["-t", "-f", "TYPE,DEVICE,STATE,CONNECTION", "device", "status"])
            .output()
        {
            if o.status.success() {
                for line in String::from_utf8_lossy(&o.stdout).lines() {
                    let cols: Vec<&str> = line.split(':').collect();
                    if cols.len() < 4 {
                        continue;
                    }
                    let (ty, dev, state, conn) = (cols[0], cols[1], cols[2], cols[3]);
                    if state != "connected" {
                        continue;
                    }
                    match ty {
                        "ethernet" if out.wired_device.is_none() => {
                            out.wired_device = Some(dev.to_owned());
                        }
                        "wifi" if out.wifi.is_none() => {
                            out.wifi = Some((conn.to_owned(), dev.to_owned()));
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Ok(o) = Command::new("nmcli").args(["networking"]).output() {
            if String::from_utf8_lossy(&o.stdout).trim() == "disabled" {
                out.airplane = true;
            }
        }
        out
    }

    /// Compact label for the tray button. Examples:
    /// `Ethernet · eno1`, `Wi-Fi · home-net`, `Offline`.
    fn label(&self) -> String {
        if self.airplane {
            return "Airplane".to_owned();
        }
        if let Some(d) = &self.wired_device {
            return format!("Ethernet · {d}");
        }
        if let Some((ssid, _)) = &self.wifi {
            return format!("Wi-Fi · {ssid}");
        }
        "Offline".to_owned()
    }

    /// Symbolic icon for the tray button.
    const fn icon_name(&self) -> &'static str {
        if self.airplane {
            return "airplane-mode-symbolic";
        }
        if self.wired_device.is_some() {
            return "network-wired-symbolic";
        }
        if self.wifi.is_some() {
            return "network-wireless-symbolic";
        }
        "network-offline-symbolic"
    }
}

/// Build the taskbar button. The widget owns its own 5 s refresh
/// timer + click handler that pops up the controls popover.
#[must_use]
pub fn build() -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name("mackes-nm-button");
    button.set_relief(gtk::ReliefStyle::None);
    button.set_focus_on_click(false);
    if let Some(atk) = button.accessible() {
        atk.set_name("Network status — click for network controls");
        atk.set_description(
            "Shows active network state (Wi-Fi / Ethernet / VPN / airplane). \
             Click to open the network controls popover.",
        );
    }

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let icon = gtk::Image::new();
    let label = gtk::Label::new(None);
    row.pack_start(&icon, false, false, 0);
    row.pack_start(&label, false, false, 0);
    button.add(&row);

    let state = Rc::new(std::cell::RefCell::new(NmState::probe()));
    apply_state(&button, &icon, &label, &state.borrow());

    let button_for_click = button.clone();
    button.connect_clicked(move |_| {
        let popover = build_popover(button_for_click.upcast_ref::<gtk::Widget>());
        popover.popup();
    });

    // 5 s state poll. Closure consumes `icon` + `label` directly;
    // only `button` needs cloning since the click handler above also
    // keeps a reference.
    {
        let button_for_timer = button.clone();
        glib::timeout_add_local(Duration::from_secs(5), move || {
            let fresh = NmState::probe();
            *state.borrow_mut() = fresh.clone();
            apply_state(&button_for_timer, &icon, &label, &fresh);
            glib::ControlFlow::Continue
        });
    }

    button
}

fn apply_state(button: &gtk::Button, icon: &gtk::Image, label: &gtk::Label, st: &NmState) {
    if let Some(pb) = crate::icons::load(st.icon_name(), 16) {
        icon.set_from_pixbuf(Some(&pb));
    } else {
        icon.set_from_icon_name(Some(st.icon_name()), gtk::IconSize::Menu);
    }
    label.set_text(&st.label());
    let tt = if st.airplane {
        "Airplane mode — click for network controls".to_owned()
    } else {
        format!("{} — click for network controls", st.label())
    };
    button.set_tooltip_text(Some(&tt));
}

fn build_popover(relative_to: &gtk::Widget) -> gtk::Popover {
    let popover = gtk::Popover::new(Some(relative_to));
    popover.set_widget_name("mackes-nm-popover");
    popover.set_position(gtk::PositionType::Top);
    popover.set_modal(true);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 10);
    column.set_margin_top(10);
    column.set_margin_bottom(10);
    column.set_margin_start(12);
    column.set_margin_end(12);

    column.pack_start(&section_header("Connections"), false, false, 0);
    column.pack_start(&build_connections_list(), false, false, 0);
    column.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    column.pack_start(&section_header("Wi-Fi networks"), false, false, 0);
    column.pack_start(&build_wifi_list(), false, false, 0);
    column.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    column.pack_start(&build_controls_row(&popover), false, false, 0);

    popover.add(&column);
    column.show_all();
    popover
}

fn section_header(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_halign(gtk::Align::Start);
    l.style_context().add_class("mackes-nm-section-header");
    l
}

fn build_connections_list() -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let lines = nmcli_lines(&["-t", "-f", "NAME,TYPE,DEVICE,STATE", "connection", "show"]);
    if lines.is_empty() {
        list.pack_start(&placeholder("(no saved connections)"), false, false, 0);
        return list;
    }
    for line in lines {
        let cols: Vec<&str> = line.split(':').collect();
        if cols.len() < 4 {
            continue;
        }
        let (name, ty, dev, state) = (cols[0], cols[1], cols[2], cols[3]);
        let active = state == "activated" || !dev.is_empty();
        list.pack_start(&build_connection_row(name, ty, active), false, false, 0);
    }
    list
}

fn build_connection_row(name: &str, ty: &str, active: bool) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let dot = gtk::Label::new(Some(if active { "●" } else { "○" }));
    dot.style_context().add_class(if active {
        "mackes-nm-row-active"
    } else {
        "mackes-nm-row-inactive"
    });
    row.pack_start(&dot, false, false, 0);

    let label = gtk::Label::new(Some(name));
    label.set_halign(gtk::Align::Start);
    label.set_max_width_chars(24);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row.pack_start(&label, true, true, 0);

    let type_label = gtk::Label::new(Some(ty));
    type_label.style_context().add_class("mackes-nm-row-type");
    row.pack_start(&type_label, false, false, 0);

    let button = gtk::Button::with_label(if active { "Down" } else { "Up" });
    button.set_relief(gtk::ReliefStyle::None);
    button.set_tooltip_text(Some(&format!(
        "nmcli connection {} {name}",
        if active { "down" } else { "up" }
    )));
    if let Some(atk) = button.accessible() {
        atk.set_name(&format!(
            "{} the {name} {ty} connection",
            if active { "Disconnect" } else { "Activate" }
        ));
    }
    let name_owned = name.to_owned();
    let was_active = active;
    button.connect_clicked(move |_| {
        let verb = if was_active { "down" } else { "up" };
        let _ = Command::new("nmcli")
            .args(["connection", verb, &name_owned])
            .spawn();
    });
    row.pack_end(&button, false, false, 0);
    row
}

fn build_wifi_list() -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let lines = nmcli_lines(&["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"]);
    if lines.is_empty() {
        list.pack_start(&placeholder("(no Wi-Fi adapters / not scanned yet)"), false, false, 0);
        return list;
    }
    for line in lines.into_iter().take(10) {
        let cols: Vec<&str> = line.split(':').collect();
        if cols.len() < 4 {
            continue;
        }
        let (in_use, ssid, signal, sec) = (cols[0], cols[1], cols[2], cols[3]);
        if ssid.trim().is_empty() {
            continue;
        }
        list.pack_start(&build_wifi_row(in_use, ssid, signal, sec), false, false, 0);
    }
    list
}

fn build_wifi_row(in_use: &str, ssid: &str, signal: &str, sec: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let active = in_use.trim() == "*";
    let dot = gtk::Label::new(Some(if active { "●" } else { "○" }));
    row.pack_start(&dot, false, false, 0);

    let label = gtk::Label::new(Some(ssid));
    label.set_halign(gtk::Align::Start);
    label.set_max_width_chars(20);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row.pack_start(&label, true, true, 0);

    let signal_label = gtk::Label::new(Some(&format!("{signal}%")));
    signal_label.style_context().add_class("mackes-nm-row-type");
    row.pack_start(&signal_label, false, false, 0);

    let sec_label = gtk::Label::new(Some(sec));
    sec_label.style_context().add_class("mackes-nm-row-type");
    row.pack_start(&sec_label, false, false, 0);

    let button = gtk::Button::with_label(if active { "Disconnect" } else { "Connect" });
    button.set_relief(gtk::ReliefStyle::None);
    button.set_tooltip_text(Some(&format!("nmcli device wifi connect {ssid}")));
    if let Some(atk) = button.accessible() {
        atk.set_name(&format!(
            "{} Wi-Fi network {ssid}",
            if active { "Disconnect from" } else { "Connect to" }
        ));
    }
    let ssid_owned = ssid.to_owned();
    let was_active = active;
    button.connect_clicked(move |_| {
        if was_active {
            let _ = Command::new("nmcli")
                .args(["device", "disconnect"])
                .spawn();
        } else {
            let _ = Command::new("nmcli")
                .args(["device", "wifi", "connect", &ssid_owned, "--ask"])
                .spawn();
        }
    });
    row.pack_end(&button, false, false, 0);
    row
}

fn build_controls_row(popover: &gtk::Popover) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);

    let airplane = gtk::Button::with_label("Toggle airplane mode");
    airplane.set_relief(gtk::ReliefStyle::None);
    airplane.set_tooltip_text(Some("nmcli networking off / on"));
    if let Some(atk) = airplane.accessible() {
        atk.set_name("Toggle airplane mode (disable / enable all networking)");
    }
    airplane.connect_clicked(|_| {
        // We don't know current state cheaply; flip via two calls
        // (off first, then on if already off). The 5 s refresh will
        // catch up.
        let state = Command::new("nmcli").args(["networking"]).output();
        let next = match state {
            Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "enabled" => "off",
            _ => "on",
        };
        let _ = Command::new("nmcli").args(["networking", next]).spawn();
    });
    row.pack_start(&airplane, true, true, 0);

    let rescan = gtk::Button::with_label("Rescan Wi-Fi");
    rescan.set_relief(gtk::ReliefStyle::None);
    rescan.set_tooltip_text(Some("nmcli device wifi rescan"));
    if let Some(atk) = rescan.accessible() {
        atk.set_name("Rescan for Wi-Fi networks");
    }
    rescan.connect_clicked(|_| {
        let _ = Command::new("nmcli").args(["device", "wifi", "rescan"]).spawn();
    });
    row.pack_start(&rescan, true, true, 0);

    let editor = gtk::Button::with_label("Open editor");
    editor.set_relief(gtk::ReliefStyle::None);
    editor.set_tooltip_text(Some("nm-connection-editor — full profile editor"));
    if let Some(atk) = editor.accessible() {
        atk.set_name("Open the NetworkManager connection editor");
    }
    let popover_for_handler = popover.clone();
    editor.connect_clicked(move |_| {
        let _ = Command::new("nm-connection-editor").spawn();
        popover_for_handler.popdown();
    });
    row.pack_start(&editor, true, true, 0);

    row
}

fn placeholder(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.style_context().add_class("mackes-nm-empty");
    l
}

fn nmcli_lines(args: &[&str]) -> Vec<String> {
    let Ok(o) = Command::new("nmcli").args(args).output() else {
        return Vec::new();
    };
    if !o.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_for_each_state_shape() {
        assert_eq!(NmState::default().label(), "Offline");
        let mut st = NmState::default();
        st.airplane = true;
        assert_eq!(st.label(), "Airplane");
        let mut st = NmState::default();
        st.wired_device = Some("eno1".into());
        assert_eq!(st.label(), "Ethernet · eno1");
        let mut st = NmState::default();
        st.wifi = Some(("home-net".into(), "wlp3s0".into()));
        assert_eq!(st.label(), "Wi-Fi · home-net");
    }

    #[test]
    fn icon_picks_right_glyph_for_each_state() {
        assert_eq!(NmState::default().icon_name(), "network-offline-symbolic");
        let mut s = NmState::default();
        s.airplane = true;
        assert_eq!(s.icon_name(), "airplane-mode-symbolic");
        let mut s = NmState::default();
        s.wired_device = Some("x".into());
        assert_eq!(s.icon_name(), "network-wired-symbolic");
        let mut s = NmState::default();
        s.wifi = Some(("x".into(), "y".into()));
        assert_eq!(s.icon_name(), "network-wireless-symbolic");
    }

    #[test]
    fn wired_takes_priority_over_wifi_in_label() {
        let mut s = NmState::default();
        s.wired_device = Some("eno1".into());
        s.wifi = Some(("net".into(), "wlan0".into()));
        assert!(s.label().starts_with("Ethernet"));
    }
}
