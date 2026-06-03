//! labwc libinput (mouse) config for Settings ▸ Devices ▸ Mouse (E12.6).
//!
//! Rewrites the `<libinput>` block of the user's `~/.config/labwc/rc.xml` from the
//! Mouse page's settings, preserving the rest of the file — crucially the
//! `<mouse><default/>` block (§7: dropping `<default/>` makes every window
//! unmovable and the titlebar buttons dead). After a rewrite it asks labwc to
//! reload (`labwc --reconfigure`), the same way `panel.rs` does after writing
//! `menu.xml`. iced-free.
//!
//! The `MDE_LABWC_RC` env var overrides the target path — a dry-run / test seam
//! (like `MDE_LOCK_CONF`) that also suppresses the live reconfigure, so a bench
//! can verify the rewrite against a temp file without touching the real session.
//! Headless entry: `mde __mouse-rc`.

use std::path::PathBuf;

/// Map the Win10-style "lines to scroll" (1–10, default 3) onto libinput's
/// `scrollFactor` multiplier (3 lines ⇒ 1.0, the neutral default).
fn scroll_factor(lines: u8) -> f32 {
    (lines.clamp(1, 10) as f32) / 3.0
}

/// The `<libinput>` block for the given mouse settings, indented to sit at the top
/// level of `rc.xml` (no trailing newline). The "scroll inactive windows"
/// preference is deliberately absent — labwc/wlroots has no such knob, so it lives
/// in menu.json as an advisory only (E12.6).
pub fn libinput_block(left_handed: bool, natural_scroll: bool, scroll_lines: u8) -> String {
    let yn = |b: bool| if b { "yes" } else { "no" };
    format!(
        "  <libinput>
    <device category=\"default\">
      <naturalScroll>{nat}</naturalScroll>
      <leftHanded>{lh}</leftHanded>
      <scrollFactor>{sf:.2}</scrollFactor>
    </device>
  </libinput>",
        nat = yn(natural_scroll),
        lh = yn(left_handed),
        sf = scroll_factor(scroll_lines),
    )
}

/// Swap `<libinput>…</libinput>` in `xml` for `block`, preserving everything else
/// (including `<mouse><default/>`). When no `<libinput>` exists, insert `block`
/// just before `</labwc_config>`. Pure — unit-tested for swap/insert/idempotence.
pub fn rewrite_libinput(xml: &str, block: &str) -> String {
    let block = block.trim_matches('\n');
    match (xml.find("<libinput"), xml.find("</libinput>")) {
        (Some(start), Some(end)) => {
            let end = end + "</libinput>".len();
            // Back up to the start of the `<libinput` line so we replace its
            // indentation too, then keep whatever followed `</libinput>` verbatim.
            let line_start = xml[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            format!("{}{}{}", &xml[..line_start], block, &xml[end..])
        }
        _ => match xml.rfind("</labwc_config>") {
            Some(pos) => format!("{}{}\n{}", &xml[..pos], block, &xml[pos..]),
            None => format!("{xml}\n{block}\n"),
        },
    }
}

/// The rc.xml path: `MDE_LABWC_RC` if set (test seam), else
/// `$XDG_CONFIG_HOME/labwc/rc.xml` (honouring `HOME` otherwise).
fn rc_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MDE_LABWC_RC") {
        return Some(PathBuf::from(p));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("labwc/rc.xml"))
}

/// Write the mouse settings into rc.xml (atomic temp+rename), then reload labwc.
/// The reconfigure is skipped under `MDE_LABWC_RC` (no live labwc to signal in a
/// test). Returns the rc.xml path written, or `None` if there was nowhere to write.
pub fn apply(left_handed: bool, natural_scroll: bool, scroll_lines: u8) -> std::io::Result<()> {
    let Some(path) = rc_path() else {
        return Ok(());
    };
    let xml = std::fs::read_to_string(&path)?;
    let block = libinput_block(left_handed, natural_scroll, scroll_lines);
    let out = rewrite_libinput(&xml, &block);
    let tmp = path.with_extension("xml.mde-tmp");
    std::fs::write(&tmp, out.as_bytes())?;
    std::fs::rename(&tmp, &path)?;
    if std::env::var_os("MDE_LABWC_RC").is_none() {
        let _ = std::process::Command::new("labwc")
            .arg("--reconfigure")
            .status();
    }
    Ok(())
}

/// Headless exercise for `mde __mouse-rc`: apply the persisted mouse settings to
/// the rc.xml (honouring `MDE_LABWC_RC`) and print the result, so the rewrite can
/// be checked end-to-end without a live session.
pub fn debug_apply() {
    let st = crate::state::load();
    if let Err(e) = apply(
        st.mouse_left_handed,
        st.mouse_natural_scroll,
        st.mouse_scroll_lines,
    ) {
        eprintln!("mde __mouse-rc: {e}");
        return;
    }
    if let Some(p) = rc_path() {
        if let Ok(s) = std::fs::read_to_string(&p) {
            print!("{s}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
<?xml version=\"1.0\"?>
<labwc_config>
  <libinput>
    <device category=\"default\">
      <naturalScroll>no</naturalScroll>
      <leftHanded>no</leftHanded>
      <scrollFactor>1.00</scrollFactor>
    </device>
  </libinput>
  <keyboard>
    <keybind key=\"W-l\"><action name=\"Execute\"><command>mde lock</command></action></keybind>
  </keyboard>
  <mouse>
    <default/>
    <context name=\"Root\">
      <mousebind button=\"Right\" action=\"Press\"><action name=\"ShowMenu\"><menu>root-menu</menu></action></mousebind>
    </context>
  </mouse>
</labwc_config>
";

    #[test]
    fn scroll_factor_maps() {
        assert!((scroll_factor(3) - 1.0).abs() < 1e-6);
        assert!((scroll_factor(6) - 2.0).abs() < 1e-6);
        assert!((scroll_factor(0) - scroll_factor(1)).abs() < 1e-6); // clamped
        assert!((scroll_factor(99) - scroll_factor(10)).abs() < 1e-6);
    }

    #[test]
    fn block_omits_the_advisory() {
        let b = libinput_block(true, true, 6);
        assert!(b.contains("<leftHanded>yes</leftHanded>"));
        assert!(b.contains("<naturalScroll>yes</naturalScroll>"));
        assert!(b.contains("<scrollFactor>2.00</scrollFactor>"));
        // The "scroll inactive windows" advisory must never reach rc.xml.
        assert!(!b.to_lowercase().contains("inactive"));
    }

    #[test]
    fn rewrite_swaps_block_and_keeps_mouse_default() {
        let block = libinput_block(true, false, 3);
        let out = rewrite_libinput(SAMPLE, &block);
        // New value in, old value gone.
        assert!(out.contains("<leftHanded>yes</leftHanded>"));
        assert!(!out.contains("<leftHanded>no</leftHanded>"));
        // Exactly one libinput block (no duplication).
        assert_eq!(out.matches("<libinput>").count(), 1);
        // The load-bearing bits survive untouched (§7).
        assert!(out.contains("<mouse>"));
        assert!(out.contains("<default/>"));
        assert!(out.contains("root-menu"));
        assert!(out.contains("mde lock"));
    }

    #[test]
    fn rewrite_inserts_when_absent_keeping_mouse() {
        let no_li = "<labwc_config>\n  <mouse>\n    <default/>\n  </mouse>\n</labwc_config>\n";
        let block = libinput_block(false, true, 5);
        let out = rewrite_libinput(no_li, &block);
        assert_eq!(out.matches("<libinput>").count(), 1);
        assert!(out.contains("<naturalScroll>yes</naturalScroll>"));
        assert!(out.contains("<default/>"));
        // Inserted before the closing tag.
        assert!(out.find("<libinput>").unwrap() < out.find("</labwc_config>").unwrap());
    }

    #[test]
    fn rewrite_is_idempotent() {
        let block = libinput_block(true, true, 7);
        let once = rewrite_libinput(SAMPLE, &block);
        let twice = rewrite_libinput(&once, &block);
        assert_eq!(once, twice);
    }
}
