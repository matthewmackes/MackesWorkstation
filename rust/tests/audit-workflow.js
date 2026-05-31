export const meta = {
  name: 'mde-completeness-audit',
  description: 'Audit every MDE-Retro Rust shell component for feature-completeness, Win2000 accuracy, bugs, and dead stubs; adversarially verify each finding; critic for whole missing areas',
  phases: [
    { title: 'Audit', detail: 'one auditor per component (parallel), each verifies its own findings' },
    { title: 'Critic', detail: 'whole-platform completeness critic over all verified findings' },
  ],
}

const ROOT = '/home/mm/MDE-Retro'
const RUST = `${ROOT}/rust`

// Shared context every auditor must hold so it grades against the real bar and
// does NOT re-flag conscious, documented trade-offs.
const CONTEXT = `
You are auditing the MDE-Retro shell: a NATIVE RUST (iced toolkit, no GTK)
reimplementation of the Windows 2000 Classic desktop shell, riding on top of
sway (sway is the compositor; mde does not draw window frames/title bars/z-order).
Repo root: ${ROOT}. Rust workspace: ${RUST}. One multiplexed binary 'mde' with
subcommands: panel, menu, files, control-panel, system-properties, run, logoff,
shutdown, properties, setup. Shared look lib: mde-ui (palette + metrics + widgets).

THE SPIRIT (the bar to grade against): "a verifiable transcription of Windows
2000 Classic — every surface obeys the Win2000 system rule for its kind (silver
DrawEdge bevels for chrome, navy-on-white highlight for menu/list rows, exact
COLOR_* RGB), the rule lives as toolkit-agnostic constants, drift is caught by
code; refuses approximation, configurability, and scope past the 2000 shell."

GOAL OF THIS AUDIT: the operator wants to release a FEATURE-COMPLETE platform in
preview. So flag anything that makes a component NOT feature-complete versus the
Windows 2000 component it transcribes, or NOT accurate to Win2000, or a latent
bug, or a dead stub / unwired control.

DO NOT FLAG these conscious, documented trade-offs (they are correct as-is):
- sway draws title bars as a FLAT navy color (no navy→blue caption gradient),
  window frames, borders, and z-order. The gradient/frame/z-order are sway-owned
  by design (read ${RUST}/ACCURACY.md §0). Never flag mde for "missing" these.
- menu items are FLAT navy-highlight (NOT 3D-beveled). This is correct Win2000.
- the GUI installer is an explicit --gui visual PREVIEW only; the real install
  runs via the verified TUI engine (in-session opens it in a themed terminal).
- Tahoma is substituted by "Droid Sans" deliberately (license); metrics name the
  target as UI_FONT_TARGET. Not a bug.
- the RPM cut and the live sway-config cutover are deliberately the LAST steps,
  done with the operator present — do not flag their absence.

Read the files yourself with your tools. Cite concrete evidence as file:line.
Be skeptical and specific. Prefer FEWER, REAL findings over a long speculative
list. Return ONLY findings that, if fixed, make the platform more complete or
more accurate.`

const FINDINGS_SCHEMA = {
  type: 'object',
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          title: { type: 'string', description: 'short imperative title' },
          component: { type: 'string' },
          category: { type: 'string', enum: ['feature-gap', 'accuracy', 'bug', 'stub', 'wiring', 'polish'] },
          severity: { type: 'string', enum: ['high', 'medium', 'low'] },
          evidence: { type: 'string', description: 'file:line and what is there now' },
          fix_sketch: { type: 'string', description: 'concretely how to fix, in 1-3 sentences' },
          achievable_autonomously: { type: 'boolean', description: 'fixable now without sudo/privilege, a clean GUI session, or operator presence' },
          blocked_by: { type: 'string', description: 'the concrete blocker if not autonomous, else ""' },
        },
        required: ['title', 'component', 'category', 'severity', 'evidence', 'fix_sketch', 'achievable_autonomously', 'blocked_by'],
        additionalProperties: false,
      },
    },
  },
  required: ['findings'],
  additionalProperties: false,
}

const VERDICT_SCHEMA = {
  type: 'object',
  properties: {
    is_real: { type: 'boolean', description: 'is this a genuine gap/bug, re-checking the cited code yourself' },
    is_already_done: { type: 'boolean', description: 'is it actually already implemented (false positive)' },
    is_conscious_tradeoff: { type: 'boolean', description: 'is it one of the documented do-not-flag trade-offs' },
    reason: { type: 'string' },
    achievable_autonomously: { type: 'boolean' },
    blocker: { type: 'string' },
    priority: { type: 'string', enum: ['high', 'medium', 'low', 'reject'] },
  },
  required: ['is_real', 'is_already_done', 'is_conscious_tradeoff', 'reason', 'achievable_autonomously', 'blocker', 'priority'],
  additionalProperties: false,
}

const CRITIC_SCHEMA = {
  type: 'object',
  properties: {
    missing_areas: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          area: { type: 'string', description: 'a whole Win2000 shell capability or component not represented at all' },
          why_it_matters: { type: 'string' },
          achievable_autonomously: { type: 'boolean' },
          blocker: { type: 'string' },
        },
        required: ['area', 'why_it_matters', 'achievable_autonomously', 'blocker'],
        additionalProperties: false,
      },
    },
    readiness_assessment: { type: 'string', description: 'is the platform feature-complete enough to release in preview? what are the top 3 blockers?' },
  },
  required: ['missing_areas', 'readiness_assessment'],
  additionalProperties: false,
}

const COMPONENTS = [
  { name: 'panel (taskbar)', files: ['mde/src/panel.rs', 'mde/src/sway.rs'],
    focus: 'Win2000 taskbar: Start button, Quick Launch, window buttons, system tray (StatusNotifier), clock, right-click context menu (Cascade/Tile/Task Manager/Properties), Start-button right-click. What of the real taskbar is missing or unwired?' },
  { name: 'menu (Start menu)', files: ['mde/src/menu.rs', 'mde/src/state.rs', 'SPEC-startmenu.md'],
    focus: 'Win2000 Start menu: Programs submenu, Documents, Settings, Search, Help, Run, Shut Down/Log Off, pinned items, keyboard nav, side banner, right-click context. Submenu cascades? Programs categories?' },
  { name: 'files (Explorer)', files: ['mde/src/files.rs'],
    focus: 'Win2000 Explorer: menubar+dropdowns, toolbar, address bar, tree pane, web-view info band, details list, status bar, context menus on files (right-click: Open/Cut/Copy/Paste/Delete/Rename/Properties), icons, double-click open, selection. What real Explorer behavior is missing/unwired?' },
  { name: 'control-panel', files: ['mde/src/control_panel.rs', 'mde/src/fedora.rs', 'mde/src/install.rs'],
    focus: 'Win2000 Control Panel: applet grid, categories, web-view info band, launch + install-missing, icons. Is the GUI complete or backend-only?' },
  { name: 'system-properties + device manager', files: ['mde/src/system_properties.rs', 'mde/src/sysinfo.rs', 'SPEC-system.md'],
    focus: 'Win2000 System Properties tabs (General/Network Identification/Hardware/User Profiles/Advanced) + Device Manager tree. Which tabs are real vs stub? Device tree completeness?' },
  { name: 'dialogs (logoff/shutdown/run/properties)', files: ['mde/src/dialogs.rs'],
    focus: 'Win2000 dialogs: Run, Shut Down, Log Off, file Properties. Correct buttons, default button, icons, behavior, error handling?' },
  { name: 'installer / setup', files: ['mde/src/installer.rs', 'mde/src/tui_setup.rs', 'SPEC-installer.md'],
    focus: 'NT-style Setup: TUI engine real steps, GUI preview, step list, error surfacing, dnf/greetd/target wiring. Asset-bundling decision (code-only fetch vs bundled) — is it resolved/consistent?' },
  { name: 'launcher / main / apps', files: ['mde/src/main.rs', 'mde/src/apps.rs'],
    focus: 'subcommand dispatch, app/desktop-entry discovery, icon resolution, argument handling. Any subcommand reachable but unimplemented? Missing --help/usage?' },
  { name: 'mde-ui widgets + palette + metrics', files: ['mde-ui/src/widget/mod.rs', 'mde-ui/src/widget/button.rs', 'mde-ui/src/widget/frame.rs', 'mde-ui/src/widget/bevel.rs', 'mde-ui/src/widget/infoband.rs', 'mde-ui/src/palette.rs', 'mde-ui/src/metrics.rs', 'mde-ui/src/font.rs'],
    focus: 'Win2000 widget set completeness: bevels, button, frame, scrollbar, sunken field/picklist, infoband, tree, list, menu, checkbox, radio, tab, groupbox, progress bar. Which standard Win2000 controls are MISSING from the toolkit and forcing apps to hand-roll? Palette/metrics coverage?' },
  { name: 'accuracy harness', files: ['mde/tests/accuracy.rs', 'mde-ui/tests/checklist.rs', 'tests/accuracy/checklist.toml', 'tests/accuracy/capture.sh', 'ACCURACY.md'],
    focus: 'Does the harness capture+check ALL components (panel, menu, files both pane modes, control-panel, system-properties, device-manager, dialogs, setup)? Which components are NOT captured/checked? Are the checks meaningful (real pixel spot-checks)?' },
]

phase('Audit')
log(`Auditing ${COMPONENTS.length} components in parallel, each self-verifying its findings…`)

const perComponent = await pipeline(
  COMPONENTS,
  // Stage 1: audit the component.
  (c) => agent(
    `${CONTEXT}\n\n=== AUDIT TARGET: ${c.name} ===\nFiles (relative to ${RUST}, read them all): ${c.files.join(', ')}\nFocus: ${c.focus}\n\nReturn structured findings. If the component is genuinely complete and accurate, return an empty findings array.`,
    { label: `audit:${c.name}`, phase: 'Audit', schema: FINDINGS_SCHEMA },
  ),
  // Stage 2: adversarially verify each finding from THIS component (no barrier
  // across components — each verifies as soon as its audit lands).
  (res, c) => parallel((res?.findings || []).map((f) => () =>
    agent(
      `${CONTEXT}\n\n=== VERIFY A FINDING (be adversarial; default to rejecting if uncertain) ===\nComponent: ${f.component}\nClaim: ${f.title}\nCategory/severity: ${f.category}/${f.severity}\nEvidence claimed: ${f.evidence}\nProposed fix: ${f.fix_sketch}\nClaimed achievable_autonomously=${f.achievable_autonomously}, blocked_by="${f.blocked_by}"\n\nRe-read the cited code yourself. Decide: is it REAL (genuine gap/bug, not already implemented, not a documented do-not-flag trade-off)? Is it truly achievable autonomously now (no sudo, no clean GUI session needed to BUILD it — pixel-confirmation later is fine, no operator presence)? Assign a priority, or 'reject'.`,
      { label: `verify:${f.title.slice(0, 32)}`, phase: 'Audit', schema: VERDICT_SCHEMA },
    ).then((v) => ({ ...f, verdict: v }))
  )),
)

// Barrier reached: pipeline awaited everything. Flatten + keep survivors.
const all = perComponent.filter(Boolean).flat().filter(Boolean)
const confirmed = all.filter((f) => f.verdict && f.verdict.is_real && !f.verdict.is_already_done && !f.verdict.is_conscious_tradeoff && f.verdict.priority !== 'reject')
log(`${all.length} findings audited; ${confirmed.length} confirmed real after adversarial verify.`)

phase('Critic')
const critic = await agent(
  `${CONTEXT}\n\n=== WHOLE-PLATFORM COMPLETENESS CRITIC ===\nThe per-component audit confirmed these real findings (JSON):\n${JSON.stringify(confirmed.map((f) => ({ component: f.component, title: f.title, category: f.category, severity: f.severity })), null, 1)}\n\nNow step back. What WHOLE AREAS or Win2000 shell capabilities are missing ENTIRELY — not a gap within a component, but a component/feature with no representation at all (e.g. a standard applet, a whole dialog, a desktop behavior, a control kind)? Then assess: is the platform feature-complete enough to release in preview, and what are the top 3 blockers? Read the repo as needed.`,
  { label: 'completeness-critic', phase: 'Critic', schema: CRITIC_SCHEMA },
)

return { confirmed, rejected_count: all.length - confirmed.length, critic }
