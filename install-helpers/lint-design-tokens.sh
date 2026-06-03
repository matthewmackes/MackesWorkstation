#!/bin/sh
# install-helpers/lint-design-tokens.sh — pre-commit lint gate.
#
# Catches hardcoded design tokens in active visual code that
# should reference the canonical token sources instead:
#
#   Hex literals (#1d1d1f, #fff, etc.) outside the canonical sites.
#   Rust Color::from_rgb / Color::from_rgba calls outside them too.
#
# Canonical sites (CLAUDE.md section 2.1 — the ONE place raw hex /
# RGB may live):
#
#   crates/shell/mde-ui/src/palette.rs   (Win2000 ground-truth
#                                          palette; the four-theme
#                                          engine remaps from here)
#   data/css/tokens.css                  (CSS design tokens)
#
# Rust attribute syntax (#[derive], #[cfg], #[allow], ...) is
# filtered out so it doesn't false-positive on the hex regex.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of
# Done) for how this gate fits the monorepo's visual direction.
#
# A snapshot allow-list at install-helpers/lint-design-tokens.allowlist
# captures the PRE-EXISTING token-drift in the merged tree at gate-
# install time, so the gate exits 0 today and catches only NET-NEW
# hardcoded tokens going forward. The allow-list is meant to SHRINK
# over time as each entry ports to palette.rs / tokens.css.
#
# Exits 0 = clean, exits 1 = net-new hardcoded-token hits.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

ALLOWLIST_FILE="${REPO_ROOT}/install-helpers/lint-design-tokens.allowlist"

# Source trees to scan. (Rust lives under crates/**/src — there is
# no rust/ dir; Python is retired to provenance/ and is not scanned.)
SCAN_PATHS='crates data/css'

# Files/paths exempt from token enforcement — the canonical token
# sources where raw hex/RGB is allowed (CLAUDE.md section 2.1).
EXEMPT_PATHS='crates/shell/mde-ui/src/palette.rs data/css/tokens.css'

# Helper: filter a list of file:line violation keys against
# EXEMPT_PATHS and the snapshot allow-list file.
filter_violations() {
    filtered="$1"
    # Strip exempt paths
    for exempt in $EXEMPT_PATHS; do
        # Escape `/` for sed
        esc=$(printf '%s' "$exempt" | sed 's|/|\\/|g')
        filtered=$(printf '%s\n' "$filtered" | sed "/${esc}/d")
    done
    # Strip allow-listed lines (by file:line key)
    if [ -f "$ALLOWLIST_FILE" ]; then
        # Allow-list format: <file>:<line> (no message)
        allow_keys=$(grep -v '^[[:space:]]*#' "$ALLOWLIST_FILE" | grep -v '^[[:space:]]*$' || true)
        if [ -n "$allow_keys" ]; then
            tmp_allow=$(mktemp)
            printf '%s\n' "$allow_keys" > "$tmp_allow"
            filtered=$(printf '%s\n' "$filtered" | grep -vFf "$tmp_allow" 2>/dev/null || true)
            rm -f "$tmp_allow"
        fi
    fi
    printf '%s' "$filtered"
}

# 1. Hex literals in source files (Rust + CSS).
HEX_HITS=$(grep -rEn '#[0-9a-fA-F]{3,8}\b' --include='*.rs' --include='*.css' $SCAN_PATHS 2>/dev/null || true)
# Strip Rust attribute syntax — not colors (#[derive(...)] etc.).
HEX_HITS=$(printf '%s\n' "$HEX_HITS" | grep -vE '#\[derive|#\[cfg|#\[allow|#\[deny|#\[warn|#\[must_use|#\[doc|#\[serde|#\[error|#\[test|#\[tokio|#\[clap|#\[command|#\[arg|#\[repr|#\[non_exhaustive|#\[proc|#\[inline|#\[track_caller|#\[automatically|#\[default' || true)
# Reduce to file:line keys for allow-list matching.
HEX_KEYS=$(printf '%s\n' "$HEX_HITS" | sed -nE 's|^([^:]+):([0-9]+):.*|\1:\2|p')

# 2. Rust Color::from_rgb / from_rgba calls.
RGB_HITS=$(grep -rEn 'Color::from_rgb' --include='*.rs' $SCAN_PATHS 2>/dev/null || true)
RGB_KEYS=$(printf '%s\n' "$RGB_HITS" | sed -nE 's|^([^:]+):([0-9]+):.*|\1:\2|p')

# Combine all keys.
ALL_KEYS=$(printf '%s\n%s\n' "$HEX_KEYS" "$RGB_KEYS" | grep -v '^[[:space:]]*$' || true)

# Apply filtering (exempt canonical sites + snapshot allow-list).
FILTERED=$(filter_violations "$ALL_KEYS")

if [ -z "$FILTERED" ] || [ "$(printf '%s' "$FILTERED" | wc -c)" = "0" ]; then
    echo "$0: clean (no net-new hardcoded design tokens)"
    exit 0
fi

NET_NEW_COUNT=$(printf '%s\n' "$FILTERED" | grep -c '^crates\|^data' || true)

if [ "$NET_NEW_COUNT" = "0" ]; then
    echo "$0: clean (no net-new hardcoded design tokens)"
    exit 0
fi

echo "$0: net-new hardcoded design tokens:"
echo
printf '%s\n' "$FILTERED" | head -20
NET_NEW_TRUNC=$(printf '%s\n' "$FILTERED" | grep -c '^' || echo 0)
if [ "$NET_NEW_TRUNC" -gt 20 ]; then
    echo "  ... and $((NET_NEW_TRUNC - 20)) more"
fi
echo
echo "Each line above hardcodes a color or other design token in"
echo "active visual code. Per CLAUDE.md section 2.1, raw hex / RGB"
echo "may only live in the canonical token sources:"
echo
echo "  crates/shell/mde-ui/src/palette.rs   (ground-truth palette)"
echo "  data/css/tokens.css                  (CSS design tokens)"
echo
echo "If the violation is intentional (e.g., a Color::from_rgb call"
echo "inside the canonical palette), reference the token source"
echo "instead. To grandfather a pre-existing line, add its <file>:<line>"
echo "key to install-helpers/lint-design-tokens.allowlist with a dated"
echo "rationale comment. The allow-list is meant to SHRINK over time as"
echo "each entry ports to palette.rs / tokens.css."
exit 1
