"""Mackes multi-monitor helper (v1.7.x).

A pure-Python facade over the XFCE `displays` xfconf channel that backs
xfsettingsd. This module is the single point of truth that the rest of
Mackes (Conky HUD, panel layout, wallpaper, login screen) consults for
"what monitors do we have?" and "apply this layout".

Why not just shell out to xrandr? Fedora 44 doesn't include xrandr in
its stock package set anymore (it's split into `xorg-x11-server-utils`),
but `xfconf-query` is a hard XFCE dependency. We therefore drive the
`displays` xfconf channel as the canonical source — xfsettingsd watches
that channel and pushes RandR commits to the X server. xrandr remains a
nice-to-have second source for live geometry verification.

xfconf `displays` channel schema (per the live audit on the dev box):

  /ActiveProfile          string   "Default" or a saved-profile name
  /AutoEnableProfiles     int      flags (3 = auto-attach + auto-detach)
  /<Profile>/<output>     string   friendly EDID-derived name
  /<Profile>/<output>/Active       bool
  /<Profile>/<output>/Primary      bool
  /<Profile>/<output>/Resolution   "WIDTHxHEIGHT"
  /<Profile>/<output>/Position/X   int
  /<Profile>/<output>/Position/Y   int
  /<Profile>/<output>/RefreshRate  double
  /<Profile>/<output>/Rotation     int    (0 / 90 / 180 / 270)
  /<Profile>/<output>/Reflection   int    (0 / 1 / 2 / 3)
  /<Profile>/<output>/Scale        double (1.0 / 1.25 / 1.5 / 1.75 / 2.0)
  /<Profile>/<output>/ModeFlags    int
  /<Profile>/<output>/EDID         hex string (256+ chars)
  /Notify                  int    Mackes/Settings dialog write 1 to commit

Writes flow:
  1. set_output(...) writes the per-output keys
  2. apply_layout() writes every per-output key for a layout dict, then
     bumps /Notify so xfsettingsd applies a single atomic RandR commit
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Optional


# ---------------------------------------------------------------------------
# Public dataclasses
# ---------------------------------------------------------------------------


@dataclass
class Mode:
    """A supported RandR mode for one output."""
    width: int
    height: int
    refresh_rate: float

    def label(self) -> str:
        return f"{self.width}x{self.height} @ {self.refresh_rate:.0f}Hz"


@dataclass
class Output:
    """One physical output enumerated via xfconf (and optionally xrandr)."""
    name: str                                # e.g. "DP-1-1", "eDP-1"
    friendly_name: str = ""                  # EDID-derived (xfconf top-level value)
    active: bool = False
    primary: bool = False
    resolution: tuple[int, int] = (0, 0)     # (w, h)
    position: tuple[int, int] = (0, 0)       # (x, y)
    scale: float = 1.0
    rotation: int = 0                        # 0 / 90 / 180 / 270
    reflection: int = 0                      # 0 / X / Y / XY
    refresh_rate: float = 0.0
    edid: str = ""
    supported_modes: list[Mode] = field(default_factory=list)
    connected: bool = True                   # xrandr "connected" flag if known

    @property
    def width(self) -> int:
        return self.resolution[0]

    @property
    def height(self) -> int:
        return self.resolution[1]

    @property
    def x(self) -> int:
        return self.position[0]

    @property
    def y(self) -> int:
        return self.position[1]

    def display_label(self) -> str:
        return self.friendly_name or self.name


# Scale values that xfce4-display-settings ships in its dropdown.
SCALE_VALUES: tuple[float, ...] = (1.0, 1.25, 1.5, 1.75, 2.0)
ROTATION_VALUES: tuple[int, ...] = (0, 90, 180, 270)


# ---------------------------------------------------------------------------
# xfconf helpers
# ---------------------------------------------------------------------------


def _have_xfconf() -> bool:
    return shutil.which("xfconf-query") is not None


def _have_xrandr() -> bool:
    return shutil.which("xrandr") is not None


def is_wayland() -> bool:
    return (os.environ.get("XDG_SESSION_TYPE", "").lower() == "wayland"
            or bool(os.environ.get("WAYLAND_DISPLAY")))


def active_profile() -> str:
    """Return the currently active display profile, defaulting to 'Default'."""
    if not _have_xfconf():
        return "Default"
    try:
        out = subprocess.check_output(
            ["xfconf-query", "-c", "displays", "-p", "/ActiveProfile"],
            text=True, stderr=subprocess.DEVNULL, timeout=4,
        ).strip()
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError):
        out = ""
    return out or "Default"


def _xfconf_dump_channel() -> dict[str, str]:
    """Read the entire displays channel into a {key: value} dict."""
    if not _have_xfconf():
        return {}
    try:
        text = subprocess.check_output(
            ["xfconf-query", "-c", "displays", "-l", "-v"],
            text=True, stderr=subprocess.DEVNULL, timeout=4,
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError):
        return {}
    out: dict[str, str] = {}
    for line in text.splitlines():
        parts = line.split(None, 1)
        if len(parts) == 2:
            out[parts[0]] = parts[1].strip()
        elif len(parts) == 1:
            out[parts[0]] = ""
    return out


def _xfconf_set(prop: str, value: str, type_hint: str) -> tuple[int, str]:
    """Create-or-set a single property in the displays channel."""
    cmd = [
        "xfconf-query", "-c", "displays", "-p", prop,
        "--create", "--type", type_hint, "--set", value,
    ]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=8)
        return r.returncode, (r.stdout + r.stderr).strip()
    except (OSError, subprocess.TimeoutExpired) as e:
        return 1, str(e)


def _xfconf_reset(prop: str, *, recursive: bool = False) -> None:
    cmd = ["xfconf-query", "-c", "displays", "-p", prop, "--reset"]
    if recursive:
        cmd.append("--recursive")
    try:
        subprocess.run(cmd, capture_output=True, timeout=8)
    except (OSError, subprocess.TimeoutExpired):
        pass


def _notify_xfsettingsd() -> None:
    """Bump /Notify so xfsettingsd commits the staged channel writes to X."""
    _xfconf_set("/Notify", "1", "int")


# ---------------------------------------------------------------------------
# xrandr fallback / mode enumeration
# ---------------------------------------------------------------------------


_XRANDR_HEAD_RE = re.compile(
    r"^(?P<name>\S+)\s+(?P<state>connected|disconnected)\b(?P<rest>.*)$"
)
_XRANDR_MODE_RE = re.compile(r"^\s+(?P<w>\d+)x(?P<h>\d+)\s+(?P<rates>.+)$")


def _xrandr_query() -> str:
    if not _have_xrandr():
        return ""
    try:
        return subprocess.check_output(
            ["xrandr", "--current"], text=True, stderr=subprocess.DEVNULL, timeout=4,
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError):
        return ""


def _parse_xrandr_modes(text: str) -> dict[str, list[Mode]]:
    """Parse xrandr --current output → {output_name: [Mode...]}."""
    modes: dict[str, list[Mode]] = {}
    current: Optional[str] = None
    for raw in text.splitlines():
        head = _XRANDR_HEAD_RE.match(raw)
        if head:
            current = head.group("name")
            modes.setdefault(current, [])
            continue
        if current is None:
            continue
        m = _XRANDR_MODE_RE.match(raw)
        if not m:
            continue
        w = int(m.group("w")); h = int(m.group("h"))
        for token in m.group("rates").split():
            # Strip xrandr's mode markers ('+' preferred, '*' current).
            clean = token.replace("+", "").replace("*", "").strip()
            if not clean:
                continue
            try:
                rate = float(clean)
            except ValueError:
                continue
            modes[current].append(Mode(width=w, height=h, refresh_rate=rate))
    return modes


def _xrandr_connected_set() -> set[str]:
    text = _xrandr_query()
    if not text:
        return set()
    out: set[str] = set()
    for line in text.splitlines():
        m = _XRANDR_HEAD_RE.match(line)
        if m and m.group("state") == "connected":
            out.add(m.group("name"))
    return out


# ---------------------------------------------------------------------------
# Enumeration
# ---------------------------------------------------------------------------


def list_outputs(profile: Optional[str] = None) -> list[Output]:
    """Enumerate every output in the active (or named) profile.

    Returns inactive outputs too — the panel needs to show them as
    "off" rectangles the user can flip on. xrandr is consulted for
    `supported_modes` when present; absent xrandr, a conservative set
    of common modes is returned so the resolution combo still works.
    """
    raw = _xfconf_dump_channel()
    prof = profile or raw.get("/ActiveProfile") or "Default"
    prefix = f"/{prof}/"

    buckets: dict[str, dict[str, str]] = {}
    for key, val in raw.items():
        if not key.startswith(prefix):
            continue
        rest = key[len(prefix):]
        if "/" in rest:
            name, sub = rest.split("/", 1)
            buckets.setdefault(name, {})[sub] = val
        else:
            # The top-level "/<Profile>/<output>" entry holds the friendly name.
            buckets.setdefault(rest, {})["__friendly__"] = val

    xrandr_text = _xrandr_query()
    modes_by_output = _parse_xrandr_modes(xrandr_text) if xrandr_text else {}
    connected = _xrandr_connected_set() if xrandr_text else set()

    outputs: list[Output] = []
    for name, props in buckets.items():
        if name in {"__friendly__"}:
            continue
        active = props.get("Active", "false").lower() == "true"
        primary = props.get("Primary", "false").lower() == "true"
        res_str = props.get("Resolution", "")
        try:
            if "x" in res_str:
                w, h = (int(x) for x in res_str.split("x", 1))
            else:
                w, h = 0, 0
        except ValueError:
            w, h = 0, 0
        try:
            x = int(props.get("Position/X", "0"))
            y = int(props.get("Position/Y", "0"))
        except ValueError:
            x, y = 0, 0
        try:
            scale = float(props.get("Scale", "1.0"))
        except ValueError:
            scale = 1.0
        try:
            rotation = int(props.get("Rotation", "0"))
        except ValueError:
            rotation = 0
        try:
            reflection = int(props.get("Reflection", "0"))
        except ValueError:
            reflection = 0
        try:
            rr = float(props.get("RefreshRate", "0"))
        except ValueError:
            rr = 0.0
        edid = props.get("EDID", "")

        supported = modes_by_output.get(name, [])
        if not supported:
            # Conservative fallback so the resolution combo isn't empty.
            supported = [Mode(w, h, rr)] if w and h else [
                Mode(3840, 2160, 60.0), Mode(2560, 1440, 60.0),
                Mode(1920, 1200, 60.0), Mode(1920, 1080, 60.0),
                Mode(1680, 1050, 60.0), Mode(1600, 900, 60.0),
                Mode(1366, 768, 60.0), Mode(1280, 800, 60.0),
                Mode(1280, 720, 60.0),
            ]

        outputs.append(Output(
            name=name,
            friendly_name=props.get("__friendly__", ""),
            active=active,
            primary=primary,
            resolution=(w, h),
            position=(x, y),
            scale=scale,
            rotation=rotation,
            reflection=reflection,
            refresh_rate=rr,
            edid=edid,
            supported_modes=supported,
            connected=(name in connected) if xrandr_text else True,
        ))

    # Stable ordering: primary first, then active, then alpha.
    outputs.sort(key=lambda o: (not o.primary, not o.active, o.name))
    return outputs


def primary_output() -> Optional[Output]:
    for o in list_outputs():
        if o.primary and o.active:
            return o
    for o in list_outputs():
        if o.active:
            return o
    return None


# ---------------------------------------------------------------------------
# Single-output writer
# ---------------------------------------------------------------------------


def set_output(
    name: str,
    *,
    active: Optional[bool] = None,
    primary: Optional[bool] = None,
    position: Optional[tuple[int, int]] = None,
    scale: Optional[float] = None,
    rotation: Optional[int] = None,
    resolution: Optional[tuple[int, int]] = None,
    refresh_rate: Optional[float] = None,
    profile: Optional[str] = None,
    notify: bool = True,
    dry_run: bool = False,
) -> list[str]:
    """Write a partial update for one output. Returns the actions taken.

    The caller can chain several `set_output(notify=False)` calls and
    then `apply_layout({}, notify=True)` for a single atomic commit —
    but the typical one-shot ergonomic is `notify=True`.
    """
    prof = profile or active_profile()
    base = f"/{prof}/{name}"
    actions: list[str] = []

    def _w(prop: str, value: str, type_hint: str) -> None:
        actions.append(f"{prop}={value}")
        if not dry_run:
            _xfconf_set(prop, value, type_hint)

    if active is not None:
        _w(f"{base}/Active", "true" if active else "false", "bool")
    if primary is not None:
        _w(f"{base}/Primary", "true" if primary else "false", "bool")
    if position is not None:
        _w(f"{base}/Position/X", str(int(position[0])), "int")
        _w(f"{base}/Position/Y", str(int(position[1])), "int")
    if scale is not None:
        _w(f"{base}/Scale", f"{float(scale):.6f}", "double")
    if rotation is not None:
        if int(rotation) not in ROTATION_VALUES:
            raise ValueError(f"rotation must be one of {ROTATION_VALUES}")
        _w(f"{base}/Rotation", str(int(rotation)), "int")
    if resolution is not None:
        _w(f"{base}/Resolution", f"{int(resolution[0])}x{int(resolution[1])}",
           "string")
    if refresh_rate is not None:
        _w(f"{base}/RefreshRate", f"{float(refresh_rate):.6f}", "double")

    if notify and actions and not dry_run:
        _notify_xfsettingsd()
    return actions


# ---------------------------------------------------------------------------
# Atomic multi-output apply
# ---------------------------------------------------------------------------


def apply_layout(layout: dict[str, dict], *, profile: Optional[str] = None,
                 dry_run: bool = False) -> list[str]:
    """Apply many outputs in one transaction, then bump /Notify.

    `layout` is keyed by output name. Each value is a dict with any of:
      active (bool), primary (bool), position ((x,y)), scale (float),
      rotation (int), resolution ((w,h)), refresh_rate (float).

    Empty layout dicts are allowed; the function still bumps /Notify
    so the caller can use it as an explicit commit after staged writes.

    Returns a flat list of action strings (suitable for the wizard
    actions log).
    """
    prof = profile or active_profile()
    actions: list[str] = []
    for name, props in layout.items():
        actions.extend(set_output(
            name,
            active=props.get("active"),
            primary=props.get("primary"),
            position=props.get("position"),
            scale=props.get("scale"),
            rotation=props.get("rotation"),
            resolution=props.get("resolution"),
            refresh_rate=props.get("refresh_rate"),
            profile=prof, notify=False, dry_run=dry_run,
        ))
    if not dry_run:
        _notify_xfsettingsd()
        actions.append("/Notify=1")
    return actions


def capture_layout() -> dict[str, dict]:
    """Snapshot the current layout in the same shape `apply_layout` consumes.

    Useful as the "before" state for the 15-second keep-this-layout
    confirmation dialog.
    """
    layout: dict[str, dict] = {}
    for o in list_outputs():
        layout[o.name] = {
            "active":     o.active,
            "primary":    o.primary,
            "position":   o.position,
            "scale":      o.scale,
            "rotation":   o.rotation,
            "resolution": o.resolution,
            "refresh_rate": o.refresh_rate,
        }
    return layout


# ---------------------------------------------------------------------------
# Named profiles (xfconf supports them natively)
# ---------------------------------------------------------------------------


def list_profiles() -> list[str]:
    raw = _xfconf_dump_channel()
    profs: set[str] = set()
    for key in raw:
        if key.startswith("/") and key.count("/") >= 2:
            top = key.split("/", 2)[1]
            if top and top not in {"ActiveProfile", "AutoEnableProfiles",
                                    "Notify", "Schemes"}:
                profs.add(top)
    return sorted(profs)


def save_profile(name: str) -> list[str]:
    """Persist the current ACTIVE profile keys under a new profile name.

    XFCE stores each profile as a sibling sub-tree under /. We copy
    every key from /Default/<output>/* (or whichever profile is active)
    into /<name>/<output>/*, preserving types.
    """
    if not name or "/" in name:
        raise ValueError("profile name must be non-empty and not contain '/'")
    src_prof = active_profile()
    src_prefix = f"/{src_prof}/"
    actions: list[str] = []
    raw = _xfconf_dump_channel()
    for key, val in raw.items():
        if not key.startswith(src_prefix):
            continue
        rest = key[len(src_prefix):]
        new_key = f"/{name}/{rest}"
        # Infer type from value.
        if val in ("true", "false"):
            t = "bool"
        else:
            try:
                int(val); t = "int"
            except ValueError:
                try:
                    float(val); t = "double"
                except ValueError:
                    t = "string"
        # Resolution is "WxH" string; Position/X+Y are ints — handled above.
        if rest.endswith("Resolution"):
            t = "string"
        _xfconf_set(new_key, val, t)
        actions.append(f"copied {key} → {new_key}")
    return actions


def load_profile(name: str) -> list[str]:
    """Activate a named profile by pointing /ActiveProfile at it + /Notify."""
    if name not in list_profiles() and name != "Default":
        raise ValueError(f"unknown profile: {name}")
    _xfconf_set("/ActiveProfile", name, "string")
    _notify_xfsettingsd()
    return [f"/ActiveProfile={name}", "/Notify=1"]


def delete_profile(name: str) -> list[str]:
    """Remove a named profile sub-tree. Cannot delete 'Default'."""
    if name == "Default":
        raise ValueError("cannot delete the Default profile")
    _xfconf_reset(f"/{name}", recursive=True)
    return [f"deleted profile /{name}"]


# ---------------------------------------------------------------------------
# Per-monitor wallpaper (xfce4-desktop channel)
# ---------------------------------------------------------------------------


def set_wallpaper(monitor: str, path: Path, *, workspace: int = 0) -> list[str]:
    """Set the per-monitor wallpaper.

    XFCE's xfdesktop reads from `/backdrop/screen0/monitor<NAME>/workspaceN/last-image`
    in the `xfce4-desktop` channel. We write the key directly — xfdesktop
    receives an xfconf change notification and repaints the desktop.
    Setting workspace -1 writes the same value across workspaces 0..3
    so wallpaper is consistent regardless of which workspace was active
    when the user picked the file.
    """
    p = Path(path)
    if not p.is_file():
        raise FileNotFoundError(p)
    actions: list[str] = []
    workspaces: Iterable[int] = range(0, 4) if workspace < 0 else (workspace,)
    for wsp in workspaces:
        prop = f"/backdrop/screen0/monitor{monitor}/workspace{wsp}/last-image"
        cmd = ["xfconf-query", "-c", "xfce4-desktop", "-p", prop,
               "--create", "--type", "string", "--set", str(p)]
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=8)
            ok = (r.returncode == 0)
        except (OSError, subprocess.TimeoutExpired):
            ok = False
        actions.append(("ok" if ok else "fail") + f" {prop}")
    return actions


def get_wallpaper(monitor: str, *, workspace: int = 0) -> Optional[Path]:
    """Read the current per-monitor wallpaper. Returns None if unset."""
    prop = f"/backdrop/screen0/monitor{monitor}/workspace{workspace}/last-image"
    try:
        r = subprocess.run(
            ["xfconf-query", "-c", "xfce4-desktop", "-p", prop],
            capture_output=True, text=True, timeout=4,
        )
        if r.returncode == 0:
            s = r.stdout.strip()
            return Path(s) if s else None
    except (OSError, subprocess.TimeoutExpired):
        pass
    return None


# ---------------------------------------------------------------------------
# LightDM greeter active-monitor (uses AdminSession for the privileged write)
# ---------------------------------------------------------------------------


GREETER_CONF = Path("/etc/lightdm/lightdm-gtk-greeter.conf")


def lightdm_active_monitor() -> Optional[str]:
    """Read the `[greeter] active-monitor =` value, or None if unset.

    Empty/missing = "show on all monitors" — that's the LightDM default.
    """
    if not GREETER_CONF.is_file():
        return None
    try:
        text = GREETER_CONF.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        m = re.match(r"\s*active-monitor\s*=\s*(.+?)\s*$", line)
        if m:
            val = m.group(1)
            return val or None
    return None


def set_lightdm_active_monitor(value: Optional[str]) -> tuple[int, str]:
    """Write `active-monitor =` in /etc/lightdm/lightdm-gtk-greeter.conf.

    `value=None` removes the setting (LightDM falls back to "all monitors").
    Routed through AdminSession so the sudoers NOPASSWD drop-in covers it.
    """
    try:
        from mackes.admin_session import AdminSession
    except Exception as e:  # noqa: BLE001
        return 127, f"admin session unavailable: {e}"

    try:
        existing = GREETER_CONF.read_text(encoding="utf-8", errors="replace")
    except OSError:
        existing = "[greeter]\n"
    lines = existing.splitlines()
    has_section = any(line.strip().lower() == "[greeter]" for line in lines)
    if not has_section:
        lines = ["[greeter]"] + lines

    # Strip any pre-existing active-monitor key from the [greeter] section.
    out: list[str] = []
    in_greeter = False
    for line in lines:
        s = line.strip()
        if s.startswith("[") and s.endswith("]"):
            in_greeter = (s.lower() == "[greeter]")
            out.append(line)
            continue
        if in_greeter and re.match(r"\s*active-monitor\s*=", line):
            continue
        out.append(line)

    if value:
        new_lines: list[str] = []
        added = False
        for line in out:
            new_lines.append(line)
            if not added and line.strip().lower() == "[greeter]":
                new_lines.append(f"active-monitor = {value}")
                added = True
        out = new_lines
    new_text = "\n".join(out).rstrip("\n") + "\n"

    import tempfile
    fd, tmp = tempfile.mkstemp(prefix="mackes-lightdm.", suffix=".conf")
    try:
        with os.fdopen(fd, "w") as f:
            f.write(new_text)
        rc, log = AdminSession.instance().run(
            ["install", "-D", "-m", "0644", tmp, str(GREETER_CONF)],
            timeout=15,
        )
        return rc, log
    finally:
        try: os.unlink(tmp)
        except OSError: pass


# ---------------------------------------------------------------------------
# Re-export the legacy helper used by mackes.conky_hud — same shape it
# already consumed, so we don't churn that caller's signature.
# ---------------------------------------------------------------------------


def xrandr_outputs_for_conky() -> list[dict]:
    """Active-only summary in the {name, primary, w, h, x, y} shape that
    conky_hud._placement() consumes. Single source of truth shim so the
    Conky module can swap to mackes.displays in a follow-up without
    duplicating xfconf parsing logic."""
    out: list[dict] = []
    for o in list_outputs():
        if not o.active or o.width == 0:
            continue
        out.append({
            "name": o.name, "primary": o.primary,
            "w": o.width, "h": o.height,
            "x": o.x, "y": o.y,
        })
    return out


__all__ = [
    "Output", "Mode",
    "SCALE_VALUES", "ROTATION_VALUES", "GREETER_CONF",
    "is_wayland", "active_profile",
    "list_outputs", "primary_output",
    "set_output", "apply_layout", "capture_layout",
    "list_profiles", "save_profile", "load_profile", "delete_profile",
    "set_wallpaper", "get_wallpaper",
    "lightdm_active_monitor", "set_lightdm_active_monitor",
    "xrandr_outputs_for_conky",
]
