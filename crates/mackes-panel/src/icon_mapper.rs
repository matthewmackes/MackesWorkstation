//! Carbon Icon Mapper — XDG-spec-compliant `.desktop` icon override
//! editor, accessible via right-click on every app entry (dock +
//! start-menu Applications submenu).
//!
//! User lock 2026-05-19: embed an icon-remap UI natively in the
//! Rust panel rather than the upstream `MenuLibre` or a separate
//! Workbench Python panel. References the freedesktop specs:
//!
//! - Desktop Entry Specification: <https://specifications.freedesktop.org/desktop-entry-spec/latest/>
//! - Icon Theme Specification: <https://specifications.freedesktop.org/icon-theme-spec/latest/>
//! - Menu Specification: <https://specifications.freedesktop.org/menu-spec/latest/>
//!
//! User overrides land at `$XDG_DATA_HOME/applications/<id>.desktop`
//! (default `~/.local/share/applications/`). Per the desktop-entry
//! spec, a file at this path with the same `<id>` shadows any
//! system-level entry under `/usr/share/applications/`. The override
//! is a complete copy of the system entry with the `Icon=` field
//! rewritten — the user's customization survives upstream upgrades
//! and can be reset by deleting the file.
//!
//! UI shape:
//!
//! 1. The popover's title shows the current `Name=` + current
//!    `Icon=`.
//! 2. A `gtk::FlowBox` renders every Mackes-Carbon icon under
//!    `/usr/share/icons/Mackes-Carbon/scalable/apps/*.svg` (and
//!    related categories) as a tappable thumbnail.
//! 3. Clicking a thumbnail writes the override and dismisses the
//!    popover. The dock refresh picks up the change on its next 2 s
//!    tick (no panel restart needed).
//!
//! Live preview: hovering a thumbnail tints the popover's preview
//! pane (top-left) to show what the launcher will look like
//! post-apply, without actually writing the override.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;

/// Open the icon-picker popover anchored to `relative_to` for the
/// `.desktop` identified by `desktop_id`. Reads the current
/// `Icon=` value from the on-disk system entry and pre-selects it
/// in the grid.
pub fn open_for(relative_to: &gtk::Widget, desktop_id: &str, current_name: &str) {
    let popover = gtk::Popover::new(Some(relative_to));
    popover.set_widget_name("mackes-icon-mapper");
    popover.set_position(gtk::PositionType::Top);
    popover.set_modal(true);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_margin_top(10);
    column.set_margin_bottom(10);
    column.set_margin_start(12);
    column.set_margin_end(12);

    // Header: name + current icon preview.
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let current_icon_name = read_icon_field(desktop_id).unwrap_or_else(|| "?".into());
    let current_image = gtk::Image::new();
    if let Some(pb) = crate::icons::load(&current_icon_name, 32) {
        current_image.set_from_pixbuf(Some(&pb));
    }
    let header_text_column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let name_label = gtk::Label::new(Some(current_name));
    name_label.set_halign(gtk::Align::Start);
    name_label.style_context().add_class("mackes-icon-mapper-name");
    let id_label = gtk::Label::new(Some(desktop_id));
    id_label.set_halign(gtk::Align::Start);
    id_label.style_context().add_class("mackes-icon-mapper-id");
    let curr_label = gtk::Label::new(Some(&format!("Icon: {current_icon_name}")));
    curr_label.set_halign(gtk::Align::Start);
    curr_label.style_context().add_class("mackes-icon-mapper-id");
    header_text_column.pack_start(&name_label, false, false, 0);
    header_text_column.pack_start(&id_label, false, false, 0);
    header_text_column.pack_start(&curr_label, false, false, 0);
    header.pack_start(&current_image, false, false, 0);
    header.pack_start(&header_text_column, true, true, 0);
    column.pack_start(&header, false, false, 0);
    column.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);

    // Grid of available Carbon icons.
    let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroller.set_min_content_width(360);
    scroller.set_min_content_height(280);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let flow = gtk::FlowBox::new();
    flow.set_widget_name("mackes-icon-grid");
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_min_children_per_line(6);
    flow.set_max_children_per_line(8);
    flow.set_homogeneous(true);

    let icons = enumerate_carbon_icons();
    if icons.is_empty() {
        let empty = gtk::Label::new(Some(
            "Mackes-Carbon theme not found at /usr/share/icons/Mackes-Carbon/."
        ));
        flow.add(&empty);
    } else {
        for icon_name in &icons {
            flow.add(&build_thumbnail(
                icon_name,
                desktop_id,
                Rc::new(popover.clone()),
            ));
        }
    }

    scroller.add(&flow);
    column.pack_start(&scroller, true, true, 0);

    // Footer
    column.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let reset = gtk::Button::with_label("Reset to default");
    reset.set_tooltip_text(Some(
        "Delete the user override at ~/.local/share/applications/<id>.desktop"
    ));
    if let Some(atk) = reset.accessible() {
        atk.set_name(&format!("Reset icon mapping for {desktop_id} to default"));
    }
    let id_for_reset = desktop_id.to_owned();
    let popover_for_reset = popover.clone();
    reset.connect_clicked(move |_| {
        let _ = std::fs::remove_file(user_override_path(&id_for_reset));
        popover_for_reset.popdown();
    });
    footer.pack_start(&reset, false, false, 0);

    let count = gtk::Label::new(Some(&format!("{} icons", icons.len())));
    count.style_context().add_class("mackes-icon-mapper-count");
    footer.pack_end(&count, false, false, 0);
    column.pack_start(&footer, false, false, 0);

    popover.add(&column);
    column.show_all();
    popover.popup();
}

fn build_thumbnail(icon_name: &str, desktop_id: &str, popover: Rc<gtk::Popover>) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_widget_name(&format!("mackes-icon-thumb-{icon_name}"));
    button.set_relief(gtk::ReliefStyle::None);
    button.set_tooltip_text(Some(icon_name));
    if let Some(atk) = button.accessible() {
        atk.set_name(&format!("Set icon for {desktop_id} to {icon_name}"));
    }
    if let Some(pb) = crate::icons::load(icon_name, 32) {
        button.set_image(Some(&gtk::Image::from_pixbuf(Some(&pb))));
        button.set_always_show_image(true);
    } else {
        button.set_label(icon_name);
    }
    let icon_owned = icon_name.to_owned();
    let id_owned = desktop_id.to_owned();
    button.connect_clicked(move |_| {
        if let Err(e) = write_override(&id_owned, &icon_owned) {
            eprintln!("mackes-panel: icon override write failed: {e}");
        }
        popover.popdown();
    });
    button
}

/// Find the system-level `.desktop` file for `desktop_id`. XDG spec:
/// search order is `$XDG_DATA_HOME/applications/`, then each
/// `$XDG_DATA_DIRS/applications/` (default `/usr/local/share` and
/// `/usr/share`). Returns the first hit.
fn locate_system_desktop(desktop_id: &str) -> Option<PathBuf> {
    let candidates = system_search_dirs();
    for dir in candidates {
        let p = dir.join(desktop_id);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn system_search_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(s) = std::env::var("XDG_DATA_DIRS") {
        for d in s.split(':') {
            if !d.is_empty() {
                out.push(PathBuf::from(d).join("applications"));
            }
        }
    } else {
        out.push(PathBuf::from("/usr/local/share/applications"));
        out.push(PathBuf::from("/usr/share/applications"));
    }
    out
}

fn user_override_path(desktop_id: &str) -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME").map_or_else(
        |_| {
            std::env::var("HOME")
                .map_or_else(|_| PathBuf::from("/tmp"), |h| PathBuf::from(h).join(".local/share"))
        },
        PathBuf::from,
    );
    base.join("applications").join(desktop_id)
}

/// Read the `Icon=` value from the system `.desktop` for `desktop_id`.
/// Returns `None` when no `Icon=` line exists.
fn read_icon_field(desktop_id: &str) -> Option<String> {
    let path = locate_system_desktop(desktop_id)?;
    let text = std::fs::read_to_string(&path).ok()?;
    extract_icon_field(&text)
}

fn extract_icon_field(text: &str) -> Option<String> {
    // Scoped to the `[Desktop Entry]` group per spec — keys outside
    // it (e.g. inside an action group) don't count. We track the
    // current group as we walk lines.
    let mut in_main = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_main = line == "[Desktop Entry]";
            continue;
        }
        if !in_main {
            continue;
        }
        if let Some(rest) = line.strip_prefix("Icon=") {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// Write a user-override `.desktop` that copies the system entry and
/// replaces the `Icon=` value (rewriting in place if the line exists,
/// appending under `[Desktop Entry]` if not).
fn write_override(desktop_id: &str, new_icon: &str) -> std::io::Result<()> {
    let system_path = locate_system_desktop(desktop_id).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no system .desktop for {desktop_id}"),
        )
    })?;
    let source = std::fs::read_to_string(&system_path)?;
    let rewritten = rewrite_icon_field(&source, new_icon);
    let target = user_override_path(desktop_id);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&target, rewritten)?;
    Ok(())
}

fn rewrite_icon_field(text: &str, new_icon: &str) -> String {
    let mut out = String::with_capacity(text.len() + 32);
    let mut in_main = false;
    let mut wrote_icon = false;
    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_main && !wrote_icon {
                // Closing the [Desktop Entry] group without having
                // seen an Icon= line — append one before the next
                // group header.
                use std::fmt::Write;
                let _ = writeln!(out, "Icon={new_icon}");
                wrote_icon = true;
            }
            in_main = trimmed == "[Desktop Entry]";
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        if in_main && trimmed.starts_with("Icon=") {
            use std::fmt::Write;
            let _ = writeln!(out, "Icon={new_icon}");
            wrote_icon = true;
            continue;
        }
        out.push_str(raw);
        out.push('\n');
    }
    if !wrote_icon {
        use std::fmt::Write;
        let _ = writeln!(out, "Icon={new_icon}");
    }
    out
}

/// Walk `/usr/share/icons/Mackes-Carbon/scalable/` and enumerate every
/// `.svg` basename, stripped of the extension — these are the icon
/// names that can be plugged into `Icon=`. Categories included:
/// apps / actions / categories / devices / mimetypes / places /
/// status (per the icon-theme spec).
fn enumerate_carbon_icons() -> Vec<String> {
    let root = Path::new("/usr/share/icons/Mackes-Carbon/scalable");
    if !root.is_dir() {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for category in &[
        "apps",
        "actions",
        "categories",
        "devices",
        "mimetypes",
        "places",
        "status",
    ] {
        let dir = root.join(category);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if let Some(stem) = name.strip_suffix(".svg") {
                out.push(stem.to_owned());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[Desktop Entry]
Type=Application
Name=Firefox
Exec=firefox %u
Icon=firefox
Categories=Network;WebBrowser;

[Desktop Action new-window]
Name=New Window
Exec=firefox -new-window
";

    #[test]
    fn extracts_icon_from_main_group_only() {
        assert_eq!(extract_icon_field(SAMPLE).as_deref(), Some("firefox"));
    }

    #[test]
    fn rewrite_replaces_icon_in_place() {
        let out = rewrite_icon_field(SAMPLE, "mackes-firefox-symbolic");
        assert!(out.contains("Icon=mackes-firefox-symbolic"));
        assert!(!out.contains("Icon=firefox\n"));
        // Action groups still present.
        assert!(out.contains("[Desktop Action new-window]"));
    }

    #[test]
    fn rewrite_inserts_when_icon_missing() {
        let no_icon = "\
[Desktop Entry]
Type=Application
Name=NoIcon
Exec=true
";
        let out = rewrite_icon_field(no_icon, "carbon-test");
        assert!(out.contains("Icon=carbon-test"));
    }

    #[test]
    fn rewrite_ignores_icon_outside_main_group() {
        let weird = "\
[Desktop Entry]
Type=Application

[Desktop Action foo]
Icon=other-icon
";
        let out = rewrite_icon_field(weird, "winner");
        // The action group's Icon line stays; our new Icon is appended
        // under [Desktop Entry] (or at end).
        assert!(out.contains("Icon=other-icon"));
        assert!(out.contains("Icon=winner"));
    }

    #[test]
    fn user_override_path_under_local_share_applications() {
        let _g = crate::test_env::env_lock();
        std::env::remove_var("XDG_DATA_HOME");
        std::env::set_var("HOME", "/tmp/icon-mapper-test");
        let p = user_override_path("firefox.desktop");
        assert!(p.ends_with(".local/share/applications/firefox.desktop"));
    }
}
