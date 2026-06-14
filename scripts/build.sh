#!/usr/bin/env bash
# Internal wrapper around the Vais compiler build command.
#
# Usage: scripts/build.sh path/to/program.vais [-o out]
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"

SRC="${1:?usage: build.sh program.vais [-o out]}"
OUT="a.out"
if [ "${2:-}" = "-o" ]; then OUT="${3:?-o needs a path}"; fi

case "$SRC" in
    *.vais) ;;
    *)
        echo "error: scripts/build.sh only accepts .vais source files" >&2
        exit 2
        ;;
esac

"$HERE/scripts/vaisc" build "$SRC" -o "$OUT"
echo "built: $OUT  (from $SRC)"
