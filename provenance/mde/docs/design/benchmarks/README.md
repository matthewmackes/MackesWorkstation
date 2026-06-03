# Reference benchmark vault

**Authority:** worklist UX-11 (locked 2026-05-21).
**Purpose:** annotated side-by-side comparisons with the apps
whose visual quality MDE is targeting. When a design question
arises during implementation ("how should focus rings look?"),
the vault is the jury — no re-litigation.

---

## Targets

| Folder | What MDE adopts from it |
|---|---|
| [`linear/`](./linear/)             | Sidebar density + active-row treatment + command-palette interaction patterns |
| [`raycast/`](./raycast/)           | Command palette UX, keyboard primacy, fuzzy-match presentation |
| [`arc/`](./arc/)                   | Motion calmness, spatial coherence, hover-revealed window controls |
| [`cursor/`](./cursor/)             | Onboarding hero polish, wizard transitions |
| [`vercel/`](./vercel/)             | Row hierarchy, empty states, table-dense surfaces |
| [`apple-settings/`](./apple-settings/) | Group layout discipline, inset-sunken active items, restrained palette |

---

## What goes in each folder

Each target folder gets:

1. **`README.md`** — a one-paragraph "what to adopt" + "what to
   not adopt" annotation block. Cite the screenshots inline.
2. **Screenshot files** — PNG, 1280 × auto-height. Capture the
   surface you're annotating (sidebar, palette, settings panel,
   onboarding). File-naming convention:
   `<target>-<surface>-<state>.png` (e.g.
   `linear-sidebar-default.png`, `raycast-palette-results.png`).
3. **Annotations** — embedded as image captions in the README,
   or as a sibling `.notes.md` if the annotation is long.

A target folder is "seeded" when it has ≥ 1 README annotation
and ≥ 1 screenshot. UX-11 acceptance: ≥ 12 annotated comparisons
across all six targets (so ≥ 2 per target on average).

---

## How the vault is used

When a downstream UX-* task hits a design question that the
50-Q + FU + NFU lock survey didn't address, the implementer
consults this vault first. The MDE call should be either:

- **"Match the benchmark exactly."** — cite the screenshot.
- **"Diverge intentionally."** — cite the screenshot, then
  document the divergence reason in a worklist note.

Either way, the answer is grounded in a concrete reference,
not a vibes argument.

---

## Status

**Skeleton landed 2026-05-21** as part of the iteration loop.
Folders are empty placeholders pending screenshot capture and
annotation. Capture work is part of UX-11's
`[ ] Open` → `[✓] Done` arc; this README closes the "vault
exists" gate but not the "vault has content" gate.
