#!/usr/bin/env bash
# End-to-end check for the IMPERATIVE code generator
# (compiler/self/fixpoint_imperative.vais): compiles `let [mut] <name> = <expr>;
# <name> = <expr>; ... return <expr>` into LLVM IR using alloca/store/load, so
# variables can be MUTATED. Foundation for loops (toward full self-compile).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_imperative.vais"
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
  legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1 \
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

# --- FP10b: while loops ---
check "let mut s = 0; let mut i = 1; while i < 6 {{ s = s + i; i = i + 1 }}; return s;" 15
check "let mut f = 1; let mut i = 1; while i < 6 {{ f = f * i; i = i + 1 }}; return f;" 120
check "let mut n = 10; let mut c = 0; while n > 0 {{ c = c + 1; n = n - 1 }}; return c;" 10
# zero-iteration loop
check "let mut s = 7; let mut i = 10; while i < 5 {{ s = s + 1 }}; return s;" 7
# two sequential loops (unique labels)
check "let mut a = 0; let mut i = 0; while i < 3 {{ a = a + 2; i = i + 1 }}; let mut j = 0; while j < 4 {{ a = a + 1; j = j + 1 }}; return a;" 10

# --- FP10c: if/else statements (control flow) ---
check "let mut s = 0; if 5 > 3 {{ s = 10 }} else {{ s = 20 }}; return s;" 10
check "let mut s = 0; if 2 > 3 {{ s = 10 }} else {{ s = 20 }}; return s;" 20
# no else
check "let mut s = 5; if 1 < 2 {{ s = 99 }}; return s;" 99
check "let mut s = 5; if 9 < 2 {{ s = 99 }}; return s;" 5
# if inside a loop: count 1..10 values > 5
check "let mut c = 0; let mut i = 1; while i < 11 {{ if i > 5 {{ c = c + 1 }}; i = i + 1 }}; return c;" 5

# Sanity: mutation genuinely uses alloca/store/load.
tmp="$(mktemp -d)"
PROG="let mut s = 1; s = s + 1; return s;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "alloca i64" "$tmp/out.ll" && grep -q "store i64" "$tmp/out.ll" && grep -q "load i64" "$tmp/out.ll"; then
  echo "  PASS emits alloca/store/load (real mutable codegen)";
else echo "  FAIL did not emit alloca/store/load"; cat "$tmp/out.ll"; fail=1; fi

# Sanity: a while loop genuinely emits a loop label + back-edge + icmp branch.
tmp="$(mktemp -d)"
PROG="let mut i = 0; while i < 3 {{ i = i + 1 }}; return i;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "loop1:" "$tmp/out.ll" && grep -q "br label %loop1" "$tmp/out.ll" && grep -q "icmp slt" "$tmp/out.ll"; then
  echo "  PASS emits loop label + back-edge + icmp (real loop codegen)";
else echo "  FAIL did not emit loop control flow"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint imperative codegen (mutable vars + while + if) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
