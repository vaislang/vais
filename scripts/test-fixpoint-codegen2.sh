#!/usr/bin/env bash
# End-to-end check for the variable code generator
# (compiler/self/fixpoint_codegen2.vais): compiles `let <name> = <expr>; ... return
# <expr>` (multi-char names) into LLVM IR that COMPUTES at runtime. Variables use
# an SSA model — each binding maps a name to the operand its expression produced.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_codegen2.vais"
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

check "let x = 10; return x;" 10
check "let x = 5; let y = x * 2; return y + 1;" 11
check "let a = 3; let b = 4; return a + b;" 7
check "let w = 4; let h = 5; return w * h;" 20
check "let base = 10; let d = base * 2; let r = d + base; return r;" 30
check "let total = 100; let half = total - 50; return half;" 50
check "let x = 2; let y = 3; let z = x * y; return z * z;" 36
check "return 7 * 6;" 42

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint codegen v2 (variables, runtime IR) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
