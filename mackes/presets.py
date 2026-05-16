"""Preset load + apply + drift detection (Q7 lock: curated presets only).

A preset is a YAML file in `data/presets/` (system) or
`~/.config/mackes-shell/presets/` (undocumented user override location).

Presets are read-only at runtime — apply pushes preset values into the live
system, but daily Workbench changes never modify the YAML. That's why drift
exists: live state diverges from the preset's declared targets.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Optional

from mackes.logging import log_action
from mackes.state import CONFIG_DIR

try:
    import yaml  # type: ignore
except ImportError:  # pragma: no cover - YAML is a hard dep at runtime
    yaml = None  # noqa: N816


# Search order: user-local first (overrides shipped), then shipped data dir.
USER_PRESET_DIR = CONFIG_DIR / "presets"
SHIPPED_PRESET_DIRS = [
    Path("/usr/share/mackes-shell/data/presets"),
    Path(__file__).resolve().parent.parent / "data" / "presets",
]

# Name of the preset that gets pre-selected in the wizard and used as the
# implicit "active" preset when nothing is set in state.json.
DEFAULT_PRESET_NAME = "hashbang"


# ---------------------------------------------------------------------------
# Preset model
# ---------------------------------------------------------------------------


@dataclass
class Preset:
    name: str
    display_name: str
    description: str
    appearance: dict[str, Any] = field(default_factory=dict)
    shell: dict[str, Any] = field(default_factory=dict)
    devices: dict[str, Any] = field(default_factory=dict)
    network: dict[str, Any] = field(default_factory=dict)
    system: dict[str, Any] = field(default_factory=dict)
    apps: dict[str, Any] = field(default_factory=dict)
    snapshot: dict[str, Any] = field(default_factory=dict)
    source_path: Optional[Path] = None

    @classmethod
    def from_dict(cls, data: dict[str, Any], source: Optional[Path] = None) -> "Preset":
        return cls(
            name=str(data.get("name", source.stem if source else "unnamed")),
            display_name=str(data.get("display_name", data.get("name", "Unnamed"))),
            description=str(data.get("description", "")).strip(),
            appearance=dict(data.get("appearance") or {}),
            shell=dict(data.get("shell") or {}),
            devices=dict(data.get("devices") or {}),
            network=dict(data.get("network") or {}),
            system=dict(data.get("system") or {}),
            apps=dict(data.get("apps") or {}),
            snapshot=dict(data.get("snapshot") or {}),
            source_path=source,
        )


# ---------------------------------------------------------------------------
# Discovery and load
# ---------------------------------------------------------------------------


def _candidate_dirs() -> Iterable[Path]:
    if USER_PRESET_DIR.is_dir():
        yield USER_PRESET_DIR
    for d in SHIPPED_PRESET_DIRS:
        if d.is_dir():
            yield d


def list_presets() -> list[Preset]:
    """All presets visible to Mackes, with user overrides shadowing shipped ones.

    The default preset (DEFAULT_PRESET_NAME) is sorted to the front so callers
    that index [0] (e.g. the wizard's preset picker) get the right starting
    selection without extra logic.
    """
    if yaml is None:
        return []
    seen: dict[str, Preset] = {}
    for d in _candidate_dirs():
        for path in sorted(d.glob("*.yaml")):
            try:
                data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
            except (OSError, yaml.YAMLError):
                continue
            preset = Preset.from_dict(data, source=path)
            seen.setdefault(preset.name, preset)
    presets = list(seen.values())
    presets.sort(key=lambda p: (p.name != DEFAULT_PRESET_NAME, p.name))
    return presets


def default_preset() -> Optional[Preset]:
    """The shipped default. Falls back to the first available if missing."""
    preset = load_preset(DEFAULT_PRESET_NAME)
    if preset is not None:
        return preset
    presets = list_presets()
    return presets[0] if presets else None


def load_preset(name: str) -> Optional[Preset]:
    for preset in list_presets():
        if preset.name == name:
            return preset
    return None


# ---------------------------------------------------------------------------
# Apply pipeline
# ---------------------------------------------------------------------------


# Map preset.appearance keys -> (xfconf channel, property)
APPEARANCE_KEYS: dict[str, tuple[str, str]] = {
    "gtk_theme":      ("xsettings", "/Net/ThemeName"),
    "icon_theme":     ("xsettings", "/Net/IconThemeName"),
    "cursor_theme":   ("xsettings", "/Gtk/CursorThemeName"),
    "cursor_size":    ("xsettings", "/Gtk/CursorThemeSize"),
    "font_ui":        ("xsettings", "/Gtk/FontName"),
    "font_monospace": ("xsettings", "/Gtk/MonospaceFontName"),
}

# Wallpaper key — per-monitor in reality; we apply to the canonical default.
WALLPAPER_KEY = ("xfce4-desktop", "/backdrop/screen0/monitor0/workspace0/last-image")


def apply_appearance(preset: Preset) -> list[str]:
    """Push the preset's appearance section into xfconf. Returns a log of writes."""
    from mackes.xfconf_bridge import get_bridge, XfconfError
    actions: list[str] = []
    try:
        xf = get_bridge()
    except XfconfError as e:
        actions.append(f"skip appearance: {e}")
        return actions
    for field_name, value in preset.appearance.items():
        if field_name == "wallpaper":
            continue  # handled below
        binding = APPEARANCE_KEYS.get(field_name)
        if binding is None:
            actions.append(f"skip unknown appearance key: {field_name}")
            continue
        channel, prop = binding
        try:
            xf.set(channel, prop, value)
            actions.append(f"set {channel}{prop} = {value!r}")
        except XfconfError as e:
            actions.append(f"failed {channel}{prop}: {e}")
    wp = preset.appearance.get("wallpaper")
    if wp and Path(str(wp)).exists():
        try:
            xf.set(*WALLPAPER_KEY, str(wp), type_hint="string")
            actions.append(f"set wallpaper -> {wp}")
        except XfconfError as e:
            actions.append(f"failed wallpaper: {e}")
    return actions


def apply_shell(preset: Preset) -> list[str]:
    from mackes.shell_profiles import apply_polybar, apply_plank, apply_rofi, set_xfce_panel_enabled
    from mackes.session_manager import apply_chupre_dotfiles
    actions: list[str] = []
    # Stage the chupre dotfiles bundle (alacritty, gtk-3.0, gtk-4.0) BEFORE
    # applying shell-stack profiles, so GTK theme/font settings are in place
    # when Polybar/Plank read them. Only runs when the preset opts in via
    # `shell.apply_chupre_dotfiles: true` — defaults on for chupre itself.
    if preset.shell.get("apply_chupre_dotfiles", False):
        actions.extend(apply_chupre_dotfiles())
    if "polybar_profile" in preset.shell:
        actions.extend(apply_polybar(preset.shell["polybar_profile"]))
    if "plank_profile" in preset.shell:
        actions.extend(apply_plank(preset.shell["plank_profile"]))
    if "rofi_profile" in preset.shell:
        actions.extend(apply_rofi(preset.shell["rofi_profile"]))
    if "xfce_panel_enabled" in preset.shell:
        actions.extend(set_xfce_panel_enabled(bool(preset.shell["xfce_panel_enabled"])))
    return actions


def apply_devices(preset: Preset) -> list[str]:
    from mackes.xfconf_bridge import get_bridge, XfconfError
    actions: list[str] = []
    try:
        xf = get_bridge()
    except XfconfError as e:
        actions.append(f"skip devices: {e}")
        return actions
    if "power_profile" in preset.devices:
        prof = preset.devices["power_profile"]
        try:
            xf.set("xfce4-power-manager", "/xfce4-power-manager/power-profile", str(prof),
                   type_hint="string")
            actions.append(f"power profile -> {prof}")
        except XfconfError as e:
            actions.append(f"power: {e}")
    if "audio_default_sink" in preset.devices:
        actions.append(f"audio sink (informational only): {preset.devices['audio_default_sink']}")
    return actions


def apply_system(preset: Preset) -> list[str]:
    from mackes.xfconf_bridge import get_bridge, XfconfError
    actions: list[str] = []
    try:
        xf = get_bridge()
    except XfconfError as e:
        actions.append(f"skip system: {e}")
        return actions
    if "workspace_count" in preset.system:
        try:
            xf.set("xfwm4", "/general/workspace_count", int(preset.system["workspace_count"]))
            actions.append(f"workspaces -> {preset.system['workspace_count']}")
        except XfconfError as e:
            actions.append(f"workspaces: {e}")
    if "window_manager_theme" in preset.system:
        try:
            xf.set("xfwm4", "/general/theme", str(preset.system["window_manager_theme"]),
                   type_hint="string")
            actions.append(f"xfwm theme -> {preset.system['window_manager_theme']}")
        except XfconfError as e:
            actions.append(f"xfwm: {e}")
    if "notifications_enabled" in preset.system:
        enabled = bool(preset.system["notifications_enabled"])
        try:
            xf.set("xfce4-notifyd", "/notify-location", 1 if enabled else 0)
            actions.append(f"notifications -> {'on' if enabled else 'off'}")
        except XfconfError as e:
            actions.append(f"notifyd: {e}")
    extras = preset.system.get("autostart_extras") or []
    if extras:
        actions.append(f"autostart hint (manual): {', '.join(extras)}")
    return actions


def apply_network(preset: Preset) -> list[str]:
    from mackes.qnm_bridge import set_qnm_enabled
    actions: list[str] = []
    if "qnm_enabled" in preset.network:
        actions.extend(set_qnm_enabled(bool(preset.network["qnm_enabled"])))
    if "firewall_default_zone" in preset.network:
        actions.append(f"firewall zone hint: {preset.network['firewall_default_zone']} "
                       "(use Network → Firewall to apply)")
    return actions


def apply_preset(preset: Preset, *, sections: Optional[set[str]] = None) -> list[str]:
    """Apply selected (or all) sections of a preset.

    sections: optional subset of {appearance, shell, devices, system, network}.
    Returns a flat list of human-readable action lines, also written to mackes.log.
    """
    sections = sections or {"appearance", "shell", "devices", "system", "network"}
    actions: list[str] = [f"--- apply preset: {preset.name} ---"]
    if "appearance" in sections:
        actions.extend(apply_appearance(preset))
    if "shell" in sections:
        actions.extend(apply_shell(preset))
    if "devices" in sections:
        actions.extend(apply_devices(preset))
    if "system" in sections:
        actions.extend(apply_system(preset))
    if "network" in sections:
        actions.extend(apply_network(preset))
    for line in actions:
        log_action(line)
    return actions


# ---------------------------------------------------------------------------
# Drift detection (read-only; informational on Dashboard §4.2)
# ---------------------------------------------------------------------------


@dataclass
class DriftItem:
    section: str
    field: str
    expected: Any
    actual: Any

    def describe(self) -> str:
        return f"{self.section}.{self.field}: preset={self.expected!r} live={self.actual!r}"


def _read_appearance_actual() -> dict[str, Any]:
    from mackes.xfconf_bridge import get_bridge, XfconfError
    try:
        xf = get_bridge()
    except XfconfError:
        return {}
    out: dict[str, Any] = {}
    for field_name, (channel, prop) in APPEARANCE_KEYS.items():
        out[field_name] = xf.get(channel, prop, None)
    out["wallpaper"] = xf.get(*WALLPAPER_KEY, "")
    return out


def _read_shell_actual() -> dict[str, Any]:
    from mackes.shell_profiles import current_polybar_profile, current_plank_profile, current_rofi_profile, xfce_panel_enabled
    return {
        "polybar_profile":   current_polybar_profile(),
        "plank_profile":     current_plank_profile(),
        "rofi_profile":      current_rofi_profile(),
        "xfce_panel_enabled": xfce_panel_enabled(),
    }


def _read_system_actual() -> dict[str, Any]:
    from mackes.xfconf_bridge import get_bridge, XfconfError
    try:
        xf = get_bridge()
    except XfconfError:
        return {}
    return {
        "workspace_count":      xf.get("xfwm4", "/general/workspace_count", None),
        "window_manager_theme": xf.get("xfwm4", "/general/theme", None),
        "notifications_enabled": (xf.get("xfce4-notifyd", "/notify-location", 1) != 0),
    }


def detect_drift(preset: Preset) -> list[DriftItem]:
    items: list[DriftItem] = []
    sections: list[tuple[str, dict[str, Any], dict[str, Any]]] = [
        ("appearance", preset.appearance, _read_appearance_actual()),
        ("shell",      preset.shell,      _read_shell_actual()),
        ("system",     preset.system,     _read_system_actual()),
    ]
    for section_name, expected_dict, actual_dict in sections:
        for field_name, expected in expected_dict.items():
            if field_name == "autostart_extras":
                continue  # informational; no drift check
            actual = actual_dict.get(field_name, None)
            if actual is None:
                continue
            if str(actual).strip() != str(expected).strip():
                items.append(DriftItem(section_name, field_name, expected, actual))
    return items


# Convenience for the Dashboard
def active_preset_drift() -> tuple[Optional[Preset], list[DriftItem]]:
    from mackes.state import MackesState
    state = MackesState.load()
    if not state.active_preset:
        return None, []
    preset = load_preset(state.active_preset)
    if preset is None:
        return None, []
    return preset, detect_drift(preset)
