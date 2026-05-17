#!/bin/sh
# Conky helper — Mesh: unified health summary from mackes.mesh.health().
#
# Reads from the same source of truth as the Mesh Health panel and the
# Get Online wizard. Output is two short lines safe to put on a single
# Conky row:
#
#   ● 6/8 ok · 1 warn · 1 fail
#   vpn: Online · 2/3 peer(s) up
#
# Falls back to a one-line "(not configured)" if the module can't load.
python3 - <<'PY' 2>/dev/null || echo "  (not configured)"
try:
    from mackes.mesh import health, summary, overall_state, _PILL_STYLES  # noqa
except Exception:
    print("  (mesh module unavailable)")
    raise SystemExit(0)

try:
    snap = health()
except Exception as e:
    print(f"  health probe failed: {e}")
    raise SystemExit(0)

worst = overall_state(snap)
glyph = {"ok": "●", "warn": "▲", "fail": "✗", "missing": "○"}.get(worst, "·")
print(f"  {glyph} {summary(snap)}")
vpn = snap.get("vpn")
if vpn:
    print(f"  vpn · {vpn.label[:38]}")
PY
