#!/bin/sh
# v2.0.0 Phase I.6 — Wayland-only gate.
#
# Runs in the v2.0.0 install / upgrade workflow + CI to assert that
# the box has cleanly cut over to Wayland: no Xwayland process
# (XWayland fallback would mask broken sway integration) and no X11
# linkage in the mde-panel binary (the Iced + libcosmic rewrite
# drops every gtk3-rs / X11 dep per Phase E.1).
#
# Exits 0 on a clean Wayland-only box, non-zero otherwise. Each
# failure prints a one-line diagnostic to stderr.

set -eu

errors=0

if pgrep -a Xwayland >/dev/null 2>&1; then
    pgrep -a Xwayland >&2
    printf '%s\n' "FAIL: Xwayland process running — mde-panel should drive Wayland natively" >&2
    errors=$((errors + 1))
fi

if [ -x /usr/bin/mde-panel ]; then
    if ldd /usr/bin/mde-panel 2>/dev/null | grep -qiE 'libx11|libxcb'; then
        ldd /usr/bin/mde-panel | grep -iE 'libx11|libxcb' >&2
        printf '%s\n' "FAIL: mde-panel still links X11 — Phase E.1 panel rewrite incomplete" >&2
        errors=$((errors + 1))
    fi
fi

if [ "${errors}" -gt 0 ]; then
    printf '%s\n' "check-wayland-only: ${errors} failure(s)" >&2
    exit 1
fi

printf '%s\n' "check-wayland-only: clean (no Xwayland, no X11 linkage in mde-panel)"
exit 0
