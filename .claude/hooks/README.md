# `.claude/hooks/` — wiring

Two classes of hook live here. They are wired differently.

## Harness-wired (active automatically via `.claude/settings.json`)

These run inside the Claude Code session; no installation step.

| Hook | Event | What it does |
|---|---|---|
| `session-start-context.sh` | `SessionStart` (startup/resume/clear) | Injects orientation: load-bearing facts, the `AI_GOVERNANCE.md` §0 master rule, the memory index, the worklist count, and the last 3 commits. Plain-text stdout, dependency-free, always exits 0. |
| `post-worklist-write.sh` | `PostToolUse` (Edit/Write/MultiEdit) | No-op unless `docs/PROJECT_WORKLIST.md` was the file written; then reminds (stderr) to capture any new lock in memory. Never blocks. |

A small inline `PostToolUse` command in `settings.json` also runs `rustfmt` on any
`.rs` file written (best-effort, `|| true`).

## Git-wired (need a one-time symlink into `.git/hooks/`)

These are **not** active until you link them. They never modify `git config`.

```sh
ln -sf ../../.claude/hooks/pre-commit-worklist.sh .git/hooks/pre-commit
ln -sf ../../.claude/hooks/commit-msg.sh          .git/hooks/commit-msg
```

| Hook | Git event | What it does |
|---|---|---|
| `pre-commit-worklist.sh` | `pre-commit` | WF-5.a — blocks a commit that adds a `docs/PROJECT_WORKLIST.md` task whose title lacks a release/workstream prefix (`v10.0.0:`, `E5:`, `MESHFS-3:`, …). No-op unless the worklist is staged. |
| `commit-msg.sh` | `commit-msg` | Extension point for commit-message lints (`install-helpers/lint-*-commitmsg.sh`). A clean no-op until linters are ported. |

These are deliberately left un-installed: the worklist doesn't exist yet, and the
`install-helpers/` lint tree is E1/E8 port work. Symlink them once that lands.
