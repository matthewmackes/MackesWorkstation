#!/usr/bin/env bash
# Mackes Shell — uninstall bootstrap (Q46–Q47 locks).
#
#   curl -L https://github.com/mattmacke/mackes-shell/releases/latest/download/uninstall.sh | bash
#
# Two modes:
#  * If `mackes` is on PATH, exec `mackes --uninstall --yes`. That's the
#    canonical uninstaller — same code as the GUI's Maintain → Uninstall.
#  * Otherwise (Mackes not installed but xfce11-unified v2.2 residue exists),
#    perform a standalone bash cleanup of known v2.2 paths (Q47 lock).
#
# Never touches Quick Network Mesh (kept per Q20).

set -euo pipefail

err() { printf '\033[31m%s\033[0m\n' "$*" >&2; exit 1; }
inf() { printf '\033[34m▸ %s\033[0m\n' "$*"; }
ok()  { printf '\033[32m✓ %s\033[0m\n' "$*"; }

[ "$(id -u)" -ne 0 ] || err "Do not pipe this to sudo. The script asks for sudo only when it needs it."

if command -v mackes >/dev/null 2>&1; then
    inf "mackes is installed — delegating to mackes --uninstall --yes"
    exec mackes --uninstall --yes
fi

inf "mackes is not installed; running standalone xfce11-unified v2.2 cleanup"

# Known v2.2 paths (must stay in sync with mackes/uninstall.py V22_KNOWN_PATHS).
V22_PATHS=(
    "$HOME/xfce11-unified"
    "$HOME/Desktop/xfce11-unified"
    "/opt/xfce11-unified"
    "/usr/local/share/xfce11-unified"
    "$HOME/Desktop/START-HERE-XFCE11-UNIFIED.desktop"
    "/usr/share/applications/START-HERE-XFCE11-UNIFIED.desktop"
    "/usr/local/share/applications/START-HERE-XFCE11-UNIFIED.desktop"
)

removed=0
for p in "${V22_PATHS[@]}"; do
    if [ -e "$p" ]; then
        inf "removing $p"
        if [ -w "$(dirname "$p")" ] || [ -O "$p" ]; then
            rm -rf "$p"
        else
            sudo rm -rf "$p"
        fi
        removed=$((removed + 1))
    fi
done

if [ "$removed" -eq 0 ]; then
    ok "Nothing to clean. No xfce11-unified v2.2 residue found."
else
    ok "Removed $removed v2.2 path(s)."
fi
