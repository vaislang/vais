#!/usr/bin/env bash
# End-to-end check for the multi-char-FUNCTION fixpoint compiler
# (compiler/self/fixpoint3.nl): tokenizes `fn <name>(<params>) {{ return <expr>
# }}; ... let/return ...` into a List<Token>, builds a List<Fn> function table
# (functions + params keyed by source-name ranges, bodies as token ranges),
# evaluates with multi-char variable + function names, and emits LLVM IR.
#
# Bodies are written with `{{`/`}}` (nl escape -> Vais `\{`/`\}`).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/fixpoint3.nl"
fail=0

check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  PROG="$prog" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'run_program\("(?:[^"\\]|\\.)*"\)', 'run_program("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 \
    || { echo "  FAIL '$prog': build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

# single multi-char fn + call
check "fn square(x) {{ return x * x }}; return square(5);" 25
# two-arg multi-char fn
check "fn add(a, b) {{ return a + b }}; return add(10, 20);" 30
# fn + top-level variable as arg
check "fn double(x) {{ return x * 2 }}; let base = 7; return double(base);" 14
# nested multi-char calls
check "fn square(x) {{ return x * x }}; fn add(a, b) {{ return a + b }}; let base = 3; return add(square(base), base);" 12
# flagship: real names + vars + nested calls
check "fn square(x) {{ return x * x }}; fn add(a, b) {{ return a + b }}; let width = 4; let height = 5; return add(square(width), square(height));" 41
# a function whose body calls another function
check "fn inc(x) {{ return x + 1 }}; fn add2(y) {{ return inc(inc(y)) }}; return add2(10);" 12
check "fn f(a) {{ return a + a }}; fn g(b) {{ return f(b) + b }}; return g(5);" 15

# --- FP3b: conditionals in bodies -> multi-char recursion ---
# non-recursive conditional body
check "fn clamp(x) {{ return if x > 100 then 100 else x }}; return clamp(7);" 7
check "fn clamp(x) {{ return if x > 100 then 100 else x }}; return clamp(250);" 100
# multi-char recursive factorial
check "fn factorial(n) {{ return if n < 2 then 1 else n * factorial(n - 1) }}; return factorial(5);" 120
# multi-char tree recursion (fibonacci)
check "fn fib(n) {{ return if n < 2 then n else fib(n - 1) + fib(n - 2) }}; return fib(10);" 55
# recursive sum
check "fn sumto(n) {{ return if n < 1 then 0 else n + sumto(n - 1) }}; return sumto(10);" 55
# recursion + variable + cross-function
check "fn fact(n) {{ return if n < 2 then 1 else n * fact(n - 1) }}; fn add(a, b) {{ return a + b }}; let base = 4; return add(fact(base), base);" 28

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint3 (multi-char functions, calls, recursion) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
