#!/usr/bin/env bash
# Produce the screenshots the accuracy harness checks (layer 2 of ACCURACY.md).
#
# Run inside a Sway session. For each component it launches the binary, lets it
# paint, grabs the active output with grim, and tears it down. Output lands in
# tests/accuracy/captures/ (gitignored); then `cargo test --test accuracy`
# spot-checks the pixels against tests/accuracy/checklist.toml.
#
# Usage:  tests/accuracy/capture.sh [desktop|panel|all]   (default: all)
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out_dir="$here/captures"
mkdir -p "$out_dir"
rust_root="$(cd "$here/../.." && pwd)"
bin="$rust_root/target/debug/mde"

if [[ -z "${WAYLAND_DISPLAY:-}" ]]; then
    echo "capture.sh: not in a Wayland session (WAYLAND_DISPLAY unset)" >&2
    exit 1
fi
command -v grim >/dev/null || { echo "capture.sh: grim not found" >&2; exit 1; }
[[ -x "$bin" ]] || { echo "capture.sh: build first (cargo build) — $bin missing" >&2; exit 1; }

output="$(swaymsg -t get_outputs | grep -o '"name": "[^"]*"' | head -1 | cut -d'"' -f4)"
echo "capture.sh: output=$output"

grab() { grim -o "$output" "$out_dir/$1"; echo "  -> $1"; }

# Snapshot the live desktop as-is (sway background + whatever taskbar is up).
cap_desktop() { echo "[desktop]"; grab desktop.png; }

# Launch the Rust layer-shell taskbar, let it paint, capture, kill it.
cap_panel() {
    echo "[panel]"
    "$bin" panel &
    local pid=$!
    sleep 1.5
    grab panel.png
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}

case "${1:-all}" in
    desktop) cap_desktop ;;
    panel)   cap_panel ;;
    all)     cap_desktop; cap_panel ;;
    *) echo "usage: capture.sh [desktop|panel|all]" >&2; exit 2 ;;
esac

echo "capture.sh: done. Verify with:  cargo test --test accuracy -- --nocapture"
