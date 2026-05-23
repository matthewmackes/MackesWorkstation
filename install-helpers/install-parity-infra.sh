#!/bin/bash
# install-parity-infra.sh — one-shot installer for the v4.0.1 PARITY
# infrastructure. Run with sudo from the repo root:
#
#     sudo install-helpers/install-parity-infra.sh
#
# What it does:
#   1. Copies install-helpers/parity-overlay.sh  → /usr/local/bin/mde-parity-overlay
#   2. Copies install-helpers/sudoers-mde-parity → /etc/sudoers.d/mde-parity
#   3. Creates /var/log/mde-parity.log with the right perms
#   4. Copies data/systemd-user/mde-parity.{path,service} into the
#      developer's ~/.config/systemd/user/ and enables the path unit
#   5. Optionally runs the overlay once now to perform the initial
#      Python+Rust sync against the v4.0.0 RPM
#
# Idempotent: re-running just refreshes any out-of-date files.

set -euo pipefail

DEVUSER="${SUDO_USER:-mm}"
DEVHOME="$(getent passwd "$DEVUSER" | cut -d: -f6)"
REPO="$(cd "$(dirname "$0")/.." && pwd)"
SYSTEMD_USER_DIR="$DEVHOME/.config/systemd/user"

[ "$EUID" -eq 0 ] || { echo "ERROR: must run as root"; exit 1; }
[ -d "$REPO" ]    || { echo "ERROR: repo not found: $REPO"; exit 1; }
[ -d "$DEVHOME" ] || { echo "ERROR: dev home not found: $DEVHOME"; exit 1; }

echo "==> installing /usr/local/bin/mde-parity-overlay"
install -D -m 0755 "$REPO/install-helpers/parity-overlay.sh" \
    /usr/local/bin/mde-parity-overlay

echo "==> installing /etc/sudoers.d/mde-parity"
install -D -m 0440 "$REPO/install-helpers/sudoers-mde-parity" \
    /etc/sudoers.d/mde-parity
visudo -c -f /etc/sudoers.d/mde-parity

echo "==> ensuring /var/log/mde-parity.log"
install -m 0664 /dev/null /var/log/mde-parity.log

echo "==> installing systemd-user units into $SYSTEMD_USER_DIR"
install -D -m 0644 "$REPO/data/systemd-user/mde-parity.path" \
    "$SYSTEMD_USER_DIR/mde-parity.path"
install -D -m 0644 "$REPO/data/systemd-user/mde-parity.service" \
    "$SYSTEMD_USER_DIR/mde-parity.service"
chown -R "$DEVUSER:$DEVUSER" "$SYSTEMD_USER_DIR"

echo "==> enabling + starting the path watch"
sudo -u "$DEVUSER" XDG_RUNTIME_DIR="/run/user/$(id -u "$DEVUSER")" \
    systemctl --user daemon-reload
sudo -u "$DEVUSER" XDG_RUNTIME_DIR="/run/user/$(id -u "$DEVUSER")" \
    systemctl --user enable --now mde-parity.path

echo
echo "Parity infra installed. Status:"
sudo -u "$DEVUSER" XDG_RUNTIME_DIR="/run/user/$(id -u "$DEVUSER")" \
    systemctl --user status mde-parity.path --no-pager || true

cat <<'POSTINSTALL'

Next steps:
  * Trigger an initial overlay now (rebuild + install all delta vs the
    RPM):
        /usr/local/bin/mde-parity-overlay
    or, equivalently, make any commit on main and the path-watch will
    fire automatically.
  * Watch the log:
        tail -f /var/log/mde-parity.log
  * To disable the auto-deploy:
        systemctl --user disable --now mde-parity.path

POSTINSTALL
