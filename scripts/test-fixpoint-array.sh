#!/usr/bin/env bash
# End-to-end check for the ARRAY code generator
# (compiler/self/fixpoint_array.vais): compiles fixed-size integer arrays into real
# LLVM IR — `let a = [v0, v1, ...]` (alloca [N x i64] + element stores), `a[expr]`
# (getelementptr + load), `a[expr] = expr` (getelementptr + store), plus scalar
# mutable vars + `while`. Arrays are the basis for the List the compiler uses.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_array.vais"
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

# array literal + indexed read
check "let a = [10, 20, 30]; return a[1] + a[2];" 50
# array element assignment
check "let a = [1, 2, 3]; a[0] = 100; return a[0] + a[1];" 102
# runtime index
check "let a = [5, 15, 25]; let mut i = 2; return a[i];" 25
# loop summing array elements
check "let a = [10, 20, 30, 40]; let mut s = 0; let mut i = 0; while i < 4 {{ s = s + a[i]; i = i + 1 }}; return s;" 100
# write in a loop, then read
check "let a = [0, 0, 0]; let mut i = 0; while i < 3 {{ a[i] = i * 10; i = i + 1 }}; return a[0] + a[1] + a[2];" 30
# larger array, partial sum
check "let a = [1, 2, 3, 4, 5]; let mut s = 0; let mut i = 0; while i < 5 {{ s = s + a[i]; i = i + 1 }}; return s;" 15

# Sanity: emitted IR genuinely uses array alloca + getelementptr.
tmp="$(mktemp -d)"
PROG="let a = [7, 8]; return a[0];" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "alloca \[2 x i64\]" "$tmp/out.ll" && grep -q "getelementptr" "$tmp/out.ll"; then
  echo "  PASS emits array alloca + getelementptr (real data-structure codegen)";
else echo "  FAIL did not emit array codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint array codegen (alloca [N x i64] + GEP) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
