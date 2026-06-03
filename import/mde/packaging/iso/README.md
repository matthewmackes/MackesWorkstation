# Mackes Desktop Environment (MDE) — ISO build

This directory contains the scaffolding for building a custom
Fedora-derivative ISO with MDE 2.0.0 baked in (Q4 lock — the v1.x
XFCE-based ISO was retired at CB-4.1).

## Quick start

```sh
sudo dnf install -y lorax pykickstart
make iso              # wraps livemedia-creator with sane defaults
```

The output lands in `dist/iso/`. Expect ~20-30 minutes on modern
hardware and ~4-6 GB of free disk for the build root.

## What's in the kickstart

`mde.ks` is a Fedora kickstart that:

1. Installs Fedora `@core` + `@base-x` (kept for Xwayland-compatible
   apps that haven't ported yet) — explicitly NOT
   `@xfce-desktop-environment` (XFCE is retired; CB-3.3 `Conflicts:`
   block prevents it from coexisting).
2. Installs the locked Wayland stack: sway, swaylock, swayidle,
   swaybg, foot, bemenu, brightnessctl, pipewire, wireplumber,
   grim, slurp, kanshi, wl-clipboard, wlr-randr.
3. Installs LightDM + the GTK greeter so the new MDE Wayland
   session entry shows up in the dropdown automatically.
4. Pulls `mde` from `https://matthewmackes.github.io/MAP2-RELEASES/`
   (the same gh-pages repo the v1.x mackes-shell package shipped
   from; the spec's `Provides: mde` line during the back-compat
   window lets the kickstart resolve cleanly).
5. In `%post`:
   - Marks `/etc/skel/.config/mde/state.json` un-provisioned so the
     first-run wizard fires on first login of every new account.
   - Drops `/etc/lightdm/lightdm.conf.d/50-mde.conf` with
     `user-session=mde` so newly-created accounts default to the
     MDE Wayland session (CB-2.3).
   - Registers the `mackes-desktop-environment` comps group
     (CB-3.4) so `dnf groupinstall` resolves on future installs.
   - Calls `add-mackes-repo.sh` (idempotent) to wire updates.
   - Calls `install-recovery.sh` to wire the GRUB recovery
     submenu.
   - Stages the MDE wallpaper to
     `/usr/share/backgrounds/mde-default.png` (CB-4.3).

## Prerequisites for the build host

- Fedora (the host release should match the target — to build an
  F44 ISO, build on F44).
- `lorax`, `pykickstart`.
- ~10 GB free disk.
- sudo (`livemedia-creator` needs root for the install image).

## Provisioning notes

The kickstart points at `https://matthewmackes.github.io/MAP2-RELEASES/...`
for the `mde` repo. Until the gh-pages content + GPG-signed RPM
metadata are live, the ISO build will fail to find `mde` in the
repo. Workarounds during pre-launch:

- Add `--addrpm $(realpath rpmbuild/RPMS/x86_64/mde-*.rpm)` to the
  `livemedia-creator` invocation.
- Or temporarily drop `mde` from `%packages` and install it
  post-boot via `dnf install -y ./mde-*.rpm`.

## What's NOT here yet

- A signed initrd / signed shim for Secure Boot — MDE ISOs don't
  ship Secure Boot signing today.
- Custom Plymouth splash (CB-4.3 wires the post-install activation;
  the splash assets themselves ship under `data/branding/` once the
  designer signs them off).
- OEM-mode bootstrap that runs the wizard *before* first login
  (today the wizard runs on first MDE-session login, which is fine
  but isn't strictly OEM-mode).
- Multi-arch (only `x86_64` is targeted today).
