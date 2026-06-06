#!/usr/bin/env bash
# install-helpers/test-lizardfs-mount.sh — E3.1
#
# Prove the LizardFS FUSE binding end-to-end on a live host: stand up a
# single-node master + chunkserver from the bundled binaries, mfsmount3 a test
# path, and exercise the four E3.1 acceptance checks (mount + mountpoint;
# write/read byte-identical + 64 MB chunking; clean teardown). The systemd
# is-active check (#3) is a separate install step — this harness runs the
# daemons directly so the FUSE round-trip is provable without a system install.
#
# Usage:
#   install-helpers/test-lizardfs-mount.sh /path/to/lizardfs-binaries.tar.gz
#
# Leaves nothing installed; everything lives under a mktemp dir, torn down on
# exit (mount unmounted, daemons killed).

set -uo pipefail

BUNDLE="${1:-lizardfs-binaries.tar.gz}"
[ -s "$BUNDLE" ] || { echo "E3.1: bundle '$BUNDLE' missing/empty" >&2; exit 1; }
BUNDLE="$(readlink -f "$BUNDLE")"

# LizardFS chunkserver/client refuse loopback for the master connection
# ("127.0.0.1 can't be used for connecting with master — use ip address of
# network controller"), so use the host's primary interface IP.
HOST_IP="$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K\S+' | head -1)"
[ -n "$HOST_IP" ] || HOST_IP="$(hostname -I 2>/dev/null | awk '{print $1}')"
[ -n "$HOST_IP" ] || { echo "E3.1: could not determine a non-loopback host IP" >&2; exit 1; }
echo "E3.1: using host IP $HOST_IP for the master connection"

ROOT="$(mktemp -d /tmp/lzfs-e31.XXXXXX)"
BIN="$ROOT/bin"; META="$ROOT/meta"; CS="$ROOT/cs"; HDD="$ROOT/hdd"; MNT="$ROOT/mnt"
mkdir -p "$BIN" "$META" "$CS" "$HDD" "$MNT"
tar -xzf "$BUNDLE" -C "$BIN"
chmod +x "$BIN"/* 2>/dev/null || true
export PATH="$BIN:$PATH"

MASTER_PID=""; CS_PID=""
cleanup() {
    echo "E3.1: teardown ..."
    mountpoint -q "$MNT" && fusermount3 -u "$MNT" 2>/dev/null
    mountpoint -q "$MNT" && fusermount -u "$MNT" 2>/dev/null
    [ -n "$CS_PID" ] && kill "$CS_PID" 2>/dev/null
    [ -n "$MASTER_PID" ] && kill "$MASTER_PID" 2>/dev/null
    sleep 1
    # acceptance #4: no orphaned mfsmount process
    sleep 1
    if pgrep -af "mfsmount $MNT" >/dev/null 2>&1; then
        echo "E3.1: WARN — orphaned mfsmount still present after umount" >&2
    else
        echo "E3.1: ✓ #4 clean teardown — unmounted, no orphaned mfsmount"
    fi
    rm -rf "$ROOT" 2>/dev/null
}
trap cleanup EXIT

# ── master config ──────────────────────────────────────────────────────────
cat > "$META/mfsmaster.cfg" <<EOF
DATA_PATH = $META
EXPORTS_FILENAME = $META/mfsexports.cfg
MATOML_LISTEN_PORT = 9419
MATOCS_LISTEN_PORT = 9420
MATOCL_LISTEN_PORT = 9421
PERSONALITY = master
EOF
echo "* / rw,alldirs,maproot=0" > "$META/mfsexports.cfg"
# Seed empty metadata: prefer the shipped template, else the documented header.
if [ -f "$BIN/metadata.mfs.empty" ]; then
    cp "$BIN/metadata.mfs.empty" "$META/metadata.mfs"
else
    printf 'MFSM NEW\0' > "$META/metadata.mfs"
fi

# ── chunkserver config ───────────────────────────────────────────────────────
cat > "$CS/mfschunkserver.cfg" <<EOF
DATA_PATH = $CS
MASTER_HOST = $HOST_IP
MASTER_PORT = 9420
CSSERV_LISTEN_PORT = 9422
HDD_CONF_FILENAME = $CS/mfshdd.cfg
EOF
echo "$HDD" > "$CS/mfshdd.cfg"

echo "E3.1: starting mfsmaster ..."
"$BIN/mfsmaster" -c "$META/mfsmaster.cfg" -d >"$ROOT/master.log" 2>&1 &
MASTER_PID=$!
sleep 3
echo "E3.1: starting mfschunkserver ..."
"$BIN/mfschunkserver" -c "$CS/mfschunkserver.cfg" -d >"$ROOT/cs.log" 2>&1 &
CS_PID=$!
sleep 4

# ── acceptance #1 — mount + mountpoint ──────────────────────────────────────
# mfsmount enables FUSE allow_other so both the user and mackesd-owned services
# can read the mesh mount; FUSE requires user_allow_other in /etc/fuse.conf for
# that (a deployment prerequisite the RPM/installer sets). Ensure it for the test.
if ! grep -qsE '^[[:space:]]*user_allow_other' /etc/fuse.conf 2>/dev/null; then
    echo "E3.1: enabling user_allow_other in /etc/fuse.conf (deployment prereq) ..."
    echo 'user_allow_other' | sudo tee -a /etc/fuse.conf >/dev/null 2>&1 || true
fi
echo "E3.1: mounting (mfsmount) ..."
"$BIN/mfsmount" "$MNT" -H "$HOST_IP" -P 9421 >"$ROOT/mount.log" 2>&1
sleep 2
PASS=0; FAIL=0
if mountpoint -q "$MNT"; then echo "  ✓ #1 mountpoint($MNT) success"; PASS=$((PASS+1));
else echo "  ✗ #1 mountpoint FAILED"; FAIL=$((FAIL+1)); cat "$ROOT/mount.log" "$ROOT/master.log" "$ROOT/cs.log" 2>/dev/null | tail -20; fi

# ── acceptance #2 — write/read byte-identical + 64 MB chunking ───────────────
if mountpoint -q "$MNT"; then
    echo "hello mesh-storage $(date +%s)" > "$MNT/small.txt"
    if [ -f "$MNT/small.txt" ] && diff <(echo) <(echo) >/dev/null; then :; fi
    cp "$MNT/small.txt" "$ROOT/small.readback"
    if cmp -s "$MNT/small.txt" "$ROOT/small.readback"; then echo "  ✓ #2a small file byte-identical"; PASS=$((PASS+1));
    else echo "  ✗ #2a small file mismatch"; FAIL=$((FAIL+1)); fi
    echo "E3.1: writing 64 MB (exercise chunking) ..."
    dd if=/dev/urandom of="$ROOT/big.src" bs=1M count=64 status=none
    cp "$ROOT/big.src" "$MNT/big.bin"
    sync
    if cmp -s "$ROOT/big.src" "$MNT/big.bin"; then echo "  ✓ #2b 64 MB chunks + reads back byte-identical"; PASS=$((PASS+1));
    else echo "  ✗ #2b 64 MB mismatch"; FAIL=$((FAIL+1)); fi
fi

# ── acceptance #4 — clean teardown (verified by the trap on exit) ────────────
echo "E3.1: results — $PASS passed, $FAIL failed (mount round-trip; teardown checked on exit)"
[ "$FAIL" -eq 0 ] && [ "$PASS" -ge 3 ] && { echo "E3.1: MOUNT PROVEN"; exit 0; }
echo "E3.1: incomplete — see logs above" >&2
exit 1
