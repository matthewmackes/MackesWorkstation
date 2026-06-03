#!/usr/bin/env bash
# WF-5.a (2026-05-21) — validate worklist task titles have a release/workstream prefix.
#
# Install: `make install-hooks` (creates a symlink at
# .git/hooks/pre-commit). Never modifies git config (per
# .claude/CLAUDE.md §0.3).
#
# This hook scans the STAGED diff of docs/PROJECT_WORKLIST.md for
# ADDED task entries (lines starting with `+- [ ]` / `+- [>]` / `+- [!]`)
# and validates that the task title starts with a recognized prefix:
#
#   - Semver version:     v\d+\.\d+(\.\d+)?:    (e.g. v2.0.1:, v2.1:)
#   - Workstream + ID:    [A-Z][A-Za-z0-9.-]*:  (e.g. UX-14:, CB-1.5.a:, WF-5.a:, NFU-2:)
#
# Pre-existing tasks in the worklist are NOT audited — only what's
# being added in this commit. Done tasks (`+- [✓]`) are also skipped
# since they're presumed to have shipped under the prior schema.
#
# Exit 0: ok (no violations or no worklist edits in this commit).
# Exit 1: violation — commit blocked. Output names the offending
#         line(s) and shows the regex.

set -u

WORKLIST="docs/PROJECT_WORKLIST.md"
PREFIX_RE='^([A-Z][A-Za-z0-9.-]*|v[0-9]+\.[0-9]+(\.[0-9]+)?):'

# Only act if the worklist is in the staged changeset
if ! git diff --cached --name-only | grep -qx "${WORKLIST}"; then
    exit 0
fi

# Get the +added active-task lines (open / in-progress / blocked)
added_tasks="$(git diff --cached -- "${WORKLIST}" \
    | grep -E '^\+- \[[ >!]\] \*\*' \
    || true)"

if [ -z "${added_tasks}" ]; then
    exit 0
fi

violations=""
while IFS= read -r line; do
    # Strip leading `+- [ ] **` to get the title fragment
    title="$(echo "${line}" | sed -E 's/^\+- \[[ >!]\] \*\*//')"
    if ! echo "${title}" | grep -qE "${PREFIX_RE}"; then
        violations="${violations}    ${title%%\*\**}"$'\n'
    fi
done <<< "${added_tasks}"

if [ -n "${violations}" ]; then
    {
        echo ""
        echo "✗ pre-commit-worklist (WF-5.a): tasks missing release/workstream prefix:"
        echo "${violations}"
        echo "  Required prefix: ${PREFIX_RE}"
        echo "  Examples:"
        echo "    - [ ] **v2.1: hotfix something — short summary** …"
        echo "    - [ ] **UX-30: new design polish task** …"
        echo "    - [ ] **WF-6.a: pre-commit hook follow-up** …"
        echo ""
        echo "  Per .claude/CLAUDE.md §1.1. Fix the title and re-stage."
        echo ""
    } >&2
    exit 1
fi

exit 0
