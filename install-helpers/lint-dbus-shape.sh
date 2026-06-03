#!/bin/sh
# install-helpers/lint-dbus-shape.sh — pre-commit lint gate.
#
# Catches NET-NEW D-Bus interface declarations on MDE-internal
# services. The canonical IPC for MDE-internal control + events
# is the mesh **Bus** (mde-bus); the internal D-Bus surface is
# retiring (it goes away under E0.3). FDO interop surfaces
# (`org.freedesktop.*`) — plus other published cross-desktop
# vendor protocols (`org.mpris.*`, `org.kde.StatusNotifier*`) —
# stay on D-Bus and are always allowed.
#
# This gate flags any net-new `#[interface]` / `#[zbus::interface]`
# block whose interface name is MDE-internal (`dev.mackes.*`,
# `org.mde.*`, `org.mackes.*`), EXCEPT inside the snapshot
# allow-list of pre-existing internal D-Bus (those retire under
# E0.3 — the allow-list shrinks as each one ports to the Bus).
#
# See CLAUDE.md section 2 (conventions) + section 3 (Definition
# of Done) for how this gate fits the monorepo's IPC direction.
#
# Exits 0 = clean, exits 1 = net-new MDE-internal D-Bus surface.

set -eu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

SCAN_INCLUDES='--include=*.rs'
SCAN_PATHS='crates/'

# D-Bus interface declaration markers (the macro that exposes a
# struct as a D-Bus interface).
DBUS_PATTERNS='#\[interface\b|#\[zbus::interface|dbus_macros::dbus_interface'

# Interop interface names that are ALWAYS allowed — these are
# published cross-desktop vendor protocols, not MDE-internal
# control surfaces. Any `#[interface(name = "...")]` whose name
# starts with one of these prefixes is interop and is skipped.
#   org.freedesktop.*           — FDO specs (Notifications, etc.)
#   org.mpris.MediaPlayer2*     — MPRIS media-player control
#   org.kde.StatusNotifier*     — StatusNotifierItem/Watcher tray
INTEROP_NAME_RE='name *= *"(org\.freedesktop\.|org\.mpris\.|org\.kde\.StatusNotifier)'

# Snapshot allow-list of files that hold PRE-EXISTING MDE-internal
# D-Bus at gate-install time (taken 2026-06-03 of the merged tree).
# These retire under E0.3; REMOVE each entry as it ports to the
# mesh Bus so the gate catches any regression. Everything outside
# this list with an internal interface name is net-new.
#
# crates/mesh/mackesd/src/ipc/*   — dev.mackes.MDE.{Shell,Settings,
#                                   Fleet,Nebula.Status,...} services
# crates/services/mde-files/src/dbus_backend.rs
#                                 — dev.mackes.MDE.Shell.* +
#                                   Fleet.Files file-transfer backend
#                                   (client side; carries no
#                                   #[interface] today but is
#                                   snapshotted per the E0.3 plan)
# crates/legacy/mde-kdc/src/dbus.rs
#                                 — legacy KDC dev.mackes.MDE.Connect1
# crates/shell/mde/src/connect.rs — org.mde.Connect1 roster surface
ALLOWED_PREFIXES='
crates/mesh/mackesd/src/ipc/
crates/services/mde-files/src/dbus_backend.rs
crates/legacy/mde-kdc/src/dbus.rs
crates/shell/mde/src/connect.rs
'

# Comment-line allow-list (talking ABOUT D-Bus, not declaring it).
# Pattern matches AFTER the `file:line:` prefix grep -n inserts —
# anchoring on `^` here would catch the file path, not the source.
# Rust-only comment leaders (`//`, `///`, `//!`, `/* */`); the `#`
# leader from the upstream gate is DELIBERATELY omitted — in Rust
# `#[interface]` is an attribute, not a comment, and including `#`
# here silently swallowed every real violation upstream.
COMMENT_PREFIXES=':[0-9]+:[[:space:]]*(///|//!|//|/\*|\*)'

# Build the grep allow-list filter from the path prefixes.
ALLOW_FILTER=""
for prefix in $ALLOWED_PREFIXES; do
  [ -z "$prefix" ] && continue
  ALLOW_FILTER="${ALLOW_FILTER}|^${prefix}"
done
ALLOW_FILTER="${ALLOW_FILTER#|}"

violations=$(
  grep -rn -E "$DBUS_PATTERNS" $SCAN_INCLUDES $SCAN_PATHS 2>/dev/null \
    | grep -vE "$INTEROP_NAME_RE" \
    | grep -vE "$ALLOW_FILTER" \
    | grep -vE "$COMMENT_PREFIXES" \
    || true
)

if [ -n "$violations" ]; then
  echo "$0: net-new MDE-internal D-Bus interface declarations:"
  echo "$violations"
  echo ""
  echo "The mesh Bus (mde-bus) is the canonical IPC for MDE-internal"
  echo "control + events; the internal D-Bus surface retires under E0.3."
  echo "Route new commands/events through the Bus instead of declaring"
  echo "a new #[interface]. FDO + cross-desktop interop (org.freedesktop.*,"
  echo "org.mpris.*, org.kde.StatusNotifier*) stays on D-Bus and is allowed."
  exit 1
fi

echo "$0: no net-new MDE-internal D-Bus interfaces — clean."
exit 0
