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
sh install-helpers/install-hooks.sh   # idempotent; symlinks both, no git-config change
```

| Hook | Git event | What it does |
|---|---|---|
| `pre-commit` | `pre-commit` | Runs `install-helpers/run-lint-gates.sh` (the lint suite, gated on the staged file types) **then** `pre-commit-worklist.sh`. Either failure blocks the commit. |
| `pre-commit-worklist.sh` | (called by `pre-commit`) | WF-5.a — blocks a commit that adds a `docs/PROJECT_WORKLIST.md` task whose title lacks a release/workstream prefix (`v10.0.0:`, `E5:`, `MESHFS-3:`, …). No-op unless the worklist is staged. |
| `commit-msg.sh` | `commit-msg` | Runs `install-helpers/lint-visual-citation.sh` (+ any `lint-*-commitmsg.sh`) against the message file. Lenient/no-op pre-release; re-enables when `docs/design/` exists. |

The lint suite (E0.10) lives in `install-helpers/lint-*.sh` with snapshot
`*.allowlist` files capturing pre-existing violations, so the gates catch only
**net-new** issues. `run-lint-gates.sh` runs the relevant subset per commit (the
slow whole-repo `lint-runtime-reachability` only when a `lib.rs`/`mod.rs` is staged).
