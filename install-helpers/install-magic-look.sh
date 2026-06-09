#!/bin/sh
# install-helpers/install-magic-look.sh — E11.1 Carbon-on-Cosmic identity installer.
#
# Sets the Magic Carbon visual identity as the desktop default (Q58/Q61/Q62):
#   • installs the Magic-Carbon freedesktop icon theme (the in-repo IBM Carbon
#     Design System icons, Apache-2.0) and makes it the icon-theme default;
#   • sets IBM Plex Sans / Mono as the UI + monospace fonts;
#   • selects the dark color-scheme + the blue accent (Carbon Blue 60, #0f62fe);
# all via gsettings (org.gnome.desktop.interface — honoured by GTK apps and by
# Cosmic's GTK/host integration). Default-on but **reversible** (Q62): prior
# values are saved and `--revert` restores them + removes the installed theme.
#
#   install-magic-look.sh            apply (per-user: ~/.local/share/icons)
#   install-magic-look.sh --system   apply system-wide (/usr/share/icons; needs root)
#   install-magic-look.sh --revert    undo (restore saved settings, remove theme)
#   install-magic-look.sh --self-test apply → verify → revert → verify-restored
#
# The exact cosmic-comp accent RON (#0f62fe) is written by the live Cosmic
# session integration; this installer sets the portable gsettings accent ("blue")
# + everything GTK/host apps read. Exit 0 = applied/clean.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_THEME="$REPO_ROOT/data/icons/Mackes-Carbon"
THEME_NAME="Magic-Carbon"
IFACE="org.gnome.desktop.interface"
STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/magic"
BACKUP="$STATE_DIR/look-backup.env"

UI_FONT="IBM Plex Sans 11"
MONO_FONT="IBM Plex Mono 11"

icon_dir() {
    if [ "${1:-}" = "--system" ]; then echo "/usr/share/icons"; else
        echo "${XDG_DATA_HOME:-$HOME/.local/share}/icons"; fi
}

# Save the current value of an interface key into the backup file (once).
save_key() {
    _k="$1"
    _v="$(gsettings get "$IFACE" "$_k" 2>/dev/null || echo "''")"
    printf '%s=%s\n' "$_k" "$_v" >> "$BACKUP"
}

apply() {
    sys="${1:-}"
    dir="$(icon_dir "$sys")"

    # 1. Install the icon theme as Magic-Carbon (rewrite the brand in index.theme).
    mkdir -p "$dir/$THEME_NAME"
    cp -r "$SRC_THEME"/. "$dir/$THEME_NAME"/
    sed -i \
        -e "s/^Name=.*/Name=$THEME_NAME/" \
        -e 's/^Comment=.*/Comment=Magic Mesh — IBM Carbon Design System icons (Apache 2.0)/' \
        "$dir/$THEME_NAME/index.theme"
    command -v gtk-update-icon-cache >/dev/null 2>&1 && \
        gtk-update-icon-cache -f -t "$dir/$THEME_NAME" >/dev/null 2>&1 || true

    # 2. Back up the settings we are about to change (so --revert is exact).
    mkdir -p "$STATE_DIR"
    : > "$BACKUP"
    for k in icon-theme font-name document-font-name monospace-font-name \
             color-scheme accent-color; do
        save_key "$k"
    done

    # 3. Apply the Magic Carbon defaults.
    gsettings set "$IFACE" icon-theme "$THEME_NAME"
    gsettings set "$IFACE" font-name "$UI_FONT"
    gsettings set "$IFACE" document-font-name "$UI_FONT"
    gsettings set "$IFACE" monospace-font-name "$MONO_FONT"
    gsettings set "$IFACE" color-scheme "prefer-dark"
    # accent-color is a named enum (GNOME 47 / Cosmic host); "blue" is the
    # portable Carbon Blue 60 selector. The exact #0f62fe is the cosmic-config step.
    gsettings set "$IFACE" accent-color "blue" 2>/dev/null || true

    echo "install-magic-look.sh: applied Magic-Carbon (icons + IBM Plex + dark + blue accent) at $dir"
}

revert() {
    if [ -f "$BACKUP" ]; then
        while IFS='=' read -r k v; do
            [ -n "$k" ] || continue
            # shellcheck disable=SC2086
            gsettings set "$IFACE" "$k" "$v" 2>/dev/null || \
                gsettings reset "$IFACE" "$k" 2>/dev/null || true
        done < "$BACKUP"
        rm -f "$BACKUP"
    fi
    for d in "$(icon_dir)" "$(icon_dir --system)"; do
        rm -rf "$d/$THEME_NAME" 2>/dev/null || true
    done
    echo "install-magic-look.sh: reverted to the prior icon theme / fonts / accent"
}

case "${1:-}" in
    --revert) revert ;;
    --self-test)
        apply
        got="$(gsettings get "$IFACE" icon-theme)"
        case "$got" in
            *"$THEME_NAME"*) : ;;
            *) echo "SELF-TEST FAIL: icon-theme not applied (got $got)"; revert; exit 1 ;;
        esac
        [ -f "$(icon_dir)/$THEME_NAME/index.theme" ] || {
            echo "SELF-TEST FAIL: theme not installed"; revert; exit 1; }
        revert
        back="$(gsettings get "$IFACE" icon-theme)"
        case "$back" in
            *"$THEME_NAME"*) echo "SELF-TEST FAIL: revert did not restore icon-theme"; exit 1 ;;
            *) : ;;
        esac
        echo "install-magic-look.sh: self-test PASS (apply set Magic-Carbon; revert restored it)"
        ;;
    --system) apply --system ;;
    *) apply ;;
esac
