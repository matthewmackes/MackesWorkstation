#!/bin/sh
# install-helpers/lint-css.sh — pre-commit gate for data/css/*.css.
#
# Generic CSS hygiene: loads each stylesheet under data/css/ through
# GtkCssProvider and reports any parse error GTK emits. This catches
# real syntax mistakes (unbalanced braces, bad selectors, malformed
# property values) before they land. Run with no args to lint the whole
# data/css/ tree, or pass explicit paths.
#
# A handful of warning classes are whitelisted as accepted (GTK CSS
# simply does not implement some properties; these are noise, not
# defects):
#   - 'text-transform'        (GTK CSS does not implement it)
#   - 'font-feature-settings' (value-parsing quirk in tokens.css)
#   - 'cursor'                (accepted, just noisy)
#   - 'line-height'           (GTK CSS computes line-height itself)
#
# GRACEFUL DEGRADATION: this gate's only checker is GTK's CssProvider,
# reached via python3 + the PyGObject (gi) GTK 3.0 binding. Python is
# RETIRED in this monorepo (it survives only under provenance/), so that
# toolchain is NOT guaranteed to be installed on a contributor's box. If
# python3 or the gi/GTK 3.0 binding is absent, this gate SKIPS WITH A
# NOTE and exits 0 rather than failing the commit — there is no
# stylelint/npm fallback in this Rust monorepo. When the binding is
# present (as in the merge environment) the lint runs in full.
#
# A snapshot allow-list at install-helpers/lint-css.allowlist captures
# PRE-EXISTING, accepted parse messages in the merged tree (matched as
# substrings) so the gate exits 0 today and catches only NET-NEW CSS
# errors going forward. The allow-list is meant to SHRINK over time.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of Done)
# for how this gate fits the monorepo's visual direction.
#
# Exit 0 on success (no new errors) or when the GTK binding is absent;
# exit 1 on real, net-new CSS parse errors.
# Run as: install-helpers/lint-css.sh [path...]
# With no args, lints data/css/*.css.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

ALLOWLIST_FILE="${REPO_ROOT}/install-helpers/lint-css.allowlist"

# Graceful degradation: no python3 -> skip (toolchain is retired here).
if ! command -v python3 >/dev/null 2>&1; then
    echo "$0: python3 not present (retired toolchain) — skipping CSS lint (exit 0)."
    exit 0
fi

# Graceful degradation: no gi/GTK 3.0 binding -> skip.
if ! python3 - <<'PROBE' >/dev/null 2>&1
import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: F401
PROBE
then
    echo "$0: PyGObject/GTK 3.0 binding not present — skipping CSS lint (exit 0)."
    exit 0
fi

if [ "$#" -eq 0 ]; then
    # shellcheck disable=SC2046
    set -- $(ls data/css/*.css 2>/dev/null || true)
fi

if [ "$#" -eq 0 ]; then
    echo "$0: no CSS files to lint (exit 0)."
    exit 0
fi

# Read the snapshot allow-list (substrings of accepted pre-existing
# messages), one per line, comments + blanks stripped, into an env var
# the Python helper can consume.
ALLOWLIST_BODY=""
if [ -f "$ALLOWLIST_FILE" ]; then
    ALLOWLIST_BODY=$(grep -v '^[[:space:]]*#' "$ALLOWLIST_FILE" 2>/dev/null | grep -v '^[[:space:]]*$' || true)
fi
export ALLOWLIST_BODY

python3 - "$@" <<'PY'
import os
import sys

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk

# Substrings that flag a pre-existing, accepted warning. The actual
# GTK error messages include these tokens.
WHITELIST = (
    "text-transform",         # GTK CSS does not implement this
    "font-feature-settings",  # value-parsing quirk in tokens.css
    "cursor",                 # accepted, just noisy
    "line-height",            # GTK CSS computes line-height itself
)

# Snapshot allow-list substrings (pre-existing accepted messages),
# passed in from the shell wrapper.
ALLOWLIST = tuple(
    line for line in os.environ.get("ALLOWLIST_BODY", "").splitlines() if line.strip()
)


def accepted(msg):
    if any(w in msg for w in WHITELIST):
        return True
    if any(a in msg for a in ALLOWLIST):
        return True
    return False


failed = False
for path in sys.argv[1:]:
    errors = []

    def on_error(provider, section, error, _errors=errors):
        msg = error.message
        if accepted(msg):
            return
        _errors.append(msg)

    p = Gtk.CssProvider()
    p.connect("parsing-error", on_error)
    try:
        p.load_from_path(path)
    except Exception as e:
        # Some GTK errors raise rather than emit. Treat exception body.
        msg = str(e)
        if not accepted(msg):
            errors.append(msg)
    if errors:
        failed = True
        print(f"FAIL  {path}")
        for m in errors:
            print(f"   - {m}")
    else:
        print(f"OK    {path}")

if failed:
    print()
    print("Net-new CSS parse errors above. Fix the stylesheet, or — if")
    print("the message is a known, accepted pre-existing condition — add")
    print("a distinctive substring of it to")
    print("install-helpers/lint-css.allowlist with a dated rationale.")
sys.exit(1 if failed else 0)
PY
