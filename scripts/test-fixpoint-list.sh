#!/usr/bin/env bash
# End-to-end check for the dynamic LIST code generator
# (compiler/self/fixpoint_list.vais): compiles `let lst = list()`, `lst.push(expr)`,
# `lst.len`, `lst[expr]`, `lst[expr] = expr` into real LLVM IR. A List is a
# fixed-capacity backing buffer (alloca [64 x i64]) + a length counter — so
# push/len/index work without true heap growth. List<T> is THE data structure
# the nl compiler is built on (List<Token>, List<Fn>, ...).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_list.vais"
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

# push + len + index
check "let xs = list(); xs.push(10); xs.push(20); xs.push(30); return xs.len + xs[1];" 23
# length tracking
check "let xs = list(); xs.push(7); xs.push(8); return xs.len;" 2
# element assignment
check "let xs = list(); xs.push(5); xs.push(6); xs[0] = 50; return xs[0] + xs[1];" 56
# push in a loop, then sum via loop over xs.len (the build+consume pattern the
# nl compiler's tokenizer/evaluator use on List<Token>)
check "let xs = list(); let mut i = 0; while i < 5 {{ xs.push(i * 10); i = i + 1 }}; let mut s = 0; let mut j = 0; while j < xs.len {{ s = s + xs[j]; j = j + 1 }}; return s;" 100

# Sanity: emitted IR uses a buffer alloca + a length alloca + push (load len /
# GEP / store / increment).
tmp="$(mktemp -d)"
PROG="let xs = list(); xs.push(9); return xs.len;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "alloca \[64 x i64\]" "$tmp/out.ll" && grep -q "getelementptr \[64 x i64\]" "$tmp/out.ll"; then
  echo "  PASS emits List buffer + push GEP (dynamic-list codegen)";
else echo "  FAIL did not emit list codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint list codegen (dynamic List push/len/index) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
