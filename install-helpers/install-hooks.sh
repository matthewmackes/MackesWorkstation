#!/bin/sh
# install-helpers/install-hooks.sh (E0.10) — symlink the repo's git hooks into
# .git/hooks. Idempotent; does NOT touch git config (per CLAUDE.md §0). Run once
# per clone:  sh install-helpers/install-hooks.sh
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

hooks_dir=".git/hooks"
[ -d "$hooks_dir" ] || { echo "no $hooks_dir — not a git checkout?"; exit 1; }

link() {  # link <target-relative-to-.git/hooks> <hook-name>
    ln -sf "$1" "$hooks_dir/$2"
    echo "  linked $hooks_dir/$2 -> $1"
}

# .git/hooks/<name> -> ../../.claude/hooks/<file>
link "../../.claude/hooks/pre-commit"     "pre-commit"
link "../../.claude/hooks/commit-msg.sh"  "commit-msg"

echo "git hooks installed: pre-commit (lint suite + worklist), commit-msg (visual-citation)."
