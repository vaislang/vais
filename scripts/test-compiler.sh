#!/usr/bin/env bash
# End-to-end self-host check: the nl compiler (compiler/self/compiler.nl) turns a
# program string (let-bindings + return over arithmetic with variables) into LLVM
# IR; we compile that IR and verify the value.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/compiler.nl"
fail=0
check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  sed "s|run_program(\"[^\"]*\")|run_program(\"$prog\")|" "$SRC" > "$tmp/c.nl"
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais"
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 || { echo "  FAIL '$prog': build"; fail=1; return; }
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
[ "$fail" -eq 0 ] && echo "RESULT: self-host compiler (vars) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
