# MDE-KDECnt-Rust

A pure-Rust, **KDE Connect-compatible** device-communication stack, shared by two
desktops: **[MDE](https://github.com/matthewmackes/MDE)** (the Mackes Shell) and
**[MDE-Retro](https://github.com/matthewmackes/mde-retro-workstation)** (the
Windows 2000 / IBM Carbon shell). It speaks the upstream KDE Connect wire protocol,
so stock **Android/iOS KDE Connect** clients and **GSConnect** pair and talk to it
without modification — plus an optional capability-negotiation header that unlocks
richer features between two MDE-family peers, with graceful fallback for stock
clients.

## Why a shared crate

Both shells want the same protocol and host logic; only the *surface* differs (MDE
binds to D-Bus + a Nebula mesh transport; MDE-Retro binds to its iced UI + a LAN
transport). Extracting the stack here keeps one tested source of truth instead of
two drifting copies. The architecture keeps strict layers:

```
Protocol  →  Transport (trait)  →  Host / Router  →  event stream  →  Surface
(this repo: protocol + host)        (per-platform: mesh vs LAN)   (per-platform: D-Bus vs iced)
```

- The **protocol** layer (`mde-kdc-proto`) is pure: zero I/O, zero D-Bus, zero
  networking. Codec, RSA-2048 pairing + AES-256-GCM session crypto (ring), UDP
  discovery announcements, and the plugin set (ping, clipboard, share,
  notification, battery, mpris, sms, telephony, findmyphone, runcommand).
- The **host** layer (coming) adds the LAN transport (UDP 1716 discovery + rustls
  TCP), the on-disk pairing store (`~/.config/mde/connect/`), and a `Transport`
  trait + event stream so each platform plugs in its own transport and surface —
  no UI, D-Bus, or mesh code lives here.

## Status

| Layer | State |
|---|---|
| `crates/mde-kdc-proto` — protocol library | ✅ extracted, builds + **181 tests pass** standalone |
| host (LAN transport + `Transport` trait + event stream) | 🚧 in progress (MDE's networking was deferred upstream; being completed here) |
| MDE / MDE-Retro wiring | ⏳ both repos repoint to this crate |

## Build

```sh
cargo build
cargo test     # 181+ protocol tests, no network needed
```

## License

GPL-3.0-or-later. Originated from MDE's `mde-kdc-proto` / `mde-kdc` crates.
