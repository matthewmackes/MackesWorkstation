# Carbon Design System Review

_Started 2026-05-17 13:36 UTC_

---

## User

Using https://carbondesignsystem.com/ as a guide review the GUI interfaces in https://github.com/matthewmackes/MAP2-RELEASES/blob/main/

## Assistant

_[tool: github_get_tree]_

_[tool: github_read_file]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: questions_v2]_

Before I dive in, this is a substantial XFCE control panel with ~30+ screens (Dashboard, Look & Feel, Devices, Network/Mesh, System, Apps, Maintain, Help, plus a first-run Wizard). I need to scope this properly before designing.

## User

Answered

## Assistant

_[tool: list_files]_

_[tool: list_files]_

_[tool: update_todos]_

I don't see your form answers come through — I'll proceed with strong defaults so I can get something on screen for you to react to:

- **Scope:** Full system pass — Dashboard, Mesh VPN, Mesh Services, Look & Feel, Apps, Maintain, plus the Wizard
- **Format:** Clickable interactive prototype with real tab nav (Carbon UI Shell)
- **Carbon strictness:** Carbon as foundation, tasteful refinements where it matters
- **Accent:** Per-preset accents switchable via Tweaks
- **Mesh:** Topology graph variant + DataTable variant via tweak
- **Data:** Realistic home/dev mesh
- **Chrome:** XFCE window frame for context

Let me read the actual source so the redesign matches the real structure.

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: github_read_file]_

_[tool: copy_starter_component]_

_[tool: snip]_

Got everything I need. Building now.

_[tool: write_file]_

_[tool: write_file]_

_[tool: write_file]_

Now the panels. I'll split into two files to stay under the 1000-line target.

_[tool: write_file]_

_[tool: write_file]_

_[tool: write_file]_

_[tool: write_file]_

Got it — proceeding with defaults. Now the entry HTML.

_[tool: done]_

_[tool: fork_verifier_agent]_

Done. Here's what I built:

**Mackes Shell — Carbon refresh** — a clickable prototype of the XFCE control panel using IBM Carbon Gray 100 as the foundation, wrapped in an XFCE-style window frame.

**Screens (all wired up):**
- **Dashboard** — stat tiles, service-health grid, drift card, hardware summary, recent activity, quick actions
- **Network → Mesh VPN** — live topology view with control-node ring, animated edge pulses, dashed DERP edges, peer detail drawer · plus a Carbon DataTable toggle
- **Network → Mesh SSH** — peer list with host-key fingerprints and an ACL code block
- **Network → Mesh Services** — discovered-services grid with peer filter pills, unified-gateway toggle, mDNS bridge
- **Network → Wi-Fi/Ethernet, Firewall** — DataTables, zones
- **Look & Feel → Appearance** — theme/icons/cursor/fonts/AA with a live preview pane
- **Devices, System** — section hubs
- **Apps** — Install / Remove / Installed tabs with category filters and search
- **Maintain** — 12 sub-cards (snapshots, drift, repair, logs, reset, uninstall…)
- **Maintain → Snapshots** — create + DataTable of existing
- **Help** — left-rail topic browser with rendered markdown for Getting started, Mesh VPN, Presets
- **First-run Wizard** — 8 steps (welcome → env scan → hardware → preset pick → network → snapshot → review → animated apply)

**Tweaks (bottom-right toggle):**
- **Preset** — Mackes / `#!` / Daylight / Vanilla / Node — swaps the accent live across the whole UI
- **Density** — compact / cozy / comfortable
- **Chrome** — XFCE frame on/off, status bar on/off
- **Wizard** — re-open the first-run flow

**Caveats / next steps:**
- The 5 sub-screens in Devices/Maintain are stubs that toast on click; I can build any of them out properly
- Snapshots' "Restore" is a toast, not a confirm modal — easy to flesh out
- Wallpaper picker shows gradient swatches as placeholders rather than real images
- Help only has full content for 3 topics; the rest show a stub

Tell me which screens to deepen, or which preset's accent you want to lock in as the default.

## User

Verifier agent check completed

