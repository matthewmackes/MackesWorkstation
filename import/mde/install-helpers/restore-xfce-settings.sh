#!/usr/bin/env bash
# install-helpers/restore-xfce-settings.sh
# Run by the RPM %preun scriptlet on uninstall (not on upgrade).
# Removes the Mackes-installed overrides so xfce4-settings menu entries
# return to visible.
set -euo pipefail

SKEL_APPS="/etc/skel/.local/share/applications"
if [[ ! -d "$SKEL_APPS" ]]; then
    exit 0
fi

for f in "$SKEL_APPS"/*.desktop; do
    [[ -f "$f" ]] || continue
    if grep -q "X-Mackes-Hidden=1" "$f"; then
        rm -f "$f"
    fi
done
