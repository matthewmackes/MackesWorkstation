#!/usr/bin/env bash
# WF-5.a (ported 2026-06-03) — validate worklist task titles have a release/workstream
# prefix. GIT pre-commit hook — NOT harness-wired. Install by symlinking into
# .git/hooks/pre-commit (see .claude/hooks/README.md). Never modifies git config.
#
# Scans the STAGED diff of docs/PROJECT_WORKLIST.md for ADDED active-task entries
# (+- [ ] / +- [>] / +- [!]) and requires the title to start with a recognized prefix:
#
#   - Semver version:     v\d+\.\d+(\.\d+)?:    (e.g. v10.0.0:, v10.1:)
#   - Epic / workstream:  [A-Z][A-Za-z0-9.-]*:  (e.g. E3:, BUS-4.2:, MESHFS-1:, WF-5.a:)
#
# Pre-existing tasks are NOT audited — only what's added in this commit. Done tasks
# (+- [✓]) are skipped. Exit 0: ok / no worklist edits. Exit 1: violation, commit blocked.

set -u

WORKLIST="docs/PROJECT_WORKLIST.md"
PREFIX_RE='^([A-Z][A-Za-z0-9.-]*|v[0-9]+\.[0-9]+(\.[0-9]+)?):'

if ! git diff --cached --name-only | grep -qx "${WORKLIST}"; then
    exit 0
fi

added_tasks="$(git diff --cached -- "${WORKLIST}" \
    | grep -E '^\+- \[[ >!]\] \*\*' \
    || true)"

if [ -z "${added_tasks}" ]; then
    exit 0
fi

violations=""
while IFS= read -r line; do
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
        echo "    - [ ] **v10.0.1: hotfix something — short summary** …"
        echo "    - [ ] **E5: new app task — short summary** …"
        echo "    - [ ] **MESHFS-3: chunk-server failover** …"
        echo ""
        echo "  Per root CLAUDE.md §5 (worklist). Fix the title and re-stage."
        echo ""
    } >&2
    exit 1
fi

exit 0
