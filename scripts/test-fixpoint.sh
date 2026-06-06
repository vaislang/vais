#!/usr/bin/env bash
# End-to-end check for the List-based fixpoint compiler
# (compiler/self/fixpoint.nl): it tokenizes a source string into a List<Token>,
# evaluates the token list recursively (by &borrow), and emits LLVM IR. We
# compile the IR and verify the value.
#
# This exercises the real pipeline (source -> List<Token> -> recursive eval),
# which the Vais `&Vec` borrow-recursion fix (compiler 214c97cf) made possible.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/fixpoint.nl"
fail=0

# check <source-program> <expected-value>
check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  PROG="$prog" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'tokenize\("(?:[^"\\]|\\.)*"\)', 'tokenize("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 \
    || { echo "  FAIL '$prog': build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

# numbers, multi-digit, whitespace
check "100" 100
check "2 + 3" 5
check "6 * 7" 42
check "12 + 3 * 4" 24
# precedence: * binds tighter than +/-
check "2 + 3 * 4 - 1" 13
# left-associativity of +/-
check "10 - 2 - 3" 5
check "20 - 5 + 2" 17
check "100 - 50 - 25 - 10" 15
# chained *
check "2 * 3 * 4" 24
# longer
check "1 + 2 + 3 + 4 + 5" 15

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint compiler (tokenize -> List<Token> -> eval) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
