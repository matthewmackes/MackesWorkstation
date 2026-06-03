//! mde-applet-sway-cluster binary — Phase E.4.1 follow-up.
//!
//! Spawns `swaymsg -t get_tree`, parses the JSON, formats the
//! chip row, prints it to stdout, exits 0. The panel-host (Phase
//! E1.3) polls this every 2s and refreshes the Cluster zone.

#![forbid(unsafe_code)]

use std::process::Command;

use mde_applet_sway_cluster::{parse_get_tree_focus, ClusterRow};

fn main() {
    // Manifest mode — emit JSON manifest then exit.
    if std::env::args().nth(1).as_deref() == Some("--manifest") {
        println!(
            "{}",
            serde_json::json!({
                "id": "sway-cluster",
                "binary": "mde-applet-sway-cluster",
                "slot": "top-bar-center",
                "summary": "SPLIT/LAYOUT/WINDOW chips from sway IPC",
                "version": env!("CARGO_PKG_VERSION"),
            })
        );
        return;
    }

    // Now mode — print the current chip row.
    let row = match Command::new("swaymsg").args(["-t", "get_tree"]).output() {
        Ok(out) if out.status.success() => {
            parse_get_tree_focus(std::str::from_utf8(&out.stdout).unwrap_or("{}"))
        }
        _ => ClusterRow::empty(),
    };
    println!("{}", row.render());
}
