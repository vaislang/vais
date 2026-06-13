#!/usr/bin/env bash
# End-to-end CX5 check: the nl compiler (compiler/self/cx5_compiler.vais) parses a
# program with user-defined functions + calls, emits LLVM IR; we compile that IR
# and verify the value. Exercises function definitions, call dispatch, and the
# struct-Env design that lets evaluation recurse under the Vais Vec-move limit.
#
# Programs embed literal braces as `{{`/`}}` (nl escape -> Vais `\{`/`\}`).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/cx5_compiler.vais"
fail=0

# check <program-string> <expected-value>
check() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  # Replace the run_program("...") argument in main with our program.
  # Use a python rewrite (sed struggles with the braces/backslashes).
  PROG="$prog" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
prog = os.environ["PROG"]
src = re.sub(r'run_program\("(?:[^"\\]|\\.)*"\)', 'run_program("' + prog + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
  legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1 \
    || { echo "  FAIL '$prog': build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

# single fn + call
check 'fn d(x) {{ return x * 2 }}; return d(21)' 42
# fn whose body is plain arithmetic on the param
check 'fn s(a) {{ return a * a }}; return s(5)' 25
# two fns, combined in the final expression
check 'fn d(x) {{ return x * 2 }}; fn s(a) {{ return a * a }}; return d(21) + s(5)' 67
# call result used inside arithmetic
check 'fn d(x) {{ return x * 2 }}; return d(10) + 1' 21
# nested call: one fn body calls another
check 'fn d(x) {{ return x * 2 }}; fn q(a) {{ return d(a) + d(a) }}; return q(5)' 20
# three fns
check 'fn d(x) {{ return x + 1 }}; fn e(a) {{ return a + 2 }}; fn f(b) {{ return b + 3 }}; return d(0) + e(0) + f(0)' 6

# --- CX6: conditionals in bodies + recursion ---
# non-recursive conditional in a function body
check 'fn d(x) {{ return if x > 0 then x * 2 else 0 }}; return d(7)' 14
# factorial (single recursion)
check 'fn f(n) {{ return if n < 2 then 1 else n * f(n - 1) }}; return f(5)' 120
# fibonacci (tree recursion)
check 'fn f(n) {{ return if n < 2 then n else f(n - 1) + f(n - 2) }}; return f(10)' 55
# sum 1..n (recursion with addition)
check 'fn s(n) {{ return if n < 1 then 0 else n + s(n - 1) }}; return s(10)' 55
# recursive fn + a helper, combined
check 'fn f(n) {{ return if n < 2 then 1 else n * f(n - 1) }}; fn d(x) {{ return x + 1 }}; return f(4) + d(5)' 30

# --- CX7: two-argument functions ---
# basic 2-arg
check 'fn m(a, b) {{ return a + b }}; return m(10, 20)' 30
check 'fn m(a, b) {{ return a * b }}; return m(6, 7)' 42
# 2-arg recursive: power
check 'fn p(b, e) {{ return if e < 1 then 1 else b * p(b, e - 1) }}; return p(3, 4)' 81
# 2-arg using both params in a condition: max
check 'fn x(a, b) {{ return if a > b then a else b }}; return x(17, 9)' 17
check 'fn x(a, b) {{ return if a > b then a else b }}; return x(4, 12)' 12
# arg expressions
check 'fn m(a, b) {{ return a + b }}; return m(1 + 2, 3 * 4)' 15
# 2-arg + recursive fn together
check 'fn a(a, b) {{ return a + b }}; fn f(n) {{ return if n < 2 then 1 else n * f(n - 1) }}; return a(3, 4) + f(5)' 127

# --- CX8: local variables (let) in bodies + top-level ---
# single local in a body
check 'fn g(x) {{ let c = x + 1; return c * c }}; return g(4)' 25
# two locals, second depends on first
check 'fn g(x) {{ let a = x + 1; let b = a * 2; return b }}; return g(4)' 10
# local in a 2-arg fn
check 'fn m(a, b) {{ let c = a + b; return c * c }}; return m(2, 3)' 25
# local passed to a recursive call
check 'fn f(n) {{ return if n < 2 then 1 else n * f(n - 1) }}; fn g(x) {{ let c = x - 1; return f(c) }}; return g(6)' 120
# bare-expression body (no return keyword)
check 'fn d(x) {{ x * 3 }}; return d(5)' 15
# top-level variables (cx5_compiler is a superset of the CX1-3 compiler)
check 'let a = 7; let b = 4; return a + b' 11
# top-level var + fn, var used as arg and in final expr
check 'let a = 3; fn d(x) {{ return x * 2 }}; return d(a) + a' 9

# --- CX9: full a-z variable slots ---
# locals t, r, s (outside the old 8-slot set)
check 'fn g(s) {{ let t = s * s; let r = t + 1; return r }}; return g(4)' 17
# param z, local w
check 'fn h(z) {{ let w = z + z; return w }}; return h(9)' 18
# three distinctly-named fns
check 'fn p(a) {{ return a + 1 }}; fn q(b) {{ return b + 2 }}; fn u(c) {{ return c + 3 }}; return p(10) + q(20) + u(30)' 66
# high-letter top-level vars
check 'let t = 5; let w = 6; return t * w' 30

# --- Flagship: every feature in one program ---
# recursion (f=fib) + local (s's t) + 2-arg (a) + top-level var (b) + nested
# calls as args. f(10)=55, s(3)=9, a(55,9)=64, +b(3) = 67.
check 'fn f(n) {{ return if n < 2 then n else f(n - 1) + f(n - 2) }}; fn s(x) {{ let t = x * x; return t }}; fn a(p, q) {{ return p + q }}; let b = 3; return a(f(10), s(b)) + b' 67

[ "$fail" -eq 0 ] && echo "RESULT: CX5-CX9 + flagship (full self-host interpreter) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
