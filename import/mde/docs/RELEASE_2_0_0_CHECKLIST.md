# Release checklist — MDE 2.0.0 cut

> CB-6.5 — single canonical gate list every release captain runs
> through before tagging `v2.0.0`. Each row is yes/no; the cut
> commit can land only when every row is YES.
>
> The checklist is one of the deliverables, NOT a runtime; the
> binaries / specs / docs it gates on are tracked under their own
> `[ ]` items in `PROJECT_WORKLIST.md`. Update both this checklist
> and the worklist when an item flips.

## A. Code-side gates

| # | Gate | Status |
|---|------|--------|
| A1 | All Phase 0 items closed (rebrand identifiers / spec / wrappers / metainfo) | [ ] |
| A2 | All Phase E items closed (Iced + libcosmic panel rewrite, sway IPC, layer-shell) | [ ] |
| A3 | All Phase H items closed (spec dep swap, recommends swap, XDG-autostart drop) | [ ] |
| A4 | All CB-1.x items closed (Workbench panels ported to Iced) | [ ] |
| A5 | All CB-2.x items closed (greeter session entry, 1.x session entries dropped) | [ ] |
| A6 | All CB-3.x items closed (spec rename + Conflicts block + group registration) | [ ] |
| A7 | All CB-4.x items closed (ISO kickstart + plymouth + branding) | [ ] |
| A8 | All CB-5.x items closed (install.sh banner + hand-off + headless hint) | [✓] |

## B. Build gates

| # | Gate | Status |
|---|------|--------|
| B1 | `make rust` green (every Rust crate compiles, no warnings beyond workspace lints) | [ ] |
| B2 | `make test` green (Python + Rust suites; workspace tests crosses 500) | [ ] |
| B3 | `make rpm` green (RPM builds for `mde-2.0.0-1.fc44.x86_64`) | [ ] |
| B4 | `make iso` green (live Fedora ISO builds with MDE pre-installed) | [ ] |

## C. Static analysis + lint gates

| # | Gate | Status |
|---|------|--------|
| C1 | `rpmlint packaging/fedora/mde.spec` green | [ ] |
| C2 | `appstreamcli validate data/metainfo/dev.mackes.MDE.metainfo.xml` green | [ ] |
| C3 | `install-helpers/lint-css.sh data/css/*.css` green | [ ] |
| C4 | `install-helpers/check-no-xfce.sh` green (I.7 gate) | [ ] |
| C5 | `install-helpers/check-wayland-only.sh` green (I.6 gate) | [ ] |
| C6 | `bash -n install.sh` green | [✓] |

## D. Live VM gates

| # | Gate | Status |
|---|------|--------|
| D1 | Fresh-install Fedora-42 VM boots MDE end-to-end (CB-7.1) | [ ] |
| D2 | Upgrade test: v1.0.8 VM → `dnf upgrade` lands on `mde-2.0.0` (CB-7.2) | [ ] |
| D3 | Wayland smoke (sway running, mde-panel layer-shell, mde-files boots) (CB-7.3) | [ ] |
| D4 | Headscale + 3 peers Docker-compose mesh smoke (Phase I.2) | [ ] |

## E. Docs gates

| # | Gate | Status |
|---|------|--------|
| E1 | `README.md` flipped from "Mackes Shell 1.x" → "MDE 2.0.0" (CB-6.1) | [ ] |
| E2 | `docs/MIGRATION_FROM_V1.md` ships (CB-6.2) | [ ] |
| E3 | `docs/help/*.md` sweep complete (CB-6.3) | [ ] |
| E4 | `CHANGELOG.md` v2.0.0 entry has BREAKING CHANGES section (CB-6.4) | [✓] |

## F. Tag + release gates

| # | Gate | Status |
|---|------|--------|
| F1 | `mde/__init__.py:__version__ == "2.0.0"` | [ ] |
| F2 | `pyproject.toml:version == "2.0.0"` | [ ] |
| F3 | `setup.py:version == "2.0.0"` | [ ] |
| F4 | `packaging/fedora/mde.spec:Version == 2.0.0` | [ ] |
| F5 | All four version files agree | [ ] |
| F6 | `git tag -a v2.0.0` pushed; GitHub Release workflow green | [ ] |
| F7 | GitHub Release artifact `mde-2.0.0-1.fc44.x86_64.rpm` published | [ ] |

## G. Post-cut bookkeeping

| # | Gate | Status |
|---|------|--------|
| G1 | CHANGELOG cut-date `(YYYY-MM-DD)` stamped | [ ] |
| G2 | `docs/PROJECT_WORKLIST.md` 2.0.0 sections archived under "Shipped releases" | [ ] |
| G3 | `<release-tag>` template at the end of the worklist refreshed for 2.1 | [ ] |
| G4 | Memory note saved: "MDE 2.0.0 cut on YYYY-MM-DD, full hard switch" | [ ] |

---

When every row above is `[✓]` (or `n/a` with a note), the release
captain can:

```bash
git tag -a v2.0.0 -m "Mackes Desktop Environment 2.0.0 — …"
git push origin v2.0.0
gh run watch <release-workflow-run-id> --exit-status
gh release view v2.0.0
```

Anything short of full-green = no cut. The hard-switch upgrade UX
makes regression handling expensive; this checklist exists so a
gate doesn't slip through to a user's box.
