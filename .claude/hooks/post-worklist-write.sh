#!/usr/bin/env bash
# WF-4 (ported 2026-06-03) — auto-detect new lock sections in the worklist.
#
# Hook input: the PostToolUse event JSON on stdin (.tool_input.file_path); falls back
# to the legacy $CLAUDE_FILE_PATH env var.
# Hook output: stderr-only reminder. Exit 0 always — never block.
# Wired in .claude/settings.json under hooks.PostToolUse (Edit|Write|MultiEdit).
#
# Pattern: any new section header containing "locked", "lock", "survey", or
# "design lock(s)" — case-insensitive. Compares git-diff against HEAD to find ADDED
# lines only (so existing locks don't re-fire on every edit). No-op unless the file
# written was docs/PROJECT_WORKLIST.md.

set -u

WORKLIST="docs/PROJECT_WORKLIST.md"

# Resolve the written file's path: stdin JSON first (current harness), then env.
stdin_json="$(cat 2>/dev/null || true)"
file_path=""
if [ -n "$stdin_json" ] && command -v jq >/dev/null 2>&1; then
    file_path="$(printf '%s' "$stdin_json" | jq -r '.tool_input.file_path // empty' 2>/dev/null)"
fi
[ -z "$file_path" ] && file_path="${CLAUDE_FILE_PATH:-}"

case "$file_path" in
    *"${WORKLIST}"|*"/${WORKLIST}") ;;
    *) exit 0 ;;
esac

if ! git -C "$(dirname "$file_path")" rev-parse --git-dir >/dev/null 2>&1; then
    exit 0
fi

cd "$(git -C "$(dirname "$file_path")" rev-parse --show-toplevel)" || exit 0

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
        echo "    ~/.claude/projects/-home-mm-MackesWorkstation/memory/"
        echo "  Pattern: project_<topic>.md + a one-line pointer in MEMORY.md."
        echo ""
    } >&2
fi

exit 0
