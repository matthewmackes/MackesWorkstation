//! Dynamic accuracy harness (layer 2 of `rust/ACCURACY.md`).
//!
//! Decodes the screenshots produced by `tests/accuracy/capture.sh` and asserts
//! that each component paints the Windows 2000 ground-truth color at known
//! locations (the checklist in `tests/accuracy/checklist.toml`). This catches
//! theming regressions in the *rendered* output that the static layer-1
//! checklist (`mde-ui/tests/checklist.rs`) cannot see.
//!
//! Behaviour:
//!   * `WAYLAND_DISPLAY` unset  -> the whole suite is skipped (headless CI).
//!   * a capture PNG is missing -> that group is skipped (not failed), so a
//!     partial capture run still verifies what it has.
//!   * a pixel is off-target    -> the test fails with the label, coordinate,
//!     expected and actual hex.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Checklist {
    capture: BTreeMap<String, Capture>,
}

#[derive(Debug, Deserialize)]
struct Capture {
    file: String,
    #[serde(default)]
    point: Vec<Point>,
}

#[derive(Debug, Deserialize)]
struct Point {
    label: String,
    x: i32,
    y: i32,
    hex: String,
    tol: u8,
}

struct Image {
    w: u32,
    h: u32,
    rgb: Vec<u8>, // tightly packed 3 bytes/pixel
}

impl Image {
    fn at(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let i = ((y * self.w + x) * 3) as usize;
        (self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
    }
}

fn accuracy_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = rust/mde ; the harness assets live in rust/tests.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("accuracy")
}

fn load_png(path: &Path) -> Image {
    let file = std::fs::File::open(path).expect("open capture");
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().expect("png header");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png frame");
    let (w, h) = (info.width, info.height);
    let channels = info.color_type.samples();
    // Repack to tight RGB regardless of source channel count (grim emits RGBA).
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for px in buf[..info.buffer_size()].chunks_exact(channels) {
        rgb.extend_from_slice(&px[..3]);
    }
    Image { w, h, rgb }
}

fn parse_hex(s: &str) -> (u8, u8, u8) {
    let n = u32::from_str_radix(s, 16).expect("hex color");
    ((n >> 16) as u8, (n >> 8) as u8, n as u8)
}

/// Resolve a checklist coordinate: negative means "from the far edge".
fn resolve(v: i32, extent: u32) -> u32 {
    let p = if v < 0 { extent as i32 + v } else { v };
    p.clamp(0, extent as i32 - 1) as u32
}

#[test]
fn rendered_components_match_win2000_palette() {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        eprintln!("accuracy: WAYLAND_DISPLAY unset — skipping dynamic screenshot checks");
        return;
    }

    let dir = accuracy_dir();
    let text = std::fs::read_to_string(dir.join("checklist.toml")).expect("read checklist.toml");
    let checklist: Checklist = toml::from_str(&text).expect("parse checklist.toml");

    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (name, cap) in &checklist.capture {
        let path = dir.join("captures").join(&cap.file);
        if !path.exists() {
            eprintln!("accuracy: [{name}] capture {} absent — skipped", cap.file);
            continue;
        }
        let img = load_png(&path);
        for p in &cap.point {
            let (x, y) = (resolve(p.x, img.w), resolve(p.y, img.h));
            let got = img.at(x, y);
            let want = parse_hex(&p.hex);
            let d = |a: u8, b: u8| (a as i16 - b as i16).unsigned_abs();
            let off = d(got.0, want.0).max(d(got.1, want.1)).max(d(got.2, want.2));
            checked += 1;
            if off as u8 > p.tol {
                failures.push(format!(
                    "[{name}] {} @ ({x},{y}): want #{:02x}{:02x}{:02x} ±{}, got #{:02x}{:02x}{:02x} (Δ{off})",
                    p.label, want.0, want.1, want.2, p.tol, got.0, got.1, got.2
                ));
            } else {
                eprintln!(
                    "accuracy: [{name}] OK {} @ ({x},{y}) #{:02x}{:02x}{:02x} (Δ{off})",
                    p.label, got.0, got.1, got.2
                );
            }
        }
    }

    if checked == 0 {
        eprintln!("accuracy: no captures present — run tests/accuracy/capture.sh first. Skipping.");
        return;
    }
    assert!(failures.is_empty(), "accuracy mismatches:\n  {}", failures.join("\n  "));
}
