#!/bin/sh
# install-helpers/lint-visual-citation.sh — commit-msg gate (lenient no-op).
#
# Invocation contract (PRESERVED):
#   - As a commit-msg hook: $1 is the path to the commit-message file.
#     Git invokes the hook this way; this script keeps that contract so
#     it can stay wired into .git/hooks/commit-msg unchanged.
#
# This gate is intentionally a LENIENT NO-OP in the MackesWorkstation
# monorepo. It accepts every commit message (exits 0).
#
# RATIONALE (recorded so the suite count + provenance stay stable):
# The upstream MDE gate required every commit that touches a visual
# surface to cite a docs/design/<spec>.md section (plus a named visual
# reference target) via a "Cite: <doc>.md §X.Y; ref: <target>" line in
# the commit body. That made sense once a design-doc corpus existed.
#
# In the monorepo two preconditions for a hard citation requirement do
# not yet hold:
#
#   1. docs/design/ does NOT exist yet — there is no design-doc corpus
#      to cite, so the requirement has nothing to point at.
#   2. The monorepo is pre-release. Visual work lands direct-to-main
#      with screenshots under the worklist process lock (no PR-branch
#      visual-review lane while review cannot run). A hard per-commit
#      citation requirement is therefore premature.
#
# So this gate is a no-op for now. RE-ENABLE the real citation check
# (port the upstream visual-surface detection + "Cite:" parsing) once
# docs/design/ exists and the platform is post-release.
#
# The commit-msg $1 contract is preserved so re-enabling later is a
# drop-in body swap with no hook-wiring change.
#
# See CLAUDE.md section 2 conventions and section 3 Definition of Done.
#
# Exits 0 always.

set -eu

# Accept the commit-message file path on $1 (commit-msg hook contract),
# but do not enforce anything against it yet. Touch it read-only so the
# arg is consumed and the contract is visibly honored.
MSG_FILE="${1:-}"
if [ -n "$MSG_FILE" ] && [ -f "$MSG_FILE" ]; then
    : # commit-message file present; citation check deferred (see header)
fi

echo "$0: lenient no-op — docs/design/ does not exist yet and visual work lands direct-to-main pre-release. Citation gate deferred; re-enable post-release. (exit 0)"
exit 0
