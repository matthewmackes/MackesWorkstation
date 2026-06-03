#!/usr/bin/env bash
# install-helpers/hide-xfce-settings.sh
# Run by the RPM %post scriptlet (and again by Maintain → Repair).
#
# For each .desktop entry that Mackes Shell replaces, write a
# system-wide override at /etc/skel/.local/share/applications/<name>
# with NoDisplay=true so the entry disappears from every menu/launcher.
# New users inherit the hidden state on account creation; existing
# users get the same effect when Mackes' first-run wizard runs (or
# manually via Workbench → Maintain → Repair).
#
# 1.1.0 expansion: the catalog now covers every XFCE component Mackes
# replaces — not just settings dialogs, but the panel preferences,
# Whisker menu, docklike-plugin preferences, xfdesktop preferences,
# xfce4-notifyd config, screensaver, and the xfconf editor. The
# symmetric restore-xfce-settings.sh script (run on uninstall)
# restores the original entries.
set -euo pipefail

SKEL_APPS="/etc/skel/.local/share/applications"
mkdir -p "$SKEL_APPS"

HIDDEN_ENTRIES=(
    # ---- XFCE Settings Manager + every sub-dialog --------------------
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
    xfce4-session-settings.desktop
    xfce4-power-manager-settings.desktop
    xfce4-notifyd-config.desktop
    xfce4-mime-settings.desktop
    xfce4-accessibility-settings.desktop
    xfce4-color-settings.desktop
    xfce4-default-applications.desktop
    xfce4-workspaces-settings.desktop

    # ---- Window manager (xfwm4 retired in 1.0.7; entries linger) -----
    xfwm4-settings.desktop
    xfwm4-tweaks-settings.desktop
    xfwm4-workspace-settings.desktop

    # ---- xfce4-panel preferences + plugin configs --------------------
    # 1.1.0: mackes-panel fully replaces xfce4-panel; the preferences
    # entry and every shipped plugin's config dialog should disappear.
    xfce4-panel.desktop
    xfce4-panel-preferences.desktop
    xfce4-whiskermenu-settings.desktop

    # ---- xfdesktop (root window + wallpaper + menu) ------------------
    # mackes-panel's Desktop layer owns wallpaper + root menu.
    xfce4-desktop-settings.desktop
    xfdesktop-settings.desktop
    xfce4-backdrop-settings.desktop

    # ---- xfce4-screensaver (we don't use it) ------------------------
    xfce4-screensaver-preferences.desktop

    # ---- Thunar add-ons we replace ----------------------------------
    thunar-volman-settings.desktop
    thunar-bulk-rename.desktop

    # ---- Low-level config tools (we wrap these via Workbench) --------
    xfce4-settings-editor.desktop
    xfconf-query.desktop

    # ---- App Finder (whisker menu replaces this) ---------------------
    xfce4-appfinder.desktop
    xfce4-run.desktop
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
