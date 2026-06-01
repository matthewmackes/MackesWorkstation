#!/usr/bin/env bash
# MDE Retro Workstation — system rebrand installer (run with sudo/root).
#
#   sudo bash install-branding.sh
#
# Rebrands a Fedora install as "MDE Retro Workstation": os-release display name,
# Plymouth boot splash, GRUB theme, LightDB login, console banner, fastfetch,
# and the default wallpaper. Everything it changes is backed up; revert with
# revert-branding.sh. ID=fedora is kept so dnf/repos keep working.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ASSETS="$(dirname "$HERE")"                       # the branding/ dir
[ -d "$ASSETS/plymouth" ] || ASSETS="/usr/share/mde/branding"
BACKUP=/var/lib/mde-branding/backup
PRODUCT="MDE Retro Workstation"
TARGET_USER="${SUDO_USER:-$USER}"

[ "$(id -u)" = 0 ] || { echo "Run as root: sudo bash $0" >&2; exit 1; }
mkdir -p "$BACKUP"
say() { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
# Back up a file once (first run wins, so revert restores the true original).
bk() { [ -e "$1" ] && [ ! -e "$BACKUP/$(basename "$1").orig" ] && cp -a "$1" "$BACKUP/$(basename "$1").orig" || true; }

# --- 1. os-release: rename the display fields, keep ID/VERSION ----------------
say "Rebranding /etc/os-release (keeping ID=fedora)"
for OSR in /etc/os-release /usr/lib/os-release; do
    [ -f "$OSR" ] || continue
    bk "$OSR"
    # real file (os-release is often a symlink to /usr/lib)
    R="$(readlink -f "$OSR")"
    # os-release is read literally (not shell-expanded), so bake the real version.
    VID="$(. "$R"; echo "${VERSION_ID:-}")"
    sed -i \
        -e "s/^NAME=.*/NAME=\"$PRODUCT\"/" \
        -e "s/^PRETTY_NAME=.*/PRETTY_NAME=\"$PRODUCT (Built on Fedora $VID)\"/" \
        -e "s|^HOME_URL=.*|HOME_URL=\"https://github.com/matthewmackes/MDE\"|" \
        -e "s/^LOGO=.*/LOGO=mde-retro/" "$R"
    grep -q '^VARIANT=' "$R" && sed -i 's/^VARIANT=.*/VARIANT="Workstation"/' "$R" || echo 'VARIANT="Workstation"' >>"$R"
done
# A hicolor 'mde-retro' logo for os-release LOGO + login.
install -Dm644 "$ASSETS/mde-logo-256.png" /usr/share/icons/hicolor/256x256/apps/mde-retro.png 2>/dev/null || true

# --- 2. Plymouth boot splash --------------------------------------------------
if command -v plymouth-set-default-theme >/dev/null; then
    say "Installing Plymouth boot splash (boot + shutdown)"
    rm -rf /usr/share/plymouth/themes/mde-retro
    cp -a "$ASSETS/plymouth/mde-retro" /usr/share/plymouth/themes/
    echo "$(plymouth-set-default-theme 2>/dev/null || echo bgrt)" >"$BACKUP/plymouth-theme.orig"
    plymouth-set-default-theme -R mde-retro || echo "  (plymouth rebuild failed; check dracut)"
else
    echo "  plymouth not installed; skipping boot splash"
fi

# --- 3. GRUB graphical theme --------------------------------------------------
if [ -d /etc/default ] && command -v grub2-mkconfig >/dev/null; then
    say "Installing GRUB theme + distributor name"
    install -d /boot/grub2/themes/mde-retro
    cp -a "$ASSETS/grub/mde-retro/." /boot/grub2/themes/mde-retro/
    bk /etc/default/grub
    sed -i '/^GRUB_THEME=/d;/^GRUB_DISTRIBUTOR=/d' /etc/default/grub
    {
        echo "GRUB_THEME=/boot/grub2/themes/mde-retro/theme.txt"
        echo "GRUB_DISTRIBUTOR=\"$PRODUCT\""
    } >>/etc/default/grub
    CFG=/boot/grub2/grub.cfg; [ -f /boot/efi/EFI/fedora/grub.cfg ] && CFG=/boot/efi/EFI/fedora/grub.cfg
    grub2-mkconfig -o "$CFG" >/dev/null 2>&1 || echo "  (grub2-mkconfig failed)"
fi

# --- 4. Console banner --------------------------------------------------------
say "Branding the console (/etc/issue + MOTD)"
bk /etc/issue; cp "$ASSETS/system/issue" /etc/issue
bk /etc/motd 2>/dev/null || true
echo "$PRODUCT — Built on Fedora" >/etc/motd

# --- 5. fastfetch -------------------------------------------------------------
say "Branding fastfetch"
install -Dm644 "$ASSETS/system/fastfetch-logo.txt" /usr/share/mde/branding/fastfetch-logo.txt
install -Dm644 "$ASSETS/system/fastfetch.jsonc" /etc/fastfetch/config.jsonc

# --- 6. Default wallpaper -----------------------------------------------------
say "Installing the default wallpaper"
install -Dm644 "$ASSETS/mde-wallpaper.png" /usr/share/backgrounds/mde-retro/mde-wallpaper.png
if [ -n "$TARGET_USER" ] && id "$TARGET_USER" >/dev/null 2>&1; then
    UH="$(getent passwd "$TARGET_USER" | cut -d: -f6)"
    install -d -o "$TARGET_USER" "$UH/.config/mde"
    printf '#!/bin/sh\nswaybg -m fill -i /usr/share/backgrounds/mde-retro/mde-wallpaper.png &\n' \
        >"$UH/.config/mde/wallpaper.sh"
    chmod +x "$UH/.config/mde/wallpaper.sh"; chown "$TARGET_USER" "$UH/.config/mde/wallpaper.sh"
fi

# --- 7. LightDM web greeter login --------------------------------------------
say "Setting up the LightDM web greeter login"
INSTALLED_GREETER=""
if ! rpm -q lightdm >/dev/null 2>&1; then dnf install -y lightdm >/dev/null 2>&1 || true; fi
# Try a web greeter (web-greeter / nody-greeter); fall back to gtk-greeter.
for pkg in web-greeter nody-greeter lightdm-webkit2-greeter; do
    rpm -q "$pkg" >/dev/null 2>&1 && INSTALLED_GREETER="$pkg" && break
    dnf install -y "$pkg" >/dev/null 2>&1 && INSTALLED_GREETER="$pkg" && break
done
if [ -n "$INSTALLED_GREETER" ]; then
    # web-greeter / nody theme dir
    for TD in /usr/share/web-greeter/themes /usr/share/web-greeter/dist/themes /opt/nody-greeter/themes; do
        [ -d "$TD" ] && { rm -rf "$TD/mde"; cp -a "$ASSETS/lightdm/mde-web-greeter" "$TD/mde"; }
    done
    bk /etc/lightdm/web-greeter.yml 2>/dev/null || true
    [ -f /etc/lightdm/web-greeter.yml ] && sed -i 's/^\(\s*theme:\).*/\1 mde/' /etc/lightdm/web-greeter.yml || true
    GREETER_SESSION="$([ -f /usr/share/xgreeters/web-greeter.desktop ] && echo web-greeter || echo nody-greeter)"
else
    echo "  No web greeter available (try COPR); falling back to lightdm-gtk-greeter."
    dnf install -y lightdm-gtk-greeter >/dev/null 2>&1 || true
    GREETER_SESSION="lightdm-gtk-greeter"
    install -Dm644 "$ASSETS/mde-wallpaper.png" /usr/share/backgrounds/mde-retro/login.png
    install -d /etc/lightdm
    bk /etc/lightdm/lightdm-gtk-greeter.conf 2>/dev/null || true
    cat >/etc/lightdm/lightdm-gtk-greeter.conf <<EOF
[greeter]
background=/usr/share/backgrounds/mde-retro/login.png
theme-name=Chicago95
icon-theme-name=Win2k
EOF
fi
if rpm -q lightdm >/dev/null 2>&1; then
    install -d /etc/lightdm
    bk /etc/lightdm/lightdm.conf 2>/dev/null || true
    sed -i '/^greeter-session=/d' /etc/lightdm/lightdm.conf 2>/dev/null || true
    printf '[Seat:*]\ngreeter-session=%s\n' "$GREETER_SESSION" >>/etc/lightdm/lightdm.conf
    # Switch the display manager: disable greetd, enable lightdm.
    systemctl disable greetd 2>/dev/null || true
    systemctl enable lightdm 2>/dev/null || true
    echo "  Display manager set to LightDM ($GREETER_SESSION)."
fi

say "Done. Reboot to see the boot splash, GRUB theme, and login."
echo "Revert any time with:  sudo bash $HERE/revert-branding.sh"
