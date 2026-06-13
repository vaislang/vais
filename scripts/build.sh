#!/usr/bin/env bash
# Legacy bootstrap build path: transpile a New Vais source, then compile+link
# with the Legacy Vais compiler.
#
# Usage:  scripts/build.sh path/to/program.nl [-o out]
#         scripts/build.sh path/to/program.vais [-o out]
# Prereqs: python3, vaisc (Vais compiler) on PATH, and the Vais source tree
#          for std resolution. Set VAIS_COMPILER_ROOT if vaisc can't find std.
#
# NOTE: this is not the New Vais compiler command. Use scripts/vaisc for the
# New Vais self-host compiler path. This script remains as Legacy bootstrap
# oracle coverage while parity is built.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
TRANSPILER="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
VAIS_COMPILER_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"

SRC="${1:?usage: build.sh program.(vais|nl) [-o out]}"
OUT="a.out"
if [ "${2:-}" = "-o" ]; then OUT="${3:?-o needs a path}"; fi

VAIS_OUT="$(mktemp -d)/$(basename "${SRC%.nl}").vais"
# 1. transpile (warnings -> stderr)
python3 "$TRANSPILER" "$SRC" > "$VAIS_OUT"
# 2. compile with vaisc (run from Vais root so `use std/...` resolves)
ABS_VAIS_OUT="$(cd "$(dirname "$VAIS_OUT")" && pwd)/$(basename "$VAIS_OUT")"
ABS_OUT="$(cd "$(dirname "$OUT")" 2>/dev/null && pwd || pwd)/$(basename "$OUT")"
legacy_vaisc_build "$ABS_VAIS_OUT" -o "$ABS_OUT"
echo "built: $ABS_OUT  (from $SRC)"
