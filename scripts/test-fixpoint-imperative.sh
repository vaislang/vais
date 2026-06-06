#!/usr/bin/env bash
# End-to-end check for the IMPERATIVE code generator
# (compiler/self/fixpoint_imperative.nl): compiles `let [mut] <name> = <expr>;
# <name> = <expr>; ... return <expr>` into LLVM IR using alloca/store/load, so
# variables can be MUTATED. Foundation for loops (toward full self-compile).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/fixpoint_imperative.nl"
fail=0

check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  PROG="$prog" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 \
    || { echo "  FAIL '$prog': compiler build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': generated IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got (runtime)";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

check "let x = 7; return x;" 7
check "let mut s = 10; s = s + 5; s = s * 2; return s;" 30
check "let mut a = 1; let mut b = 2; a = a + b; b = a * b; return a + b;" 9
check "let mut n = 100; n = n - 1; n = n - 1; return n;" 98
check "let mut acc = 0; acc = acc + 10; acc = acc + 20; acc = acc + 30; return acc;" 60

# Sanity: mutation genuinely uses alloca/store/load.
tmp="$(mktemp -d)"
PROG="let mut s = 1; s = s + 1; return s;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "alloca i64" "$tmp/out.ll" && grep -q "store i64" "$tmp/out.ll" && grep -q "load i64" "$tmp/out.ll"; then
  echo "  PASS emits alloca/store/load (real mutable codegen)";
else echo "  FAIL did not emit alloca/store/load"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint imperative codegen (mutable vars, alloca) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
