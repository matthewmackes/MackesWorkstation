#!/bin/sh
# install-helpers/lint-public-ports.sh — pre-commit lint gate.
#
# Catches NET-NEW listeners that bind a public-facing port outside
# the locked allow-list. The only public ports a MackesWorkstation
# peer exposes are:
#
#   - UDP/4242  — Nebula overlay (lighthouse listens; peers may
#                 also bind for NAT-traversal hole-punch)
#   - TCP/443   — Nebula HTTPS-tunnel fallback (lighthouses only,
#                 for peers behind UDP-blocking firewalls)
#
# Every other listener MUST bind on the Nebula overlay interface
# (e.g. `nebula0` / the overlay IP) — never on `0.0.0.0` or a real
# public interface. mde-bus brokers, LizardFS mounts, mackesd IPC,
# KDC listeners — all overlay-only.
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition of
# Done) for how this gate fits the monorepo's security direction.
#
# Exits 0 = clean, exits 1 = net-new public-port binds.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Scan source + config that could declare listeners. Python is
# retired (lives only under provenance/), so .py is NOT scanned —
# only the Rust workspace under crates/ plus shipped config in data/.
SCAN_INCLUDES='--include=*.rs --include=*.toml --include=*.yaml --include=*.yml --include=*.conf --include=*.service'
SCAN_PATHS='crates/ data/'

# Patterns that suggest a public-facing listener:
# - `bind` to `0.0.0.0` (catch-all)
# - `bind` to `[::]` (IPv6 catch-all)
# - sshd_config `ListenAddress 0.0.0.0`
# - systemd `BindAddress=0.0.0.0`
# - Dockerfile/podman `EXPOSE` (cross-checked against the allow-list)
# - explicit string literal "0.0.0.0" in source — strong signal
PATTERNS='bind.*"?0\.0\.0\.0|bind.*"?\[::\]|ListenAddress[[:space:]]+0\.0\.0\.0|BindAddress=0\.0\.0\.0|^EXPOSE[[:space:]]|"0\.0\.0\.0"'

# Allow-listed path prefixes — files where 0.0.0.0 binds are
# legitimate. Snapshot taken 2026-06-03 against the pre-existing
# merged tree, matching the snapshot-allow-list discipline used by
# lint-dbus-shape.sh. Each broad prefix carries an inline rationale
# below so future security audits can re-evaluate; net-new files NOT
# in this list trigger the gate.
#
# Adapted from the upstream MDE gate: Python (mackes/*.py) allow-list
# entries are dropped (Python is retired), the legacy GTK panel
# (crates/mackes-panel/, absent here) is dropped, and the mackesd /
# nebula-https-tunnel / voice-config crates are remapped into the
# monorepo's crates/{mesh,services}/ layout.
ALLOWED_PREFIXES='
crates/mesh/mackesd/src/workers/nebula_supervisor.rs
crates/mesh/mackesd/src/workers/nebula_https_listener.rs
crates/mesh/mackesd/src/workers/sshd_overlay_bind.rs
crates/mesh/mackesd/src/workers/firewall_preset.rs
crates/mesh/mackes-nebula-https-tunnel/
crates/mesh/mackesd/src/https_fallback.rs
crates/mesh/mackesd/src/transport/
crates/mesh/mackesd/src/topology/
data/systemd/
crates/shell/mde-panel/
crates/mesh/mackesd/src/workers/voice_config.rs
crates/mesh/mackesd/src/workers/wol.rs
crates/mesh/mackesd/src/voice/materialize.rs
crates/services/mde-voice-config/src/lib.rs
crates/services/mde-voice-hud/src/sip.rs
crates/mesh/mackesd/tests/
'

# Snapshot-allow-list rationales (security-audit reference):
#
# - mackes-nebula-https-tunnel/ : the TCP/443 HTTPS-tunnel fallback
#   crate itself — error-string mentions "bind 0.0.0.0:443"; this is
#   one of the two locked public ports.
# - workers/nebula_https_listener.rs : binds 0.0.0.0:443 on
#   lighthouses (the TCP/443 fallback) — a locked public port.
# - workers/nebula_supervisor.rs / sshd_overlay_bind.rs /
#   firewall_preset.rs / transport/ / topology/ / https_fallback.rs :
#   Nebula overlay + firewall plumbing for the locked UDP/4242 +
#   TCP/443 surface.
# - workers/voice_config.rs : voice-config test fixture with the
#   "0.0.0.0" placeholder mesh_bind_address (in-source test).
# - workers/wol.rs : WoL magic packet REQUIRES a broadcast UDP
#   socket bound to 0.0.0.0:0 (RFC 2153 / WoL spec). Not a listener.
# - services/mde-voice-hud/src/sip.rs : the SIP REGISTER client binds an
#   EPHEMERAL UDP socket (0.0.0.0:0) then immediately connect()s to the
#   registrar — the kernel filters incoming datagrams to that one peer and
#   selects the source interface by route (the Nebula overlay for a mesh
#   registrar). An outbound client socket, NOT a service listener (same
#   class as wol.rs above).
# - voice/materialize.rs : default mesh_bind_address placeholder
#   gets replaced with the per-peer overlay IP by voice_config.rs
#   on materialize; "0.0.0.0" is a fallback for tests + initial
#   bootstrap before Nebula enrolment publishes the overlay IP.
# - services/mde-voice-config/src/lib.rs : same — default config
#   struct placeholder that the materializer overrides.
# - data/systemd/ : shipped unit files for the Nebula/overlay
#   listeners above.
# - crates/mesh/mackesd/tests/ : test-only binds.

# Comment-line allow-list (talking ABOUT public ports, not binding them).
# Pattern matches AFTER grep's `file:line:` prefix injection.
COMMENT_PREFIXES=':[0-9]+:[[:space:]]*(///|//!|//|#|<!--|/\*|\*)|"0\.0\.0\.0"[[:space:]]*//|allow-list|allowlist|carve-out|retired|legacy|superseded|allow this'

# Build the grep allow-list filter
ALLOW_FILTER=""
for prefix in $ALLOWED_PREFIXES; do
  [ -z "$prefix" ] && continue
  ALLOW_FILTER="${ALLOW_FILTER}|^${prefix}"
done
ALLOW_FILTER="${ALLOW_FILTER#|}"

violations=$(
  grep -rn -E "$PATTERNS" $SCAN_INCLUDES $SCAN_PATHS 2>/dev/null \
    | grep -vE "$ALLOW_FILTER" \
    | grep -vE "$COMMENT_PREFIXES" \
    || true
)

if [ -n "$violations" ]; then
  echo "$0: net-new public-port binds detected:"
  echo "$violations"
  echo ""
  echo "MackesWorkstation peers must bind every listener on the Nebula"
  echo "overlay interface (e.g. nebula0 / the overlay IP), NEVER on"
  echo "0.0.0.0 or a real public interface. The only public ports"
  echo "allowed are UDP/4242 (Nebula) + TCP/443 (HTTPS fallback on"
  echo "lighthouses). If your listener legitimately needs a public"
  echo "bind, add its source file to the allow-list in this script"
  echo "with a comment citing the security rationale."
  exit 1
fi

echo "$0: no net-new public-port binds — clean."
exit 0
