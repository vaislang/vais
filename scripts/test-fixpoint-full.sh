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

# --- integration: functions + imperative bodies + dynamic LISTS ---
# function builds a List in a loop then consumes it (the tokenizer pattern)
check "fn build(n) {{ let xs = list(); let mut i = 0; while i < n {{ xs.push(i * 10); i = i + 1 }}; let mut s = 0; let mut j = 0; while j < xs.len {{ s = s + xs[j]; j = j + 1 }}; return s }}; return build(5);" 100
# List length tracking in a function
check "fn cnt(n) {{ let xs = list(); let mut i = 0; while i < n {{ xs.push(i); i = i + 1 }}; return xs.len }}; return cnt(7);" 7
# function using BOTH an array and a List
check "fn mix(n) {{ let a = [100, 200]; let xs = list(); xs.push(a[0]); xs.push(a[1]); xs.push(n); return xs[0] + xs[2] }}; return mix(5);" 105

# --- full integration: functions + imperative + arrays + Lists + STRUCTS ---
# struct (Token-like) built in a function, fields summed
check "struct Tok {{ kind, start, len }}; fn dist(n) {{ let t = Tok {{ kind: 1, start: n, len: 3 }}; return t.kind + t.start + t.len }}; return dist(5);" 9
# struct field write in a function
check "struct P {{ x, y }}; fn f(n) {{ let p = P {{ x: n, y: 0 }}; p.y = n * 2; return p.x + p.y }}; return f(4);" 12
# struct AND List together in one function
check "struct P {{ a, b }}; fn g(n) {{ let p = P {{ a: 10, b: 20 }}; let xs = list(); xs.push(p.a); xs.push(p.b); xs.push(n); return xs[0] + xs[2] }}; return g(5);" 15

# --- FP12: multi-param (0..4) + zero-param functions + nested call args ---
check "fn add3(a, b, c) {{ return a + b + c }}; fn answer() {{ return 42 }}; return add3(1, 2, 3) + answer();" 48
check "fn add(a, b) {{ return a + b }}; return add(3, 4);" 7
check "fn one() {{ return 1 }}; return one() + one() + one();" 3
check "fn s4(a, b, c, d) {{ return a + b + c + d }}; return s4(10, 20, 30, 40);" 100
check "fn dbl(x) {{ return x * 2 }}; fn add(a, b) {{ return a + b }}; return add(dbl(3), dbl(4));" 14

# --- FP12c: STRING literals + s[i] byte load + s.len() (the source-tokenization
# primitive). Strings use backtick as the delimiter (escaped \` for bash). A
# string literal becomes a module-level @.sN global; s[i] is GEP i8 + load i8 +
# zext; s.len() is the compile-time length. ---
# top-level: byte index + length
check "let s = \`ABC\`; return s[1] + s.len();" 69
# top-level scan summing bytes (tokenizer scan shape): 'A'+'B'+'C' = 198
check "let s = \`ABC\`; let mut i = 0; let mut acc = 0; while i < s.len() {{ acc = acc + s[i]; i = i + 1 }}; return acc;" 198
# two strings, independent length tracking: 'X'88+'Z'90+2+1 = 181
check "let a = \`XY\`; let b = \`Z\`; return a[0] + b[0] + a.len() + b.len();" 181
# string INSIDE a function body
check "fn f() {{ let s = \`ABC\`; return s[1] + s.len() }}; return f();" 69
# string + struct in one function: scan length into a struct field
check "struct C {{ n }}; fn run() {{ let s = \`hello\`; let mut i = 0; let c = C {{ n: 0 }}; while i < s.len() {{ i = i + 1 }}; c.n = i; return c.n }}; return run();" 5
# THE TOKENIZER CORE: a function scans a string byte by byte into a List
# (exactly the shape fixpoint.nl's own tokenizer takes), returns count + first byte
check "fn tok() {{ let s = \`Hi\`; let xs = list(); let mut i = 0; while i < s.len() {{ xs.push(s[i]); i = i + 1 }}; return xs.len + xs[0] }}; return tok();" 74
# THE LEXER INNER LOOP: if-on-byte inside a while-over-string. Counts 'l'(108)
# in "yellow" -> 2 (string indexing + comparison + conditional in a scan loop).
check "fn cnt() {{ let s = \`yellow\`; let mut i = 0; let mut c = 0; while i < s.len() {{ if s[i] == 108 {{ c = c + 1 }}; i = i + 1 }}; return c }}; return cnt();" 2
# A WORKING TOKENIZER: count whitespace-separated token runs in "ab cd ef" -> 3.
# This is the real lexer state machine (if/else + nested if + an in-word flag over
# a string scan) -- the unified compiler codegens a complete tokenizer in the subset.
check "fn ntok() {{ let s = \`ab cd ef\`; let mut i = 0; let mut n = 0; let mut inw = 0; while i < s.len() {{ if s[i] == 32 {{ inw = 0 }} else {{ if inw == 0 {{ n = n + 1 }}; inw = 1 }}; i = i + 1 }}; return n }}; return ntok();" 3

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
if grep -q "define i64 @sum_to(i64 %a0)" "$tmp/out.ll" && grep -q "store i64 %a0" "$tmp/out.ll" && grep -q "br label %loop" "$tmp/out.ll" && grep -q "call i64 @sum_to" "$tmp/out.ll"; then
  echo "  PASS emits function(param-alloca) + loop + call (functions-with-imperative-bodies codegen)";
else echo "  FAIL did not emit function+imperative codegen"; cat "$tmp/out.ll"; fail=1; fi

# --- FP12b: putchar — generated program emits output ---
check_out() {
  local prog="$1" want="$2" tmp; tmp="$(mktemp -d)"
  PROG="$prog" python3 - "$SRC" "$tmp/c.nl" <<'PYEOF'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PYEOF
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
  ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 || { echo "  FAIL '$prog': compiler build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  local got; got="$("$tmp/bin")"
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> stdout [$got]";
  else echo "  FAIL '$prog': stdout got [$got] want [$want]"; fail=1; fi
}
check_out "fn show() {{ putchar(72); putchar(73); return 0 }}; return show();" "HI"
check_out "fn stars(n) {{ let mut i = 0; while i < n {{ putchar(42); i = i + 1 }}; return 0 }}; return stars(5);" "*****"

# Sanity: a string program emits a module-level string global + i8* alloca +
# byte load (GEP i8 / load i8 / zext) — string codegen integrated, not scalar.
tmp="$(mktemp -d)"
PROG="let s = \`Hi\`; return s[0];" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q 'private constant \[' "$tmp/out.ll" && grep -q 'alloca i8\*' "$tmp/out.ll" && grep -q 'getelementptr i8, i8\*' "$tmp/out.ll" && grep -q 'zext i8' "$tmp/out.ll"; then
  echo "  PASS emits string global + i8* alloca + byte load [string codegen integrated]";
else echo "  FAIL did not emit integrated string codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint full codegen (functions with imperative bodies) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
