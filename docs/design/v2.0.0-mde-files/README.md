# MDE Files — "Artifact Manager" design (v2.0.0)

**Status:** locked 2026-05-19. The visual + interaction contract for the
mesh-first MDE file manager. Rust implementation lives at
`crates/mde-files/` (forked from `pop-os/cosmic-files`).

This directory holds the **canonical design source** for the file
manager. The Rust implementation must match the prototype's structure,
visual rhythm, and interaction model. When the implementation drifts
from the prototype, the prototype wins unless the change is logged here
with an "override" note.

## Files

- `upstream-bundle/` — exact handoff bundle from the Claude Design tool
  (claude.ai/design). Do not edit:
  - `README.md` — handoff README (read first, original instructions).
  - `Artifact-Manager.html` — the single-file React prototype that is
    the implementation contract. Read top-to-bottom before changing
    `crates/mde-files/`.
  - `chats/chat1.md` — desktop-layout iteration (notification center).
    Not directly relevant to the file manager, but kept for context on
    the visual system.
  - `chats/chat2.md` — file-manager iteration history. **Read this** —
    it documents every design pivot (mesh predominance, subdued home,
    Downloads as primary local pin, the explainer-card local veil) and
    explains *why* the layout ended up the way it did.
- `design-spec.md` — Rust implementation contract: data model, view
  router, theme tokens, icon registry, dimensions. Generated from the
  prototype and the chat transcripts; this is what the Rust code
  follows day-to-day.

## What the design says, in one paragraph

The mesh is the home base, not the local filesystem. The sidebar's MESH
section dominates (peers, inbox, outbox, "Network overview" landing).
The LOCAL section is pinned at the bottom of the sidebar with only one
first-class pin — **Downloads**, accented amber — and the rest of the
local filesystem hidden behind a dashed "Browse filesystem…" disclosure
that opens an explainer card ("private to this node") rather than a flat
folder listing. Files that arrived from a peer wear an amber `↘
peer.mesh` pill in every list so the visual rhythm immediately conveys
what came from where.

## How to use this design

When working on `crates/mde-files/`:

1. **Open `upstream-bundle/Artifact-Manager.html`** in a text editor (do
   not render it — the handoff README is explicit on this) and find the
   section you're implementing. The CSS is the source-of-truth for
   spacing, color, type; the JSX is the source-of-truth for layout
   structure and interaction.
2. **Cross-check `design-spec.md`** for the Rust translation of that
   section's data + tokens.
3. If the prototype is silent on a detail (e.g., focus state for an
   item that can never be tabbed to in the prototype), prefer the
   PatternFly v6 dark default + Mackes warm-dark accent, and add the
   detail to `design-spec.md` so the next implementer doesn't have to
   guess again.

## Provenance

The prototype was generated in the Claude Design tool from chat2's
iteration history. The fork target is
`https://github.com/pop-os/cosmic-files`. License compatibility (GPLv3)
is tracked by the v2.0.0 MDE Files worklist item Phase 0.1 in
`docs/PROJECT_WORKLIST.md`.
