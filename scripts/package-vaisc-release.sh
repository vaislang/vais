#!/usr/bin/env bash
# Build a standalone Vais compiler release archive.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
exec "$HERE/scripts/vaisc" run "$HERE/tools/package_vaisc_release.vais" -- \
    "$HERE" "${OUT_DIR:-}" "${VAIS_VERSION:-}" "$@"
