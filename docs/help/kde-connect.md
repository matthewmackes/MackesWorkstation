# KDE Connect on Mackes

Mackes ships first-class KDE Connect integration so your Android phones,
iPads, and other paired devices show up alongside your mesh peers in the
Workbench Connect panel — and stay reachable even when they're on a
different LAN.

## What we ship vs. what KDE upstream ships

Mackes does **not** re-implement the KDE Connect protocol. The official
`kdeconnectd` daemon owns pairing, encryption, file transfer, SMS, MPRIS
control, and every other on-the-wire feature. Mackes adds three pieces
on top:

1. **Mackes Workbench Connect panels** — a Carbon/PatternFly UI talking
   `org.kde.kdeconnect.*` over D-Bus. Replaces the KDE tray indicator,
   which Mackes suppresses via `kdeconnect-indicator.desktop`'s
   `Hidden=true` autostart override.
2. **`mackesd-kdc-bridge`** — a per-peer user-systemd service that
   re-announces every paired phone seen on any mesh peer's LAN onto the
   local LAN as if it were directly attached.
3. **`mackes-kdc`** (Rust crate) — typed value model + first-launch
   importer that walks `~/.config/kdeconnect/` and seeds Mackes-side
   `kdeconnect.toml`.

This is the **Option A** integration locked on 2026-05-19: wrap upstream
rather than fork. The upstream daemon stays user-session-autostarted by
its own `.desktop` file; only its tray indicator is hidden.

## Setting up your first device

1. **Install KDE Connect on your phone.** Use the F-Droid build for
   Android (recommended) or the Play Store / App Store release.
2. **Open the phone app while your Mackes machine is on the same Wi-Fi
   as the phone.** The Mackes hostname appears in the phone app's
   "Available devices" list within a few seconds.
3. **Tap the Mackes machine → Pair.** A notification fires on the
   Mackes machine; the Workbench Connect panel surfaces an "Accept
   pair request from … ?" prompt at the same moment.
4. **Accept the prompt.** Pairing completes; the device shows up in
   Workbench Connect → Devices as a row with a phone glyph, the device
   name, a battery gauge (if applicable), and a green reachability
   pip.

First launch after a Mackes install also runs an importer that scans
`~/.config/kdeconnect/` for any pairings KDE Connect already knew about
(common on upgrades from a vanilla Fedora install) and seeds the
Mackes-side `kdeconnect.toml` with their UUIDs. You don't need to
re-pair existing devices when adopting Mackes.

## Workbench Connect panels (shipping in 1.2.0)

Until 1.2.0 ships, use the upstream `kdeconnect-app` GUI for
device-level operations; the Mackes Workbench Connect panels are
listed below so you know what's coming.

> Upstream documentation:
> <https://userbase.kde.org/KDEConnect>

| Panel | What it does |
|---|---|
| **Devices** | Rows for every paired device with name + glyph + battery + reachability pip. Drill-in opens the Device detail page. Pair / unpair buttons live here. |
| **Clipboard** | Per-device clipboard view + push-to-device + pull-from-device + 50-entry rolling history (mirrors Mackes' own mesh-clipboard 100-entry ring on a separate axis). |
| **Files** | Drag-drop send to any reachable device. Receive surface shows incoming files routed to `~/Downloads/<device>/` by default; configurable per-device in `panel.toml:[kdeconnect.destinations]`. |
| **SMS** | Android-only. Threaded SMS / MMS view per paired phone with send-from-desktop. Talks to KDE Connect's SMS plugin; relays via the phone. |
| **Phone** | Battery gauge + Find-my-phone (rings the phone even on silent) + MPRIS remote (use the phone as a play / pause / skip remote for desktop media) + call-silencer (silence the desktop when an incoming call rings the phone) + remote-input pairing (use the phone trackpad / keyboard to drive the desktop). |
| **Device detail** | Per-device deep view reached by drilling into a Devices row. Shows full capability table (which plugins the device supports), pairing fingerprint, last-seen history, and per-feature on/off toggles. |

Notifications mirrored from phones do not get their own panel — they
land in the **Mackes Drawer's Notifications section** with a phone
glyph badge so they sit alongside desktop notifications instead of
competing for attention with them.

## The mesh-mDNS bridge

KDE Connect discovers peers via mDNS on the local broadcast domain
(`_kdeconnect._udp.local`). That fails the moment a phone is on a
different LAN — at the coffee shop, on cellular, behind a corporate
firewall. The `mackesd-kdc-bridge` user-service fixes this.

### How it works

```
┌───────────────────────────────────────────────────────────────────────┐
│ peer-anvil (your laptop on home Wi-Fi)                                │
│                                                                       │
│   kdeconnectd ─── mDNS ───┐                                           │
│                           ▼                                           │
│   mackesd-kdc-bridge  ───  ~/QNM-Shared/peer-anvil/kdc/announce.jsonl │
└────────────────────────────────┬──────────────────────────────────────┘
                                 │  mesh-fs (SSHFS via mackesd)
                                 ▼
┌───────────────────────────────────────────────────────────────────────┐
│ peer-forge (your desktop at the office)                               │
│                                                                       │
│   mackesd-kdc-bridge  ──  reads peer-anvil's announce.jsonl           │
│         │                                                             │
│         └── re-announces every device on the office LAN's mDNS ──┐    │
│                                                                  ▼    │
│                                                       kdeconnectd     │
└───────────────────────────────────────────────────────────────────────┘
```

Concretely:

- **Discover** — bridge subscribes to `_kdeconnect._udp.local` over the
  local Avahi daemon. Every TXT-record-bearing announcement is parsed
  into a `Device` record.
- **Publish** — bridge writes the live device set to
  `~/QNM-Shared/<this-peer>/kdc/announce.jsonl` (one JSON record per
  line). The file rewrites on every mDNS event; readers see consistent
  snapshots.
- **Re-announce** — on every other peer, the bridge reads the
  *remote* `announce.jsonl` files and registers each remote device
  with the local Avahi daemon as a synthetic mDNS service. To
  `kdeconnectd` on this peer, the remote phone now looks like it's on
  the local LAN.
- **Forward** — when `kdeconnectd` opens a connection to the synthetic
  service, the bridge allocates a TCP shuttle through the mesh VPN
  (Tailscale data plane) to the origin peer, then onward to the real
  phone. Latency: ~5–15 ms over a healthy mesh.

The bridge runs as `mackesd-kdc-bridge.service` in the user-systemd
graphical-session target. It auto-restarts on failure, depends on
`avahi-daemon.service`, and is enabled by the `90-mackes.preset`
shipped in `data/systemd/`. No manual setup needed.

### File format

`~/QNM-Shared/<peer>/kdc/announce.jsonl` is JSON-Lines: one `Device`
record per line. Same shape as `mackes_kdc::Device`:

```json
{"id":"a1b2c3d4e5f6a1b2","name":"Pixel 8","kind":"phone","reachable":true,"battery_pct":73,"last_seen_s":1700000000}
{"id":"deadbeefcafebabe1234","name":"iPad mini","kind":"tablet","reachable":false,"battery_pct":12,"last_seen_s":1699900000}
```

Notifications mirrored from phones get an adjacent file,
`~/QNM-Shared/<peer>/kdc/notifications.jsonl`, using the
`MirroredNotification` schema.

The format is committed-public and unit-tested in
`crates/mackes-kdc/tests/integration.rs` — third-party tools can read
and write it directly.

## Troubleshooting

### Paired device doesn't show up in Workbench Connect

1. Confirm the upstream daemon is running:
   ```sh
   systemctl --user status kdeconnect.service
   ```
   If it's not, start it with
   `systemctl --user start kdeconnect.service`.
2. List devices via the upstream CLI:
   ```sh
   kdeconnect-cli --list-devices
   ```
   If the device is there, the issue is on the Mackes side — restart
   the Workbench: `pkill mackes-shell; mackes`.
3. If the upstream CLI doesn't see the device either, re-pair from the
   phone (open KDE Connect on the phone → tap Mackes host → Pair).

### Mesh bridge isn't running

1. Check the unit status:
   ```sh
   systemctl --user status mackesd-kdc-bridge.service
   ```
2. Check the journal for crash loops:
   ```sh
   journalctl --user -u mackesd-kdc-bridge.service -n 100 --no-pager
   ```
3. Verify Avahi is up — the bridge depends on it:
   ```sh
   systemctl status avahi-daemon.service
   ```
4. Re-enable the preset if the unit isn't even loaded:
   ```sh
   systemctl --user enable --now mackesd-kdc-bridge.service
   ```

### Remote phone shows up but file transfer fails

This usually means the per-peer TCP shuttle isn't able to reach the
origin peer. Check:

1. **Mesh VPN status** — `mackes status` should show every peer
   reachable. If the origin peer is down, file transfers naturally
   can't complete.
2. **Origin peer's bridge** — log into the origin peer and check
   `systemctl --user status mackesd-kdc-bridge.service`. The bridge
   has to be running on *both* ends.
3. **Firewall on the origin LAN** — the shuttle uses an ephemeral port
   allocated at connection time. If your origin LAN blocks outbound
   high-port traffic, the connection negotiates but never carries
   bytes. Open outbound TCP on the Tailscale interface.
4. **`~/Downloads/<device>` permissions** — KDE Connect refuses to
   transfer if the destination is read-only. Default destination is
   `~/Downloads/<device>` and Mackes' birthright step creates the
   directory with the user's own ownership; if you've changed the
   destination in `panel.toml:[kdeconnect.destinations]`, double-check
   the new path is writable.

### Device shows the wrong name or wrong battery

The Workbench reads device metadata from a snapshot the upstream
daemon publishes over D-Bus. Both fields update on the daemon's own
cadence — typically every 60 s. If a phone's been off for hours the
battery gauge shows the last-known value rather than going to zero;
`reachable: false` is the signal that the gauge is stale.

### Bridge announce files exist but look stale

The bridge rewrites `announce.jsonl` whenever a device joins, leaves,
or changes state. If the file's mtime is hours old:

1. Restart the bridge: `systemctl --user restart mackesd-kdc-bridge`.
2. If the issue persists, the upstream daemon may not be publishing
   events. Restart it: `systemctl --user restart kdeconnect.service`.
3. As a last resort, force a re-pair from the phone — that always
   triggers a fresh mDNS announce.

## Related help pages

- **[Devices](devices.md)** — the broader Mackes Devices panel (display
  / keyboard / mouse / sound / power); the KDE Connect Devices view
  lives in Workbench Connect, a separate panel.
- **[Mesh in Thunar](mesh-thunar.md)** — `~/QNM-Shared/` lives under
  the `mesh:///` filesystem the bridge uses for inter-peer announce
  files.
- **[Mesh VPN](mesh-vpn.md)** — the data plane the bridge's TCP
  shuttles ride on.
- **[Troubleshooting](troubleshooting.md)** — general fault-finding
  steps that apply beyond KDE Connect.
