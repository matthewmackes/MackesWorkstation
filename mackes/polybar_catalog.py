"""Polybar theme catalog — discovery + parsing of vendored adi1090x themes.

Reads `data/shell-profiles/polybar/upstream/{simple,bitmap}/<family>/` and
exposes:

- `list_families()` -> all theme families across both variants
- `list_modules(family)` -> module names defined by a family
- `palette(family)` -> dict of color-name -> hex value
- `bar_layout(family)` -> default modules-left/center/right tuples

This module is pure read-only over the vendored data; it does no I/O against
~/.config/polybar/. The generator (mackes.polybar_gen) consumes the catalog
to assemble a self-contained config.ini.
"""
from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


# ---------------------------------------------------------------------------
# Discovery roots
# ---------------------------------------------------------------------------

_UPSTREAM_ROOTS = (
    Path("/usr/share/mackes-shell/data/shell-profiles/polybar/upstream"),
    Path(__file__).resolve().parent.parent
    / "data" / "shell-profiles" / "polybar" / "upstream",
)


def _upstream_root() -> Path | None:
    for root in _UPSTREAM_ROOTS:
        if root.is_dir():
            return root
    return None


# ---------------------------------------------------------------------------
# Model
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Family:
    name: str                    # e.g. "forest"
    variant: str                 # "simple" | "bitmap"
    path: Path                   # absolute path to the family dir

    @property
    def key(self) -> str:
        """Catalog-wide unique key, e.g. `simple/forest`."""
        return f"{self.variant}/{self.name}"


@dataclass
class Module:
    name: str                    # module name without the `module/` prefix
    type: str = ""               # internal/network, custom/script, etc.
    family: str = ""             # owning family key
    source: str = ""             # 'modules' | 'user_modules'


@dataclass
class BarLayout:
    width: str = "100%"
    height: str = "32"
    radius: str = "0"
    modules_left: tuple[str, ...] = ()
    modules_center: tuple[str, ...] = ()
    modules_right: tuple[str, ...] = ()


# ---------------------------------------------------------------------------
# Discovery
# ---------------------------------------------------------------------------


def list_families() -> list[Family]:
    root = _upstream_root()
    if root is None:
        return []
    families: list[Family] = []
    for variant in ("simple", "bitmap"):
        v_root = root / variant
        if not v_root.is_dir():
            continue
        for sub in sorted(v_root.iterdir()):
            if not sub.is_dir():
                continue
            # require a config.ini to count as a family
            if not (sub / "config.ini").is_file():
                continue
            families.append(Family(name=sub.name, variant=variant, path=sub))
    return families


def get_family(key: str) -> Family | None:
    """Look up a family by `<variant>/<name>` key or just `<name>` (preferring simple)."""
    if "/" in key:
        variant, name = key.split("/", 1)
        for f in list_families():
            if f.variant == variant and f.name == name:
                return f
        return None
    # bare name — prefer simple, fall back to bitmap
    candidates = [f for f in list_families() if f.name == key]
    candidates.sort(key=lambda f: 0 if f.variant == "simple" else 1)
    return candidates[0] if candidates else None


# ---------------------------------------------------------------------------
# Lightweight INI parsing
# ---------------------------------------------------------------------------
# polybar's config.ini uses a slightly non-strict INI dialect (`;` and `;;`
# comments, multi-line values, `${section.key}` references). configparser
# chokes on some of it; rolling our own is cheaper than fighting it.


_SECTION_RE = re.compile(r"^\s*\[([^\]]+)\]\s*$")
_KEY_RE = re.compile(r"^\s*([\w.-]+)\s*=\s*(.*?)\s*$")


def _iter_sections(path: Path) -> Iterable[tuple[str, dict[str, str]]]:
    """Yield (section_name, {key: value}) pairs from a polybar-style INI."""
    if not path.is_file():
        return
    current: str | None = None
    body: dict[str, str] = {}
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            line = raw.split(";", 1)[0].rstrip()  # strip trailing comments
            if not line.strip():
                continue
            m = _SECTION_RE.match(line)
            if m:
                if current is not None:
                    yield current, body
                current = m.group(1)
                body = {}
                continue
            m = _KEY_RE.match(line)
            if m and current is not None:
                body[m.group(1)] = m.group(2)
    if current is not None:
        yield current, body


# ---------------------------------------------------------------------------
# Module extraction
# ---------------------------------------------------------------------------


def list_modules(family: Family) -> list[Module]:
    out: list[Module] = []
    for source_name in ("modules", "user_modules"):
        for section, body in _iter_sections(family.path / f"{source_name}.ini"):
            if not section.startswith("module/"):
                continue
            out.append(Module(
                name=section[len("module/"):],
                type=body.get("type", ""),
                family=family.key,
                source=source_name,
            ))
    return out


# ---------------------------------------------------------------------------
# Palette extraction
# ---------------------------------------------------------------------------


def palette(family: Family) -> dict[str, str]:
    """Return the [color] section of the family's colors.ini as a dict."""
    for section, body in _iter_sections(family.path / "colors.ini"):
        if section == "color":
            return dict(body)
    return {}


# ---------------------------------------------------------------------------
# Bar layout extraction
# ---------------------------------------------------------------------------


def _split_modules(spec: str) -> tuple[str, ...]:
    return tuple(m for m in spec.split() if m)


def bar_layout(family: Family) -> BarLayout:
    """Read the first [bar/...] section from config.ini and return its layout."""
    for section, body in _iter_sections(family.path / "config.ini"):
        if not section.startswith("bar/"):
            continue
        return BarLayout(
            width=body.get("width", "100%"),
            height=body.get("height", "32"),
            radius=body.get("radius", "0"),
            modules_left=_split_modules(body.get("modules-left", "")),
            modules_center=_split_modules(body.get("modules-center", "")),
            modules_right=_split_modules(body.get("modules-right", "")),
        )
    return BarLayout()


# ---------------------------------------------------------------------------
# Whole-catalog dump (used by `python -m mackes.polybar_catalog`)
# ---------------------------------------------------------------------------


def _summary() -> str:
    lines: list[str] = []
    root = _upstream_root()
    lines.append(f"upstream root: {root}")
    families = list_families()
    lines.append(f"families: {len(families)} "
                 f"({sum(1 for f in families if f.variant == 'simple')} simple, "
                 f"{sum(1 for f in families if f.variant == 'bitmap')} bitmap)")
    lines.append("")
    for f in families:
        mods = list_modules(f)
        layout = bar_layout(f)
        pal = palette(f)
        lines.append(f"  {f.key:20s} modules={len(mods):2d} palette-colors={len(pal):2d} "
                     f"bar={len(layout.modules_left)}L/{len(layout.modules_center)}C/"
                     f"{len(layout.modules_right)}R")
    return "\n".join(lines)


if __name__ == "__main__":
    print(_summary())
