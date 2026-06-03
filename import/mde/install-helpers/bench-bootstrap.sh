#!/usr/bin/env bash
# v2.0.3 — bench bootstrap.
#
# One-shot installer for the dependencies the mde-2.0.x spec does NOT
# yet hard-Require but that an operator iterating on the bench needs
# in practice. Runs idempotently — re-invocation is a no-op if every
# package is already installed.
#
# Invoke as:
#
#   sudo bash install-helpers/bench-bootstrap.sh
#
# (or from a Claude Code session, prefix with `! ` so the line reaches
# a real TTY: `! sudo bash install-helpers/bench-bootstrap.sh`.)
#
# What this script installs:
#
#   * `mako` — Wayland-native notification daemon. Replaces dunst on
#     MDE sessions; dunst's `org.freedesktop.Notifications` D-Bus name
#     gets owned by mako going forward. The mde-2.0.3 spec will
#     promote mako to a hard Requires; until then this script is the
#     bridge.
#
#   * `wlr-randr` — sway/wlroots-friendly `xrandr` equivalent. Needed
#     for ad-hoc dual-monitor scaling tweaks while v2.0.3's auto-scale
#     helper is being designed.
#
#   * `wayland-utils` — provides `wayland-info`, the canonical way to
#     probe the running compositor's globals (xdg-shell version,
#     wlr-layer-shell-v1 presence, foreign-toplevel manager
#     availability). Required reading every time mde-panel misbehaves.
#
#   * `wev` — `xev` for Wayland. Tap on `Super+L` and confirm sway is
#     dispatching the binding, in those debug sessions where the panel
#     swallows the keystroke before it reaches the user's
#     `loginctl lock-session` exec.
#
#   * `sway-contrib` — assorted helper scripts (grimshot, etc).
#
# What this script masks:
#
#   * `dunst.service` (user unit) — D-Bus activated, X11-only, crash-
#     loops on every Wayland login. Replaced by mako above. Masking is
#     reversible via `systemctl --user unmask dunst.service`.
#
# Hooks for v2.0.3 spec changes:
#
#   * `packaging/fedora/mackes-shell.spec` will gain
#     `Requires: mako` + `Conflicts: dunst` once this script's
#     behavior is stable enough to ship as the default. The script
#     stays around for v1.x → v2.0.x in-place upgrades that skip the
#     full RPM dep refresh.

set -euo pipefail

if [ "${EUID:-$(id -u)}" -ne 0 ]; then
    printf 'bench-bootstrap.sh must run as root (`sudo bash %s`)\n' "$0" >&2
    exit 1
fi

# Resolve the original (non-root) user so we can run `systemctl --user`
# against the right session bus. SUDO_USER is the canonical hint;
# fall back to LOGNAME, then to scraping /var/run/utmp via `who`.
target_user="${SUDO_USER:-${LOGNAME:-}}"
if [ -z "$target_user" ]; then
    target_user=$(who | awk 'NR==1{print $1}')
fi
if [ -z "$target_user" ]; then
    printf 'could not determine the non-root login user (SUDO_USER unset, no `who` output)\n' >&2
    exit 1
fi
target_uid=$(id -u "$target_user")
printf 'bench-bootstrap: target user is %s (uid %d)\n' "$target_user" "$target_uid"

PACKAGES=(
    mako
    wlr-randr
    wayland-utils
    wev
    sway-contrib
)

printf 'bench-bootstrap: installing %d packages via dnf...\n' "${#PACKAGES[@]}"
dnf install -y "${PACKAGES[@]}"

# Mask the X11-only dunst notification daemon for the target user.
# This is reversible (`systemctl --user unmask dunst.service`).
mask_dunst() {
    if ! command -v systemctl >/dev/null 2>&1; then
        printf 'bench-bootstrap: systemctl missing, skipping dunst mask\n' >&2
        return 0
    fi
    if ! sudo -u "$target_user" \
            XDG_RUNTIME_DIR="/run/user/$target_uid" \
            systemctl --user is-active dunst.service >/dev/null 2>&1
    then
        # Service not running — masking it is still useful to prevent
        # D-Bus activation later, but doesn't need a status check.
        true
    fi
    sudo -u "$target_user" \
        XDG_RUNTIME_DIR="/run/user/$target_uid" \
        systemctl --user mask dunst.service >/dev/null
    printf 'bench-bootstrap: masked dunst.service for %s\n' "$target_user"
}

# Enable mako as the user's notification daemon so it owns the
# org.freedesktop.Notifications D-Bus name on next login. (Mako ships a
# user unit at /usr/lib/systemd/user/mako.service.)
enable_mako() {
    if [ ! -f /usr/lib/systemd/user/mako.service ]; then
        printf 'bench-bootstrap: WARNING — /usr/lib/systemd/user/mako.service not found; mako install may be incomplete\n' >&2
        return 0
    fi
    sudo -u "$target_user" \
        XDG_RUNTIME_DIR="/run/user/$target_uid" \
        systemctl --user enable mako.service >/dev/null 2>&1 || true
    printf 'bench-bootstrap: enabled mako.service for %s (start on next login)\n' "$target_user"
}

mask_dunst
enable_mako

printf '\nbench-bootstrap: done.\n'
printf '  Next: log out + back in (or `pkill -USR1 mde-session` if available) so mako picks up the org.freedesktop.Notifications name.\n'
