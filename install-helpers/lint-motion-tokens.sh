#!/bin/sh
# install-helpers/lint-motion-tokens.sh — pre-commit lint gate.
#
# Catches ad-hoc motion-grid Duration literals in active animation
# code that should reference the canonical motion tokens instead.
#
# The canonical animation-timing contract lives in the mde-theme
# crate's motion presets:
#
#   crates/shared/mde-theme/src/motion.rs    (Motion::* presets:
#                                             panel_mount / dialog_mount /
#                                             tooltip_fade / ...)
#   crates/shared/mde-theme/src/animation.rs (Tween + named timings)
#   data/css/motion-vocabulary.css           (central documentation
#                                             those constants point back
#                                             to; GTK CSS is doc-only)
#
# Motion grid values (data/css/motion-vocabulary.css §1):
#   100 ms — active / button press / toggle / exit one-tier-faster
#   120 ms — info dismissal (toast / popover close)
#   150 ms — standard state change (default)
#   180 ms — panel / dialog mount
#   200 ms — info arrival (toast / popover open)
#
# Bare `Duration::from_millis(N)` for these grid values in
# animation-path code should reference a Motion::* preset's
# `.duration` or a named constant from mde_theme::animation instead.
#
# NOTE ON SCOPE: this gate targets *animation* durations only. The
# mde-theme crate is the canonical source and is excluded from the
# scan. Non-animation Durations — poll/tick cadences, debounce
# windows, sleeps, timeouts, reconnect backoff, and test fixtures —
# are NOT motion tokens; they are filtered out heuristically, and any
# pre-existing residue is captured in the snapshot allow-list so the
# gate exits 0 today and catches only NET-NEW animation-grid literals
# going forward.
#
# A motion-token source DOES exist in this tree
# (data/css/motion-vocabulary.css), so this gate is ACTIVE. If that
# file is ever absent (e.g. before the Win10 animation work lands the
# tokens), the gate degrades to a no-op that exits 0 with a note.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of
# Done) for how this gate fits the monorepo's visual direction.
#
# Exits 0 = clean, exits 1 = net-new motion-grid Duration literals.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

MOTION_TOKEN_FILE="${REPO_ROOT}/data/css/motion-vocabulary.css"
ALLOWLIST_FILE="${REPO_ROOT}/install-helpers/lint-motion-tokens.allowlist"

# If the motion-token file is absent in this tree, the motion tokens
# have not landed yet (they arrive with the Win10 animation work).
# Degrade to a no-op that exits 0 with a note.
if [ ! -f "$MOTION_TOKEN_FILE" ]; then
    echo "$0: no-op — data/css/motion-vocabulary.css not present yet."
    echo "  (motion tokens land with the Win10 animation work; nothing"
    echo "   to enforce against until then.)"
    exit 0
fi

# Rust source lives under crates/**/src (there is no rust/ dir; Python
# is retired to provenance/ and is not scanned). The mde-theme crate is
# the canonical motion source and is excluded below.
SCAN_PATHS='crates'

# Motion-grid Duration literals. egrep-compatible.
PATTERN='Duration::from_millis\((100|120|150|180|200)\b'

TMPFILE="$(mktemp)"
trap 'rm -f "$TMPFILE"' EXIT

# Scan. grep exits 1 if no match — suppress that.
# shellcheck disable=SC2086
grep -rnE "$PATTERN" --include='*.rs' $SCAN_PATHS 2>/dev/null > "$TMPFILE" || true

# Exclude the canonical motion source crate (mde-theme) — it is where
# the grid values are legitimately defined.
grep -v '/mde-theme/' "$TMPFILE" > "$TMPFILE.f" && mv "$TMPFILE.f" "$TMPFILE"

# Filter out test files / directories / cfg(test) lines.
grep -v '_tests\.rs:' "$TMPFILE" > "$TMPFILE.f" && mv "$TMPFILE.f" "$TMPFILE"
grep -v '/tests/' "$TMPFILE" > "$TMPFILE.f" && mv "$TMPFILE.f" "$TMPFILE"
grep -v '#\[cfg(test' "$TMPFILE" > "$TMPFILE.f" && mv "$TMPFILE.f" "$TMPFILE"

# Filter out infrastructure patterns (poll ticks, sleeps, timeouts,
# debounce, reconnect backoff) — these are not animation durations.
grep -vE 'sleep\(|timeout\(|every\(|tick_interval|poll_interval|reconnect' \
    "$TMPFILE" > "$TMPFILE.f" && mv "$TMPFILE.f" "$TMPFILE"

# Filter pure comment lines (file:line:   // ...).
grep -vE '^[^:]*:[0-9]*:[[:space:]]*//' "$TMPFILE" > "$TMPFILE.f" \
    && mv "$TMPFILE.f" "$TMPFILE"

# Reduce to file:line keys for allow-list matching.
KEYS="$(sed -nE 's|^([^:]+):([0-9]+):.*|\1:\2|p' "$TMPFILE")"

# Apply the snapshot allow-list (sibling .allowlist file, file:line
# keys, '#' comments + blank lines ignored). Same mechanism as
# lint-design-tokens.sh.
if [ -f "$ALLOWLIST_FILE" ]; then
    ALLOW_KEYS="$(grep -v '^[[:space:]]*#' "$ALLOWLIST_FILE" \
        | grep -v '^[[:space:]]*$' || true)"
    if [ -n "$ALLOW_KEYS" ]; then
        TMP_ALLOW="$(mktemp)"
        printf '%s\n' "$ALLOW_KEYS" > "$TMP_ALLOW"
        KEYS="$(printf '%s\n' "$KEYS" | grep -vFf "$TMP_ALLOW" 2>/dev/null || true)"
        rm -f "$TMP_ALLOW"
    fi
fi

KEYS="$(printf '%s\n' "$KEYS" | grep -v '^[[:space:]]*$' || true)"

if [ -z "$KEYS" ]; then
    echo "$0: clean (no net-new motion-grid Duration literals)"
    exit 0
fi

echo "$0: net-new motion-grid Duration literals:"
echo
# Re-print full context lines for the surviving keys.
printf '%s\n' "$KEYS" | while IFS= read -r key; do
    grep -F "$key:" "$TMPFILE" || true
done
echo
echo "Motion-grid Duration literals must come from the mde-theme motion"
echo "presets, not bare from_millis() calls. Use:"
echo "  Motion::panel_mount().duration    // 180 ms"
echo "  Motion::dialog_mount().duration   // 180 ms"
echo "  Motion::tooltip_fade().duration   // 120 ms"
echo "Or add a named const in mde_theme::motion if the preset is new."
echo
echo "If this is a NON-animation Duration (poll tick, debounce, sleep,"
echo "timeout, test fixture), move it out of the pattern match or add"
echo "its <file>:<line> key to install-helpers/lint-motion-tokens.allowlist"
echo "with a dated rationale comment."
exit 1
