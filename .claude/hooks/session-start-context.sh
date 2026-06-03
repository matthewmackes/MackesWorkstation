#!/usr/bin/env bash
# SessionStart context injector (ported from MDE + MDE-Retro, 2026-06-03).
#
# Surfaces, for every fresh/resumed/cleared session: the load-bearing facts the
# prose READMEs still get wrong, the governance compass (root CLAUDE.md + §0 master
# rule), the auto-memory index, the worklist count (once it exists), and the last 3
# commits. Output is plain text on stdout — Claude Code injects a SessionStart hook's
# stdout as session context, so no jq/python dependency is needed.
#
# Wired in .claude/settings.json under hooks.SessionStart (startup|resume|clear — NOT
# compact, which already carries context forward). Degrades gracefully: every section
# is optional, and the hook always exits 0 (a non-zero SessionStart hook surfaces an
# error to the operator on every launch).

set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." 2>/dev/null && pwd)" || exit 0
[ -n "$repo_root" ] && cd "$repo_root" || exit 0

gov="docs/AI_GOVERNANCE.md"
worklist="docs/PROJECT_WORKLIST.md"
rulebook="CLAUDE.md"
mem="${HOME}/.claude/projects/-home-mm-MackesWorkstation/memory/MEMORY.md"

echo "# MackesWorkstation — orientation (auto-injected by session-start-context.sh)"
echo
echo "Read ${rulebook} (root). Load-bearing facts the prose lags on:"
echo "  • ONE cargo workspace at the repo root; sources under crates/{platform,mesh,shell,"
echo "    workbench,services,shared,applets,kdc,legacy}/. No rust/ dir, no Python mackes/ tree."
echo "  • Shell = one multiplexed 'mde <subcommand>' binary (crates/shell/mde); look lib ="
echo "    crates/shell/mde-ui. Compositor is labwc. Storage is LizardFS (NOT Gluster)."
echo "  • Four themes via the single palette::color() edge — Win2000 / Carbon (default dark) /"
echo "    Win10 / BeOS. No raw hex outside crates/shell/mde-ui/src/palette.rs."
echo "  • Version is v10.0.0; single remote 'origin'; commit + push are SEPARATE asks."
echo "  • A green 'cargo test' does NOT verify a render — use the preview/accuracy harness."

if [ -f "$gov" ]; then
    master="$(grep -m1 -E '^> \*\*"Secure, Simple, No-Fixed-Center' "$gov" 2>/dev/null \
        || grep -m1 -E '^> \*\*"Secure, Simple, Centerless' "$gov" 2>/dev/null || true)"
    echo
    echo "## Governance — ${gov}"
    [ -n "$master" ] && echo "  • §0 master rule: ${master#> }"
    echo "  • Roadmap is the E0–E8 epic sequence (docs/MACKES-WORKSTATION-PLAN.md §11); E0 done."
    echo "    The RPM (E8) is held until every feature is §3-complete; HW bench is post-release."
    echo "  • Authority on conflict (newest wins): Memory > CLAUDE.md > AI_GOVERNANCE.md >"
    echo "    MACKES-WORKSTATION-PLAN.md / design docs > PROJECT_WORKLIST.md."
    echo "  • Live skills: plan · ship · release · audit · preview (.claude/skills/)."
fi

if [ -f "$mem" ]; then
    echo
    echo "## Memory index — ${mem}"
    while IFS= read -r line; do [ -n "$line" ] && echo "  $line"; done < "$mem"
fi

if [ -f "$worklist" ]; then
    open="$(grep -cE '^\s*- \[[ >]\]' "$worklist" 2>/dev/null || echo 0)"
    echo
    echo "## Worklist — ${worklist}: ${open} open/in-progress item(s)."
fi

if git rev-parse --git-dir >/dev/null 2>&1; then
    log="$(git log -3 --oneline 2>/dev/null || true)"
    if [ -n "$log" ]; then
        echo
        echo "## Last 3 commits"
        while IFS= read -r line; do echo "  - $line"; done <<< "$log"
    fi
fi

exit 0
