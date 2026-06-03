#!/usr/bin/env bash
# commit-msg hook (ported 2026-06-03) — GIT hook, NOT harness-wired. Install by
# symlinking into .git/hooks/commit-msg (see .claude/hooks/README.md).
#
# Runs commit-msg-class lints AFTER the message is written + BEFORE the commit lands.
# Each lint receives the commit-message file path as $1 (the standard commit-msg
# contract). This is an EXTENSION POINT: drop executable lints into install-helpers/
# (e.g. a visual-citation lint) and they run here. With no lints present it is a
# clean no-op, so it is safe to symlink before the linters are ported.

set -u

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MSG_FILE="$1"

# Run every executable lint in install-helpers/ that opts into the commit-msg contract
# by being named lint-*-commitmsg.sh. None exist yet — this is the port hook.
if [ -d "${REPO_ROOT}/install-helpers" ]; then
    for lint in "${REPO_ROOT}"/install-helpers/lint-*-commitmsg.sh; do
        [ -x "$lint" ] || continue
        if ! "$lint" "$MSG_FILE"; then
            exit 1
        fi
    done
fi

exit 0
