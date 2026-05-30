# MDE-Retro Rust shell

A native **Rust** rewrite of the MDE-Retro Windows 2000 desktop shell. Runs on
top of **sway** (sway stays the compositor); replaces the Python/shell scripts,
Waybar, and wofi with one lean binary.

> Status: **scaffold** on branch `rust-shell`. Components are stubbed and print
> "not yet implemented" until built. Needs a Rust toolchain — see
> [`DEV-SETUP.md`](DEV-SETUP.md).

## Workspace

| Crate     | What                                                                 |
| --------- | ------------------------------------------------------------------- |
| `mde-ui`  | Win2000 Classic palette, metrics, and the 3D-bevel widget model (iced) |
| `mde`     | the single `mde` binary: `panel`, `menu`, `files`, `control-panel`, `install` |

- **Toolkit:** iced (pure Rust, wgpu). Taskbar + Start menu use `iced_layershell`
  (wlr-layer-shell); the file manager is a normal xdg-toplevel window.
- **Look:** Windows 2000 Classic — palette/metrics transcribed from
  `../assets/reference/win2000-classic-colors.ini`; verified by the
  [accuracy harness](ACCURACY.md).
- **Binary:** one `mde` multiplexed by subcommand (or by `mde-*` symlink).
- **Packaging:** `cargo generate-rpm -p mde` (code-only RPM; assets fetched on
  first run via `mde install --assets`).

## Build

```sh
cd rust
cargo build --release      # -> target/release/mde
cargo test                 # unit tests (+ accuracy harness in a Wayland session)
cargo generate-rpm -p mde  # -> target/generate-rpm/mde-*.rpm
```

## Cutover (big-bang)

When the components are done, the sway `config` swaps the script/Waybar/wofi
launchers for `mde panel` / `mde menu` / `mde files`, and `rust-shell` merges to
`main`. Until then `main` remains the working script-based desktop.
