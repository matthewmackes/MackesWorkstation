# Polybar themes — upstream attribution

These theme files are vendored from **adi1090x/polybar-themes**, licensed under
GPL-3.0. mackes-shell is also GPL-3.0; the licenses are compatible.

- **Upstream:** https://github.com/adi1090x/polybar-themes
- **Commit vendored:** `a4b8d48500b368122fd010aa6418d8835389871e`
- **Date of upstream commit:** 2026-04-30
- **License:** GPL-3.0 (preserved as `LICENSE` in this directory)

## What's vendored

- `simple/` — 12 theme families: blocks, colorblocks, cuts, docky, forest,
  grayblocks, hack, material, panels, pwidgets, shades, shapes
- `bitmap/` — 10 theme families (the bitmap-font variants; panels and
  pwidgets are simple-only)

## What's *not* vendored

- `fonts/` (3.7 MB of TTF/OTF) — polybar will fall back to system fonts.
  Users wanting upstream's exact fonts can install them separately.
- `wallpapers/` (32 MB) — not related to polybar themes proper.
- `launch.sh` and `setup.sh` — upstream's installer scripts; mackes-shell
  ships its own apply logic in `mackes/shell_profiles.py`.

## Refreshing

To pull a newer upstream snapshot, see `tools/import_adi1090x_polybar.py`
(Phase 2 of the polybar editor build — not yet written). Until then:

```sh
tmpdir=$(mktemp -d)
git clone --depth=1 https://github.com/adi1090x/polybar-themes.git "$tmpdir/polybar-themes"
rm -rf data/shell-profiles/polybar/upstream/{simple,bitmap}
cp -r "$tmpdir/polybar-themes/simple" data/shell-profiles/polybar/upstream/
cp -r "$tmpdir/polybar-themes/bitmap" data/shell-profiles/polybar/upstream/
cp    "$tmpdir/polybar-themes/LICENSE" data/shell-profiles/polybar/upstream/LICENSE
# delete the launch.sh files in simple/ and bitmap/ if they were copied
find data/shell-profiles/polybar/upstream/{simple,bitmap} -maxdepth 1 -name 'launch.sh' -delete
# Update commit hash + date in this file.
rm -rf "$tmpdir"
```
