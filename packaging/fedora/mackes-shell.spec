# Mackes Workstation (MDE) — v10.0.0 RPM spec.
#
# E8.5 (2026-06-05): rewritten for the Rust monorepo. The historical
# Python-era spec (mackes-shell, GTK3 + birthright.py) is in git history; this
# packages the Rust workspace's release binaries + the LizardFS mesh-storage
# bundle + the shipped data. The role-subpackage split (mde-headless /
# mde-desktop) is a follow-up; this base package is the installable platform.
#
# Source0 is produced by `git archive --prefix=mde-core-%{version}/`; Source1
# (lizardfs-binaries.tar.gz) by install-helpers/build-lizardfs.sh — both staged
# under rpmbuild/SOURCES/.

# No separate -debuginfo subpackage: the release binaries are the shipped
# artifact; debuginfo extraction over the whole Rust workspace is slow and
# unwanted for the platform RPM.
%global debug_package %{nil}

Name:           mde-core
Version:        10.0.0
Release:        1%{?dist}
Summary:        Mackes Workstation (MDE) — native-Rust mesh desktop environment

License:        GPL-3.0-or-later
URL:            https://github.com/matthewmackes/MackesWorkstation
Source0:        mackes-shell-%{version}.tar.gz
# LizardFS mesh-storage binaries, built from the pinned tag 3.13.0-rc2 by
# install-helpers/build-lizardfs.sh (or the lizardfs-build.yml CI job).
Source1:        lizardfs-binaries.tar.gz

# Back-compat names (the platform was `mackes-shell` / `mackes-xfce-workstation`
# / `mde`; `dnf install mde` keeps resolving here).
Provides:       mde = %{version}-%{release}
Provides:       mackes-shell = %{version}-%{release}
Provides:       mackes-xfce-workstation = %{version}-%{release}
Obsoletes:      mde < 10.0.0
Obsoletes:      mackes-shell < 10.0.0
Obsoletes:      mackes-xfce-workstation < 10.0.0
# MDE absorbs KDE Connect (the native mde-kdc-host replaces it) and the legacy
# XFCE stack.
Obsoletes:      kdeconnect < 999
Obsoletes:      kdeconnectd < 999
Obsoletes:      kdeconnect-cli < 999
Obsoletes:      kdeconnect-indicator < 999
Conflicts:      kdeconnect
Conflicts:      gsconnect

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  pkgconfig
BuildRequires:  gtk3-devel
BuildRequires:  alsa-lib-devel
BuildRequires:  fuse3-devel
BuildRequires:  openssl-devel

# Hard runtime deps kept minimal so the package installs cleanly; rpm's ELF
# dependency generator pulls the shared-library deps automatically. The desktop
# stack (compositor, greeter, tools) is weak so a headless box can install the
# base without dragging in the GUI.
Requires:       fuse3
Recommends:     labwc
Recommends:     greetd
Recommends:     grim
Recommends:     foot
Recommends:     ibm-plex-mono-fonts

%description
Mackes Workstation (MDE) is the native-Rust mesh operating environment: a
multiplexed shell (Win2000 / IBM Carbon / Windows 10 / BeOS looks) over labwc,
the mackesd control plane with the mde-bus backbone, the Nebula encrypted
overlay, LizardFS mesh-storage, and the native KDE Connect host. One install,
an install-time role chooser (Lighthouse / Server / Workstation).

%prep
# Source0 is a git-archive with the %{name}-%{version} prefix, so a plain
# %autosetup uses the default name.
%autosetup

# E8.5 #2 / CLAUDE.md §7 — DISCLAIMER.md pre-flight gate. The educational-
# mission disclaimer is a release invariant (single-sourced via the
# mde-disclaimer crate's include_str!); refuse to build an RPM without it.
if [ ! -s DISCLAIMER.md ]; then
    echo "ERROR: DISCLAIMER.md is missing or empty — refusing to build the RPM." >&2
    echo "       (CLAUDE.md §7 / E8.5 release pre-flight gate.)" >&2
    exit 1
fi
echo "mde-core: DISCLAIMER.md pre-flight gate passed ($(wc -c < DISCLAIMER.md) bytes)."

%build
# The whole workspace, release mode. (.cargo/config.toml carries the CMake-4
# Opus fix; rust-toolchain.toml pins the compiler.)
cargo build --release --workspace

%install
# 1. Rust release binaries → %{_bindir}.
install -d %{buildroot}%{_bindir}
for b in target/release/*; do
    [ -f "$b" ] && [ -x "$b" ] || continue
    install -m 0755 "$b" %{buildroot}%{_bindir}/
done

# 2. The mde shell's subcommand symlink farm (argv[0] dispatch): mde-<sub> -> mde
#    for the subcommands that aren't already standalone binaries, so .desktop
#    Exec= / labwc keybinds resolve.
for sub in panel menu popup start-win10 action-center task-view search settings \
           personalization jumplist net-flyout connect phone control-panel \
           add-remove browser-default browser-jumplist display filedialog run \
           system-properties security greeter clipboard devices-monitor project \
           snip taskbar-properties setup oobe; do
    if [ ! -e "%{buildroot}%{_bindir}/mde-${sub}" ]; then
        ln -s mde "%{buildroot}%{_bindir}/mde-${sub}"
    fi
done

# 3. LizardFS mesh-storage binaries (Source1) → %{_sbindir}.
install -d %{buildroot}%{_sbindir}
tar -xzf %{SOURCE1} -C %{buildroot}%{_sbindir}
chmod 0755 %{buildroot}%{_sbindir}/*

# 4. Shipped read-only data → %{_datadir}/mde/.
install -d %{buildroot}%{_datadir}/mde
if [ -d data ]; then
    cp -a data/. %{buildroot}%{_datadir}/mde/
fi
if [ -d assets ]; then
    install -d %{buildroot}%{_datadir}/mde/assets
    cp -a assets/. %{buildroot}%{_datadir}/mde/assets/
fi
# The single-source disclaimer alongside the data.
install -m 0644 DISCLAIMER.md %{buildroot}%{_datadir}/mde/DISCLAIMER.md

%files
%doc DISCLAIMER.md
%{_bindir}/mde
%{_bindir}/mde-*
%{_bindir}/mackesd
%{_sbindir}/mfsmaster
%{_sbindir}/mfschunkserver
%{_sbindir}/mfsmetarestore
%{_sbindir}/mfsmount
%{_sbindir}/lizardfs
%{_sbindir}/lizardfs-admin
%{_datadir}/mde/

%changelog
* Fri Jun 05 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-1
- v10.0.0: the MackesWorkstation monorepo — native-Rust mesh desktop. Spec
  rewritten from the Python-era mackes-shell for the Rust workspace; bundles
  the LizardFS mesh-storage binaries (FUSE binding proven, E3.1).
