"""v2.0.0 Phase 0.4 + 0.13 — D-Bus service file presence + alias parity tests.

Asserts the data/dbus-1/services/ directory ships the expected
``dev.mackes.MDE.*`` service files AND the one-release legacy
``org.mackes.*`` aliases. Each file must reference the right Exec=
and SystemdService= so the session bus actually starts mded when
something asks for the name.
"""
from __future__ import annotations

from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DBUS_DIR = REPO / "data" / "dbus-1" / "services"


# (filename, expected Name=, expected Exec= prefix)
MDE_SERVICES = [
    ("dev.mackes.MDE.Shell.service",        "dev.mackes.MDE.Shell",        "/usr/bin/mded"),
    ("dev.mackes.MDE.Settings.service",     "dev.mackes.MDE.Settings",     "/usr/bin/mded"),
    ("dev.mackes.MDE.Session.service",      "dev.mackes.MDE.Session",      "/usr/bin/mde-session"),
    ("dev.mackes.MDE.Fleet.service",        "dev.mackes.MDE.Fleet",        "/usr/bin/mded"),
    ("dev.mackes.MDE.Notifications.service","dev.mackes.MDE.Notifications","/usr/bin/mded"),
]

LEGACY_ALIASES = [
    ("org.mackes.Shell.service",    "org.mackes.Shell",    "/usr/bin/mded"),
    ("org.mackes.Settings.service", "org.mackes.Settings", "/usr/bin/mded"),
    ("org.mackes.Session.service",  "org.mackes.Session",  "/usr/bin/mde-session"),
    ("org.mackes.Fleet.service",    "org.mackes.Fleet",    "/usr/bin/mded"),
]


def _read_kv(path: Path) -> dict[str, str]:
    out = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith(("#", "[")):
            continue
        if "=" in line:
            k, v = line.split("=", 1)
            out[k.strip()] = v.strip()
    return out


def test_every_mde_service_file_ships():
    for filename, name, _exec in MDE_SERVICES:
        path = DBUS_DIR / filename
        assert path.is_file(), f"missing D-Bus service file: {path}"
        kv = _read_kv(path)
        assert kv["Name"] == name


def test_every_mde_service_file_has_systemd_activation():
    for filename, _name, _exec in MDE_SERVICES:
        path = DBUS_DIR / filename
        kv = _read_kv(path)
        assert "SystemdService" in kv, f"{filename} must specify SystemdService="
        assert kv["SystemdService"].endswith(".service")


def test_mde_service_exec_targets_match_binary_name():
    for filename, _name, exec_prefix in MDE_SERVICES:
        path = DBUS_DIR / filename
        kv = _read_kv(path)
        assert kv["Exec"].startswith(exec_prefix), (
            f"{filename}: Exec={kv['Exec']!r} should start with {exec_prefix!r}"
        )


def test_every_legacy_alias_ships_for_one_release_backward_compat():
    for filename, name, _exec in LEGACY_ALIASES:
        path = DBUS_DIR / filename
        assert path.is_file(), (
            f"missing legacy alias D-Bus service file: {path}. "
            "Phase 0.4 lock keeps the v1.x service name resolvable "
            "for one release."
        )
        kv = _read_kv(path)
        assert kv["Name"] == name


def test_legacy_alias_resolves_to_same_systemd_service_as_new_name():
    """Each org.mackes.* alias must route to the same systemd unit
    as the matching dev.mackes.MDE.* file so the session bus brings
    up exactly one mded process regardless of which name was
    requested."""
    new_by_concern = {
        "Shell":    _read_kv(DBUS_DIR / "dev.mackes.MDE.Shell.service"),
        "Settings": _read_kv(DBUS_DIR / "dev.mackes.MDE.Settings.service"),
        "Session":  _read_kv(DBUS_DIR / "dev.mackes.MDE.Session.service"),
        "Fleet":    _read_kv(DBUS_DIR / "dev.mackes.MDE.Fleet.service"),
    }
    for filename, _name, _exec in LEGACY_ALIASES:
        concern = filename.removeprefix("org.mackes.").removesuffix(".service")
        kv = _read_kv(DBUS_DIR / filename)
        assert kv["SystemdService"] == new_by_concern[concern]["SystemdService"]


def test_legacy_alias_files_carry_phase_0_4_comment():
    """The alias files are temporary (drop in v2.1). Each must
    include a comment explaining why so a future cleanup pass
    knows to delete them."""
    for filename, _name, _exec in LEGACY_ALIASES:
        text = (DBUS_DIR / filename).read_text(encoding="utf-8")
        assert "Phase 0.4" in text or "backward-compat" in text.lower()
