#!/usr/bin/env bash
# End-to-end check for the STRUCT code generator
# (compiler/self/fixpoint_struct.nl): compiles `struct Name {{ f0, f1, ... }}`
# declarations, `Name {{ f0: v0, ... }}` literals, `p.field` reads, and `p.field
# = expr` writes into real LLVM IR (struct = [N x i64], field name -> index ->
# getelementptr). Structs are the records the nl compiler itself uses.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_struct.nl"
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

# struct literal + field reads
check "struct Point {{ x, y }}; let p = Point {{ x: 3, y: 4 }}; return p.x + p.y;" 7
# field assignment
check "struct Point {{ x, y }}; let p = Point {{ x: 3, y: 4 }}; p.x = 100; return p.x + p.y;" 104
# 3-field struct (like the compiler's Token)
check "struct Tok {{ kind, start, len }}; let t = Tok {{ kind: 1, start: 5, len: 3 }}; return t.kind + t.start + t.len;" 9
# struct fields in arithmetic
check "struct Box {{ w, h }}; let b = Box {{ w: 4, h: 5 }}; return b.w * b.h;" 20
# expression field values
check "struct P {{ a, b }}; let p = P {{ a: 2 + 3, b: 4 * 5 }}; return p.a + p.b;" 25

# Sanity: emitted IR uses array-style struct alloca + getelementptr by field index.
tmp="$(mktemp -d)"
PROG="struct Point {{ x, y }}; let p = Point {{ x: 3, y: 4 }}; return p.y;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "alloca \[2 x i64\]" "$tmp/out.ll" && grep -q "getelementptr \[2 x i64\]" "$tmp/out.ll"; then
  echo "  PASS emits struct alloca + field GEP (record codegen)";
else echo "  FAIL did not emit struct codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint struct codegen (records via [N x i64] + field GEP) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
