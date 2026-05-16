# Mackes XFCE — ISO build

This directory contains the scaffolding for building a custom Fedora-derivative
ISO with mackes-shell baked in (Option 10 of the platform review).

## Quick start

```sh
sudo dnf install -y lorax pykickstart
make iso              # wraps livemedia-creator with sane defaults
```

The output lands in `dist/iso/`. Expect ~20-30 minutes on modern hardware
and ~4-6 GB of free disk for the build root.

## What's in the kickstart

`mackes-xfce.ks` is a Fedora kickstart that:

1. Installs Fedora XFCE base + the polybar/plank/rofi/dunst/picom shell stack
2. Pulls `mackes-shell` from `https://matthewmackes.github.io/MAP2-RELEASES/`
3. In `%post`:
   - Marks `/etc/skel/.config/mackes-shell/state.json` un-provisioned so the
     wizard runs on first login of every new user
   - Calls `add-mackes-repo.sh` (idempotent)
   - Calls `install-recovery.sh` to wire the GRUB recovery submenu
   - Stages curated wallpaper to `/usr/share/backgrounds/mackes-default.png`

## Prerequisites for the build host

- Fedora (the host fedora release should match the target — to build an
  F44 ISO, build on F44)
- `lorax`, `pykickstart`
- ~10 GB free disk
- sudo (livemedia-creator needs root for the install image)

## Provisioning notes

The kickstart points at `https://matthewmackes.github.io/MAP2-RELEASES/...`
for the `mackes-shell` repo. Until that gh-pages content + GPG-signed RPM
metadata is actually live, the ISO build will fail to find `mackes-shell` in
the repo. Workarounds during pre-launch:

- Add `--addrpm $(realpath rpmbuild/RPMS/noarch/mackes-shell-*.rpm)` to the
  livemedia-creator invocation
- Or temporarily remove `mackes-shell` from `%packages` and install it
  post-boot via `dnf install -y ./mackes-shell-*.rpm`

## What's NOT here yet

- A signed initrd / signed shim for Secure Boot — Mackes Shell ISOs don't
  ship Secure Boot signing today
- Custom Plymouth splash (default Fedora splash is used)
- OEM-mode bootstrap that runs the wizard *before* first login (today the
  wizard runs on first GUI login, which is fine but isn't strictly OEM-mode)
- Multi-arch (only `x86_64` is targeted today)
