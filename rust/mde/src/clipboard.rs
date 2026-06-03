//! Clipboard history daemon + index (E16.2).
//!
//! `mde clipboard daemon` runs two `wl-paste --watch` watchers (text + image/png)
//! that re-invoke this binary to append each new clipboard item to a 25-entry ring
//! at `~/.local/share/mde/clipboard/index.json` (atomic write). It is
//! lockfile-idempotent: a second `daemon` launch sees the live PID and exits
//! without starting a duplicate watcher. The history popup (E16.1) reads the same
//! index and re-copies a chosen entry via `wl-copy`.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::{Deserialize, Serialize};

/// Maximum entries kept in the ring.
pub const RING: usize = 25;

/// One clipboard history entry: inline text, or a path to a saved PNG blob.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipEntry {
    Text(String),
    Image(String),
}

/// `~/.local/share/mde/clipboard/` (honours `$XDG_DATA_HOME`).
pub fn dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("mde").join("clipboard"))
}

fn index_path() -> Option<PathBuf> {
    dir().map(|d| d.join("index.json"))
}

/// The current history (most-recent first), or empty on any problem.
pub fn load_index() -> Vec<ClipEntry> {
    index_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_index(idx: &[ClipEntry]) -> std::io::Result<()> {
    let Some(path) = index_path() else {
        return Ok(());
    };
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let json = serde_json::to_string_pretty(idx)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}

/// Prepend `entry` to `idx`, moving an identical existing entry to the front
/// (dedup) rather than stacking duplicates, and cap at [`RING`]. Pure — unit-tested.
pub fn ring_push(mut idx: Vec<ClipEntry>, entry: ClipEntry) -> Vec<ClipEntry> {
    idx.retain(|e| e != &entry);
    idx.insert(0, entry);
    idx.truncate(RING);
    idx
}

fn push_entry(entry: ClipEntry) {
    let _ = save_index(&ring_push(load_index(), entry));
}

/// `mde clipboard __add` — append the stdin text (skips empty/whitespace).
fn add_text() {
    let mut s = String::new();
    if std::io::stdin().read_to_string(&mut s).is_ok() {
        let s = s.trim_end_matches('\n').to_string();
        if !s.trim().is_empty() {
            push_entry(ClipEntry::Text(s));
        }
    }
}

/// `mde clipboard __add-image` — save the stdin PNG bytes to a blob + append a ref.
fn add_image() {
    let mut bytes = Vec::new();
    if std::io::stdin().read_to_end(&mut bytes).is_err() || bytes.is_empty() {
        return;
    }
    let Some(d) = dir() else {
        return;
    };
    let _ = std::fs::create_dir_all(&d);
    let path = d.join(format!("img-{}.png", next_img_id(&d)));
    if std::fs::write(&path, &bytes).is_ok() {
        push_entry(ClipEntry::Image(path.display().to_string()));
    }
}

/// A monotonic image id from a counter file (no clock/RNG, so it's resume-safe).
fn next_img_id(d: &Path) -> u64 {
    let cf = d.join(".img-counter");
    let n = std::fs::read_to_string(&cf)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
        + 1;
    let _ = std::fs::write(&cf, n.to_string());
    n
}

/// The clipboard watcher daemon. Lockfile-idempotent: if a live PID holds the
/// lock, exit. Otherwise spawn the text + image watchers and run until the
/// compositor (and thus `wl-paste`) goes away.
fn run_daemon() -> ExitCode {
    let Some(d) = dir() else {
        return ExitCode::FAILURE;
    };
    let _ = std::fs::create_dir_all(&d);
    let lock = d.join("daemon.lock");
    if let Some(pid) = std::fs::read_to_string(&lock)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
    {
        if Path::new(&format!("/proc/{pid}")).exists() {
            eprintln!("mde clipboard daemon: already running (pid {pid}).");
            return ExitCode::SUCCESS;
        }
    }
    let _ = std::fs::write(&lock, std::process::id().to_string());

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "mde".to_string());
    // `wl-paste --watch CMD…` runs CMD with the new clipboard content on its stdin.
    let watch = |ty: &str, sub: &str| {
        Command::new("wl-paste")
            .args(["--type", ty, "--watch", &exe, "clipboard", sub])
            .spawn()
            .ok()
    };
    let mut text = watch("text", "__add");
    let mut img = watch("image/png", "__add-image");
    // Block on the text watcher (it lives as long as the Wayland session); then
    // tidy up the image watcher and the lock.
    if let Some(c) = text.as_mut() {
        let _ = c.wait();
    }
    if let Some(c) = img.as_mut() {
        let _ = c.kill();
        let _ = c.wait();
    }
    let _ = std::fs::remove_file(&lock);
    ExitCode::SUCCESS
}

/// Headless dump of the history (one line per entry) for `mde clipboard --list`.
fn debug_list() {
    for e in load_index() {
        match e {
            ClipEntry::Text(t) => println!("text: {}", t.replace('\n', "⏎")),
            ClipEntry::Image(p) => println!("image: {p}"),
        }
    }
}

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("daemon") => run_daemon(),
        Some("__add") => {
            add_text();
            ExitCode::SUCCESS
        }
        Some("__add-image") => {
            add_image();
            ExitCode::SUCCESS
        }
        Some("--list") => {
            debug_list();
            ExitCode::SUCCESS
        }
        // No subcommand → the Win10 history popup (Win+V).
        _ => popup::run(),
    }
}

/// Re-copy a history entry to the clipboard via `wl-copy` (text on stdin, or the
/// PNG blob as `image/png`).
pub fn copy_entry(entry: &ClipEntry) {
    match entry {
        ClipEntry::Text(s) => {
            if let Ok(mut child) = Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut si) = child.stdin.take() {
                    use std::io::Write;
                    let _ = si.write_all(s.as_bytes());
                }
                let _ = child.wait();
            }
        }
        ClipEntry::Image(p) => {
            if let Ok(f) = std::fs::File::open(p) {
                let _ = Command::new("wl-copy")
                    .args(["--type", "image/png"])
                    .stdin(f)
                    .status();
            }
        }
    }
}

/// The Windows 10 clipboard-history popup (E16.1): a flat card of recent items;
/// click one to re-copy it (with a "Copied" toast) and close.
mod popup {
    use super::{copy_entry, load_index, ClipEntry};
    use iced::widget::{container, image, mouse_area, scrollable, text, Column};
    use iced::{event, keyboard, Background, Border, Color, Element, Event, Length, Padding, Task};
    use iced_layershell::build_pattern::{application, MainSettings};
    use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
    use iced_layershell::settings::LayerShellSettings;
    use iced_layershell::{to_layer_message, Appearance};
    use mde_ui::{metrics, palette};
    use std::process::{exit, Command, ExitCode};

    struct Clip {
        entries: Vec<ClipEntry>,
    }

    #[to_layer_message]
    #[derive(Debug, Clone)]
    enum Message {
        Copy(usize),
        Event(Event),
    }

    fn exe() -> std::path::PathBuf {
        std::env::current_exe().unwrap_or_else(|_| "mde".into())
    }

    pub fn run() -> ExitCode {
        // The history popup is a Windows 10-era surface (the daemon itself is
        // era-neutral). Other eras have no Win+V clipboard history.
        if !palette::is_windows10() {
            eprintln!("mde clipboard: the history popup is a Windows 10-era surface.");
            return ExitCode::SUCCESS;
        }
        // Make sure the watcher is running so the history actually fills (idempotent).
        let _ = Command::new(exe()).args(["clipboard", "daemon"]).spawn();

        let r = application(namespace, update, view)
            .style(style)
            .subscription(|_: &Clip| {
                event::listen_with(|e, _s, _w| match e {
                    Event::Keyboard(_) => Some(Message::Event(e)),
                    _ => None,
                })
            })
            .font(mde_ui::font::REGULAR_BYTES)
            .font(mde_ui::font::BOLD_BYTES)
            .font(mde_ui::font::PLEX_REGULAR_BYTES)
            .font(mde_ui::font::PLEX_BOLD_BYTES)
            .default_font(mde_ui::font::ui())
            .settings(MainSettings {
                layer_settings: LayerShellSettings {
                    anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                    exclusive_zone: 0,
                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                    ..Default::default()
                },
                ..Default::default()
            })
            .run_with(|| {
                (
                    Clip {
                        entries: load_index(),
                    },
                    Task::none(),
                )
            });
        match r {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }

    fn namespace(_: &Clip) -> String {
        "mde-clipboard".to_string()
    }

    fn style(_: &Clip, _: &iced::Theme) -> Appearance {
        Appearance {
            background_color: Color::TRANSPARENT,
            text_color: palette::color(palette::WINDOW_TEXT),
        }
    }

    fn update(state: &mut Clip, message: Message) -> Task<Message> {
        match message {
            Message::Copy(i) => {
                if let Some(e) = state.entries.get(i) {
                    copy_entry(e);
                    let _ = Command::new(exe()).args(["toast", "Copied"]).spawn();
                }
                exit(0)
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })) => exit(0),
            _ => Task::none(),
        }
    }

    fn row_style(_: &iced::Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(palette::color(palette::BUTTON_FACE))),
            border: Border {
                color: palette::color(palette::WINDOW_FRAME),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        }
    }

    fn truncate(s: &str, n: usize) -> String {
        let one = s.replace('\n', " ");
        if one.chars().count() > n {
            format!("{}…", one.chars().take(n - 1).collect::<String>())
        } else {
            one
        }
    }

    fn view(state: &Clip) -> Element<'_, Message> {
        let mut list = Column::new().spacing(6.0);
        if state.entries.is_empty() {
            list = list.push(
                text("Clipboard history is empty. Copy something to see it here.")
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::GRAY_TEXT)),
            );
        } else {
            for (i, e) in state.entries.iter().enumerate() {
                let body: Element<Message> = match e {
                    ClipEntry::Text(t) => text(truncate(t, 48))
                        .size(metrics::UI_PX)
                        .color(palette::color(palette::WINDOW_TEXT))
                        .into(),
                    ClipEntry::Image(p) => image(p.clone()).height(Length::Fixed(48.0)).into(),
                };
                list = list.push(
                    mouse_area(
                        container(body)
                            .width(Length::Fill)
                            .padding(8.0)
                            .style(|t: &iced::Theme| row_style(t)),
                    )
                    .on_press(Message::Copy(i)),
                );
            }
        }

        let card = container(
            Column::new()
                .spacing(8.0)
                .push(
                    text("Clipboard")
                        .size(metrics::INFO_TITLE_PX)
                        .color(palette::color(palette::WINDOW_TEXT)),
                )
                .push(
                    container(scrollable(list).style(mde_ui::scrollbar))
                        .height(Length::Fixed(360.0)),
                ),
        )
        .width(Length::Fixed(320.0))
        .padding(10.0)
        .style(|_| container::Style {
            background: Some(Background::Color(palette::color(palette::WINDOW))),
            border: Border {
                color: palette::color(palette::WINDOW_FRAME),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        });

        // Bottom-left, above the taskbar — the Win+V position.
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 44.0,
                left: 8.0,
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> ClipEntry {
        ClipEntry::Text(s.to_string())
    }

    #[test]
    fn ring_prepends_dedups_and_caps() {
        let mut idx = Vec::new();
        idx = ring_push(idx, t("a"));
        idx = ring_push(idx, t("b"));
        // Most-recent first.
        assert_eq!(idx, vec![t("b"), t("a")]);
        // Re-copying "a" moves it to the front (dedup, not a duplicate).
        idx = ring_push(idx, t("a"));
        assert_eq!(idx, vec![t("a"), t("b")]);
        // Capped at RING.
        let mut full = Vec::new();
        for i in 0..(RING + 5) {
            full = ring_push(full, t(&format!("e{i}")));
        }
        assert_eq!(full.len(), RING);
        assert_eq!(full[0], t(&format!("e{}", RING + 4))); // newest kept
    }
}
