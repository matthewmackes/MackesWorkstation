#!/usr/bin/env bash
# WF-4 (2026-05-21) — auto-detect new lock sections in the worklist.
#
# Hook input: $CLAUDE_FILE_PATH (the file just written/edited).
# Hook output: stderr-only reminder. Exit 0 always — never block.
#
# Wired in .claude/settings.json under hooks.PostToolUse.
#
# Pattern: any new section header containing "locked", "lock",
# "survey", or "design lock(s)" — case-insensitive. The hook compares
# git-diff against HEAD to find ADDED lines only (so existing locks
# don't re-fire on every edit).

set -u

WORKLIST="docs/PROJECT_WORKLIST.md"

# Only act if the file written was the worklist
case "${CLAUDE_FILE_PATH:-}" in
    *"${WORKLIST}"|*"/${WORKLIST}")
        ;;
    *)
        exit 0
        ;;
esac

# Only act in a git repo
if ! git -C "$(dirname "${CLAUDE_FILE_PATH}")" rev-parse --git-dir >/dev/null 2>&1; then
    exit 0
fi

# Find ADDED lines in the worklist diff that look like a lock header
cd "$(git -C "$(dirname "${CLAUDE_FILE_PATH}")" rev-parse --show-toplevel)"

new_locks="$(git diff -- "${WORKLIST}" 2>/dev/null \
    | grep -E '^\+' \
    | grep -v '^\+\+\+' \
    | grep -iE '^\+.*(locked|lock|survey|design.lock)' \
    | grep -iE '^\+(##|###|####|\*\*)' \
    || true)"

if [ -n "${new_locks}" ]; then
    {
        echo ""
        echo "⚠ post-worklist-write hook (WF-4): new lock-pattern header(s) detected:"
        echo "${new_locks}" | sed 's/^/    /'
        echo ""
        echo "→ Consider surfacing this in memory:"
        echo "    ~/.claude/projects/-home-mm-Desktop-files-mackes-shell/memory/"
        echo "  Pattern: project_<topic>.md + entry in MEMORY.md."
        echo ""
    } >&2
fi

exit 0
