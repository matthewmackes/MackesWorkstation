#!/bin/sh
# install-helpers/lint-spacing-tokens.sh — pre-commit lint gate (E9.6).
#
# Catches NET-NEW raw spacing / size literals — `.spacing(N)`, `.padding(N)`,
# `.size(N)` written with a bare number instead of a Carbon token from the
# canonical metrics module (E9.2 substrate: SPACING_01..13 + the type scale in
# crates/shell/mde-ui/src/metrics.rs).
#
# COUNT-based, deliberately: it tracks a per-file MAX count in
# `lint-spacing-tokens.snapshot`, so it is immune to the line-shift desync that
# plagues the file:line allowlists (lint-design-tokens / lint-voice) — a
# `cargo fmt` reflow that MOVES a literal never trips it; only a genuine
# INCREASE (a net-new raw literal) does. The snapshot ratchets DOWN as E9.3
# migrates surfaces to tokens; it should never go up without a recorded reason
# (a genuine off-grid / dense-chrome value, the E9.2 pragmatic-exception clause).
#
# Zero literals (`(0.0)` / `(0)`) are "no gap", not a design token, and are not
# counted. The metrics module itself and test trees are excluded from the scan.

set -eu
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"
SNAP="install-helpers/lint-spacing-tokens.snapshot"

# Per-file occurrence count of raw spacing/size literals, minus zero literals.
cur="$(mktemp)"
trap 'rm -f "$cur"' EXIT
find crates -type f -name '*.rs' \
    ! -path '*/mde-ui/src/metrics.rs' \
    ! -path '*/tests/*' \
    -exec sh -c '
        for f; do
            all=$(grep -hoE "\.(spacing|padding|size)\([0-9]" "$f" 2>/dev/null | wc -l)
            zero=$(grep -hoE "\.(spacing|padding|size)\((0\.0|0)\)" "$f" 2>/dev/null | wc -l)
            n=$((all - zero))
            [ "$n" -gt 0 ] && printf "%s %s\n" "$f" "$n"
        done
    ' sh {} + | sort > "$cur"

# Bootstrap: first run records the snapshot and passes.
if [ ! -f "$SNAP" ]; then
    {
        echo "# Snapshot for lint-spacing-tokens.sh (E9.6) — per-file MAX count of"
        echo "# raw .spacing()/.padding()/.size() numeric literals (Carbon token"
        echo "# migration, E9.3). Format: <file> <count>. Ratchets DOWN as surfaces"
        echo "# migrate; a rise needs a recorded off-grid/dense-chrome rationale."
        cat "$cur"
    } > "$SNAP"
    echo "lint-spacing-tokens.sh: snapshot bootstrapped ($(wc -l < "$cur" | tr -d ' ') files)."
    exit 0
fi

# Compare current counts to the snapshot; fail on any net-new increase.
regress=0
while read -r f n; do
    snap=$(awk -v F="$f" '$1==F {print $2; exit}' "$SNAP")
    [ -n "$snap" ] || snap=0
    if [ "$n" -gt "$snap" ]; then
        if [ "$regress" -eq 0 ]; then
            echo "lint-spacing-tokens.sh: net-new raw spacing/size literals:"
            echo ""
        fi
        echo "  $f: $n raw .spacing/.padding/.size literals (snapshot allows $snap)"
        regress=1
    fi
done < "$cur"

if [ "$regress" -eq 1 ]; then
    echo ""
    echo "Migrate the new literal(s) to a metrics:: Carbon token (E9.2), or — for a"
    echo "genuine off-grid / dense-chrome value (E9.2 exception) — raise that file's"
    echo "count in $SNAP with a one-line rationale. Re-stage and commit again."
    exit 1
fi

echo "lint-spacing-tokens.sh: clean (no net-new raw spacing/size literals)"
exit 0
