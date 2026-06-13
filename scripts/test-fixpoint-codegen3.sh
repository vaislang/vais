#!/usr/bin/env bash
# End-to-end check for the FUNCTION code generator
# (compiler/self/fixpoint_codegen3.vais): compiles `fn <name>(<param>) {{ return
# <expr> }}; ... return <expr>` into real multi-function LLVM IR — each function
# becomes a `define i64 @<name>(i64 %<param>)` and each call a `call` instruction.
# Multi-char names are emitted as LLVM identifiers. The generated program runs.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_codegen3.vais"
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

check "fn double(x) {{ return x * 2 }}; return double(21);" 42
check "fn inc(x) {{ return x + 1 }}; return inc(41);" 42
check "fn square(n) {{ return n * n }}; return square(7);" 49
check "fn double(x) {{ return x * 2 }}; return double(10 + 11);" 42
check "fn double(x) {{ return x * 2 }}; return double(20) + 2;" 42
check "fn f(x) {{ return x * 3 - 1 }}; return f(5);" 14

# Sanity: the emitted IR genuinely contains a `define @<name>` and a `call`
# (proves multi-function codegen, not inlining/folding).
tmp="$(mktemp -d)"
PROG="fn double(x) {{ return x * 2 }}; return double(21);" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "define i64 @double" "$tmp/out.ll" && grep -q "call i64 @double" "$tmp/out.ll"; then
  echo "  PASS emits 'define @double' + 'call @double' (real function codegen)";
else echo "  FAIL did not emit a function define+call"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint codegen v3 (functions: define/call) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
