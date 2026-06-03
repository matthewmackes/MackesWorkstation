#!/bin/sh
# install-helpers/lint-voice.sh — voice-and-tone verb-discipline +
# forbidden-strings pre-commit gate.
#
# Adapted from the MDE pre-commit gate of the same name. Enforces:
#
#   1. Forbidden marketing/celebratory strings: Oops, Whoops, Yikes,
#      Lorem ipsum, etc.
#   2. Forbidden placeholder strings reachable from user-visible
#      code: foo/bar/baz/qux, test123, placeholder.
#   3. Aspirational "coming soon" / "TBD" / "WIP" labels leaked into
#      user-visible copy.
#   4. Verb-discipline misuse in user-visible button-label-shaped
#      strings: e.g. "Create" / "New" where "Add" is the lock,
#      "Save" where "Apply" is the lock for config changes.
#
# Scans (the surfaces that exist in this monorepo):
#
#   - crates/**/src/             (Iced views, panel labels)
#   - data/applications/*.desktop (launcher Name= / Comment=)
#
# Python is RETIRED (it survives only under provenance/, which is
# NOT scanned), so this gate looks at Rust + .desktop only — there
# is no rust/ or mackes/ tree.
#
# The verb-discipline check is intentionally conservative — it
# targets clear button-label shapes (UPPER first letter, <= 3 words,
# ends with no punctuation) to avoid false positives in narrative
# prose / comments / log strings.
#
# NOTE (monorepo policy reversal): the upstream MDE gate also forbade
# Carbon-branded iconography vocab ("Carbon icon" / "carbon-<name>"),
# because MDE was migrating Carbon -> Material Symbols. The monorepo
# REVERSED that decision — Carbon is KEPT as the default dark theme
# (one of four: Win2000 / Carbon / Win10 / BeOS) and Material Symbols
# was dropped. That FORBIDDEN-CARBON-VOCAB rule is therefore INVERTED
# for this tree and is deliberately NOT ported.
#
# Three allowlist layers, all narrowing the gate to NET-NEW hits:
#   1. install-helpers/lint-voice.allowlist — a <file>:<line> snapshot
#      of the PRE-EXISTING violations in the merged tree (same
#      mechanism as lint-design-tokens.allowlist). Meant to SHRINK.
#   2. ALLOWLIST_PREFIXES (inline below) — path prefixes exempt from
#      ALL checks (e.g. the wholesale legacy crate tree).
#   3. Per-line `voice-allow` annotation — append it to a source line
#      to silence the rare compliant case.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of
# Done) for how this gate fits the monorepo's voice-and-tone
# direction.
#
# Exit 0 = clean, exit 1 = violations found.
# Run as: install-helpers/lint-voice.sh [path...]

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ $# -gt 0 ]; then
    SCAN_PATHS="$*"
    ACTIVE_PATHS="$*"
else
    # All scanned surfaces that exist in this tree. Rust lives under
    # crates/**/src; .desktop launchers live under data/applications.
    # provenance/ (archived MDE/Python source) is never scanned.
    SCAN_PATHS="crates data/applications"
    # Verb-discipline scans run against the same active surfaces.
    ACTIVE_PATHS="crates data/applications"
fi

# ──────────────────────────────────────────────────────────────
# Path-prefix allowlist for PRE-EXISTING violations.
#
# Lines whose file path starts with ANY of these prefixes are
# exempt from ALL checks. Snapshot taken 2026-06-03 of the merged
# tree so the gate exits 0 and catches only NET-NEW violations.
# REMOVE each entry as its file is cleaned up.
#
# `crates/legacy/` — the wholesale legacy crate tree (drawer, kdc,
# portal, virtual, kdc-proto); retires under its own epic, so its
# label vocabulary predates the voice lock.
# ──────────────────────────────────────────────────────────────
ALLOWLIST_PREFIXES='
crates/legacy/
'

# Snapshot allow-list file (file:line keys for pre-existing hits).
ALLOWLIST_FILE="${REPO_ROOT}/install-helpers/lint-voice.allowlist"
ALLOW_KEYS=""
if [ -f "$ALLOWLIST_FILE" ]; then
    ALLOW_KEYS=$(grep -v '^[[:space:]]*#' "$ALLOWLIST_FILE" \
        | grep -v '^[[:space:]]*$' || true)
fi

# Filter pattern for source lines: exclude comments + tests.
# Iced strings are typically inside text("..."), button(text("...")),
# .placeholder("...") — we grep across all and let the verb
# pattern + word-boundary do the filtering.

EXIT_CODE=0
TOTAL_VIOLATIONS=0
TMPFILE=$(mktemp)
FILTERED=$(mktemp)
trap 'rm -f "$TMPFILE" "$FILTERED"' EXIT

# Drop lines whose file path is under an allowlisted prefix.
# Reads grep "file:line:body" lines on stdin, writes survivors out.
apply_path_allowlist() {
    while IFS= read -r raw; do
        [ -n "$raw" ] || continue
        fname=$(printf '%s' "$raw" | cut -d: -f1)
        allowed=0
        while IFS= read -r prefix; do
            [ -n "$prefix" ] || continue
            case "$fname" in
                "$prefix"*) allowed=1; break ;;
            esac
        done <<EOF
$ALLOWLIST_PREFIXES
EOF
        [ "$allowed" = 1 ] && continue
        printf '%s\n' "$raw"
    done
}

scan() {
    label="$1"
    pattern="$2"
    description="$3"
    args="$4"
    # 5th positional arg: which path set to scan. "active" for
    # verb-discipline checks, "all" (default) for forbidden-strings.
    path_set="${5:-all}"
    case "$path_set" in
        active) paths="$ACTIVE_PATHS" ;;
        *)      paths="$SCAN_PATHS" ;;
    esac
    > "$TMPFILE"
    # shellcheck disable=SC2086
    grep -rn -E "$pattern" $args $paths 2>/dev/null \
        | grep -v 'voice-allow' \
        | apply_path_allowlist \
        > "$TMPFILE" || true
    # Strip snapshot-allow-listed lines by <file>:<line> key.
    if [ -n "$ALLOW_KEYS" ] && [ -s "$TMPFILE" ]; then
        # Reduce hits to their file:line key, drop any that match a
        # snapshot key, keep the survivors' full lines.
        : > "$FILTERED"
        while IFS= read -r raw; do
            [ -n "$raw" ] || continue
            key=$(printf '%s' "$raw" | sed -E 's/^([^:]+:[0-9]+):.*/\1/')
            if printf '%s\n' "$ALLOW_KEYS" | grep -qxF "$key"; then
                continue
            fi
            printf '%s\n' "$raw" >> "$FILTERED"
        done < "$TMPFILE"
        cp "$FILTERED" "$TMPFILE"
    fi
    if [ -s "$TMPFILE" ]; then
        count=$(wc -l < "$TMPFILE")
        TOTAL_VIOLATIONS=$((TOTAL_VIOLATIONS + count))
        printf '\n[%s] %d hit(s) — %s\n' "$label" "$count" "$description"
        head -10 "$TMPFILE"
        if [ "$count" -gt 10 ]; then
            printf '  ... %d more hits suppressed\n' $((count - 10))
        fi
        EXIT_CODE=1
    fi
}

# ──────────────────────────────────────────────────────────────
# Forbidden marketing / celebratory strings
# ──────────────────────────────────────────────────────────────

scan FORBIDDEN-MARKETING \
    '\b(Oops|Whoops|Yikes)\b' \
    'celebratory/apologetic words banned in user strings' \
    '--include=*.rs --include=*.desktop'

scan FORBIDDEN-LOREM \
    '\b(Lorem ipsum|dolor sit amet)\b' \
    'lorem ipsum placeholder reached production' \
    '--include=*.rs --include=*.desktop'

scan FORBIDDEN-FOO \
    '"(foo|bar|baz|qux)"' \
    'metasyntactic variables as visible strings (use real names)' \
    '--include=*.rs'

scan FORBIDDEN-TEST \
    '"(test123|testing123|placeholder)"' \
    'placeholder/test default values shipping in production' \
    '--include=*.rs --include=*.desktop'

# coming-soon discipline. User-visible strings must not advertise
# aspirational state ("coming soon", "TBD", "WIP", etc.). Pairs with
# the code-side lint-no-stubs.sh. The pattern below requires the
# forbidden word to be the WHOLE quoted string so technical
# sentences don't false-positive. -i so case variants are caught.
scan FORBIDDEN-COMING-SOON \
    '"(coming soon|TBD|tbd|WIP|work in progress|not yet implemented|soon™|early access)"' \
    'aspirational "coming soon" / "TBD" / "WIP" labels leaked into user-visible copy' \
    '-i --include=*.rs --include=*.desktop'

# Aspirational labels embedded INSIDE quoted user strings — e.g.
# "Networks (coming soon)" / "Voice (beta)" / "Files [WIP]". The
# parenthetical / bracketed form is the operator-facing
# aspirational-label idiom; bare technical mentions of beta/alpha/
# etc. inside long descriptive strings are not caught.
scan FORBIDDEN-LABEL-SUFFIX \
    '"[^"]*[\[\(](coming soon|TBD|WIP|work in progress|early access|alpha|beta|preview|experimental)[\]\)][^"]*"' \
    'aspirational label suffix leaked into user-visible copy' \
    '-i --include=*.rs --include=*.desktop'

# ──────────────────────────────────────────────────────────────
# Verb discipline
# Targets clear button-label-shape strings only: capitalized
# first letter inside double-quotes, ends with quote (no
# punctuation), <= 3 words. Logs / errors / multi-sentence prose
# fall outside the shape and are not matched.
# ──────────────────────────────────────────────────────────────

# "Create" / "New" → use "Add"
scan VERB-CREATE-VS-ADD \
    '"\b(Create|New)\b( \w+){0,2}"' \
    'use "Add ..." not "Create/New ..." (verb discipline)' \
    '--include=*.rs --include=*.desktop' \
    active

# "Delete X" where the action is removal-from-set, not destruction.
# Flag bare "Delete" in button-shape strings so the author can choose
# Remove or keep Delete.
scan VERB-DELETE-VS-REMOVE \
    '"\bDelete\b( \w+){0,2}"' \
    'consider "Remove ..." for set-removal; "Delete ..." reserved for destroy (verb discipline)' \
    '--include=*.rs --include=*.desktop' \
    active

# "Save" / "Confirm" → use "Apply" for config changes
scan VERB-SAVE-VS-APPLY \
    '"\b(Save|Confirm)\b( \w+){0,2}"' \
    'use "Apply ..." not "Save/Confirm ..." for config changes (verb discipline)' \
    '--include=*.rs --include=*.desktop' \
    active

# "Abort" → use "Cancel"
scan VERB-STOP-VS-CANCEL \
    '"\b(Abort)\b( \w+){0,2}"' \
    'use "Cancel ..." not "Abort ..." (verb discipline)' \
    '--include=*.rs --include=*.desktop' \
    active

# "Execute" / "Trigger" → use "Run"
scan VERB-EXECUTE-VS-RUN \
    '"\b(Execute|Trigger)\b( \w+){0,2}"' \
    'use "Run ..." not "Execute/Trigger ..." (verb discipline)' \
    '--include=*.rs --include=*.desktop' \
    active

# ──────────────────────────────────────────────────────────────
# Summary
# ──────────────────────────────────────────────────────────────

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "lint-voice.sh: clean ($SCAN_PATHS)"
else
    printf '\nlint-voice.sh: %d total violation(s) across the scanned paths.\n' "$TOTAL_VIOLATIONS" >&2
    printf 'See CLAUDE.md section 2 conventions for the verb table + forbidden strings.\n' >&2
fi

exit "$EXIT_CODE"
