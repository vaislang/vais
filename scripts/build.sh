#!/usr/bin/env bash
# Build a .nl program: transpile to Vais, then compile+link with vaisc.
#
# Usage:  scripts/build.sh path/to/program.nl [-o out]
# Prereqs: python3, vaisc (Vais compiler) on PATH, and the Vais source tree
#          for std resolution. Set VAIS_COMPILER_ROOT if vaisc can't find std.
#
# NOTE (prototype): the backend is the reused Vais compiler. The transpiler
# (compiler/transpiler/nl2vais.py) maps the nl surface subset to Vais; see
# docs/design/aria-prototype README / NEW-LANGUAGE-README for coverage & limits.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
TRANSPILER="$HERE/compiler/transpiler/nl2vais.py"
VAIS_COMPILER_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"

SRC="${1:?usage: build.sh program.nl [-o out]}"
OUT="a.out"
if [ "${2:-}" = "-o" ]; then OUT="${3:?-o needs a path}"; fi

VAIS_OUT="$(mktemp -d)/$(basename "${SRC%.nl}").vais"
# 1. transpile (warnings -> stderr)
python3 "$TRANSPILER" "$SRC" > "$VAIS_OUT"
# 2. compile with vaisc (run from Vais root so `use std/...` resolves)
ABS_VAIS_OUT="$(cd "$(dirname "$VAIS_OUT")" && pwd)/$(basename "$VAIS_OUT")"
ABS_OUT="$(cd "$(dirname "$OUT")" 2>/dev/null && pwd || pwd)/$(basename "$OUT")"
( cd "$VAIS_COMPILER_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$ABS_VAIS_OUT" -o "$ABS_OUT" )
echo "built: $ABS_OUT  (from $SRC)"
