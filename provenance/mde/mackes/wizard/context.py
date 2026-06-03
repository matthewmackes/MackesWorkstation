"""Shared mutable context passed between wizard pages.

Each page reads/writes fields here. The Apply page reduces the context down
to the final actions to take.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from mackes.presets import Preset


@dataclass
class WizardContext:
    selected_preset: Optional[Preset] = None
    create_initial_snapshot: bool = True
    snapshot_label: str = "initial"
    enable_qnm: bool = True
    firewall_zone: str = "FedoraWorkstation"
    imported_vpn_path: Optional[str] = None
    # User overrides on the preset's defaults (applied after apply_preset)
    overrides: dict[str, dict[str, object]] = field(default_factory=dict)
    # Filled in by env_scan; consumed by review
    missing_packages: list[str] = field(default_factory=list)
    detected: dict[str, str] = field(default_factory=dict)
