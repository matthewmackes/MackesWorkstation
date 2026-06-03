---
name: plan
description: Design-thinking + survey + worklist-management skill for the MackesWorkstation monorepo. Use when the user asks to scope an E0–E8 epic, run an N-Q AskUserQuestion survey, audit the worklist for drift, rescue dead crate-modules, lift design-doc actions into worklist tasks, draft a design document, or otherwise PLAN work before any code lands. Absorbs the prior `mackes-worklist-management` + the design-thinking / rescue-pass portions of `iteration`. Sister skills: `ship` (drains the queue), `release` (the operator-gated RPM cut), plus `audit` + `preview`.
---

# Plan

The design + worklist-management skill. **Everything before code
lands runs through plan.** Surveys design forks via
`AskUserQuestion`, lifts design-doc actions into worklist tasks,
audits the worklist for drift, drafts new design documents.

Consolidates the planning portions of the retired
`mackes-worklist-management` + `iteration` (rescue-pass) skills.

## Triggers

- "Design [X]" / "Survey [X]" / "Lock [X]"
- "Audit the worklist" / "Rescue the worklist" / "Find dead modules"
- "Lift the design doc actions into the worklist"
- "Run an N-Q survey on [X]" / "Fire 25 questions about [X]"
- "Plan the next [epic]"

## Method

### Survey pattern (≥3-option design forks)

When the user asks to lock a non-trivial design decision (≥3
plausible options), fire an `AskUserQuestion` survey **one
question at a time**. Group into rounds (e.g., 10 questions per
round). After every round, recap the locks before proceeding.

After all questions:

1. Write `docs/design/<epic>.md` capturing every lock in a table
   + the resulting architecture + acceptance criteria + risks +
   out-of-scope items.
2. Lift every actionable item into `docs/PROJECT_WORKLIST.md` as
   a new `### EPIC-NAME` section with user-story tasks
   (As/I want/so that + runtime-observable acceptance bullets per
   the no-stubs rule + CLAUDE.md §3). The worklist is created when
   execution past E0 begins (CLAUDE.md §5) — until then, the plan
   doc (`docs/MACKES-WORKSTATION-PLAN.md` §11) is the tracker.
3. Update `docs/AI_GOVERNANCE.md` if the survey locks
   platform-wide direction (not just per-epic).
4. Commit + `git push origin main` per CLAUDE.md §0 + standing
   auth. Single remote — there is no dual-remote push.

### Worklist rescue pass

Before any large ship effort, scan for:

- **Dead modules** — `pub mod foo;` in a crate under `crates/**`
  with zero external `foo::` / `crate::foo::` references (and no
  `pub use foo::*` re-export). Tests inside `foo.rs` itself don't
  count — they reference the module from within.
- **Misleading `[✓]` marks** — tasks marked done where the
  runtime-reachability gate (CLAUDE.md §3) doesn't actually hold:
  the code exists but no `mde <subcommand>` path, iced
  `update`/`view`/subscription, or `mackesd` worker / Bus
  subscription can invoke it.
- **Mockup-only features** — UI that renders but the underlying
  state never updates; `demo_data`/placeholder constants or
  "coming soon" strings standing in for real behavior.
- **Deferred markers** — code or worklist text saying "lands in
  a follow-up", "wired in Phase N", "deferred to", "stub for now",
  "todo!()", "unimplemented!()", `panic!("not yet …")`.
- **Design-doc actions never lifted** — items in
  `docs/design/*.md` that don't have matching worklist entries.

Each finding becomes a new worklist task (user-story shape) BEFORE
any new code lands. CLAUDE.md §3 (Definition of Done — no stubs,
runtime-reachable) is the upstream prevention; the rescue pass is
the downstream catch.

### Authority

When two locks contradict (CLAUDE.md governance — newest wins
silently):
1. **Memory** (`~/.claude/projects/-home-mm-MackesWorkstation/memory/*.md`)
   — operator live preferences, highest.
2. **root `CLAUDE.md`** — the operational rulebook (at the repo
   root, NOT `.claude/CLAUDE.md`).
3. **`docs/AI_GOVERNANCE.md`** — platform identity + compass.
4. **`docs/MACKES-WORKSTATION-PLAN.md` + `docs/design/*.md`** —
   the E0–E8 plan + per-epic locks.
5. **`docs/PROJECT_WORKLIST.md`** body — actionable state.

Newest wins. When in doubt: the §0 master rule from
`docs/AI_GOVERNANCE.md` ("Secure, Simple, No-Fixed-Center
Workgroup").

## Worklist schema (inherited from retired mackes-worklist-management)

```
- [ ] **<PREFIX>-N.M: <release/epic> — <short title>** *(optional carve-out tag)*
  **As** <role>,
  **I want** <capability>,
  **so that** <outcome>.
  **Acceptance** (each runtime-observable):
    - [ ] specific runtime-observable bullet
    - [ ] specific runtime-observable bullet
```

Status legend: `[ ] Open`, `[>] In Progress` (carry a
`session=<id>` marker when a `/ship` session claims it),
`[✓] Done`, `[!] Blocked`. `[~] Deferred` is RETIRED — no silent
deferrals.

Every task carries an epic prefix tying it to the E0–E8 sequence
(plan §11). The RPM (E8) is held until every feature is
§3-complete; HW bench is post-release.

## Companion skills

The live `.claude/skills/` set is exactly five — cross-reference
only these:

- `ship` — when planning is done, switch to ship to drain the
  queue (the autonomous loop + completeness rules).
- `release` — operator-gated RPM cut/push/tag flow; never
  auto-trigger it from a ship run.
- `audit` — integrity sweep of the Rust shell (dead/unreachable
  code, stubs, convention violations) with FINISH-or-REMOVE
  verdicts.
- `preview` — visual / accuracy harness; verify a render actually
  looks right rather than trusting a green `cargo test`.

## Retired ancestors

Not carried into the monorepo — do not point users at them as
live:

- `mackes-worklist-management` — the worklist schema part, now
  embedded here.
- `iteration` — the rescue-pass + N-Q survey parts, now embedded
  here; the LOOP part moved to `ship`.
- `autonomous-worker` — the planning part, embedded here.
- `complete-remaining-work` — the prioritization rules, moved to
  `ship`.
