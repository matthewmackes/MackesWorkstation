"""Unified mackes log.

A single rotating log under `~/.local/share/mackes-shell/logs/mackes.log` that
every Mackes module writes to. The Maintain → Logs panel tails this file; the
Dashboard's "Recent activity" reads its last lines.

Format is intentionally human-readable rather than JSON — it's read by users
in a Gtk.TextView, not parsed.
"""
from __future__ import annotations

import logging
from logging.handlers import RotatingFileHandler

from mackes.state import LOG_DIR, ensure_dirs


_LOGGER: logging.Logger | None = None


def get_logger() -> logging.Logger:
    global _LOGGER
    if _LOGGER is not None:
        return _LOGGER

    ensure_dirs()
    log = logging.getLogger("mackes")
    log.setLevel(logging.INFO)
    log.propagate = False

    handler = RotatingFileHandler(
        LOG_DIR / "mackes.log",
        maxBytes=512_000,
        backupCount=3,
        encoding="utf-8",
    )
    handler.setFormatter(
        logging.Formatter("%(asctime)s  %(levelname)-5s  %(name)s :: %(message)s",
                          datefmt="%Y-%m-%d %H:%M:%S")
    )
    log.addHandler(handler)
    _LOGGER = log
    return log


def log_action(message: str, *, level: int = logging.INFO) -> None:
    """Convenience: log a single user-visible action."""
    get_logger().log(level, message)
