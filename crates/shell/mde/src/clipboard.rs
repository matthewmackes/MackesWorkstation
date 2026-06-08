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

/// What a history entry holds: inline text, or a path to a saved PNG blob.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipKind {
    Text(String),
    Image(String),
}

/// One clipboard history entry: its content + whether it's pinned (E16.3 — pinned
/// entries survive "Clear all" and the ring cap).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipEntry {
    pub kind: ClipKind,
    #[serde(default)]
    pub pinned: bool,
}

impl ClipEntry {
    fn new(kind: ClipKind) -> Self {
        ClipEntry {
            kind,
            pinned: false,
        }
    }
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

pub(crate) fn save_index(idx: &[ClipEntry]) -> std::io::Result<()> {
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

/// Prepend `entry` to `idx`, moving an identical-content existing entry to the
/// front (dedup, inheriting its pinned flag) rather than stacking duplicates, then
/// cap at [`RING`] — keeping ALL pinned entries plus the most-recent unpinned ones
/// (order preserved). Pure — unit-tested.
pub fn ring_push(mut idx: Vec<ClipEntry>, mut entry: ClipEntry) -> Vec<ClipEntry> {
    if let Some(pos) = idx.iter().position(|e| e.kind == entry.kind) {
        entry.pinned |= idx[pos].pinned; // a re-copied pinned item stays pinned
        idx.remove(pos);
    }
    idx.insert(0, entry);
    // Cap: pinned entries never count against the budget and are never dropped.
    let budget = RING.saturating_sub(idx.iter().filter(|e| e.pinned).count());
    let mut kept_unpinned = 0;
    idx.retain(|e| {
        if e.pinned {
            true
        } else if kept_unpinned < budget {
            kept_unpinned += 1;
            true
        } else {
            false
        }
    });
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
            push_entry(ClipEntry::new(ClipKind::Text(s)));
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
        push_entry(ClipEntry::new(ClipKind::Image(path.display().to_string())));
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
        let pin = if e.pinned { "[pinned] " } else { "" };
        match &e.kind {
            ClipKind::Text(t) => println!("{pin}text: {}", t.replace('\n', "⏎")),
            ClipKind::Image(p) => println!("{pin}image: {p}"),
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
    match &entry.kind {
        ClipKind::Text(s) => {
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
        ClipKind::Image(p) => {
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
    use super::{save_index, ClipKind};
    use iced::widget::{button, container, image, mouse_area, scrollable, text, Column, Row};
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
        TogglePin(usize),
        Delete(usize),
        ClearAll,
        Event(Event),
    }

    fn exe() -> std::path::PathBuf {
        std::env::current_exe().unwrap_or_else(|_| "mde".into())
    }

    pub fn run() -> ExitCode {
        // E9: the Win+V history popup is now a universal Carbon surface (was
        // Windows-10-era-gated; the daemon itself was always era-neutral).
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
            // Pin / delete / clear mutate the on-screen list and persist it in place.
            Message::TogglePin(i) => {
                if let Some(e) = state.entries.get_mut(i) {
                    e.pinned = !e.pinned;
                }
                let _ = save_index(&state.entries);
                Task::none()
            }
            Message::Delete(i) => {
                if i < state.entries.len() {
                    state.entries.remove(i);
                }
                let _ = save_index(&state.entries);
                Task::none()
            }
            Message::ClearAll => {
                state.entries.retain(|e| e.pinned);
                let _ = save_index(&state.entries);
                Task::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })) => exit(0),
            _ => Task::none(),
        }
    }

    /// A small flat glyph button (pin / delete), Nerd Font.
    fn glyph_btn(glyph: &'static str, accent: bool, msg: Message) -> Element<'static, Message> {
        let color = if accent {
            palette::accent()
        } else {
            palette::color(palette::GRAY_TEXT)
        };
        button(
            text(glyph)
                .font(mde_ui::font::NERD)
                .size(metrics::UI_PX)
                .color(color),
        )
        .on_press(msg)
        .padding(Padding::from([2.0, 6.0]))
        .style(mde_ui::button_ghost)
        .into()
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
        let mut list = Column::new().spacing(metrics::SPACING_03);
        if state.entries.is_empty() {
            list = list.push(
                text("Clipboard history is empty. Copy something to see it here.")
                    .size(metrics::UI_PX)
                    .color(palette::color(palette::GRAY_TEXT)),
            );
        } else {
            for (i, e) in state.entries.iter().enumerate() {
                let body: Element<Message> = match &e.kind {
                    ClipKind::Text(t) => text(truncate(t, 38))
                        .size(metrics::UI_PX)
                        .color(palette::color(palette::WINDOW_TEXT))
                        .into(),
                    ClipKind::Image(p) => image(p.clone()).height(Length::Fixed(40.0)).into(),
                };
                // Content (click to re-copy) + pin toggle + delete.
                let row = Row::new()
                    .spacing(metrics::SPACING_02)
                    .align_y(iced::alignment::Vertical::Center)
                    .push(
                        mouse_area(
                            container(body)
                                .width(Length::Fill)
                                .padding(metrics::SPACING_03),
                        )
                        .on_press(Message::Copy(i)),
                    )
                    .push(glyph_btn("\u{f08d}", e.pinned, Message::TogglePin(i))) // thumbtack
                    .push(glyph_btn("\u{f00d}", false, Message::Delete(i))); // times
                list = list.push(
                    container(row)
                        .width(Length::Fill)
                        .style(|t: &iced::Theme| row_style(t)),
                );
            }
        }

        let header = Row::new()
            .align_y(iced::alignment::Vertical::Center)
            .push(
                text("Clipboard")
                    .size(metrics::INFO_TITLE_PX)
                    .width(Length::Fill)
                    .color(palette::color(palette::WINDOW_TEXT)),
            )
            .push(
                button(text("Clear all").size(metrics::UI_PX))
                    .on_press(Message::ClearAll)
                    .padding(Padding::from([metrics::SPACING_01, metrics::SPACING_03]))
                    .style(mde_ui::button_ghost),
            );

        let card = container(
            Column::new()
                .spacing(metrics::SPACING_03)
                .push(header)
                .push(
                    container(scrollable(list).style(mde_ui::scrollbar))
                        .height(Length::Fixed(360.0)),
                ),
        )
        .width(Length::Fixed(320.0))
        .padding(metrics::SPACING_04)
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
        ClipEntry::new(ClipKind::Text(s.to_string()))
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

    #[test]
    fn pinned_entries_survive_the_cap() {
        // Start with a pinned entry, then push RING+5 newer unpinned ones.
        let mut idx = vec![ClipEntry {
            kind: ClipKind::Text("keep me".into()),
            pinned: true,
        }];
        for i in 0..(RING + 5) {
            idx = ring_push(idx, t(&format!("u{i}")));
        }
        // The pinned entry is never dropped, even though far more than RING
        // unpinned items have arrived; the rest are capped to fit the budget.
        assert!(idx
            .iter()
            .any(|e| e.pinned && e.kind == ClipKind::Text("keep me".into())));
        assert_eq!(idx.iter().filter(|e| e.pinned).count(), 1);
        assert_eq!(idx.iter().filter(|e| !e.pinned).count(), RING - 1);
    }
}
