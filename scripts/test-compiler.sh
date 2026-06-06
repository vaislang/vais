#!/usr/bin/env bash
# End-to-end self-host check: the nl compiler (compiler/self/compiler.nl) turns an
# arithmetic SOURCE STRING into LLVM IR; we compile that IR and verify the value.
# Tests several expressions by swapping the source in main().
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/compiler.nl"
fail=0
check() {
  local expr="$1" want="$2" tmp; tmp="$(mktemp -d)"
  sed "s|compile_eval(\"[^\"]*\")|compile_eval(\"$expr\")|" "$SRC" > "$tmp/c.nl"
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais"
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 || { echo "  FAIL '$expr': nl compiler build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null || { echo "  FAIL '$expr': generated IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$expr' -> IR -> $got"; else echo "  FAIL '$expr': got=$got want=$want"; fail=1; fi
}
check "1+2*3" 7
check "2+3*4" 14
check "10-2-3" 5
check "20/2/5" 2
check "12+34" 46
check "100" 100
[ "$fail" -eq 0 ] && echo "RESULT: self-host compiler end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
