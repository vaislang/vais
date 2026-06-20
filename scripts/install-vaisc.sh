#!/usr/bin/env bash
# Install the standalone native Vais compiler and checker binaries.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
exec "$HERE/scripts/vaisc" run "$HERE/tools/install_vaisc.vais" -- \
    "$HERE" "${PREFIX:-}" "${DESTDIR:-}" "${BINDIR:-}" "$@"
