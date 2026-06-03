#!/bin/sh
# install-helpers/lint-material-symbols.sh — pre-commit gate (N/A no-op).
#
# This gate is intentionally a no-op in the MackesWorkstation monorepo.
#
# RATIONALE (recorded so the suite count + provenance stay stable):
# The upstream MDE gate forbade Carbon icon references and mandated a
# Carbon -> Material Symbols migration. The monorepo REVERSED that
# design decision:
#
#   - Carbon is KEPT — it is the default dark theme and one of the four
#     supported themes (Win2000 / Carbon / Win10 / BeOS).
#   - Material Symbols was DROPPED entirely.
#
# So the original policy (forbid Carbon / mandate Material) is INVERTED
# here. Porting it as an active gate would block legitimate, supported
# Carbon references. The file is retained — rather than deleted — so the
# pre-commit suite count is stable and this rationale is on record.
#
# See CLAUDE.md section 2 conventions (Carbon is a supported theme;
# the four-theme palette engine) and section 3 Definition of Done.
#
# Exits 0 always.

set -eu

echo "$0: N/A in the monorepo — Carbon is a supported theme; Material Symbols was dropped. No-op (exit 0)."
exit 0
