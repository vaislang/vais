#!/usr/bin/env bash
# End-to-end self-host check: the Vais compiler (compiler/self/compiler.vais) turns a
# program string (let-bindings + return over arithmetic with variables) into LLVM
# IR; we compile that IR and verify the value.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
source "$HERE/scripts/vais-build-env.sh"
SRC="$HERE/compiler/self/compiler.vais"
fail=0
check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  sed "s|run_program(\"[^\"]*\")|run_program(\"$prog\")|" "$SRC" > "$tmp/c.input.vais"
  cp "$tmp/c.input.vais" "$tmp/c.vais"
  vais_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1 || { echo "  FAIL '$prog': build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got"; else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}
check "return 6 * 7" 42
check "return 1+2*3" 7
check "return 10-2-3" 5
check "let a = 2; let b = 3; return a + b * 4" 14
check "let x = 10; return x - 3" 7
check "let a = 5; let b = a * 2; return b + 1" 11
check "let n = 100; return n" 100
# CX4: conditionals (if <arith> <cmp> <arith> then <arith> else <arith>)
check "return if 7 > 4 then 11 else 99" 11
check "return if 2 > 4 then 11 else 99" 99
check "return if 3 < 5 then 100 else 200" 100
check "return if 5 == 5 then 7 else 8" 7
check "return if 5 == 6 then 7 else 8" 8
check "let a = 7; let b = 4; return if a > b then a + b else a - b" 11
check "let a = 2; let b = 4; return if a > b then a + b else b - a" 2
check "let x = 3; return if x < 5 then x * 2 else x" 6
[ "$fail" -eq 0 ] && echo "RESULT: self-host compiler (vars+if) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
