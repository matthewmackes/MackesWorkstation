#!/usr/bin/env bash
# install-helpers/hide-xfce-settings.sh
# Run by the RPM %post scriptlet (and again by Maintain → Repair).
#
# For each xfce4-settings menu .desktop entry, write a system-wide override
# at /etc/skel/.local/share/applications/<name> with NoDisplay=true. New
# users inherit the hidden state on account creation; existing users get
# the same effect when Mackes' first-run wizard runs.
set -euo pipefail

SKEL_APPS="/etc/skel/.local/share/applications"
mkdir -p "$SKEL_APPS"

HIDDEN_ENTRIES=(
    xfce-settings-manager.desktop
    xfce4-settings-manager.desktop
    xfce-display-settings.desktop
    xfce4-display-settings.desktop
    xfce-keyboard-settings.desktop
    xfce4-keyboard-settings.desktop
    xfce-mouse-settings.desktop
    xfce4-mouse-settings.desktop
    xfce-appearance-settings.desktop
    xfce4-appearance-settings.desktop
    xfwm4-settings.desktop
    xfwm4-tweaks-settings.desktop
    xfwm4-workspace-settings.desktop
    xfce4-session-settings.desktop
    xfce4-power-manager-settings.desktop
    xfce4-notifyd-config.desktop
    thunar-volman-settings.desktop
    xfce4-mime-settings.desktop
    xfce4-accessibility-settings.desktop
)

for name in "${HIDDEN_ENTRIES[@]}"; do
    target="$SKEL_APPS/$name"
    # Skip if a non-Mackes override already exists at this path
    if [[ -f "$target" ]] && ! grep -q "X-Mackes-Hidden=1" "$target"; then
        continue
    fi
    cat > "$target" <<EOF
[Desktop Entry]
Hidden=true
NoDisplay=true
X-Mackes-Hidden=1
EOF
done
