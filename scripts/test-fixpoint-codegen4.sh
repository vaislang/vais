#!/usr/bin/env bash
# End-to-end check for the CONTROL-FLOW code generator
# (compiler/self/fixpoint_codegen4.vais): compiles functions whose bodies may be
# `return if <cond> then <e> else <e>` into real LLVM IR with icmp / conditional
# br / labeled basic blocks / phi — so RECURSIVE functions (factorial, fib) are
# generated as native code. The generated program runs.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais-legacy/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_codegen4.vais"
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

# recursive factorial
check "fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);" 120
# recursive fibonacci (tree recursion)
check "fn fib(n) {{ return if n < 2 then n else fib(n - 1) + fib(n - 2) }}; return fib(10);" 55
# recursive sum
check "fn sumto(n) {{ return if n < 1 then 0 else n + sumto(n - 1) }}; return sumto(10);" 55
# non-recursive conditional (>)
check "fn clamp(x) {{ return if x > 100 then 100 else x }}; return clamp(7);" 7
check "fn clamp(x) {{ return if x > 100 then 100 else x }}; return clamp(250);" 100
# == comparison
check "fn iszero(x) {{ return if x == 0 then 1 else 0 }}; return iszero(0);" 1
check "fn iszero(x) {{ return if x == 0 then 1 else 0 }}; return iszero(5);" 0

# Sanity: emitted IR genuinely contains control-flow + a recursive call.
tmp="$(mktemp -d)"
PROG="fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "icmp slt" "$tmp/out.ll" && grep -q "phi i64" "$tmp/out.ll" && grep -q "call i64 @factorial" "$tmp/out.ll"; then
  echo "  PASS emits icmp + phi + recursive call (real control-flow codegen)";
else echo "  FAIL did not emit control-flow IR"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint codegen v4 (recursion via icmp/br/phi) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
