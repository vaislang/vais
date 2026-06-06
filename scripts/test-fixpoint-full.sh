#!/usr/bin/env bash
# End-to-end check for the FULL code generator (compiler/self/fixpoint_full.nl):
# functions with IMPERATIVE bodies — `fn name(param) { let mut ...; while ...;
# if ...; return ... }` plus calls. Each function emits `define i64 @name(i64
# %p_in)` with the param copied to an alloca, body locals alloca'd, the
# imperative body via gen_stmts, and calls as `call` instructions. This is the
# shape the nl compiler's own functions take — the core of the self-compile path.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/fixpoint_full.nl"
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

# function with a loop body
check "fn sum_to(n) {{ let mut s = 0; let mut i = 1; while i < n {{ s = s + i; i = i + 1 }}; return s }}; return sum_to(6);" 15
# factorial via loop
check "fn fact(n) {{ let mut f = 1; let mut i = 1; while i < n {{ f = f * i; i = i + 1 }}; return f }}; return fact(6);" 120
# function with if in body
check "fn clamp(x) {{ let mut r = x; if x > 100 {{ r = 100 }}; return r }}; return clamp(250);" 100
check "fn clamp(x) {{ let mut r = x; if x > 100 {{ r = 100 }}; return r }}; return clamp(7);" 7
# recursion (early-return base case) in an imperative-function body
check "fn fac(n) {{ if n < 2 {{ return 1 }}; return n * fac(n - 1) }}; return fac(5);" 120
# two functions, one calls the other, with mutable locals
check "fn dbl(x) {{ return x + x }}; fn quad(y) {{ let mut r = dbl(y); r = dbl(r); return r }}; return quad(5);" 20

# --- integration: functions + imperative bodies + ARRAYS ---
# function whose body builds a local array and loops summing it
check "fn sumarr(n) {{ let a = [10, 20, 30]; let mut s = 0; let mut i = 0; while i < n {{ s = s + a[i]; i = i + 1 }}; return s }}; return sumarr(3);" 60
# function writes array elements in a loop, then reads
check "fn build(n) {{ let a = [0, 0, 0]; let mut i = 0; while i < n {{ a[i] = i * 5; i = i + 1 }}; return a[0] + a[1] + a[2] }}; return build(3);" 15
# function with an array and an if/else
check "fn pick(k) {{ let a = [7, 8, 9]; let mut r = 0; if k > 1 {{ r = a[2] }} else {{ r = a[0] }}; return r }}; return pick(5);" 9

# Sanity: emitted IR has a function define with param-alloca + a loop + a call.
tmp="$(mktemp -d)"
PROG="fn sum_to(n) {{ let mut s = 0; let mut i = 1; while i < n {{ s = s + i; i = i + 1 }}; return s }}; return sum_to(6);" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "define i64 @sum_to(i64 %p_in)" "$tmp/out.ll" && grep -q "store i64 %p_in" "$tmp/out.ll" && grep -q "br label %loop" "$tmp/out.ll" && grep -q "call i64 @sum_to" "$tmp/out.ll"; then
  echo "  PASS emits function(param-alloca) + loop + call (functions-with-imperative-bodies codegen)";
else echo "  FAIL did not emit function+imperative codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint full codegen (functions with imperative bodies) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
