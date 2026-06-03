#!/bin/sh
# install-helpers/lint-no-stubs.sh — pre-commit gate (no-stubs).
#
# Catches CLAUDE.md section 3 "Definition of Done" / no-stubs
# violations in committed Rust code:
#
#   - todo!()
#   - unimplemented!()
#   - panic!("not yet …")
#   - panic!("todo …")
#   - panic!("not implemented …")
#
# Every commit must ship END-TO-END (CLAUDE.md section 3). A stub
# left in committed code is a regression this gate blocks.
#
# Exits 0 = clean, exits 1 = violations found.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Scan Rust source only. Python is retired (lives only under
# provenance/); no need to catch stub violations in dead code.
SCAN_INCLUDES='--include=*.rs'
SCAN_PATHS='crates/'

# Patterns that signal a stub. egrep-compatible.
PATTERNS='(\<todo!\s*\()|(\<unimplemented!\s*\()|(panic!\s*\(\s*"\s*not yet)|(panic!\s*\(\s*"\s*todo)|(panic!\s*\(\s*"\s*not implemented)'

# Snapshot allow-list. Captures pre-existing hits in the merged tree
# at lint introduction (see dated entries below). Going forward, any
# match outside the allow-list is a regression the gate must catch.
#
# Format: one path per line, leading + trailing whitespace ignored.
# Lines starting with `#` are comments.
ALLOWLIST_PREFIXES='
'

TMPFILE="$(mktemp)"
trap 'rm -f "$TMPFILE"' EXIT

# `grep -rnE` for portable POSIX regex (no PCRE dep).
grep -rnE $SCAN_INCLUDES "$PATTERNS" $SCAN_PATHS > "$TMPFILE" 2>/dev/null || true

# Filter out allow-listed paths.
echo "$ALLOWLIST_PREFIXES" | while IFS= read -r prefix; do
    prefix="$(printf '%s' "$prefix" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    case "$prefix" in
        ''|'#'*) continue ;;
    esac
    # Escape for sed pattern.
    esc="$(printf '%s' "$prefix" | sed 's/[\/&]/\\&/g')"
    sed -i.bak "/^${esc}/d" "$TMPFILE"
    rm -f "$TMPFILE.bak"
done

# Filter out lines inside #[cfg(test)] modules (tests can use
# unimplemented!() / todo!() as placeholders for test fixtures).
# This is approximate (sed line-based) but catches the common
# case: lines inside files whose path contains `/tests/`.
sed -i.bak '/\/tests\//d' "$TMPFILE"
rm -f "$TMPFILE.bak"

# Filter out lines inside test modules (files ending in
# `*_tests.rs` or paths under any `tests/` subdirectory).
sed -i.bak '/_tests\.rs:/d' "$TMPFILE"
rm -f "$TMPFILE.bak"

if [ -s "$TMPFILE" ]; then
    echo "lint-no-stubs.sh: no-stubs violations found:"
    echo
    cat "$TMPFILE"
    echo
    echo "Each line above ships a stub (todo!() / unimplemented!() /"
    echo "panic-not-yet). Per CLAUDE.md section 3 every commit must ship"
    echo "END-TO-END. Either complete the implementation, split the"
    echo "task at write-time so each sub-commit is complete, or add"
    echo "the path to the snapshot allow-list above with a one-line"
    echo "rationale comment."
    exit 1
fi

echo "lint-no-stubs.sh: clean (no net-new todo!() / unimplemented!() / panic-not-yet hits)"
exit 0
