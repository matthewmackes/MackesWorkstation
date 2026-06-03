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
TARGET_USER="${SUDO_USER:-${USER:-}}"
# When run non-interactively (the mde-activate-branding.service one-shot) there's
# no SUDO_USER and $USER is root, so fall back to the first real login user — the
# per-user wallpaper hook (section 6) needs a real home, not root's.
if [ -z "$TARGET_USER" ] || [ "$TARGET_USER" = root ]; then
    TARGET_USER="$(getent passwd | awk -F: '$3>=1000 && $3<60000 {print $1; exit}')"
fi

[ "$(id -u)" = 0 ] || { echo "Run as root: sudo bash $0" >&2; exit 1; }
mkdir -p "$BACKUP"
say() { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
# Back up a file once (first run wins, so revert restores the true original).
# cp -aL: dereference symlinks so the backup is a real file. /etc/os-release is a
# symlink into /usr/lib; a plain `cp -a` copied the link and left a dangling
# backup/os-release.orig (revert then restored nothing).
bk() { [ -e "$1" ] && [ ! -e "$BACKUP/$(basename "$1").orig" ] && cp -aL "$1" "$BACKUP/$(basename "$1").orig" || true; }
# Tracks whether a network-dependent / critical step silently failed. The
# .activated marker (which permanently disarms the boot one-shot) is written
# only if this stays 0 — so an offline first boot that can't fetch the LightDM
# greeter retries on a later boot instead of locking in half-applied branding.
INCOMPLETE=0

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
    # mde-retro is a SCRIPTED theme (ModuleName=script) → needs the script plugin
    # module /usr/lib64/plymouth/script.so. It's a package Require, but install
    # here too in case of an offline/partial state (mirrors the LightDM section).
    # Without it, plymouth-set-default-theme -R aborts with "script.so does not
    # exist" and the splash silently stays on the stock theme.
    if [ ! -e /usr/lib64/plymouth/script.so ]; then
        rpm -q plymouth-plugin-script >/dev/null 2>&1 || dnf install -y plymouth-plugin-script >/dev/null 2>&1 || true
    fi
    plymouth-set-default-theme -R mde-retro || { echo "  (plymouth rebuild failed; check dracut / plymouth-plugin-script)"; INCOMPLETE=1; }
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

# --- 7. LightDM login (lightdm-gtk-greeter, themed Windows 2000) -------------
# We use lightdm-gtk-greeter, not a web greeter: web-greeter (Nuitka) is
# unsupported on Fedora and ships .deb-only, and nody-greeter is a ~90MB Electron
# blob with no Fedora rpm. gtk-greeter is native, offline, and gives a clean
# Win2000 login via the Chicago95 GTK theme + Win2k icons.
say "Setting up the LightDM login (lightdm-gtk-greeter, Win2000 theme)"
GREETER_SESSION=lightdm-gtk-greeter
# These are package Requires, so normally already present; install if somehow not.
for pkg in lightdm lightdm-gtk-greeter; do
    rpm -q "$pkg" >/dev/null 2>&1 || dnf install -y "$pkg" >/dev/null 2>&1 || true
done
install -Dm644 "$ASSETS/mde-wallpaper.png" /usr/share/backgrounds/mde-retro/login.png
# The greeter runs as the unprivileged 'lightdm' user, so its theme + icons must
# be SYSTEM-wide. Win2k icons are shipped by the package (/usr/share/icons/Win2k);
# best-effort copy the Chicago95 GTK theme system-wide if the user has fetched it
# (mde install --assets puts it under ~/.local/share/themes), so the login is
# themed too. A missing theme just renders the default — it never breaks login.
if [ -n "$TARGET_USER" ]; then
    UTHEME="$(getent passwd "$TARGET_USER" | cut -d: -f6)/.local/share/themes/Chicago95"
    { [ -d "$UTHEME" ] && [ ! -d /usr/share/themes/Chicago95 ] && cp -a "$UTHEME" /usr/share/themes/; } || true
fi
GTHEME=Adwaita; [ -d /usr/share/themes/Chicago95 ] && GTHEME=Chicago95
GICON=Adwaita; [ -d /usr/share/icons/Win2k ] && GICON=Win2k
install -d /etc/lightdm
bk /etc/lightdm/lightdm-gtk-greeter.conf 2>/dev/null || true
cat >/etc/lightdm/lightdm-gtk-greeter.conf <<EOF
[greeter]
background=/usr/share/backgrounds/mde-retro/login.png
theme-name=$GTHEME
icon-theme-name=$GICON
EOF
if rpm -q lightdm >/dev/null 2>&1; then
    install -d /etc/lightdm
    bk /etc/lightdm/lightdm.conf 2>/dev/null || true
    sed -i '/^greeter-session=/d' /etc/lightdm/lightdm.conf 2>/dev/null || true
    printf '[Seat:*]\ngreeter-session=%s\n' "$GREETER_SESSION" >>/etc/lightdm/lightdm.conf
    # Switch the display manager: disable greetd, enable lightdm.
    systemctl disable greetd 2>/dev/null || true
    systemctl enable lightdm 2>/dev/null || true
    echo "  Display manager set to LightDM (gtk-greeter, theme=$GTHEME, icons=$GICON)."
    [ -f /usr/share/xgreeters/lightdm-gtk-greeter.desktop ] || {
        echo "  WARNING: lightdm-gtk-greeter not installed; login not branded."
        INCOMPLETE=1
    }
else
    echo "  WARNING: LightDM not installed (offline?); the login was left unchanged."
    INCOMPLETE=1
fi

# Mark activation complete ONLY if nothing critical silently failed. The marker
# permanently disarms the mde-activate-branding.service one-shot (it's gated on
# this path via ConditionPathExists), so writing it after a half-applied run
# would lock the system half-branded. Leaving it unwritten makes the one-shot
# retry on the next boot — re-running install-branding.sh is idempotent. This
# guards the remaining `|| true` steps (Plymouth initramfs rebuild, LightDM);
# install-branding.sh runs under `set -e`, so hard failures already exit first.
if [ "$INCOMPLETE" = 0 ]; then
    mkdir -p /var/lib/mde-branding
    : >/var/lib/mde-branding/.activated
    say "Done. Reboot to see the boot splash, GRUB theme, and login."
else
    say "Branding applied PARTIALLY — a step (Plymouth or the LightDM greeter) failed."
    echo "  NOT marking activation complete; mde-activate-branding.service will retry on the next boot."
    echo "  Or re-run manually:  sudo bash $HERE/install-branding.sh"
fi
echo "Revert any time with:  sudo bash $HERE/revert-branding.sh"
