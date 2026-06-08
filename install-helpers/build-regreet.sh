#!/usr/bin/env bash
# install-helpers/build-regreet.sh — DM-2.1 (2026-06-08)
#
# Build ReGreet (the Rust + GTK4 greetd greeter) from a pinned tag and produce
# regreet-bin.tar.gz suitable for use as Source2 in the MDE RPM.
#
# Usage:
#   install-helpers/build-regreet.sh [tag]
#   install-helpers/build-regreet.sh v0.4.0
#
# Output:
#   regreet-bin.tar.gz  (in the current directory) — contains a single
#   `regreet` executable, extracted into %{_bindir} by the spec's %install.
#
# Why bundled (not a dnf dep): ReGreet is NOT packaged in Fedora and is not a
# published crate, so — exactly like the LizardFS bundle (Source1) — the
# platform builds it once and ships the binary. The mde-desktop subpackage owns
# /usr/bin/regreet; greetd + cage come from Fedora as ordinary Requires.
#
# Build deps (Fedora): rust cargo gtk4-devel gtk4-layer-shell-devel gcc.
# The operator runs this once before the RPM build so
# rpmbuild/SOURCES/regreet-bin.tar.gz is in place.

set -euo pipefail

REGREET_TAG="${1:-v0.4.0}"
REGREET_REPO="https://github.com/rharish101/ReGreet.git"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

echo "build-regreet.sh: cloning tag ${REGREET_TAG} ..."
git clone --depth 1 --branch "${REGREET_TAG}" "${REGREET_REPO}" "${WORK}/src"

echo "build-regreet.sh: cargo build --release ..."
( cd "${WORK}/src" && cargo build --release )

BIN="${WORK}/src/target/release/regreet"
if [ ! -x "${BIN}" ]; then
    echo "build-regreet.sh: ERROR — regreet binary not found at ${BIN}" >&2
    exit 1
fi

echo "build-regreet.sh: packaging regreet-bin.tar.gz ..."
install -d "${WORK}/stage"
install -m 0755 "${BIN}" "${WORK}/stage/regreet"
tar -C "${WORK}/stage" -czf regreet-bin.tar.gz regreet

echo "build-regreet.sh: wrote $(pwd)/regreet-bin.tar.gz"
