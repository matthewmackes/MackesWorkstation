"""In-process TTL cache for stable system probes.

Many panel constructors shell out to xrandr / fc-list / rpm -q /
xinput / pactl to populate static-ish lists (output names, font
families, installed packages, input devices, audio sinks). Those
results are stable for the duration of a session — invalidate them
and re-query only when state actually changes.

Pattern:

    from mackes.probe_cache import cached

    def list_outputs() -> list[str]:
        return cached("xrandr.outputs", ttl_s=60,
                      factory=lambda: _shell_out_xrandr())

The cache is process-scoped (lives in module-level dict), thread-safe
(single Lock), and time-bounded by TTL. Set ttl_s=None for a value
that should live for the entire process.

Eviction: there is none beyond TTL. Mackes panels make ~dozens of
unique cache keys, not thousands; no LRU needed.
"""
from __future__ import annotations

import threading
import time
from typing import Any, Callable, Optional


_CACHE: dict[str, tuple[float, Any]] = {}
_LOCK = threading.Lock()


def cached(key: str, *, factory: Callable[[], Any],
           ttl_s: Optional[float]) -> Any:
    """Return cached value for `key`, computing it via `factory()` on miss.

    `ttl_s=None` means infinite (until `invalidate(key)` or process exit).
    """
    now = time.monotonic()
    with _LOCK:
        entry = _CACHE.get(key)
        if entry is not None:
            expires_at, value = entry
            if expires_at == 0 or expires_at > now:
                return value
    # Compute outside the lock so a slow factory doesn't serialise
    # every other cache call.
    value = factory()
    expires_at = 0.0 if ttl_s is None else (now + ttl_s)
    with _LOCK:
        _CACHE[key] = (expires_at, value)
    return value


def invalidate(key: str) -> None:
    """Drop a single cache entry. Safe if key is absent."""
    with _LOCK:
        _CACHE.pop(key, None)


def invalidate_prefix(prefix: str) -> None:
    """Drop every cache entry whose key starts with `prefix`.

    Useful when a write happens (e.g. apply_layout) — invalidate the
    whole "xrandr.*" family rather than enumerating known keys.
    """
    with _LOCK:
        for k in list(_CACHE.keys()):
            if k.startswith(prefix):
                del _CACHE[k]


def clear() -> None:
    """Drop the entire cache. Mostly for tests."""
    with _LOCK:
        _CACHE.clear()
