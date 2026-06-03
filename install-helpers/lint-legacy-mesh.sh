#!/bin/sh
# install-helpers/lint-legacy-mesh.sh — pre-commit lint gate.
#
# Catches NET-NEW `tailscale` / `headscale` / `derper` references
# in Nebula-native Rust source. The legacy tree
# (`crates/legacy/**`) and the provenance archive (`provenance/**`)
# are wholesale allow-listed because those files still reference
# the legacy stack for backward-compat or as deletion targets.
# Specific pre-existing files in the live mesh crates that still
# carry the legacy vocabulary (deletion / migration targets) are
# snapshot-allow-listed by path prefix; the allow-list shrinks as
# each one retires. Retraction-comment lines and pure comment
# lines are also allow-listed. Going forward, any
# tailscale/headscale/derper hit OUTSIDE the allow-list is a
# regression — net-new code shouldn't reach for the dead stack.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of
# Done) for how this gate fits the monorepo's Nebula-native mesh
# direction.
#
# Exits 0 = clean, exits 1 = violations found.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Where to look. Rust source only — Python is RETIRED (it survives
# only under provenance/, which is allow-listed below), and docs
# are not scanned.
SCAN_INCLUDES='--include=*.rs'
SCAN_PATHS='crates/'

# Directory + file-prefix allow-list. Lines whose file path
# starts with ANY of these are allowed to mention the legacy
# vocabulary. Each entry is a prefix (matched via `case`-style
# pattern); add new entries one per line.
#
# `crates/legacy/**` — the wholesale legacy tree (drawer, kdc,
# portal, virtual, kdc-proto); retires under its own epic, so its
# tailscale references are pre-existing, not net-new.
#
# `provenance/**` — the archived MDE/Python source kept for
# provenance; never scanned for live policy.
#
# The remaining entries are PRE-EXISTING deletion / migration
# targets in the live mesh crates that still carry the legacy
# vocabulary in non-comment code. Snapshot taken 2026-06-03 of
# the merged tree; REMOVE each entry as its file retires so the
# gate catches any regression. Everything outside this list is
# net-new.
ALLOWLIST_PREFIXES='
crates/legacy/
provenance/
crates/mesh/mackesd/src/legacy_inventory.rs
crates/mesh/mackesd/src/transport/https443.rs
crates/mesh/mackesd/tests/
crates/workbench/mde-workbench/src/panels/mesh_services.rs
'

LEGACY_PATTERN='\btailscale\|\bheadscale\|\bderper'

TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

# shellcheck disable=SC2086
grep -rIn -i ${SCAN_INCLUDES} -e "$LEGACY_PATTERN" \
    $SCAN_PATHS 2>/dev/null > "$TMPFILE" || true

VIOLATIONS=0

while IFS= read -r raw; do
    [ -n "$raw" ] || continue
    fname=$(printf '%s' "$raw" | cut -d: -f1)
    body=$(printf '%s' "$raw" | cut -d: -f3-)

    # Allow lines that include a retraction-comment tag — these
    # are intentional "we retired X" notes left behind in
    # otherwise-Nebula-native files. Matches the legacy MDE epic
    # tags (NF-N.M, GF-N.M, RD-N, KDC2-N) plus the monorepo's E0
    # epic tags, or any of the verbs "retired", "retire",
    # "superseded", "deprecat", "legacy" appearing in the line.
    if printf '%s' "$body" | grep -qE '(NF-[0-9]+(\.[0-9]+)?|GF-[0-9]+(\.[0-9]+)?|RD-[0-9]+|KDC2-[0-9]+|E[0-9]+(\.[0-9]+)?|\blegacy\b|\bretired\b|\bretire\b|\bsuperseded\b|\bdeprecat)'; then
        continue
    fi

    # Allow lines that are entirely inside a // or # comment.
    trimmed=$(printf '%s' "$body" | sed -e 's/^[[:space:]]*//')
    case "$trimmed" in
        '#'*|'//'*|'///'*|'//!'*|'/*'*|'*'*) continue ;;
    esac

    # Walk the allow-list. If the file path starts with any
    # prefix, allow.
    allowed=0
    while IFS= read -r prefix; do
        [ -n "$prefix" ] || continue
        case "$fname" in
            "$prefix"*) allowed=1; break ;;
        esac
    done <<EOF
$ALLOWLIST_PREFIXES
EOF
    if [ "$allowed" = 1 ]; then
        continue
    fi

    printf '%s\n' "$raw"
    VIOLATIONS=$((VIOLATIONS + 1))
done < "$TMPFILE"

if [ "$VIOLATIONS" -eq 0 ]; then
    printf 'lint-legacy-mesh.sh: clean (no net-new tailscale/headscale/derper hits in Nebula-native source)\n'
    exit 0
fi

printf '\nlint-legacy-mesh.sh: %d violation(s) — net-new legacy mesh vocabulary in Nebula-native source.\n' \
    "$VIOLATIONS" >&2
printf 'See CLAUDE.md section 2 conventions + the allow-list inside this script.\n' >&2
exit 1
