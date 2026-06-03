#!/bin/sh
# v2.0.0 Phase I.7 — No-XFCE gate.
#
# Runs in the v2.0.0 install / upgrade workflow + CI to assert that
# the box has cleanly retired XFCE: no xfce4-* packages installed
# beyond optional icon themes / language packs.
#
# Allowlist: anything matching one of the locked patterns is OK
# (icon themes, terminal themes — these are pure data and don't
# pull in xfconfd or any of the retired panel/desktop/session
# stack).
#
# Exits 0 on a clean no-XFCE box, non-zero otherwise.

set -eu

if ! command -v rpm >/dev/null 2>&1; then
    printf '%s\n' "check-no-xfce: rpm not on PATH; skipping" >&2
    exit 0
fi

# Pull every xfce-prefixed package and filter the allowlist.
unexpected=$(
    rpm -qa --qf '%{NAME}\n' \
        | grep -iE '^xfce|^xf(conf|desktop|wm|settings|notifyd|panel|session|power)' \
        | grep -viE '^xfce4-icon-theme-|^xfce4-icon-|^xfce-icon|theme$|^xfce4-dev-tools$' \
        || true
)

if [ -n "${unexpected}" ]; then
    printf '%s\n' "FAIL: unexpected xfce* packages installed:" >&2
    printf '%s\n' "${unexpected}" >&2
    exit 1
fi

printf '%s\n' "check-no-xfce: clean (no XFCE panel / session / settings stack)"
exit 0
