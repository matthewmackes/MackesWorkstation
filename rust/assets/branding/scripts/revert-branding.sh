#!/usr/bin/env bash
# Revert the MDE Retro Workstation rebrand — restore stock Fedora branding.
#   sudo bash revert-branding.sh
set -euo pipefail
BACKUP=/var/lib/mde-branding/backup
[ "$(id -u)" = 0 ] || { echo "Run as root: sudo bash $0" >&2; exit 1; }
[ -d "$BACKUP" ] || { echo "No backup at $BACKUP — nothing to revert." >&2; exit 1; }
say() { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
restore() { [ -f "$BACKUP/$(basename "$1").orig" ] && cp -a "$BACKUP/$(basename "$1").orig" "$1" && echo "  restored $1" || true; }

say "Restoring os-release / GRUB / issue / MOTD / login from backup"
restore /etc/os-release
restore /usr/lib/os-release
restore /etc/default/grub
restore /etc/issue
restore /etc/motd
restore /etc/lightdm/lightdm.conf
restore /etc/lightdm/lightdm-gtk-greeter.conf
restore /etc/lightdm/web-greeter.yml
rm -f /usr/share/icons/hicolor/256x256/apps/mde-retro.png

# Plymouth: restore the previous default theme + rebuild initramfs.
if [ -f "$BACKUP/plymouth-theme.orig" ] && command -v plymouth-set-default-theme >/dev/null; then
    say "Restoring Plymouth theme"
    plymouth-set-default-theme -R "$(cat "$BACKUP/plymouth-theme.orig")" || true
    rm -rf /usr/share/plymouth/themes/mde-retro
fi

# GRUB: regenerate config from the restored /etc/default/grub.
if command -v grub2-mkconfig >/dev/null; then
    say "Regenerating GRUB config"
    rm -rf /boot/grub2/themes/mde-retro
    CFG=/boot/grub2/grub.cfg; [ -f /boot/efi/EFI/fedora/grub.cfg ] && CFG=/boot/efi/EFI/fedora/grub.cfg
    grub2-mkconfig -o "$CFG" >/dev/null 2>&1 || true
fi

# Display manager: back to greetd if it was the original.
say "Restoring the display manager (greetd)"
systemctl disable lightdm 2>/dev/null || true
systemctl enable greetd 2>/dev/null || true

rm -f /etc/fastfetch/config.jsonc
say "Reverted. Reboot to return to stock Fedora branding."
