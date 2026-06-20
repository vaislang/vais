#!/usr/bin/env bash
# Remove the standalone native Vais compiler and checker binaries.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
exec "$HERE/scripts/vaisc" run "$HERE/tools/uninstall_vaisc.vais" -- \
    "$HERE" "${PREFIX:-/usr/local}" "${DESTDIR:-}" "${BINDIR:-}" "$@"
