# Accuracy harness — "accuracy is job 1"

The Win2000 look is verified, not eyeballed. Two layers:

## 1. Metric checklist (static)

`mde-ui` encodes the targets in code (`palette.rs`, `metrics.rs`). Unit tests
assert internal consistency (e.g. the bevel raised/sunken mirror). The checklist
below is what a rendered component must satisfy:

- [ ] Desktop background `#3a6ea5`
- [ ] Window frame silver `#d4d0c8`; sizing frame 3px, fixed frame 1px
- [ ] Active title bar `#0a246a` → gradient to `#a6caf0`; height 18px; Tahoma Bold
- [ ] Inactive title bar `#808080`
- [ ] 3D bevel: raised = white/`#dfdfdf` (TL) over `#808080`/`#404040` (BR)
- [ ] Selection / highlight `#0a246a`, text white
- [ ] Taskbar height 28px, raised bevel; sunken clock well
- [ ] Scrollbars 16px; menu rows 18px
- [ ] UI font Tahoma 8pt everywhere

## 2. Screenshot diff (dynamic)

`tests/accuracy/` compares live captures to reference Win2000 screenshots.

```
tests/accuracy/
  refs/             reference Win2000 PNGs (taskbar, start-menu, window, ...)
  captures/         grim output (gitignored)
  checklist.toml    per-component tolerances + crop regions
```

Flow (run inside a Sway session):

1. Launch the component (e.g. `mde panel`) on a fixed-size headless output.
2. `grim -o HEADLESS-1 captures/taskbar.png`
3. Compare crop regions against `refs/` with an SSIM + per-pixel-ΔE tolerance.
4. Spot-check exact colors at known coordinates (title bar, bevel lines).

The comparator is a small Rust test (`cargo test --test accuracy`) so it can
gate CI. Reference PNGs are added as each component is built.

> Needs a running Wayland session + `grim`; skipped automatically when
> `WAYLAND_DISPLAY` is unset (e.g. in a headless CI without a nested compositor).
