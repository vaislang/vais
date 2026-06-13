#!/usr/bin/env bash
# Verify the self-host codegen: run compiler/self/codegen.vais, capture the LLVM IR
# it emits to stdout, compile that IR with clang, run it, and check the value.
# (The value-correctness runner only checks codegen.vais's own exit code; this
# checks that the IR it GENERATES is valid and computes the right answer.)
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais-legacy/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
tmp="$(mktemp -d)"
python3 "$TR" "$HERE/compiler/self/codegen.vais" > "$tmp/cg.vais"
legacy_vaisc_build "$tmp/cg.vais" -o "$tmp/cg" >/dev/null 2>&1 || { echo "FAIL: codegen.vais build"; exit 1; }
"$tmp/cg" > "$tmp/out.ll"
clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null || { echo "FAIL: generated IR invalid"; cat "$tmp/out.ll"; exit 1; }
"$tmp/bin"; got=$?
if [ "$got" = "7" ]; then echo "PASS: self-host codegen — generated IR runs, value=$got"; exit 0
else echo "FAIL: generated IR value=$got (expect 7)"; exit 1; fi
