# Mackes Workstation (MDE) — v10.0.0 RPM spec.
#
# E8.5 (2026-06-05, role-split 2026-06-08): rewritten for the Rust monorepo. The
# historical Python-era spec (mackes-shell, GTK3 + birthright.py) is in git
# history; this packages the Rust workspace's release binaries + the LizardFS
# mesh-storage bundle + the shipped data, split across three role subpackages —
# mde-core (Lighthouse base) ⊂ mde-headless (Server) ⊂ mde-desktop (Workstation)
# — each a strict superset (§12).
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
Release:        6%{?dist}
Summary:        Mackes Workstation (MDE) — native-Rust mesh desktop environment

License:        GPL-3.0-or-later
URL:            https://github.com/matthewmackes/MackesWorkstation
Source0:        mackes-shell-%{version}.tar.gz
# LizardFS mesh-storage binaries, built from the pinned tag 3.13.0-rc2 by
# install-helpers/build-lizardfs.sh (or the lizardfs-build.yml CI job).
Source1:        lizardfs-binaries.tar.gz

# Back-compat names (the platform was `mackes-shell` / `mde`; `dnf install mde`
# keeps resolving here). `mackes-xfce-workstation` → the mde-desktop subpackage.
Provides:       mde = %{version}-%{release}
Provides:       mackes-shell = %{version}-%{release}
Obsoletes:      mde < 10.0.0
Obsoletes:      mackes-shell < 10.0.0
# MDE absorbs KDE Connect (the native host runs in mackesd, which is core).
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
# Provides the user-unit dir + user-unit scriptlet macros used below.
BuildRequires:  systemd-rpm-macros

# mde-core is the minimal base every role installs (Lighthouse included): the
# `mde` dispatcher, the mackesd control plane, the mde-bus backbone, and the
# core CLI utilities. rpm's ELF dep generator pulls shared-library deps
# automatically. No desktop or mesh-storage deps here — those live on the
# mde-desktop / mde-headless subpackages so a Lighthouse relay stays lean.

%description
Mackes Workstation (MDE) is the native-Rust mesh operating environment: a
multiplexed shell with a strict IBM Carbon look (Gray 10 / 90 / 100) over labwc,
the mackesd control plane with the mde-bus backbone, the Nebula encrypted
overlay, LizardFS mesh-storage, and the native KDE Connect host. One install,
an install-time role chooser (Lighthouse / Server / Workstation).

This base package (mde-core) is the Lighthouse-role tier: the dispatcher,
control plane, and bus that every role needs. Add mde-headless for the Server
role (mesh storage + fleet) and mde-desktop for the full Workstation desktop.

# ── Server role: headless mesh storage + fleet ──────────────────────────────
%package -n mde-headless
Summary:        Mackes Workstation — headless mesh storage + fleet (Server role)
Requires:       mde-core = %{version}-%{release}
# The LizardFS FUSE mount client needs fuse3 at runtime.
Requires:       fuse3
Provides:       mde-server = %{version}-%{release}

%description -n mde-headless
The Server-role tier over mde-core: the LizardFS mesh-storage binaries (master,
chunkserver, metadata restore, FUSE mount client, admin CLI) and the fleet
(ansible-pull) plumbing. Adds no desktop. Requires mde-core.

# ── Workstation role: the full IBM Carbon desktop ───────────────────────────
%package -n mde-desktop
Summary:        Mackes Workstation — full IBM Carbon desktop (Workstation role)
Requires:       mde-core = %{version}-%{release}
# Workstation is a strict superset of Server (§12), so it pulls the headless
# tier too.
Requires:       mde-headless = %{version}-%{release}
# The desktop stack (compositor, greeter, screenshot, terminal, fonts) is weak
# so the role can come up but the operator can swap components.
Recommends:     labwc
Recommends:     greetd
Recommends:     grim
Recommends:     foot
Recommends:     ibm-plex-mono-fonts
Provides:       mackes-xfce-workstation = %{version}-%{release}
Obsoletes:      mackes-xfce-workstation < 10.0.0

%description -n mde-desktop
The Workstation-role desktop over mde-core + mde-headless: the labwc session
orchestrator, the IBM Carbon shell surfaces (the mde-* subcommand symlink farm
+ the standalone Files / Music / Voice-HUD / Workbench apps), the status
applets, the media + voice + clipboard service daemons, the labwc skel config +
window-frame theme + wayland-session entry, and the session/greeter systemd
units. Requires mde-core + mde-headless.

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

# E8.5 #3 — held-release guard. The RPM is HELD until the preceding E8 gates
# (E8.1 accuracy/gallery, E8.4 runtime-reachability/no-stubs) report green; the
# operator releases the hold by exporting MDE_RELEASE_READY=1 ONLY after
# confirming them. The default-held posture means a stray `rpmbuild` can't emit
# a release artifact before the platform is verified (CLAUDE.md §7). E8.2
# (disclaimer) and E8.3 (clippy-deny/fmt/test green) are already satisfied;
# E8.1 + E8.4 are display-bench / E3 (LizardFS) gated.
if [ "${MDE_RELEASE_READY:-0}" != "1" ]; then
    echo "ERROR: RPM build is HELD (E8.5 #3 release gate)." >&2
    echo "       Held pending the E8.1 (accuracy/gallery) + E8.4 (runtime-" >&2
    echo "       reachability, E3/LizardFS-gated) sign-off. Export" >&2
    echo "       MDE_RELEASE_READY=1 to release the hold once they are green." >&2
    exit 1
fi
echo "mde-core: held-release guard cleared (MDE_RELEASE_READY=1)."

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
for sub in panel menu popup action-center task-view search settings \
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
# Bug 4 (2026-06-06) — drop the retired MDE-internal D-Bus service files. The
# `cp -a data/.` above drags `data/dbus-1/services/{dev.mackes.MDE.*,org.mackes.*}`
# into %{_datadir}/mde/dbus-1, a path D-Bus never scans (it reads
# %{_datadir}/dbus-1/services), so they were inert AND contradicted the
# "no MDE-internal D-Bus, FDO interop only" architecture lock. The shell talks
# over mde-bus, not D-Bus — these are dead legacy from the Python MDE.
rm -rf %{buildroot}%{_datadir}/mde/dbus-1
# The systemd unit trees under data/ are likewise inert at %{_datadir}/mde/systemd*
# (systemd reads %{_unitdir} / %{_userunitdir}, not here). They stay as reference
# copies; step 4b installs the one unit this deployment actually activates.

# 4b. Bug 3 (2026-06-06) — the per-user control plane. The E8.5 spec shipped this
# unit only to the inert %{_datadir}/mde/systemd-user path, so nothing started
# `mackesd serve`. Install it to the active %{_userunitdir} + an ENABLING preset
# so it runs at every login and owns the per-user Bus surface. (Bug 6 resolved:
# the fabric workers prereq-gate + self-skip on a non-enrolled box, so the daemon
# starts clean — no crash-loop to auto-enable.)
install -d %{buildroot}%{_userunitdir}
install -m 0644 data/systemd-user/mackesd.service \
    %{buildroot}%{_userunitdir}/mackesd.service
install -d %{buildroot}%{_userpresetdir}
install -m 0644 data/systemd-user-preset/80-mde-mackesd.preset \
    %{buildroot}%{_userpresetdir}/80-mde-mackesd.preset
if [ -d assets ]; then
    install -d %{buildroot}%{_datadir}/mde/assets
    cp -a assets/. %{buildroot}%{_datadir}/mde/assets/
fi
# The single-source disclaimer alongside the data.
install -m 0644 DISCLAIMER.md %{buildroot}%{_datadir}/mde/DISCLAIMER.md

# 5. Wayland-session entry → the FHS path every greeter scans
#    (%{_datadir}/wayland-sessions = /usr/share/wayland-sessions). Owning the
#    file here is greeter-agnostic: lightdm-gtk-greeter, greetd, and regreet all
#    read this dir, whereas the old /var/lib/mde/wayland-sessions plan only ever
#    worked for regreet — which is why the "MDE" option vanished under lightdm.
#    The session-name string ("MDE") is what shows in the greeter dropdown; Exec
#    points at the /usr/bin/mde-session orchestrator installed in step 1.
install -d %{buildroot}%{_datadir}/wayland-sessions
install -m 0644 data/wayland-sessions/mde.desktop \
    %{buildroot}%{_datadir}/wayland-sessions/mde.desktop

# 6. labwc skel config (autostart + menu.xml + rc.xml + scripts + the Win2000-MDE
#    Openbox theme). This IS what draws the shell on login: mde-session falls back
#    to %{_datadir}/mde/skel/.config/labwc (its compiled-in SYSTEM_LABWC_CONFIG_DIR,
#    passed to `labwc -C`) whenever the user has no ~/.config/labwc. The autostart
#    file launches `mde panel`; menu.xml is the desktop right-click menu.
#    The E8.5 spec rewrite dropped this tree (the retired cargo generate-rpm assets
#    in crates/shell/mde/Cargo.toml shipped it) — so labwc came up against an EMPTY
#    config dir: no autostart (black desktop, no panel/wallpaper) and labwc's
#    compiled-in fallback root menu (the lone-item right-click menu). Restoring it
#    here is the fix.
install -d %{buildroot}%{_datadir}/mde/skel
cp -a crates/shell/mde/skel/. %{buildroot}%{_datadir}/mde/skel/
# The labwc autostart + brightness helper must stay executable.
chmod 0755 %{buildroot}%{_datadir}/mde/skel/.config/labwc/autostart
[ -f %{buildroot}%{_datadir}/mde/skel/.config/labwc/scripts/brightness.sh ] && \
    chmod 0755 %{buildroot}%{_datadir}/mde/skel/.config/labwc/scripts/brightness.sh

# 7. The Win2000-MDE Openbox window-frame theme -> %{_datadir}/themes. rc.xml
#    references <theme><name>Win2000-MDE</name>, and labwc resolves theme names on
#    its standard search path (XDG_DATA_DIRS/themes), which includes
#    %{_datadir}/themes = /usr/share/themes — even when launched with `-C` against
#    the skel config. The skel copy above lands the theme under
#    .local/share/themes (where `mde setup` deploys it per-user), which is NOT on
#    labwc's path under -C; without this system copy the title-bar frames fall back
#    to labwc's default look on a fresh install.
install -d %{buildroot}%{_datadir}/themes
cp -a crates/shell/mde/skel/.local/share/themes/Win2000-MDE \
    %{buildroot}%{_datadir}/themes/

# ── mde-core: the Lighthouse base (dispatcher + control plane + bus) ─────────
# Every role installs this. Only the role-agnostic binaries + the shared data +
# the control-plane unit live here; mde-* GUI binaries are owned by mde-desktop
# and the LizardFS sbin set by mde-headless.
%files
%doc DISCLAIMER.md
%{_bindir}/mde
%{_bindir}/mackesd
%{_bindir}/mde-bus
%{_bindir}/mde-alert-emit
%{_bindir}/mde-update
# Shipped read-only data the shell + mackesd read (tokens / icons / assets);
# the desktop skel subtree is owned by mde-desktop.
%dir %{_datadir}/mde
%{_datadir}/mde/*
%exclude %{_datadir}/mde/skel
# The per-user control-plane unit + its enabling preset.
%{_userunitdir}/mackesd.service
%{_userpresetdir}/80-mde-mackesd.preset

# ── mde-headless: LizardFS mesh storage (Server role) ───────────────────────
%files -n mde-headless
# Fedora 42 merged /usr/sbin into /usr/bin, so %{_sbindir} now resolves to
# /usr/bin — a bare %{_sbindir}/* glob would sweep the WHOLE bindir and steal
# mde-core's + mde-desktop's binaries (file conflicts that abort any multi-role
# install). Own the LizardFS set (Source1's six binaries) explicitly instead.
%{_sbindir}/lizardfs
%{_sbindir}/lizardfs-admin
%{_sbindir}/mfschunkserver
%{_sbindir}/mfsmaster
%{_sbindir}/mfsmetarestore
%{_sbindir}/mfsmount

# ── mde-desktop: the full IBM Carbon desktop (Workstation role) ──────────────
# Every mde-* binary + the subcommand symlink farm EXCEPT the role-agnostic
# utilities mde-core owns. (`mde` itself has no hyphen, so the glob skips it.)
%files -n mde-desktop
%{_bindir}/mde-*
%exclude %{_bindir}/mde-bus
%exclude %{_bindir}/mde-alert-emit
%exclude %{_bindir}/mde-update
%{_datadir}/mde/skel
%{_datadir}/wayland-sessions/mde.desktop
%{_datadir}/themes/Win2000-MDE/

# Bug 3 (2026-06-06) — register + enable the per-user control-plane unit. The
# post scriptlet applies the shipped enabling preset (Bug 6 is resolved, so the
# daemon starts clean); the preun scriptlet cleans up on erase.
%post
%systemd_user_post mackesd.service

%preun
%systemd_user_preun mackesd.service

%changelog
* Mon Jun 08 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-6
- fix (E8.5): mde-headless %files used a bare %{_sbindir}/* glob, but Fedora 42
  merged /usr/sbin into /usr/bin so %{_sbindir} now resolves to /usr/bin — the
  glob swept the entire bindir into mde-headless (stealing mde-core's mde /
  mackesd / mde-bus and all of mde-desktop's mde-* GUI binaries), so any
  multi-role install hit file conflicts and aborted. Own the six LizardFS
  binaries (lizardfs, lizardfs-admin, mfs{chunkserver,master,metarestore,mount})
  explicitly so the role superset chain installs cleanly.

* Mon Jun 08 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-5
- packaging (E8.5 #1): split into the three role subpackages — mde-core
  (Lighthouse base: mde / mackesd / mde-bus + core utils + control-plane unit +
  shared data), mde-headless (Server: the LizardFS sbin set + fuse3), and
  mde-desktop (Workstation: every mde-* GUI binary + the subcommand symlink farm
  + skel/theme/wayland-session). Each is a strict superset via Requires
  (desktop → headless → core), matching §12. Moved the desktop Recommends +
  mackes-xfce-workstation Provides/Obsoletes onto mde-desktop; fuse3 onto
  mde-headless. Completes E8.5 (held-guard #3 + disclaimer #2 already in place).
- docs (E9.7): drop the stale "Win2000 / Windows 10 / BeOS" framing from the
  %description (Carbon-only).
* Sat Jun 06 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-4
- fix (Bug 6): prereq-gate the mackesd workers that can't run off an enrolled
  box so `mackesd serve` no longer crash-loops as a per-user daemon —
  nebula_https_listener (skip without a relay cert), fs_sync (skip without the
  retired Python mesh-gvfs module), voice_config (skip when the system voice dir
  isn't writable), ansible-pull (skip without MDE_ANSIBLE_PULL_URL).
- fix (Bug 3, completes -3): with the daemon now starting clean, flip the
  mackesd.service user-preset from disable -> enable so the control plane runs
  at every login by default.
* Sat Jun 06 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-3
- fix (Bug 3, partial): install the per-user mackesd.service to the active
  %{_userunitdir} (+ a user-preset). The control plane had only been cp -a'd to
  the inert %{_datadir}/mde/systemd-user path, so nothing ran `mackesd serve`.
  Ships DISABLED pending Bug 6 (per-user serve spawns fabric workers that
  crash-loop off an enrolled box); flip the preset to `enable` once that lands.
- fix (Bug 4): stop shipping the retired MDE-internal D-Bus service files
  (data/dbus-1/services/{dev.mackes.MDE.*,org.mackes.*}). They landed under
  %{_datadir}/mde/dbus-1 — a path D-Bus never scans — and contradict the
  "no MDE-internal D-Bus, mde-bus only" architecture lock. Removed in %install.
* Sat Jun 06 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-2
- packaging: install the Wayland-session entry into %{_datadir}/wayland-sessions
  so the "MDE" option appears in the greeter. The session .desktop previously
  only landed under %{_datadir}/mde/ (scanned by no greeter); the regreet-only
  /var/lib/mde/wayland-sessions plan left it invisible under lightdm.
- fix: install the labwc skel config (autostart/menu.xml/rc.xml + Win2000-MDE
  Openbox theme) into %{_datadir}/mde/skel — mde-session's SYSTEM_LABWC_CONFIG_DIR
  fallback. The E8.5 rewrite dropped it, so labwc launched against an empty config
  dir: black desktop (autostart never ran `mde panel`) + labwc's built-in one-item
  fallback root menu. This is what makes the shell actually come up on login.
- fix: also install the Win2000-MDE Openbox theme into %{_datadir}/themes so labwc
  resolves the rc.xml <theme> on its standard search path under `-C` — otherwise
  title-bar frames fell back to labwc's default look on a fresh install.
* Fri Jun 05 2026 Matthew Mackes <matthewmackes@gmail.com> - 10.0.0-1
- v10.0.0: the MackesWorkstation monorepo — native-Rust mesh desktop. Spec
  rewritten from the Python-era mackes-shell for the Rust workspace; bundles
  the LizardFS mesh-storage binaries (FUSE binding proven, E3.1).
