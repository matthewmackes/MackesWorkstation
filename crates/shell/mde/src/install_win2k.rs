//! RETIRE-PY.6a — native Win2k icon-theme installer.
//!
//! A faithful Rust port of the retired `install-win2k-icons.py` (the sole
//! `python3` consumer in the asset-install path — see RETIRE-PY.6). It
//! installs the KDE-Store "Windows 2000" icon set (item 1120706) as a
//! spec-compliant freedesktop icon theme usable by GTK apps:
//!
//!   1. fetch the upstream tarball (when not already cached) — per locked
//!      decision #7 the asset *bytes* are pulled at install time, never
//!      redistributed, so only this code ships;
//!   2. extract it into `~/.local/share/icons/Win2k`, stripping the leading
//!      `Win2k-2.2.2-1/` archive component;
//!   3. copy freedesktop-named aliases (firefox → konqueror, terminal →
//!      konsole, folder, trash, mimetypes …) so modern apps resolve icons;
//!   4. write an `index.theme` (`Inherits=hicolor,Adwaita` so anything we
//!      don't cover falls back gracefully);
//!   5. refresh the GTK icon cache when `gtk-update-icon-cache` is present.
//!
//! Re-running is idempotent: the theme dir is rebuilt from the tarball.
//!
//! Reachable via `mde install --only win2k` (and the default `mde install
//! --assets`, where it is the native Win2k step). The cache moved from the
//! python's stale `~/.config/sway/resources` to `$XDG_CACHE_HOME/mde` — a
//! downloaded asset belongs in the cache, and the sway path is retired under
//! labwc. Override the source with `$MDE_WIN2K_URL`.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

const THEME_NAME: &str = "Win2k";
const TARBALL_NAME: &str = "133-Win2k-2.2.2-1.tgz";
const DEFAULT_URL: &str = "https://files.kde.org/store/133-Win2k-2.2.2-1.tgz";

/// KDE-2 category dir → freedesktop `Context`. A size-dir subdir is only
/// listed in `index.theme` when its name is one of these.
const CONTEXT: &[(&str, &str)] = &[
    ("actions", "Actions"),
    ("apps", "Applications"),
    ("devices", "Devices"),
    ("filesystems", "Places"),
    ("mimetypes", "MimeTypes"),
];

/// freedesktop icon name → source file (relative to a size dir). The alias is
/// only created when the source actually exists at that size.
const ALIASES: &[(&str, &str)] = &[
    // --- Applications --------------------------------------------------
    ("firefox", "apps/konqueror.png"),
    ("web-browser", "apps/konqueror.png"),
    ("foot", "apps/konsole.png"),
    ("org.codeberg.dnkl.foot", "apps/konsole.png"),
    ("utilities-terminal", "apps/konsole.png"),
    ("terminal", "apps/konsole.png"),
    ("text-editor", "apps/kate.png"),
    ("accessories-text-editor", "apps/kate.png"),
    ("system-file-manager", "apps/kfm.png"),
    ("preferences-system", "apps/kcontrol.png"),
    ("systemsettings", "apps/kcontrol.png"),
    ("preferences-desktop", "apps/kcontrol.png"),
    ("help-browser", "apps/khelpcenter.png"),
    ("preferences-desktop-font", "apps/fonts.png"),
    ("preferences-desktop-keyboard", "apps/keyboard.png"),
    ("utilities-system-monitor", "apps/ksysguard.png"),
    ("gparted", "apps/kcmpartitions.png"),
    ("printer", "devices/printer1.png"),
    ("preferences-system-printer", "devices/printer1.png"),
    ("clock", "apps/clock.png"),
    ("preferences-desktop-screensaver", "apps/kscreensaver.png"),
    // --- Places --------------------------------------------------------
    ("folder", "filesystems/folder.png"),
    ("folder-open", "filesystems/folder_open.png"),
    ("inode-directory", "filesystems/folder.png"),
    ("user-home", "filesystems/folder_home.png"),
    ("folder-home", "filesystems/folder_home.png"),
    ("user-desktop", "filesystems/desktop.png"),
    ("user-trash", "filesystems/trashcan_empty.png"),
    ("user-trash-full", "filesystems/trashcan_full.png"),
    ("network-workgroup", "filesystems/network.png"),
    ("network-server", "filesystems/network.png"),
    ("folder-remote", "filesystems/network.png"),
    ("emblem-important", "filesystems/folder_important.png"),
    // --- Generic executable fallback -----------------------------------
    ("application-x-executable", "filesystems/exec.png"),
    ("application-default-icon", "filesystems/exec.png"),
    ("exec", "filesystems/exec.png"),
    // --- MimeTypes -----------------------------------------------------
    ("text-x-generic", "mimetypes/txt.png"),
    ("text-plain", "mimetypes/txt.png"),
    ("text-html", "mimetypes/html.png"),
    ("image-x-generic", "mimetypes/image.png"),
    ("audio-x-generic", "mimetypes/sound.png"),
    ("video-x-generic", "mimetypes/video.png"),
    ("font-x-generic", "mimetypes/font.png"),
    ("application-x-shellscript", "mimetypes/shellscript.png"),
    ("text-x-script", "mimetypes/shellscript.png"),
    ("package-x-generic", "mimetypes/rpm.png"),
    ("application-x-rpm", "mimetypes/rpm.png"),
];

/// CLI entry point for the native Win2k step. Returns SUCCESS / FAILURE so
/// `install.rs` can route `mde install --only win2k` straight here with no
/// `python3` and no orchestrator script.
pub fn run() -> ExitCode {
    let theme_dir = theme_dir();
    let tarball = tarball_path();
    match install(&win2k_url(), &tarball, &theme_dir) {
        Ok((aliases, dirs)) => {
            println!(
                "Win2k icon theme installed: {aliases} aliases, {dirs} directories → {}\n\
                 Set gtk-icon-theme-name={THEME_NAME} in your GTK settings.",
                theme_dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mde install (win2k): {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Full install pipeline. Returns `(aliases_created, theme_directories)`.
fn install(url: &str, tarball: &Path, theme_dir: &Path) -> Result<(usize, usize)> {
    fetch_if_missing(url, tarball)?;
    extract(tarball, theme_dir)?;
    let aliases = make_aliases(theme_dir)?;
    let dirs = write_index_theme(theme_dir)?;
    refresh_cache(theme_dir);
    Ok((aliases, dirs))
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

/// `$XDG_DATA_HOME/icons/Win2k`, else `~/.local/share/icons/Win2k`.
fn theme_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
        .join("icons")
        .join(THEME_NAME)
}

/// `$XDG_CACHE_HOME/mde/<tarball>`, else `~/.cache/mde/<tarball>`.
fn tarball_path() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".cache"))
        .join("mde")
        .join(TARBALL_NAME)
}

fn win2k_url() -> String {
    std::env::var("MDE_WIN2K_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

/// Download the tarball when it isn't cached. A pre-seeded cache (offline
/// mirror) is used as-is. Downloads to a `.part` sibling and renames on
/// success so a partial transfer never looks cached.
fn fetch_if_missing(url: &str, tarball: &Path) -> Result<()> {
    if tarball.is_file() {
        return Ok(());
    }
    if let Some(parent) = tarball.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating cache dir {}", parent.display()))?;
    }
    println!("fetching Win2k icon set: {url}");
    let resp = ureq::get(url).call().with_context(|| {
        format!(
            "fetching {url} (set $MDE_WIN2K_URL to a mirror, or pre-seed {})",
            tarball.display()
        )
    })?;
    let mut reader = resp.into_body().into_reader();
    let part = tarball.with_extension("part");
    {
        let mut out =
            fs::File::create(&part).with_context(|| format!("creating {}", part.display()))?;
        std::io::copy(&mut reader, &mut out)
            .with_context(|| format!("writing {}", part.display()))?;
    }
    fs::rename(&part, tarball).with_context(|| format!("finalizing {}", tarball.display()))?;
    println!("  → cached at {}", tarball.display());
    Ok(())
}

/// Extract the `.tgz` into `theme_dir`, stripping the leading archive
/// component (`Win2k-2.2.2-1/`). The theme dir is rebuilt from scratch so a
/// re-run is idempotent.
fn extract(tarball: &Path, theme_dir: &Path) -> Result<()> {
    if theme_dir.exists() {
        fs::remove_dir_all(theme_dir)
            .with_context(|| format!("clearing {}", theme_dir.display()))?;
    }
    fs::create_dir_all(theme_dir).with_context(|| format!("creating {}", theme_dir.display()))?;

    let f = fs::File::open(tarball).with_context(|| format!("opening {}", tarball.display()))?;
    let mut archive = tar::Archive::new(GzDecoder::new(f));
    for entry in archive.entries().context("reading tar entries")? {
        let mut entry = entry.context("reading a tar entry")?;
        let path = entry.path().context("entry path")?.into_owned();
        let Some(stripped) = strip_top_component(&path) else {
            continue;
        };
        let dest = theme_dir.join(&stripped);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        entry
            .unpack(&dest)
            .with_context(|| format!("unpacking {}", dest.display()))?;
    }
    Ok(())
}

/// Drop the first path component and reject any traversal (`..`) — returns
/// `None` for the bare top dir or an unsafe path.
fn strip_top_component(path: &Path) -> Option<PathBuf> {
    let mut comps = path.components();
    comps.next(); // drop the leading "Win2k-2.2.2-1"
    let rest: PathBuf = comps.as_path().into();
    if rest.as_os_str().is_empty() {
        return None;
    }
    if rest
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
    {
        return None;
    }
    Some(rest)
}

/// For every `NxN` size dir, copy each existing alias source to its
/// freedesktop name in the same category dir. Returns the count created.
fn make_aliases(theme_dir: &Path) -> Result<usize> {
    let mut created = 0;
    for sdir in sorted_size_dirs(theme_dir)? {
        for (name, src) in ALIASES {
            let src_path = sdir.join(src);
            if !src_path.is_file() {
                continue;
            }
            // The alias lands in the same category dir as its source.
            let Some(cat_dir) = src_path.parent() else {
                continue;
            };
            let dst = cat_dir.join(format!("{name}.png"));
            if dst == src_path || dst.exists() {
                continue;
            }
            fs::copy(&src_path, &dst)
                .with_context(|| format!("aliasing {} → {}", src_path.display(), dst.display()))?;
            created += 1;
        }
    }
    Ok(created)
}

/// Write a spec-compliant `index.theme`. Returns the directory count listed.
/// Output is byte-compatible with the python it replaces.
fn write_index_theme(theme_dir: &Path) -> Result<usize> {
    let mut dirs: Vec<String> = Vec::new();
    let mut blocks: Vec<String> = Vec::new();

    for sdir in sorted_size_dirs(theme_dir)? {
        let size = size_of_name(&dir_name(&sdir)).unwrap_or(0);
        let mut cats: Vec<String> = fs::read_dir(&sdir)
            .with_context(|| format!("reading {}", sdir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .map(|p| dir_name(&p))
            .collect();
        cats.sort();
        for cat in cats {
            let Some(ctx) = CONTEXT.iter().find(|(k, _)| *k == cat).map(|(_, v)| *v) else {
                continue;
            };
            let rel = format!("{}/{cat}", dir_name(&sdir));
            blocks.push(format!(
                "[{rel}]\nSize={size}\nContext={ctx}\nType=Threshold\n"
            ));
            dirs.push(rel);
        }
    }

    let header = format!(
        "[Icon Theme]\n\
         Name={THEME_NAME}\n\
         Comment=Windows 2000 icon theme (KDE-Store 1120706), bridged to freedesktop naming\n\
         Inherits=hicolor,Adwaita\n\
         Directories={}\n\n",
        dirs.join(",")
    );
    let body = format!("{header}{}\n", blocks.join("\n"));
    fs::write(theme_dir.join("index.theme"), body)
        .with_context(|| format!("writing {}/index.theme", theme_dir.display()))?;
    Ok(dirs.len())
}

/// Refresh the GTK icon cache (best-effort — absent tool is fine).
fn refresh_cache(theme_dir: &Path) {
    if which("gtk-update-icon-cache").is_none() {
        println!("gtk-update-icon-cache not found (cache optional)");
        return;
    }
    let _ = Command::new("gtk-update-icon-cache")
        .args(["-f", "-t"])
        .arg(theme_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    println!("refreshed icon cache");
}

/// Size dirs (`NxN`) under `theme_dir`, sorted by name (matches the python's
/// `sorted(os.listdir(...))` ordering).
fn sorted_size_dirs(theme_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(theme_dir)
        .with_context(|| format!("reading {}", theme_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && size_of_name(&dir_name(p)).is_some())
        .collect();
    dirs.sort();
    Ok(dirs)
}

fn dir_name(p: &Path) -> String {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Parse an `NxM` directory name to its leading pixel size, else `None`
/// (`"16x16"` → `Some(16)`, `"scalable"`/`"apps"` → `None`).
fn size_of_name(name: &str) -> Option<u32> {
    let (w, h) = name.split_once('x')?;
    if w.is_empty() || !w.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if h.is_empty() || !h.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    w.parse().ok()
}

/// Locate an executable on `PATH`.
fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("mde-win2k-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn size_of_name_only_matches_size_dirs() {
        assert_eq!(size_of_name("16x16"), Some(16));
        assert_eq!(size_of_name("64x64"), Some(64));
        assert_eq!(size_of_name("128x128"), Some(128));
        assert_eq!(size_of_name("scalable"), None);
        assert_eq!(size_of_name("apps"), None);
        assert_eq!(size_of_name("x16"), None);
        assert_eq!(size_of_name("16x"), None);
    }

    #[test]
    fn strip_top_component_drops_archive_root_and_rejects_traversal() {
        assert_eq!(
            strip_top_component(Path::new("Win2k-2.2.2-1/16x16/apps/k.png")),
            Some(PathBuf::from("16x16/apps/k.png"))
        );
        assert_eq!(strip_top_component(Path::new("Win2k-2.2.2-1/")), None);
        assert_eq!(
            strip_top_component(Path::new("Win2k-2.2.2-1/../escape")),
            None
        );
    }

    #[test]
    fn make_aliases_copies_existing_sources_only() {
        let base = scratch("aliases");
        let theme = base.join("Win2k");
        let apps = theme.join("16x16/apps");
        let fsd = theme.join("16x16/filesystems");
        fs::create_dir_all(&apps).unwrap();
        fs::create_dir_all(&fsd).unwrap();
        fs::write(apps.join("konqueror.png"), b"k").unwrap();
        fs::write(apps.join("konsole.png"), b"t").unwrap();
        fs::write(fsd.join("folder.png"), b"f").unwrap();
        // No kate.png → no text-editor alias.

        let created = make_aliases(&theme).unwrap();
        assert!(created > 0);
        // konqueror → firefox/web-browser
        assert!(apps.join("firefox.png").is_file());
        assert!(apps.join("web-browser.png").is_file());
        // konsole → foot/terminal/utilities-terminal
        assert!(apps.join("foot.png").is_file());
        assert!(apps.join("terminal.png").is_file());
        // folder → inode-directory (the literal "folder" alias is its own
        // source, so it's skipped, not duplicated)
        assert!(fsd.join("inode-directory.png").is_file());
        // missing source → no alias
        assert!(!apps.join("text-editor.png").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn write_index_theme_lists_context_dirs_only() {
        let base = scratch("index");
        let theme = base.join("Win2k");
        fs::create_dir_all(theme.join("16x16/apps")).unwrap();
        fs::create_dir_all(theme.join("16x16/filesystems")).unwrap();
        fs::create_dir_all(theme.join("32x32/mimetypes")).unwrap();
        fs::create_dir_all(theme.join("16x16/ignoreme")).unwrap(); // not in CONTEXT

        let n = write_index_theme(&theme).unwrap();
        assert_eq!(n, 3);
        let idx = fs::read_to_string(theme.join("index.theme")).unwrap();
        assert!(idx.starts_with("[Icon Theme]\nName=Win2k\n"));
        assert!(idx.contains("Inherits=hicolor,Adwaita\n"));
        assert!(idx.contains("Directories=16x16/apps,16x16/filesystems,32x32/mimetypes\n"));
        assert!(idx.contains("[16x16/apps]\nSize=16\nContext=Applications\nType=Threshold\n"));
        assert!(idx.contains("[16x16/filesystems]\nSize=16\nContext=Places\nType=Threshold\n"));
        assert!(idx.contains("[32x32/mimetypes]\nSize=32\nContext=MimeTypes\nType=Threshold\n"));
        assert!(!idx.contains("ignoreme"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn extract_strips_top_and_lands_files() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let base = scratch("extract");
        let tgz = base.join("set.tgz");
        {
            let f = fs::File::create(&tgz).unwrap();
            let gz = GzEncoder::new(f, Compression::default());
            let mut b = tar::Builder::new(gz);
            for rel in [
                "Win2k-2.2.2-1/16x16/apps/konqueror.png",
                "Win2k-2.2.2-1/16x16/filesystems/folder.png",
            ] {
                let data = b"icon-bytes";
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                b.append_data(&mut h, rel, &data[..]).unwrap();
            }
            let gz = b.into_inner().unwrap();
            gz.finish().unwrap();
        }

        let theme = base.join("Win2k");
        extract(&tgz, &theme).unwrap();
        // top component stripped
        assert!(theme.join("16x16/apps/konqueror.png").is_file());
        assert!(theme.join("16x16/filesystems/folder.png").is_file());
        assert!(!theme.join("Win2k-2.2.2-1").exists());
        // re-run is idempotent (rebuilds cleanly)
        extract(&tgz, &theme).unwrap();
        assert!(theme.join("16x16/apps/konqueror.png").is_file());
        let _ = fs::remove_dir_all(&base);
    }

    /// End-to-end on a synthetic tarball (no network): extract → alias →
    /// index.theme. Exercises the whole pipeline minus `fetch_if_missing`.
    #[test]
    fn pipeline_extract_alias_index_on_synthetic_set() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let base = scratch("pipeline");
        let tgz = base.join("set.tgz");
        {
            let f = fs::File::create(&tgz).unwrap();
            let gz = GzEncoder::new(f, Compression::default());
            let mut b = tar::Builder::new(gz);
            for rel in [
                "Win2k-2.2.2-1/16x16/apps/konqueror.png",
                "Win2k-2.2.2-1/16x16/apps/konsole.png",
                "Win2k-2.2.2-1/16x16/mimetypes/txt.png",
            ] {
                let data = b"x";
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                b.append_data(&mut h, rel, &data[..]).unwrap();
            }
            b.into_inner().unwrap().finish().unwrap();
        }
        let theme = base.join("Win2k");
        extract(&tgz, &theme).unwrap();
        let aliases = make_aliases(&theme).unwrap();
        let dirs = write_index_theme(&theme).unwrap();
        assert!(aliases >= 3); // firefox, foot, terminal, text-x-generic, …
        assert_eq!(dirs, 2); // apps + mimetypes
        assert!(theme.join("16x16/apps/firefox.png").is_file());
        assert!(theme.join("16x16/mimetypes/text-x-generic.png").is_file());
        assert!(theme.join("index.theme").is_file());
        let _ = fs::remove_dir_all(&base);
    }
}
