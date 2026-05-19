# Audio / video compliance — Fedora 44 PulseAudio + PipeWire

**Locked 2026-05-19. Task #17, 1.1.0 release.**

Audit of Mackes Shell's audio probes against the actual stack on
Fedora 44.

## What Fedora 44 ships

- **PipeWire 1.6.5** as the canonical audio server.
- **`pipewire-pulse`** as the compatibility shim that exposes the
  PulseAudio API (`/run/user/$UID/pulse/native`) over PipeWire.
- `pactl` (PulseAudio 17.0 client library) speaks the same protocol
  against either real PulseAudio or the PipeWire shim — no code
  changes needed to support both.
- **`wireplumber`** as the policy engine (replaces pulseaudio-modules).

## Where Mackes hits this stack

1. **Status-cluster volume probe** — `crates/mackes-panel/src/status_cluster.rs::probe_volume`
   runs `pactl get-sink-volume @DEFAULT_SINK@` and parses the first
   `NN%` token. Verified working against pipewire-pulse on Fedora 44.
2. **Drawer volume slider** — `mackes/drawer/volume.py` uses
   `pactl set-sink-volume @DEFAULT_SINK@ <percent>%`. Works under both
   backends; the shim implements both `get-sink-volume` and
   `set-sink-volume`.
3. **Media client autoplay** — `mackes/birthright/apply_media_clients`
   ensures GStreamer 1 + open-codec plugins are present. PipeWire
   integrates with GStreamer via `pipewiresrc` / `pipewiresink`
   automatically; nothing Mackes-specific to do.

## What we verified on the dev host (`mm@fedora`, F44)

```text
$ pactl --version
pactl 17.0
Compiled with libpulse 17.0.0
Linked with libpulse 17.0.0

$ pactl info | grep Server
Server String: /run/user/1000/pulse/native
Server Protocol Version: 35
Server Name: PulseAudio (on PipeWire 1.6.5)

$ pactl get-sink-volume @DEFAULT_SINK@
Volume: front-left: 53076 /  81% / -5.49 dB,
        front-right: 53076 /  81% / -5.49 dB
```

The status-cluster probe correctly extracts `81` from this output.

## Edge cases that just work

- **Headphone hotplug rerouting**: `wireplumber` reroutes the
  default sink to `analog-output-headphones` automatically. The
  Mackes probe re-runs every 2 s and picks up the new volume on the
  new sink without any Mackes-specific event subscription.
- **Bluetooth audio (`bluez5`)**: PipeWire's bluez5 module
  registers BT sinks as additional outputs; `@DEFAULT_SINK@`
  follows wireplumber's selection. Volume probe works on both
  routes.
- **Video playback (mpv / firefox / vlc)**: all use GStreamer or
  ffmpeg under the hood; GStreamer 1.20+ has native `pipewiresink`,
  and ffmpeg uses libpulse against the shim. No Mackes
  intervention required.

## What didn't change in 1.1.0

- The probe code in `status_cluster.rs` works on both PA and PW;
  no rewrite needed.
- The drawer slider's `pactl set-sink-volume` path works on both;
  no rewrite needed.
- We don't ship a wireplumber config or override its policy.

## Follow-ups for 1.2.0

- **Per-app volume** in the Workbench → Devices → Sound panel. The
  drawer surface currently only shows the master sink volume; a
  full per-app row would need `pactl list sink-inputs` parsing.
- **Sink-switch UI**: clicking the volume tray icon could drop a
  popover with every output sink + a click-to-route radio group.
  Today the user goes through `pavucontrol` or `helvum` for this.
