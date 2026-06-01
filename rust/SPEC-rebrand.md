# MDE-Retro rebrand spec (captured from 20 Q&A, 2026-05-31)

**Delivery:** apply to this machine now via a `sudo install-branding.sh` **AND**
bake into the RPM **AND** run automatically as a step of `mde setup`. A full
backup + one-command `revert-branding.sh` (restores stock Fedora) is mandatory.

**Brand mapping:** *Everything is MDE Retro.* Product = "MDE Retro Workstation";
Fedora appears only as the small-print base ("Built on Fedora 44"). No Microsoft.

**Brand identity (one look everywhere):** the Start-menu **side-banner** style —
black + blue-glow background, "MDE Retro" white + "Workstation" light blue
(#3a6ad0 family). Used by the logo, boot splash, login, About, wallpaper.

**Logo mark:** design a new square MDE Retro mark (carbon layout-grid motif on
the black/blue brand background). Shared by About, Plymouth, login.

**Version:** lead with MDE version (e.g. "MDE Retro Workstation 1.0", from the
package version) + "Built on Fedora 44" small print.

## 1. About page
- **Both** a standalone `mde about` winver-style dialog (Start ▸ Help) **and** the
  Control Panel ▸ System Properties ▸ General surface.
- **Content (full winver):** logo + product + version + "Built on Fedora 44" +
  "Registered to: <current user's GECOS full name → username>" + live specs
  (CPU, RAM) + kernel.

## 2. Plymouth boot splash
- Win2000 look: **black** screen, centered MDE logo + wordmark, the classic
  **indeterminate sliding-blue-blocks** progress strip, "Built on Fedora 44" footer.
- Covers **boot + shutdown/reboot**; style the LUKS password prompt if encryption present.
- Needs root: install theme, `plymouth-set-default-theme -R` + `dracut -f`.

## 3. Login (LightDM)
- **Switch greetd → LightDM** with **lightdm-gtk-greeter**, themed for a Windows
  2000 "Log On to Windows" feel: Chicago95 GTK theme + Win2k icons over the
  branded login wallpaper. Both are hard `Requires`, so the login works offline.
- DECISION (2026-06-01): a **web greeter was rejected**. web-greeter (Nuitka) is
  unsupported on Fedora and ships .deb-only; nody-greeter is a ~90MB Electron
  blob with no Fedora rpm. gtk-greeter is native, lightweight, and offline-safe.
  The greeter runs as the unprivileged `lightdm` user, so its theme/icons are
  installed **system-wide** (Win2k icons ship in the package; install-branding.sh
  best-effort copies a fetched Chicago95 GTK theme into /usr/share/themes).

## 4. OS rebrand (full cosmetic)
- `/etc/os-release` + `/usr/lib/os-release`: NAME/PRETTY_NAME/LOGO/HOME_URL → MDE
  Retro; **keep `ID=fedora` + VERSION** (so dnf/repos/tooling keep working).
- **GRUB:** full graphical GRUB theme matching the brand (bg + logo + menu).
- `/etc/issue` + MOTD console banner → MDE Retro.
- **fastfetch:** custom MDE ASCII logo + rebranded name/version.

## 5. Desktop
- Generate a **subtle branded wallpaper** (logo/wordmark, brand colors), set as
  default (still overridable in Display).
- **Startup sound:** keep the current Chicago95 login chime.

## Packaging
- All assets + scripts bundled in the RPM; licenses already covered (assets are
  MDE-original).
- **Auto-activation:** the RPM ships `mde-activate-branding.service` (disabled)
  and enables it from `%posttrans`. On the next boot the one-shot runs
  `install-branding.sh`, then writes `/var/lib/mde-branding/.activated` and
  self-disables (so upgrades never re-fire it). This keeps the greeter dnf
  install, Plymouth initramfs rebuild, and greetd→LightDM switch OUT of the rpm
  transaction — never run them inline in `%post` (dnf-inside-dnf deadlocks; and
  `%post` would re-fire on every upgrade). `mde setup` runs the same script
  directly for the interactive install; either path satisfies the marker.
