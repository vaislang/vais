#!/usr/bin/env bash
# End-to-end check for the REAL code generator (compiler/self/fixpoint_codegen.nl):
# it tokenizes an arithmetic source into a List<Token> and emits LLVM IR that
# COMPUTES the result at runtime (mul/add/sub instructions + SSA temps), rather
# than pre-evaluating to a constant. We compile the generated IR and verify the
# runtime value.
#
# Requires the Vais fixes: &Vec borrow recursion (214c97cf) + literal-% escaping
# (e711dac1) so the emitted `%tN` register names survive.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_codegen.nl"
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
  # The output must be REAL IR (contain a runtime instruction for non-trivial exprs).
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': generated IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got (runtime)";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

check "100" 100
check "2 + 3" 5
check "6 * 7" 42
check "12 + 3 * 4" 24
check "2 + 3 * 4 - 1" 13
check "10 - 2 - 3" 5
check "20 - 5 + 2" 17
check "2 * 3 * 4" 24
check "1 + 2 + 3 + 4 + 5" 15

# Sanity: the emitted IR for a binop genuinely contains a runtime instruction
# (not a pre-computed constant) — proves this is codegen, not interpretation.
tmp="$(mktemp -d)"
PROG="6 * 7" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "mul i64" "$tmp/out.ll"; then echo "  PASS emits runtime 'mul i64' (real codegen, not constant-folded)";
else echo "  FAIL '6 * 7' did not emit a runtime mul"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint code generator (emits runtime IR) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
