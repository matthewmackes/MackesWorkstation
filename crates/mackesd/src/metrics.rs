//! Prometheus textfile collector writer (Phase 12.1.5).
//!
//! Per the 12.1.5 lock: "written to a local Prometheus textfile
//! collector path (`/var/lib/node_exporter/textfile_collector/
//! mackesd.prom`). No HTTP endpoint."
//!
//! The textfile collector picks up `.prom` files written to its
//! configured directory and exposes them on the node_exporter
//! scrape. Operators wire scrape themselves; mackesd just writes
//! the file.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};

/// One Prometheus counter — monotonically increasing, never resets
/// (except on process restart).
#[derive(Debug, Clone, Default)]
pub struct Counter {
    /// Stable Prometheus name like `mackesd_apply_total`.
    pub name: &'static str,
    /// One-line human description for the `# HELP` row.
    pub help: &'static str,
    /// Cumulative count.
    pub value: u64,
    /// Optional labels (`{key="val"}`).
    pub labels: BTreeMap<String, String>,
}

/// One Prometheus histogram bucket — `le` (less-than-or-equal)
/// upper bound + cumulative count.
#[derive(Debug, Clone)]
pub struct Bucket {
    /// Upper bound this bucket counts up to (inclusive).
    pub le: f64,
    /// Cumulative count.
    pub count: u64,
}

/// One Prometheus histogram. Buckets, sum, and overall count.
#[derive(Debug, Clone)]
pub struct Histogram {
    /// Stable Prometheus name.
    pub name: &'static str,
    /// Help text.
    pub help: &'static str,
    /// Buckets in ascending `le` order.
    pub buckets: Vec<Bucket>,
    /// Sum of all observed values (Prometheus `_sum`).
    pub sum: f64,
    /// Total observation count (Prometheus `_count`).
    pub count: u64,
}

/// Render a counter as Prometheus text-format.
fn render_counter(out: &mut String, c: &Counter) {
    let _ = writeln!(out, "# HELP {} {}", c.name, c.help);
    let _ = writeln!(out, "# TYPE {} counter", c.name);
    if c.labels.is_empty() {
        let _ = writeln!(out, "{} {}", c.name, c.value);
    } else {
        let labels = render_labels(&c.labels);
        let _ = writeln!(out, "{}{} {}", c.name, labels, c.value);
    }
}

/// Render a histogram (Prometheus text format requires `_bucket`,
/// `_sum`, and `_count` rows per series).
fn render_histogram(out: &mut String, h: &Histogram) {
    let _ = writeln!(out, "# HELP {} {}", h.name, h.help);
    let _ = writeln!(out, "# TYPE {} histogram", h.name);
    for b in &h.buckets {
        let _ = writeln!(out, r#"{}_bucket{{le="{}"}} {}"#, h.name, b.le, b.count);
    }
    let _ = writeln!(out, r#"{}_bucket{{le="+Inf"}} {}"#, h.name, h.count);
    let _ = writeln!(out, "{}_sum {}", h.name, h.sum);
    let _ = writeln!(out, "{}_count {}", h.name, h.count);
}

fn render_labels(labels: &BTreeMap<String, String>) -> String {
    let mut out = String::from("{");
    let mut first = true;
    for (k, v) in labels {
        if !first {
            out.push(',');
        }
        let _ = write!(out, r#"{k}="{}""#, escape_label_value(v));
        first = false;
    }
    out.push('}');
    out
}

fn escape_label_value(v: &str) -> String {
    v.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

/// Write the canonical mackesd metrics file at
/// `<dir>/mackesd.prom`. Atomic: writes to a temp file first, then
/// renames into place so the collector never reads a half-written
/// snapshot.
///
/// # Errors
/// Returns `std::io::Error` if the directory isn't writable or the
/// rename fails.
pub fn write_textfile(
    dir: &Path,
    counters: &[Counter],
    histograms: &[Histogram],
) -> std::io::Result<PathBuf> {
    let final_path = dir.join("mackesd.prom");
    let tmp_path = dir.join("mackesd.prom.tmp");

    let mut body = String::new();
    for c in counters {
        render_counter(&mut body, c);
    }
    for h in histograms {
        render_histogram(&mut body, h);
    }

    let mut f = std::fs::File::create(&tmp_path)?;
    f.write_all(body.as_bytes())?;
    f.sync_data()?;
    std::fs::rename(&tmp_path, &final_path)?;
    Ok(final_path)
}

/// Default textfile collector directory per the 12.1.5 lock.
#[must_use]
pub fn default_textfile_dir() -> PathBuf {
    PathBuf::from("/var/lib/node_exporter/textfile_collector")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_counter_without_labels() {
        let c = Counter {
            name: "mackesd_apply_total",
            help: "Total applied revisions",
            value: 42,
            labels: BTreeMap::new(),
        };
        let mut out = String::new();
        render_counter(&mut out, &c);
        assert!(out.contains("# HELP mackesd_apply_total"));
        assert!(out.contains("# TYPE mackesd_apply_total counter"));
        assert!(out.contains("mackesd_apply_total 42"));
    }

    #[test]
    fn render_counter_with_labels() {
        let mut labels = BTreeMap::new();
        labels.insert("severity".to_owned(), "auto".to_owned());
        let c = Counter {
            name: "mackesd_drift_detected_total",
            help: "Drift events detected",
            value: 7,
            labels,
        };
        let mut out = String::new();
        render_counter(&mut out, &c);
        assert!(out.contains(r#"mackesd_drift_detected_total{severity="auto"} 7"#));
    }

    #[test]
    fn render_histogram_includes_inf_bucket() {
        let h = Histogram {
            name: "mackesd_probe_seconds",
            help: "Probe latency",
            buckets: vec![
                Bucket { le: 0.1, count: 5 },
                Bucket { le: 0.5, count: 10 },
                Bucket { le: 1.0, count: 12 },
            ],
            sum: 3.42,
            count: 14,
        };
        let mut out = String::new();
        render_histogram(&mut out, &h);
        assert!(out.contains(r#"mackesd_probe_seconds_bucket{le="0.1"} 5"#));
        assert!(out.contains(r#"mackesd_probe_seconds_bucket{le="+Inf"} 14"#));
        assert!(out.contains("mackesd_probe_seconds_sum 3.42"));
        assert!(out.contains("mackesd_probe_seconds_count 14"));
    }

    #[test]
    fn escape_label_value_handles_quotes_and_backslash() {
        assert_eq!(escape_label_value(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn write_textfile_creates_atomic_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let counters = vec![Counter {
            name: "x_total",
            help: "X",
            value: 1,
            labels: BTreeMap::new(),
        }];
        let path = write_textfile(dir.path(), &counters, &[]).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("x_total 1"));
        // No `.tmp` leftover.
        assert!(!dir.path().join("mackesd.prom.tmp").exists());
    }
}
