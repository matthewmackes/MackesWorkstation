#!/usr/bin/env bash
# Launch exactly one `mde panel` as the taskbar; retire waybar AND swaybar (so
# `mde panel` can own the StatusNotifier tray). Idempotent — sway runs this via
# exec_always on startup and every reload.
pkill -x waybar 2>/dev/null || true
pkill -x swaybar 2>/dev/null || true
pkill -f 'mde panel' 2>/dev/null || true
sleep 0.4
exec "$HOME/.local/bin/mde" panel
