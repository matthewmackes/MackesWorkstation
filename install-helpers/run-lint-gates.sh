#!/bin/sh
# install-helpers/run-lint-gates.sh — pre-commit lint-gate runner (E0.10).
#
# Runs the install-helpers/lint-*.sh gates that are relevant to the staged
# changeset. Each gate scans the whole tree (allowlisting pre-existing
# violations), so it catches net-new violations anywhere; the staged-file
# triggers below just decide WHICH gates to run, so an unrelated commit
# doesn't pay for the slow whole-repo gates (notably runtime-reachability).
#
# Invoked by .claude/hooks/pre-commit. Exit 0 = all clean, 1 = a gate failed.
# The commit-msg gate (lint-visual-citation.sh) runs separately from
# .claude/hooks/commit-msg.sh — it needs the message file.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"
H="install-helpers"

staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
[ -n "$staged" ] || exit 0

has() { printf '%s\n' "$staged" | grep -qE "$1"; }

# Categories of staged change.
RS=0;  has '\.rs$'                 && RS=1
MODS=0; has '/(lib|mod)\.rs$'      && MODS=1
CSS=0; has 'data/css/.*\.css$'     && CSS=1
DESIGN=0; has '^docs/design/.*\.md$' && DESIGN=1

failed=""
run() {  # run <gate> only if it exists + is executable
    gate="$H/$1.sh"
    [ -x "$gate" ] || return 0
    if ! sh "$gate"; then failed="$failed $1"; fi
}

# Rust source touched → the Rust-policy gates.
if [ "$RS" -eq 1 ]; then
    run lint-no-stubs
    run lint-dbus-shape
    run lint-legacy-mesh
    run lint-public-ports
    run lint-voice
    run lint-material-symbols   # no-op in the monorepo (Carbon kept), kept for suite stability
fi

# Rust or CSS touched → the token gates.
if [ "$RS" -eq 1 ] || [ "$CSS" -eq 1 ]; then
    run lint-design-tokens
    run lint-motion-tokens
fi

# Module graph touched → the (slow) whole-repo reachability sweep.
[ "$MODS" -eq 1 ] && run lint-runtime-reachability

# CSS touched → the CSS hygiene gate.
[ "$CSS" -eq 1 ] && run lint-css

# A design doc touched → the design-doc↔worklist sync gate.
[ "$DESIGN" -eq 1 ] && run lint-design-doc-sync

if [ -n "$failed" ]; then
    echo ""
    echo "✗ pre-commit lint gates FAILED:${failed}"
    echo "  Fix the reported violations, or add a pre-existing path to that gate's"
    echo "  allowlist with a one-line rationale. Re-stage and commit again."
    exit 1
fi
exit 0
