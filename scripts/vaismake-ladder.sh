#!/usr/bin/env bash
# Run the full Vais gate ladder through the vaismake tool itself: build the
# packaged binary, then let tools/gates.tasks drive every gate via !needs.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"
dist="$HERE/build/vaismake-dist"
"$HERE/scripts/vaisc" package "$HERE/examples/e344_vaismake_package" -o "$dist" >/dev/null
exec "$dist/bin/vaismake" tools/gates.tasks "${1:-ladder}"
