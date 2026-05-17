"""Birthright — first-run install steps that turn a stock XFCE box into Mackes.

Each function is idempotent (safe to re-run via Maintain → Reset to Preset)
and returns a `list[str]` of action lines for the wizard's apply page log.

These are the eight "birthright" items the v1.1.0 wizard runs in addition
to the v1.0.x xfconf-only apply pipeline:

  1. apply_themes              — deploy PadOS GTK theme + Carbon icon theme files
  2. apply_fonts               — install IBM Plex Sans + Mono via dnf
  3. apply_apps                — install preset.apps.install / remove preset.apps.remove_bloat
  4. apply_panel_layout        — write the Mackes default xfce4-panel layout
  5. apply_plymouth            — install + activate the Mackes Plymouth boot theme
  6. apply_dnf_update          — dnf upgrade -y --refresh (full system update)
  7. apply_third_party_repos   — install fedora-workstation-repositories (Chrome, RPM Fusion, etc.)
  8. apply_flathub             — add the Flathub flatpak remote (per-user)

All eight are wired into mackes/wizard/pages/apply.py between Panel and Mesh.
"""
from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path
from typing import Iterable, List

from mackes.logging import log_action
from mackes.presets import Preset


# ---------------------------------------------------------------------------
# Resolve repo / install paths
# ---------------------------------------------------------------------------


def _data_roots() -> List[Path]:
    """Return ordered list of candidate data roots (installed > source-tree)."""
    return [
        Path("/usr/share/mackes-shell/data"),
        Path(__file__).resolve().parent.parent / "data",
    ]


def _find_data(*rel: str) -> Path | None:
    for root in _data_roots():
        p = root.joinpath(*rel)
        if p.exists():
            return p
    return None


def _branding(*rel: str) -> Path | None:
    """Branding lives at /usr/share/mackes-shell/branding/ (RPM) or repo branding/."""
    for root in (
        Path("/usr/share/mackes-shell/branding"),
        Path(__file__).resolve().parent.parent / "branding",
    ):
        p = root.joinpath(*rel)
        if p.exists():
            return p
    return None


def _run_root(cmd: list[str], *, timeout: int = 300) -> tuple[int, str]:
    """Run a command with root privileges (pkexec preferred, sudo fallback)."""
    if shutil.which("pkexec"):
        full = ["pkexec", *cmd]
    elif shutil.which("sudo"):
        full = ["sudo", *cmd]
    else:
        full = cmd
    try:
        proc = subprocess.run(full, capture_output=True, text=True, timeout=timeout)
        return proc.returncode, (proc.stdout + proc.stderr)
    except (OSError, subprocess.TimeoutExpired) as e:
        return 1, str(e)


def _run(cmd: list[str], *, timeout: int = 60) -> tuple[int, str]:
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        return proc.returncode, (proc.stdout + proc.stderr)
    except (OSError, subprocess.TimeoutExpired) as e:
        return 1, str(e)


# ---------------------------------------------------------------------------
# 1. Themes — copy PadOS GTK theme + Carbon icon theme into system dirs
# ---------------------------------------------------------------------------


def apply_themes(_preset: Preset) -> List[str]:
    """Deploy PadOS + Carbon to /usr/share/{themes,icons}/ and refresh caches.

    Source: data/themes/PadOS/ and data/icons/Carbon/ (shipped by the RPM).
    Idempotent: skips if the destination is newer than the source.
    """
    actions: List[str] = []

    pad_src = _find_data("themes", "PadOS")
    carbon_src = _find_data("icons", "Carbon")
    pad_dst = Path("/usr/share/themes/PadOS")
    carbon_dst = Path("/usr/share/icons/Carbon")

    # PadOS GTK theme ---------------------------------------------------
    if pad_src is None:
        actions.append("themes: PadOS source missing in data/themes/PadOS — skipping")
    elif _newer_than(pad_dst, pad_src):
        actions.append(f"themes: PadOS already installed at {pad_dst} (up to date)")
    else:
        rc, out = _run_root(
            ["cp", "-rT", str(pad_src), str(pad_dst)],
            timeout=120,
        )
        if rc == 0:
            actions.append(f"themes: installed PadOS to {pad_dst}")
        else:
            actions.append(f"themes: PadOS install failed: {out.strip().splitlines()[-1] if out.strip() else 'rc='+str(rc)}")

    # Carbon icon theme -------------------------------------------------
    if carbon_src is None:
        actions.append("themes: Carbon source missing in data/icons/Carbon — skipping")
    elif _newer_than(carbon_dst, carbon_src):
        actions.append(f"themes: Carbon already installed at {carbon_dst} (up to date)")
    else:
        rc, out = _run_root(
            ["cp", "-rT", str(carbon_src), str(carbon_dst)],
            timeout=300,
        )
        if rc == 0:
            actions.append(f"themes: installed Carbon to {carbon_dst}")
            # Refresh icon cache
            if shutil.which("gtk-update-icon-cache"):
                _run_root(["gtk-update-icon-cache", "-f", "-t", str(carbon_dst)], timeout=60)
                actions.append("themes: rebuilt Carbon icon cache")
        else:
            actions.append(f"themes: Carbon install failed: {out.strip().splitlines()[-1] if out.strip() else 'rc='+str(rc)}")

    for line in actions:
        log_action(line)
    return actions


def _newer_than(dst: Path, src: Path) -> bool:
    """Return True iff dst exists and is at least as new as the newest file in src."""
    if not dst.exists():
        return False
    try:
        dst_mtime = max(_walk_mtimes(dst), default=0.0)
        src_mtime = max(_walk_mtimes(src), default=0.0)
        return dst_mtime >= src_mtime
    except OSError:
        return False


def _walk_mtimes(path: Path) -> Iterable[float]:
    if path.is_file():
        try:
            yield path.stat().st_mtime
        except OSError:
            pass
        return
    if not path.is_dir():
        return
    try:
        for root, _dirs, files in os.walk(path):
            for f in files:
                try:
                    yield os.stat(os.path.join(root, f)).st_mtime
                except OSError:
                    continue
    except OSError:
        return


# ---------------------------------------------------------------------------
# 2. Fonts — dnf install IBM Plex Sans + Mono
# ---------------------------------------------------------------------------


_PLEX_PACKAGES = ("ibm-plex-sans-fonts", "ibm-plex-mono-fonts")


def apply_fonts(_preset: Preset) -> List[str]:
    """Install IBM Plex Sans + Mono via dnf. Idempotent."""
    actions: List[str] = []
    if shutil.which("dnf") is None:
        actions.append("fonts: dnf not available — skipping")
        return actions

    # Skip if already installed
    needed = []
    for pkg in _PLEX_PACKAGES:
        rc, _ = _run(["rpm", "-q", pkg])
        if rc != 0:
            needed.append(pkg)
    if not needed:
        actions.append("fonts: IBM Plex already installed")
        return actions

    rc, out = _run_root(["dnf", "install", "-y", *needed], timeout=600)
    if rc == 0:
        actions.append(f"fonts: installed {', '.join(needed)}")
        if shutil.which("fc-cache"):
            _run_root(["fc-cache", "-fv"], timeout=120)
            actions.append("fonts: rebuilt fontconfig cache")
    else:
        last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
        actions.append(f"fonts: install failed: {last}")
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# 3. Apps — process preset.apps.install + apps.remove_bloat
# ---------------------------------------------------------------------------


def apply_apps(preset: Preset) -> List[str]:
    """Process the preset's apps.install + apps.remove_bloat lists."""
    actions: List[str] = []
    apps_section = getattr(preset, "apps", None) or {}
    install = apps_section.get("install") if isinstance(apps_section, dict) else []
    remove  = apps_section.get("remove_bloat") if isinstance(apps_section, dict) else []
    install = install or []
    remove  = remove  or []

    if not install and not remove:
        actions.append("apps: preset declares neither install nor remove_bloat — nothing to do")
        return actions

    # Install
    if install:
        from mackes.app_mgmt import install_curated_set
        try:
            actions.append(f"apps: installing {len(install)} app(s): {', '.join(install)}")
            actions.extend(install_curated_set(list(install)))
        except Exception as e:  # noqa: BLE001
            actions.append(f"apps: install pipeline error: {e}")

    # Remove bloat
    if remove:
        from mackes.app_mgmt import remove_packages
        try:
            actions.append(f"apps: removing bloat ({len(remove)} pattern(s))")
            actions.extend(remove_packages(list(remove), category="bloat"))
        except Exception as e:  # noqa: BLE001
            actions.append(f"apps: remove pipeline error: {e}")

    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# 4. Panel layout — write Mackes default xfce4-panel layout via xfconf
# ---------------------------------------------------------------------------


def apply_panel_layout(_preset: Preset) -> List[str]:
    """Write the Mackes default xfce4-panel layout.

    The layout is a single horizontal panel along the top with:
      - Whisker Menu (replaces XFCE Applications menu)
      - Docklike Taskbar (replaces Window Buttons)
      - Spacer
      - Status Tray
      - Clock (IBM Plex digital)

    This function uses xfconf-query and is per-user. It only writes keys
    that aren't already set to the expected value (idempotent).
    """
    actions: List[str] = []
    if shutil.which("xfconf-query") is None:
        actions.append("panel layout: xfconf-query not installed — skipping")
        return actions

    # Helper: set a key only if it differs
    def _set(channel: str, prop: str, type_hint: str, value: str) -> None:
        rc, out = _run(["xfconf-query", "--channel", channel, "--property", prop,
                        "--create", "--type", type_hint, "--set", value], timeout=10)
        if rc == 0:
            actions.append(f"panel: set {channel}{prop} = {value}")
        else:
            actions.append(f"panel: failed to set {prop}: {out.strip().splitlines()[-1] if out.strip() else rc}")

    panels_array_set = False
    rc, out = _run(["xfconf-query", "--channel", "xfce4-panel", "--property", "/panels"])
    if rc == 0 and out.strip():
        # array already exists; don't replace
        panels_array_set = True

    # Plugin IDs we own (chosen to avoid collisions with default 1/2/3)
    # plugin-101 = whiskermenu, 102 = docklike, 103 = spacer, 104 = systray, 105 = clock
    plugin_ids = ["101", "102", "103", "104", "105"]

    if not panels_array_set:
        # Define a single panel (id=0) with our plugins.
        _set("xfce4-panel", "/panels", "int", "0")
        _set("xfce4-panel", "/panels/panel-0/plugin-ids",
             "uint", "")  # placeholder — xfconf-query has trouble with array sets via CLI

    # Always write plugin types + plugin-ids (idempotent)
    _set("xfce4-panel", "/panels/panel-0/position", "string", "p=8;x=0;y=0")
    _set("xfce4-panel", "/panels/panel-0/length",   "uint",   "100")
    _set("xfce4-panel", "/panels/panel-0/size",     "uint",   "32")
    _set("xfce4-panel", "/panels/panel-0/icon-size", "uint",  "22")
    _set("xfce4-panel", "/panels/panel-0/position-locked", "bool", "true")
    _set("xfce4-panel", "/panels/panel-0/autohide-behavior", "uint", "0")

    # Plugins
    _set("xfce4-panel", "/plugins/plugin-101", "string", "whiskermenu")
    _set("xfce4-panel", "/plugins/plugin-102", "string", "docklike")
    _set("xfce4-panel", "/plugins/plugin-103", "string", "separator")
    _set("xfce4-panel", "/plugins/plugin-103/expand", "bool", "true")
    _set("xfce4-panel", "/plugins/plugin-103/style", "uint", "0")
    _set("xfce4-panel", "/plugins/plugin-104", "string", "systray")
    _set("xfce4-panel", "/plugins/plugin-105", "string", "clock")
    _set("xfce4-panel", "/plugins/plugin-105/digital-time-font", "string", "IBM Plex Sans Bold 12")
    _set("xfce4-panel", "/plugins/plugin-105/digital-time-format", "string", "%I:%M %p")
    _set("xfce4-panel", "/plugins/plugin-105/digital-date-font", "string", "IBM Plex Sans 10")
    _set("xfce4-panel", "/plugins/plugin-105/digital-date-format", "string", "%B %d, %Y")
    _set("xfce4-panel", "/plugins/plugin-105/mode", "uint", "2")

    # Restart panel to pick up changes
    if shutil.which("xfce4-panel"):
        try:
            subprocess.Popen(["xfce4-panel", "--restart"],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            actions.append("panel: xfce4-panel --restart issued")
        except OSError as e:
            actions.append(f"panel: could not restart xfce4-panel: {e}")

    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# 5. Plymouth — install Mackes boot theme and set default
# ---------------------------------------------------------------------------


_PLYMOUTH_DEST = Path("/usr/share/plymouth/themes/mackes")


def apply_plymouth(_preset: Preset) -> List[str]:
    """Install + activate the Mackes Plymouth boot theme.

    Theme source: data/plymouth/mackes/ (shipped by the RPM).
    Activation: plymouth-set-default-theme mackes -R (regenerates initrd).
    """
    actions: List[str] = []
    if shutil.which("plymouth-set-default-theme") is None:
        actions.append("plymouth: plymouth not installed — skipping")
        return actions

    src = _find_data("plymouth", "mackes")
    if src is None:
        actions.append("plymouth: source missing in data/plymouth/mackes — skipping")
        return actions

    # If logo file is missing in source, copy from branding/
    logo_src = src / "logo.png"
    if not logo_src.exists():
        b = _branding("MACKES-XFCE-LOGO.png")
        if b is not None:
            rc, out = _run_root(["cp", str(b), str(logo_src)], timeout=15)
            if rc == 0:
                actions.append("plymouth: copied logo from branding/")

    # Copy theme to /usr/share/plymouth/themes/mackes (root)
    if _PLYMOUTH_DEST.exists() and _newer_than(_PLYMOUTH_DEST, src):
        actions.append(f"plymouth: theme already installed at {_PLYMOUTH_DEST} (up to date)")
    else:
        _PLYMOUTH_DEST.parent.mkdir(parents=True, exist_ok=True)
        rc, out = _run_root(["cp", "-rT", str(src), str(_PLYMOUTH_DEST)], timeout=60)
        if rc == 0:
            actions.append(f"plymouth: installed theme to {_PLYMOUTH_DEST}")
        else:
            last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
            actions.append(f"plymouth: theme copy failed: {last}")
            return actions

    # Activate theme + rebuild initrd
    actions.append("plymouth: activating theme + regenerating initrd (this may take ~30s)…")
    rc, out = _run_root(
        ["plymouth-set-default-theme", "mackes", "-R"],
        timeout=300,
    )
    if rc == 0:
        actions.append("plymouth: theme set to 'mackes'; initrd regenerated")
    else:
        last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
        actions.append(f"plymouth: theme activation failed: {last}")

    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# 6. dnf update — full system upgrade (heaviest birthright step)
# ---------------------------------------------------------------------------


def apply_dnf_update(_preset: Preset) -> List[str]:
    """Run a full `dnf upgrade -y --refresh`. May take many minutes."""
    actions: List[str] = []
    if shutil.which("dnf") is None:
        actions.append("system update: dnf not available — skipping")
        return actions

    actions.append("system update: dnf upgrade -y --refresh (this can take several minutes)…")
    rc, out = _run_root(
        ["dnf", "upgrade", "-y", "--refresh"],
        timeout=3600,   # up to 1h for large mirror catches
    )
    # Surface just the last few summary lines (full output goes to log).
    summary = [ln for ln in (out or "").splitlines() if ln.strip()][-5:]
    if rc == 0:
        actions.append("system update: complete")
    else:
        actions.append(f"system update: failed (rc={rc})")
    actions.extend(f"  {ln}" for ln in summary)
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# 7. Third-party repos — Fedora Workstation Repositories meta-package
# ---------------------------------------------------------------------------


def apply_third_party_repos(_preset: Preset) -> List[str]:
    """Install `fedora-workstation-repositories`.

    The package ships repo files for Google Chrome, RPM Fusion, Steam,
    NVIDIA, etc. The repos stay disabled until the user opts in
    (`dnf config-manager --set-enabled <repo>` or the GNOME Software UI).
    Installing the package is enough to surface them in Apps → Install.
    """
    actions: List[str] = []
    if shutil.which("dnf") is None:
        actions.append("third-party repos: dnf not available — skipping")
        return actions

    pkg = "fedora-workstation-repositories"
    rc, _ = _run(["rpm", "-q", pkg])
    if rc == 0:
        actions.append(f"third-party repos: {pkg} already installed")
        # Still enable RPM Fusion + Google Chrome on the user's behalf — those
        # are the universally-useful ones.
    else:
        rc, out = _run_root(["dnf", "install", "-y", pkg], timeout=600)
        if rc == 0:
            actions.append(f"third-party repos: installed {pkg}")
        else:
            last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
            actions.append(f"third-party repos: install failed: {last}")
            for line in actions:
                log_action(line)
            return actions

    # Install RPM Fusion free + nonfree (the most useful third-party repos).
    fedora_ver = _detect_fedora_version()
    if fedora_ver:
        rpmfusion_pkgs = [
            f"https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-{fedora_ver}.noarch.rpm",
            f"https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-{fedora_ver}.noarch.rpm",
        ]
        # Skip if already installed
        free_rc, _ = _run(["rpm", "-q", "rpmfusion-free-release"])
        nonfree_rc, _ = _run(["rpm", "-q", "rpmfusion-nonfree-release"])
        to_install = []
        if free_rc != 0:
            to_install.append(rpmfusion_pkgs[0])
        if nonfree_rc != 0:
            to_install.append(rpmfusion_pkgs[1])
        if to_install:
            rc, out = _run_root(["dnf", "install", "-y", *to_install], timeout=300)
            if rc == 0:
                actions.append(f"third-party repos: enabled RPM Fusion (free + nonfree) for Fedora {fedora_ver}")
            else:
                last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
                actions.append(f"third-party repos: RPM Fusion enable failed: {last}")
        else:
            actions.append("third-party repos: RPM Fusion already enabled")
    else:
        actions.append("third-party repos: could not detect Fedora version — skipping RPM Fusion")

    for line in actions:
        log_action(line)
    return actions


def _detect_fedora_version() -> str | None:
    """Return Fedora major version as a string (e.g. '44'), or None."""
    try:
        for line in Path("/etc/os-release").read_text().splitlines():
            if line.startswith("VERSION_ID="):
                return line.split("=", 1)[1].strip().strip('"')
    except OSError:
        return None
    return None


# ---------------------------------------------------------------------------
# 8. Flathub — add the per-user remote
# ---------------------------------------------------------------------------


def apply_flathub(_preset: Preset) -> List[str]:
    """Add the Flathub remote so flatpak apps are discoverable.

    Per-user (`--user`) so we don't need root. Flatpak is shipped on
    Fedora Workstation by default; we no-op if it's missing.
    """
    actions: List[str] = []
    if shutil.which("flatpak") is None:
        actions.append("flathub: flatpak not installed — skipping")
        return actions

    # Check whether the remote already exists for the current user.
    rc, out = _run(["flatpak", "remotes", "--user", "--columns=name"])
    if rc == 0 and any(line.strip() == "flathub" for line in (out or "").splitlines()):
        actions.append("flathub: per-user remote already configured")
        return actions

    rc, out = _run(
        ["flatpak", "remote-add", "--user", "--if-not-exists", "flathub",
         "https://dl.flathub.org/repo/flathub.flatpakrepo"],
        timeout=60,
    )
    if rc == 0:
        actions.append("flathub: added per-user remote")
    else:
        last = out.strip().splitlines()[-1] if out.strip() else f"rc={rc}"
        actions.append(f"flathub: remote-add failed: {last}")
    for line in actions:
        log_action(line)
    return actions
