//! Text-mode installer (`mde setup` on a headless server) styled after the
//! Windows 2000 NT text-mode Setup: full-screen blue, white text, a bottom
//! key-hint bar. Runs as root, installs everything, registers the greetd
//! session, and switches the machine to graphical startup.

use std::process::{Command, ExitCode};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Gauge, Paragraph, Wrap};
use ratatui::Frame;

const BLUE: Color = Color::Indexed(18); // deep NT setup blue
const TITLE: &str = "MDE-Retro Professional Setup";

/// Core runtime packages (everything else = the 40 system tools).
const CORE: &[&str] = &[
    "sway", "foot", "swaybg", "grim", "wmenu", "NetworkManager",
    "NetworkManager-applet", "greetd", "tuigreet", "pipewire", "wireplumber",
    "xkeyboard-config", "google-droid-sans-fonts", "polkit",
];

#[derive(PartialEq)]
enum Screen {
    NotRoot,
    Welcome,
    Summary,
    Progress,
    Finish,
}

struct Step {
    label: &'static str,
    done: bool,
}

struct App {
    screen: Screen,
    steps: Vec<Step>,
    current: usize,
    dry_run: bool,
}

pub fn run(dry_run: bool) -> ExitCode {
    let mut app = App {
        screen: if dry_run || is_root() { Screen::Welcome } else { Screen::NotRoot },
        steps: vec![
            Step { label: "Collecting information", done: false },
            Step { label: "Installing packages", done: false },
            Step { label: "Deploying configuration", done: false },
            Step { label: "Installing visual assets", done: false },
            Step { label: "Registering session and login manager", done: false },
            Step { label: "Finalizing installation", done: false },
        ],
        current: 0,
        dry_run,
    };

    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> ExitCode {
    loop {
        let _ = terminal.draw(|f| ui(f, app));

        if app.screen == Screen::Progress {
            // Run one step per frame so the screen updates between steps.
            if app.current < app.steps.len() {
                run_step(app.current, app.dry_run);
                app.steps[app.current].done = true;
                app.current += 1;
            } else {
                app.screen = Screen::Finish;
            }
            // Drain any pending key (allow F3 abort) without blocking.
            if let Ok(true) = event::poll(Duration::from_millis(10)) {
                if let Ok(Event::Key(k)) = event::read() {
                    if k.code == KeyCode::F(3) {
                        return ExitCode::from(1);
                    }
                }
            }
            continue;
        }

        if let Ok(Event::Key(k)) = event::read() {
            match (&app.screen, k.code) {
                (Screen::NotRoot, _) => return ExitCode::from(1),
                (Screen::Welcome, KeyCode::Enter) => app.screen = Screen::Summary,
                (Screen::Summary, KeyCode::Enter) => app.screen = Screen::Progress,
                (Screen::Finish, KeyCode::Enter) => return ExitCode::SUCCESS,
                (_, KeyCode::F(3)) | (_, KeyCode::Esc) => return ExitCode::from(1),
                _ => {}
            }
        }
    }
}

// --- install steps ---------------------------------------------------------

fn run_step(i: usize, dry_run: bool) {
    if dry_run {
        std::thread::sleep(Duration::from_millis(450));
        return;
    }
    match i {
        0 => {} // collecting information (root already checked)
        1 => {
            let mut pkgs: Vec<String> = CORE.iter().map(|s| s.to_string()).collect();
            for t in crate::fedora::TOOLS {
                pkgs.push(t.package.to_string());
            }
            pkgs.sort();
            pkgs.dedup();
            let mut args = vec!["install", "-y", "--skip-unavailable"];
            let refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
            args.extend(refs);
            let _ = Command::new("dnf").args(&args).status();
        }
        2 => {
            // configs are shipped by the RPM under /usr/share/mde/skel
            let _ = Command::new("sh")
                .arg("-c")
                .arg("cp -rn /usr/share/mde/skel/. /etc/skel/ 2>/dev/null; true")
                .status();
        }
        3 => {
            let _ = Command::new("sh")
                .arg("-c")
                .arg("cp -rn /usr/share/mde/assets/icons/. /usr/share/icons/ 2>/dev/null; cp -rn /usr/share/mde/assets/sounds/. /usr/share/sounds/ 2>/dev/null; true")
                .status();
        }
        4 => register_session(),
        5 => {
            let _ = Command::new("systemctl").args(["set-default", "graphical.target"]).status();
            let _ = Command::new("systemctl").args(["enable", "--now", "greetd"]).status();
        }
        _ => {}
    }
}

fn register_session() {
    let session = "[Desktop Entry]\nName=MDE-Retro\nComment=Windows 2000 desktop\nExec=sway\nType=Application\n";
    let _ = std::fs::create_dir_all("/usr/share/wayland-sessions");
    let _ = std::fs::write("/usr/share/wayland-sessions/mde-retro.desktop", session);
    let greetd = "[terminal]\nvt = 1\n\n[default_session]\ncommand = \"tuigreet --remember --sessions /usr/share/wayland-sessions\"\nuser = \"greetd\"\n";
    let _ = std::fs::create_dir_all("/etc/greetd");
    let _ = std::fs::write("/etc/greetd/config.toml", greetd);
}

// --- rendering -------------------------------------------------------------

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    // Paint the whole screen NT-setup blue.
    f.render_widget(Block::default().style(Style::default().bg(BLUE)), area);

    let rows = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Min(0),    // body
        Constraint::Length(1), // key bar
    ])
    .split(area);

    let title = Paragraph::new(Line::from(Span::styled(
        TITLE,
        Style::default().fg(Color::White).bg(BLUE).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(title, rows[0]);

    let (body, keys) = match app.screen {
        Screen::NotRoot => (not_root_body(), " Press any key to exit "),
        Screen::Welcome => (welcome_body(), " ENTER=Continue    F3=Exit "),
        Screen::Summary => (summary_body(), " ENTER=Begin Installation    F3=Exit "),
        Screen::Progress => (progress_body(app), " Installing, please wait... "),
        Screen::Finish => (finish_body(), " ENTER=Finish "),
    };

    let inner = centered(rows[1], 64, 16);
    f.render_widget(
        Paragraph::new(body)
            .style(Style::default().fg(Color::White).bg(BLUE))
            .wrap(Wrap { trim: false }),
        inner,
    );

    let keybar = Paragraph::new(Line::from(Span::styled(
        keys,
        Style::default().fg(Color::Black).bg(Color::Gray),
    )))
    .alignment(Alignment::Left);
    f.render_widget(keybar, rows[2]);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w, height: h }
}

fn not_root_body() -> ratatui::text::Text<'static> {
    "Setup cannot continue.\n\nMDE-Retro Setup must be run with administrator (root) privileges.\n\n    Please run:   sudo mde setup\n".into()
}

fn welcome_body() -> ratatui::text::Text<'static> {
    "Welcome to Setup.\n\nThis portion of Setup prepares MDE-Retro -- a Windows 2000 desktop for Fedora -- to run on your computer.\n\n  - To set up MDE-Retro now, press ENTER.\n  - To quit Setup without installing, press F3.".into()
}

fn summary_body() -> ratatui::text::Text<'static> {
    "Setup will perform the following on this computer:\n\n  - Install the desktop runtime and all system tools (dnf)\n  - Deploy the MDE-Retro configuration (system-wide and per user)\n  - Install the Windows 2000 icons, cursors, sounds and fonts\n  - Register the MDE-Retro session and the greetd login manager\n  - Switch the system to graphical startup\n\n  To begin installation, press ENTER.".into()
}

fn progress_body(app: &App) -> ratatui::text::Text<'static> {
    let mut lines: Vec<Line> = Vec::new();
    for (i, s) in app.steps.iter().enumerate() {
        let mark = if s.done {
            "[done]    "
        } else if i == app.current {
            "[working] "
        } else {
            "[ ]       "
        };
        lines.push(Line::from(format!("{mark}{}", s.label)));
    }
    ratatui::text::Text::from(lines)
}

fn finish_body() -> ratatui::text::Text<'static> {
    "MDE-Retro has been installed on this computer.\n\nThe graphical environment (greetd) will now start, and the MDE-Retro logon screen will appear.\n\n  Press ENTER to start the graphical environment.".into()
}

#[allow(dead_code)]
fn _gauge(app: &App) -> Gauge<'static> {
    Gauge::default().ratio(app.current as f64 / app.steps.len() as f64)
}
