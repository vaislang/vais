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

# --- FP12g: comparison as a VALUE (return a == b / a < b / a > b) -> icmp + zext
# i1->i64. Previously the compiler dropped the comparison (returned just the LHS);
# now it produces 1/0. (The self-host's own source returns boolean comparisons.) ---
check "fn eq(a, b) {{ return a == b }}; return eq(3, 3);" 1
check "fn eq(a, b) {{ return a == b }}; return eq(3, 4);" 0
check "fn lt(a, b) {{ return a < b }}; return lt(3, 4);" 1
check "fn gt(n) {{ return n > 5 }}; return gt(8);" 1

# --- FP12h: parenthesized grouping in expressions -- gen_factor parses `( <expr> )`
# (overrides precedence, nests, and combines with comparison-as-value). ---
check "fn f() {{ return (2 + 3) * 4 }}; return f();" 20
check "fn f(n) {{ return (n + 1) * (n + 2) }}; return f(3);" 20
check "fn f() {{ return ((1 + 2) * 3) }}; return f();" 9
# grouped comparisons combined with arithmetic: (a<b) + (b<c) = 1 + 1 = 2
check "fn f(a, b, c) {{ return (a < b) + (b < c) }}; return f(1, 2, 3);" 2

# --- FP12i: `>=` / `<=` (two-char comparisons) -- tokenized as single ops, emit
# sge/sle, both as values and in if/while conditions. The is_digit bootstrap
# pattern (c >= 48, c <= 57). ---
check "fn ge(a, b) {{ return a >= b }}; return ge(5, 5);" 1
check "fn le(a, b) {{ return a <= b }}; return le(6, 5);" 0
check "fn is_d(c) {{ if c >= 48 {{ if c <= 57 {{ return 1 }} }}; return 0 }}; return is_d(53);" 1
check "fn is_d(c) {{ if c >= 48 {{ if c <= 57 {{ return 1 }} }}; return 0 }}; return is_d(99);" 0
check "fn f(n) {{ let mut s = 0; let mut i = 1; while i <= n {{ s = s + i; i = i + 1 }}; return s }}; return f(5);" 15

# --- FP12j: `and`/`or` as values with correct precedence (lower than comparison).
# The complete is_digit shape: `return c >= 48 and c <= 57`. ---
check "fn is_d(c) {{ return c >= 48 and c <= 57 }}; return is_d(53);" 1
check "fn is_d(c) {{ return c >= 48 and c <= 57 }}; return is_d(99);" 0
check "fn is_d(c) {{ return c >= 48 and c <= 57 }}; return is_d(48);" 1
check "fn f(a) {{ return a < 0 or a > 100 }}; return f(150);" 1
check "fn f(a) {{ return a < 0 or a > 100 }}; return f(50);" 0
# and-chain ordered/unordered
check "fn f(a, b, c) {{ return a < b and b < c }}; return f(1, 2, 3);" 1
check "fn f(a, b, c) {{ return a < b and b < c }}; return f(1, 5, 3);" 0

# --- FP12k: compound conditions in if/while (the condition is evaluated as a
# whole value, then branched on nonzero). The is_alpha shape `if c >= 97 and
# c <= 122`. ---
check "fn is_a(c) {{ if c >= 97 and c <= 122 {{ return 1 }}; return 0 }}; return is_a(100);" 1
check "fn is_a(c) {{ if c >= 97 and c <= 122 {{ return 1 }}; return 0 }}; return is_a(50);" 0
check "fn f(x) {{ if x < 0 or x > 100 {{ return 1 }}; return 0 }}; return f(150);" 1
check "fn f(n) {{ let mut s = 0; let mut i = 0; while i < n and s < 100 {{ s = s + 10; i = i + 1 }}; return s }}; return f(5);" 50

# --- INTEGRATION: a complete 4-function lexer fragment (the real self-host lexer
# shape) -- is_digit/is_alpha/is_space (compound conditions) + a classify
# dispatcher (helper-call dispatch). classify(53)=1, classify(104)=2,
# classify(32)=3 -> 1 + 2*10 + 3*50 = 171. Proves the self-host compiler codegens
# its own lexer's core. ---
check "fn is_digit(c) {{ return c >= 48 and c <= 57 }}; fn is_alpha(c) {{ return c >= 97 and c <= 122 }}; fn is_space(c) {{ return c == 32 }}; fn classify(c) {{ if is_digit(c) {{ return 1 }}; if is_alpha(c) {{ return 2 }}; if is_space(c) {{ return 3 }}; return 0 }}; return classify(53) + classify(104) * 10 + classify(32) * 50;" 171

# --- FP12l: STRING PARAMETERS (`fn f(s: Str)`) -- the #1 bootstrap pattern (the
# self-host source uses Str params 176x). Param typed i8*, string-literal arg
# passed as a pointer, s[i] byte-load. (s.len() on a param needs runtime length;
# tracked separately -- these pass an explicit index or length.) ---
check "fn f(s: Str) {{ return s[0] }}; return f(\`A\`);" 65
check "fn f(s: Str) {{ return s[1] }}; return f(\`AB\`);" 66
# name_eq shape: compare two bytes of a string parameter
check "fn name_eq(s: Str, a: Int, b: Int) {{ return s[a] == s[b] }}; return name_eq(\`abca\`, 0, 3);" 1
# string param + int params + byte arithmetic
check "fn f(s: Str, i: Int) {{ return s[i] + s[i + 1] }}; return f(\`AB\`, 0);" 131

# --- FP12m: s.len() on a string PARAMETER (runtime strlen -- walk to NUL). This
# completes string params: a function can now take the source as `s: Str` and
# scan it with `while i < s.len()`. The actual self-host tokenizer signature. ---
check "fn slen(s: Str) {{ return s.len() }}; return slen(\`hello\`);" 5
check "fn cd(s: Str) {{ let mut i = 0; let mut n = 0; while i < s.len() {{ if s[i] >= 48 and s[i] <= 57 {{ n = n + 1 }}; i = i + 1 }}; return n }}; return cd(\`a1b2c3\`);" 3
# a REAL tokenizer over a string PARAMETER: count whitespace-separated tokens
check "fn ntok(s: Str) {{ let mut i = 0; let mut n = 0; let mut inw = 0; while i < s.len() {{ if s[i] == 32 {{ inw = 0 }} else {{ if inw == 0 {{ n = n + 1 }}; inw = 1 }}; i = i + 1 }}; return n }}; return ntok(\`ab cd ef\`);" 3

# --- FP12n: LIST PARAMETERS (`fn f(xs: List<Int>)`) -- the #2 bootstrap pattern
# (the self-host uses &List<Token> params 177x). A local List is passed by buffer
# POINTER (length written to buf[63] at the call); the callee indexes via
# getelementptr i64 and reads xs.len from buf[63]. The parser/evaluator signature
# `fn eval_expr(toks: &List<Token>, ...)`. ---
check "fn sum_list(xs: List<Int>) {{ let mut s = 0; let mut i = 0; while i < xs.len {{ s = s + xs[i]; i = i + 1 }}; return s }}; fn run() {{ let xs = list(); xs.push(10); xs.push(20); xs.push(30); return sum_list(xs) }}; return run();" 60
check "fn first(xs: List<Int>) {{ return xs[0] }}; fn run() {{ let xs = list(); xs.push(7); xs.push(8); return first(xs) }}; return run();" 7
check "fn cnt(xs: List<Int>) {{ return xs.len }}; fn run() {{ let xs = list(); xs.push(1); xs.push(2); xs.push(3); return cnt(xs) }}; return run();" 3

# --- FP12o: 5-10 PARAMETERS (the self-host core arity plus fixpoint2.nl's
# 10-param word_is helper). Plus the REAL helpers name_eq (5) and kw3 (6). ---
check "fn s8(a, b, c, d, e, f, g, h) {{ return a + b + c + d + e + f + g + h }}; return s8(1, 2, 3, 4, 5, 6, 7, 8);" 36
check "fn s10(a, b, c, d, e, f, g, h, i, j) {{ return a + b + c + d + e + f + g + h + i + j }}; return s10(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);" 55
check "fn fill(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int, out: List<Int>, n: Int) {{ out.push(n); out.push(n + 2); return 0 }}; fn run() {{ let xs = list(); fill(1, 2, 3, 4, 5, 6, 7, 8, xs, 20); return xs.len * 10 + xs[1] }}; return run();" 42
# name_eq: the parser's identifier matcher (string param + 5 params + byte loop)
check "fn name_eq(s: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen == blen {{ let mut k = 0; let mut ok = 1; while k < alen {{ if s[a + k] == s[b + k] {{ ok = ok }} else {{ ok = 0 }}; k = k + 1 }}; return ok }}; return 0 }}; return name_eq(\`letlet\`, 0, 3, 3, 3);" 1
check "fn name_eq(s: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen == blen {{ let mut k = 0; let mut ok = 1; while k < alen {{ if s[a + k] == s[b + k] {{ ok = ok }} else {{ ok = 0 }}; k = k + 1 }}; return ok }}; return 0 }}; return name_eq(\`letmut\`, 0, 3, 3, 3);" 0
# kw3: the keyword recognizer (string param + 6 params + 3 byte compares)
check "fn kw3(s: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int) {{ if alen == 3 {{ if s[a] == w0 {{ if s[a + 1] == w1 {{ if s[a + 2] == w2 {{ return 1 }} }} }} }}; return 0 }}; return kw3(\`let\`, 0, 3, 108, 101, 116);" 1
# fixpoint2.nl's real word_is helper: string param + 9 scalar params.
check "fn word_is(src: Str, a: Int, alen: Int, w0: Int, w1: Int, w2: Int, w3: Int, w4: Int, w5: Int, wlen: Int) {{ if alen != wlen {{ return 0 }}; if alen >= 1 {{ if src[a] != w0 {{ return 0 }} }}; if alen >= 2 {{ if src[a + 1] != w1 {{ return 0 }} }}; if alen >= 3 {{ if src[a + 2] != w2 {{ return 0 }} }}; if alen >= 4 {{ if src[a + 3] != w3 {{ return 0 }} }}; if alen >= 5 {{ if src[a + 4] != w4 {{ return 0 }} }}; if alen >= 6 {{ if src[a + 5] != w5 {{ return 0 }} }}; return 1 }}; return word_is(\`return\`, 0, 6, 114, 101, 116, 117, 114, 110, 6);" 1

# --- FP12p: bare call statements + push-to-List-param (the out-param pattern,
# which sidesteps List return). A function fills a caller-provided List, and the
# caller reads it back. The full self-host tokenizer shape `fn tokenize(src: Str,
# out: List<Int>)`. ---
# bare call statement (discarded result) is emitted (the call fires)
check "fn add1(x) {{ return x + 1 }}; fn run() {{ let mut a = 0; add1(a); return 5 }}; return run();" 5
# push to a List parameter, callee reads its own length
check "fn fill(out: List<Int>, n: Int) {{ let mut i = 0; while i < n {{ out.push(i * 10); i = i + 1 }}; return out.len }}; fn run() {{ let xs = list(); return fill(xs, 3) }}; return run();" 3
# THE OUT-PARAM TOKENIZER: scan a string param, fill the out List, caller reads tokens
check "fn tokenize(src: Str, out: List<Int>) {{ let mut i = 0; while i < src.len() {{ if src[i] == 32 {{ i = i + 1 }} else {{ out.push(src[i]); i = i + 1 }} }}; return 0 }}; fn run() {{ let toks = list(); tokenize(\`ab cd\`, toks); return toks[0] + toks[3] }}; return run();" 197

# --- FP12q: List length-sync after out-param calls + the FULL tokenize->consume
# pipeline across two functions. After an out-param fill, the caller's xs.len
# reflects the pushes (synced from buf[63]), so the filled List can be passed on. ---
check "fn fill(out: List<Int>, n: Int) {{ let mut i = 0; while i < n {{ out.push(i + 1); i = i + 1 }}; return 0 }}; fn run() {{ let xs = list(); fill(xs, 3); return xs.len }}; return run();" 3
# THE PIPELINE: tokenize(out-param) fills a List, then count_digits(List-param) scans it
check "fn is_d(c) {{ return c >= 48 and c <= 57 }}; fn tokenize(src: Str, out: List<Int>) {{ let mut i = 0; while i < src.len() {{ if src[i] == 32 {{ i = i + 1 }} else {{ out.push(src[i]); i = i + 1 }} }}; return 0 }}; fn count_digits(toks: List<Int>) {{ let mut j = 0; let mut n = 0; while j < toks.len {{ if is_d(toks[j]) {{ n = n + 1 }}; j = j + 1 }}; return n }}; fn run() {{ let toks = list(); tokenize(\`a1 b2 c3\`, toks); return count_digits(toks) }}; return run();" 3

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

# THE PARSER CORE: byte-by-byte string equality (the name_eq shape -- compare two
# byte ranges). "let" == "let" -> 1; "let" != "mut" -> 0.
check "fn eq() {{ let a = \`let\`; let b = \`let\`; let mut i = 0; let mut ok = 1; while i < a.len() {{ if a[i] == b[i] {{ ok = ok }} else {{ ok = 0 }}; i = i + 1 }}; return ok }}; return eq();" 1
check "fn ne() {{ let a = \`let\`; let b = \`mut\`; let mut i = 0; let mut ok = 1; while i < a.len() {{ if a[i] == b[i] {{ ok = ok }} else {{ ok = 0 }}; i = i + 1 }}; return ok }}; return ne();" 0
# KEYWORD RECOGNITION: length check + per-byte compare (the kw3/kw5 pattern the
# real lexer uses to classify identifiers as keywords). Recognizes "let" -> 7.
check "fn kw() {{ let s = \`let\`; if s.len() == 3 {{ if s[0] == 108 {{ if s[1] == 101 {{ if s[2] == 116 {{ return 7 }} }} }} }}; return 0 }}; return kw();" 7

# --- FP12r: List-of-structs (the real self-host List<Token> shape) ---
# A List whose elements are whole structs: buffer is [64*nfields x i64] with
# element stride = field count; push stores each field, toks[i].field reads
# buf[i*stride + field_index]. This is exactly how the self-host tokenizer
# builds and consumes List<Token>.
# push a struct + read length
check "struct Tok {{ kind, val }}; fn run() {{ let toks = list(); toks.push(Tok {{ kind: 1, val: 10 }}); return toks.len }}; return run();" 1
# index a List-of-structs element field, two terms in one expression (skip_factor)
check "struct Tok {{ kind, val }}; fn run() {{ let toks = list(); toks.push(Tok {{ kind: 1, val: 10 }}); toks.push(Tok {{ kind: 2, val: 20 }}); return toks[1].val + toks[0].kind }}; return run();" 21
# dynamic build-then-consume loop over List<struct> (tokenize -> eval shape)
check "struct Tok {{ kind, val }}; fn run() {{ let toks = list(); let mut i = 0; while i < 3 {{ toks.push(Tok {{ kind: 1, val: i + 1 }}); i = i + 1 }}; let mut s = 0; let mut j = 0; while j < toks.len {{ s = s + toks[j].val; j = j + 1 }}; return s }}; return run();" 6
# the actual 4-field self-host Token struct
check "struct Token {{ kind, value, nstart, nlen }}; fn run() {{ let toks = list(); toks.push(Token {{ kind: 5, value: 100, nstart: 2, nlen: 3 }}); toks.push(Token {{ kind: 7, value: 50, nstart: 0, nlen: 1 }}); return toks[0].value + toks[1].value + toks[0].nlen }}; return run();" 153

# --- FP12s: List-of-structs PARAMETER (the full self-host tokenizer shape) ---
# A List<struct> passed by-pointer to a function that pushes structs into it
# (out-param) and a caller that consumes them by field. The buffer carries its
# length at index 64*nf (past the strided data, avoiding the buf[63] collision),
# and the caller infers the local List's element type from the callee's
# `List<Struct>` param annotation (the push happens in the callee, not the caller).
# callee fills an empty out-param List<struct>, caller reads length
check "struct Tok {{ kind, val }}; fn fill(out: List<Tok>) {{ out.push(Tok {{ kind: 1, val: 10 }}); out.push(Tok {{ kind: 2, val: 20 }}); return 0 }}; fn run() {{ let toks = list(); fill(toks); return toks.len }}; return run();" 2
# caller reads struct fields the callee pushed
check "struct Tok {{ kind, val }}; fn fill(out: List<Tok>) {{ out.push(Tok {{ kind: 1, val: 10 }}); out.push(Tok {{ kind: 2, val: 20 }}); return 0 }}; fn run() {{ let toks = list(); fill(toks); return toks[0].val + toks[1].val + toks[0].kind }}; return run();" 31
# callee reads its own List<struct> param element field
check "struct Tok {{ kind, val }}; fn fill(out: List<Tok>) {{ out.push(Tok {{ kind: 5, val: 100 }}); return out[0].val }}; fn run() {{ let toks = list(); return fill(toks) }}; return run();" 100
# THE full self-host tokenizer shape: scan string -> push Token{{kind,value}} into
# out-param List<Token> -> caller consumes by field (digit-token value sum)
check "struct Token {{ kind, value }}; fn tok(s: Str, out: List<Token>) {{ let mut i = 0; while i < s.len() {{ let c = s[i]; if c >= 48 and c <= 57 {{ out.push(Token {{ kind: 1, value: c - 48 }}) }} else {{ out.push(Token {{ kind: 0, value: 0 }}) }}; i = i + 1 }}; return 0 }}; fn run() {{ let toks = list(); tok(\`a1b2\`, toks); let mut s = 0; let mut j = 0; while j < toks.len {{ if toks[j].kind == 1 {{ s = s + toks[j].value }}; j = j + 1 }}; return s }}; return run();" 3

# --- FP12t: the full tokenize->eval pipeline (List<Token> SHARED across functions) ---
# The self-host shape: tokenize fills a List<Token> out-param, then a SEPARATE
# consumer function takes that List<Token> as a param and dispatches on
# toks[i].kind. The consumer is read-only (no push), so its element type must be
# inferred from its OWN `List<Token>` param annotation, not a body push-scan.
# tokenize -> eval(toks: List<Token>): sum the numeric tokens (dispatch on kind)
check "struct Token {{ kind, value }}; fn tokenize(s: Str, out: List<Token>) {{ let mut i = 0; while i < s.len() {{ let c = s[i]; if c >= 48 and c <= 57 {{ out.push(Token {{ kind: 1, value: c - 48 }}) }} else {{ out.push(Token {{ kind: 2, value: 0 }}) }}; i = i + 1 }}; return 0 }}; fn eval(toks: List<Token>) {{ let mut s = 0; let mut j = 0; while j < toks.len {{ if toks[j].kind == 1 {{ s = s + toks[j].value }}; j = j + 1 }}; return s }}; fn run() {{ let toks = list(); tokenize(\`1+2+3\`, toks); return eval(toks) }}; return run();" 6
# a mini calculator: eval walks tokens, dispatches on kind, accumulates
check "struct Token {{ kind, value }}; fn tokenize(s: Str, out: List<Token>) {{ let mut i = 0; while i < s.len() {{ let c = s[i]; if c >= 48 and c <= 57 {{ out.push(Token {{ kind: 1, value: c - 48 }}) }} else {{ out.push(Token {{ kind: 2, value: 0 }}) }}; i = i + 1 }}; return 0 }}; fn eval(toks: List<Token>) {{ let mut acc = 0; let mut j = 0; while j < toks.len {{ let k = toks[j].kind; if k == 1 {{ acc = acc + toks[j].value }} else {{ acc = acc + 0 }}; j = j + 1 }}; return acc }}; fn run() {{ let toks = list(); tokenize(\`2+3+4\`, toks); return eval(toks) }}; return run();" 9
# TWO separate consumers over the same tokenize-filled List<Token> (multi-pass)
check "struct Token {{ kind, value }}; fn tokenize(s: Str, out: List<Token>) {{ let mut i = 0; while i < s.len() {{ let c = s[i]; if c >= 48 and c <= 57 {{ out.push(Token {{ kind: 1, value: c - 48 }}) }} else {{ out.push(Token {{ kind: 2, value: 0 }}) }}; i = i + 1 }}; return 0 }}; fn count_nums(toks: List<Token>) {{ let mut n = 0; let mut j = 0; while j < toks.len {{ if toks[j].kind == 1 {{ n = n + 1 }}; j = j + 1 }}; return n }}; fn sum_nums(toks: List<Token>) {{ let mut s = 0; let mut j = 0; while j < toks.len {{ if toks[j].kind == 1 {{ s = s + toks[j].value }}; j = j + 1 }}; return s }}; fn run() {{ let toks = list(); tokenize(\`5+6\`, toks); return count_nums(toks) * 100 + sum_nums(toks) }}; return run();" 211

# --- FP12u: typed let bindings (`let name: Type = rhs`) -- real self-host source ---
# Actual self-host source uses typed lets like `let mut toks: List<Token> = []`.
# The let parser must skip the `: Type` annotation (incl. `List<...>`) to find the
# RHS. An empty list literal `[]` is recognized as a List; element type comes from
# the annotation. (Untyped `let name = rhs` keeps working.)
# typed scalar let
check "fn run() {{ let x: Int = 42; return x }}; return run();" 42
# typed mutable scalar let
check "fn run() {{ let mut x: Int = 40; x = x + 2; return x }}; return run();" 42
# typed empty-list-literal of structs (the `let mut toks: List<Token> = []` shape)
check "struct Tok {{ kind, val }}; fn run() {{ let mut xs: List<Tok> = []; xs.push(Tok {{ kind: 0, val: 42 }}); return xs[0].val }}; return run();" 42
# typed list() of structs + typed struct-literal let
check "struct P {{ x, y }}; fn run() {{ let p: P = P {{ x: 40, y: 2 }}; return p.x + p.y }}; return run();" 42
# typed empty-list of scalars
check "fn run() {{ let xs: List<Int> = []; xs.push(20); xs.push(22); return xs[0] + xs[1] }}; return run();" 42
# top-level typed let
check "let n: Int = 42; return n;" 42
# the real self-host shape: typed-list tokenizer (toks: List<Token> = [] then push)
check "struct Token {{ kind, value }}; fn tokenize(s: Str) {{ let mut toks: List<Token> = []; let mut i = 0; while i < s.len() {{ let c = s[i]; if c >= 48 and c <= 57 {{ toks.push(Token {{ kind: 1, value: c - 48 }}) }}; i = i + 1 }}; let mut sum = 0; let mut j = 0; while j < toks.len {{ sum = sum + toks[j].value; j = j + 1 }}; return sum }}; return tokenize(\`1a2a3\`);" 6

# --- FP12v: boolean literals `true`/`false` -- real self-host loop-flag pattern ---
# fixpoint.nl uses `let mut go = true; ... go = false` for digit-run / scan loops.
# true/false are tokenized as identifiers (kind 1); gen_factor treats them as the
# integer constants 1/0 (nl bools are i64) instead of looking them up as variables.
# bool flag controlling a while loop (the `while go { ...; go = false }` pattern)
check "fn run() {{ let mut go = true; let mut v = 0; while go {{ v = v + 1; if v >= 5 {{ go = false }} }}; return v }}; return run();" 5
# true as an if condition
check "fn run() {{ let b = true; if b {{ return 42 }}; return 0 }}; return run();" 42
# false as an if condition
check "fn run() {{ let b = false; if b {{ return 0 }}; return 42 }}; return run();" 42
# the real self-host digit-run tokenize: nested `while go` consuming a multi-digit
# number into one token (v = v*10 + (d-48)), with `go` a bool loop flag
check "struct Token {{ kind, value }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c >= 48 and c <= 57 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else {{ let d = src[i]; if d >= 48 and d <= 57 {{ v = v * 10 + (d - 48); i = i + 1 }} else {{ go = false }} }} }}; out.push(Token {{ kind: 0, value: v }}) }} else {{ i = i + 1 }} }}; return 0 }}; fn run() {{ let toks = list(); tokenize(\`12a34\`, toks); return toks[0].value + toks[1].value }}; return run();" 46

# --- FP12w: `else if` chains inside a loop body -- real self-host tokenize ---
# An `if A {} else if B {} else {}` chain inside a `while` body was mis-lowered:
# the else-region only covered the first inner then-block, hoisting the final
# `else` to run unconditionally (loop ran once). Fix: if_stmt_end() walks the
# whole else-if chain so the outer else-region spans it; the `else if` body is
# emitted as the nested if statement.
# 3-way else-if chain in a loop (the dispatch shape)
check "fn run() {{ let mut i = 0; let mut v = 0; let mut go = true; while go {{ if i >= 3 {{ go = false }} else if i >= 0 {{ v = v + 1; i = i + 1 }} else {{ go = false }} }}; return v }}; return run();" 3
# else-if with a function-call condition in a loop
check "fn pos(x: Int) {{ if x >= 0 {{ return 1 }}; return 0 }}; fn run() {{ let mut i = 0; let mut v = 0; let mut go = true; while go {{ if i >= 3 {{ go = false }} else if pos(i) == 1 {{ v = v + 1; i = i + 1 }} else {{ go = false }} }}; return v }}; return run();" 3
# THE real self-host tokenizer: 4 token kinds (digit-run/+/*/-), else-if chain,
# is_space/is_digit helper calls -- fixpoint.nl's tokenize, out-param form.
# token count (5: num12, +, num3, *, num4)
check "fn is_space(c: Int) {{ if c == 32 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; struct Token {{ kind, value }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if is_space(c) == 1 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ out.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else if c == 45 {{ out.push(Token {{ kind: 3, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn run() {{ let toks = list(); tokenize(\`12 + 3 * 4\`, toks); return toks.len }}; return run();" 5
# token values: toks[0].value(12) + toks[2].value(3) + toks[4].value(4) = 19
check "fn is_space(c: Int) {{ if c == 32 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; struct Token {{ kind, value }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if is_space(c) == 1 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ out.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else if c == 45 {{ out.push(Token {{ kind: 3, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn run() {{ let toks = list(); tokenize(\`12 + 3 * 4\`, toks); return toks[0].value + toks[2].value + toks[4].value }}; return run();" 19

# --- FP12x: `let t = toks[i]` binding a List-of-structs element + t.field ---
# fixpoint.nl's evaluator does `let t = toks[i]; if t.kind == 2 { ... }` -- it
# binds a whole Token struct to a local and reads its fields. The let must copy
# all nf fields into a [nf x i64] struct local (sty=elem type). Works for both a
# local List-of-structs (is_arr=2) and a List-of-structs param (is_arr=4).
# bind a local List-of-structs element, read its fields
check "struct Tok {{ kind, val }}; fn run() {{ let toks = list(); toks.push(Tok {{ kind: 7, val: 30 }}); toks.push(Tok {{ kind: 9, val: 12 }}); let a = toks[0]; let b = toks[1]; return a.val + b.val + a.kind }}; return run();" 49
# bind a List-of-structs PARAM element
check "struct Tok {{ kind, val }}; fn fk(toks: List<Tok>) {{ let t = toks[0]; return t.kind }}; fn run() {{ let xs = list(); xs.push(Tok {{ kind: 7, val: 42 }}); return fk(xs) }}; return run();" 7
# THE real fixpoint.nl evaluator: mutually-recursive eval over List<Token> with
# `let t = toks[i]` kind dispatch -- evaluates 2+3*4 = 14 (precedence + left-fold)
check "struct Token {{ kind, value }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_expr_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc + term) }}; if op.kind == 3 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc - term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_expr_fold(toks, after, n, acc) }}; fn run() {{ let toks = list(); toks.push(Token {{ kind: 0, value: 2 }}); toks.push(Token {{ kind: 1, value: 0 }}); toks.push(Token {{ kind: 0, value: 3 }}); toks.push(Token {{ kind: 2, value: 0 }}); toks.push(Token {{ kind: 0, value: 4 }}); return eval_expr(toks, 5) }}; return run();" 14
# the same evaluator, left-assoc + precedence: 10 - 2 * 3 = 4
check "struct Token {{ kind, value }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_expr_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc + term) }}; if op.kind == 3 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc - term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_expr_fold(toks, after, n, acc) }}; fn run() {{ let toks = list(); toks.push(Token {{ kind: 0, value: 10 }}); toks.push(Token {{ kind: 3, value: 0 }}); toks.push(Token {{ kind: 0, value: 2 }}); toks.push(Token {{ kind: 2, value: 0 }}); toks.push(Token {{ kind: 0, value: 3 }}); return eval_expr(toks, 5) }}; return run();" 4

# --- FP12y: the COMPLETE self-host pipeline integrated end-to-end ---
# A single program: run(src: Str) tokenizes the expression string into a
# List<Token> (out-param), then evaluates it (eval over the List<Token>), and
# returns the value. This is fixpoint.nl's whole tokenize+eval compiler, run as
# one integrated program -- the smallest complete self-host compiler, end to end.
# (No new compiler capability -- it composes FP12w tokenize + FP12x evaluator.)
FP12Y_PIPE='fn is_space(c: Int) {{ if c == 32 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; struct Token {{ kind, value }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if is_space(c) == 1 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ out.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else if c == 45 {{ out.push(Token {{ kind: 3, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_expr_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc + term) }}; if op.kind == 3 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_expr_fold(toks, after, n, acc - term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_expr_fold(toks, after, n, acc) }}; fn run(src: Str) {{ let toks = list(); tokenize(src, toks); return eval_expr(toks, toks.len) }};'
# 2+3*4 = 14 (precedence)
check "$FP12Y_PIPE fn main2() {{ return run(\`2+3*4\`) }}; return main2();" 14
# 10 - 2 * 3 = 4 (left-assoc + precedence + spaces)
check "$FP12Y_PIPE fn main2() {{ return run(\`10 - 2 * 3\`) }}; return main2();" 4
# 7 + 8 + 9 = 24 (left-fold over a longer expression)
check "$FP12Y_PIPE fn main2() {{ return run(\`7 + 8 + 9\`) }}; return main2();" 24

# --- FP12z: `!=` operator + name_eq + List<Var> symbol table (fixpoint2.nl) ---
# `!=` was entirely missing from the tokenizer (`!` was skipped, `=` became an
# assignment), so `src[a+k] != src[b+k]` lowered to `... != 0` (RHS dropped).
# This broke name_eq (source-byte comparison) and the symbol-table lookup it
# powers -- the core of fixpoint2.nl (arithmetic + multi-char variables).
# != as a value / condition
check "fn run() {{ let a = 5; let b = 3; let r = a != b; if a != 5 {{ return 0 }}; return r * 42 }}; return run();" 42
# name_eq: compares two source-byte ranges (the symbol-table key check)
check "fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn run() {{ let src = \`foo bar foo\`; return name_eq(src, 0, 3, 8, 3) * 10 + name_eq(src, 0, 3, 4, 3) }}; return run();" 10
# THE fixpoint2.nl symbol table: List<Var> looked up by name_eq on source bytes
check "struct Var {{ nstart, nlen, value }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 - 1 }}; fn run() {{ let vars = list(); vars.push(Var {{ nstart: 0, nlen: 3, value: 10 }}); vars.push(Var {{ nstart: 4, nlen: 3, value: 20 }}); let src = \`foo bar foo\`; return lookup(vars, src, 8, 3) + lookup(vars, src, 4, 3) }}; return run();" 30

# --- FP12aa: variable evaluation -- eval_factor resolves idents via lookup ---
# fixpoint2.nl's evaluator: eval_factor returns t.value for a num, else
# lookup(vars, ...) for an identifier; the vars: List<Var> symbol table is
# threaded through the whole recursive eval chain (eval_expr -> eval_fold ->
# eval_term -> eval_factor -> lookup). This is the arithmetic+variables tier.
# x + 3 with x=5 -> 8 (identifier operand resolved via lookup)
check "struct Token {{ kind, value, nstart, nlen }}; struct Var {{ nstart, nlen, value }}; fn lookup(vars: List<Var>, q: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if v.nstart == q {{ return v.value }}; i = i + 1 }}; return 0 }}; fn eval_factor(toks: List<Token>, i: Int, vars: List<Var>) {{ let t = toks[i]; if t.kind == 0 {{ return t.value }}; return lookup(vars, t.nstart) }}; fn eval_term(toks: List<Token>, i: Int, n: Int, vars: List<Var>, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rf = eval_factor(toks, i + 1, vars); return eval_term(toks, i + 2, n, vars, acc * rf) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, stop: Int, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let op = toks[i]; if op.kind == 4 {{ let rf = eval_factor(toks, i + 1, vars); let term = eval_term(toks, i + 2, stop, vars, rf); let after = skip_term(toks, i + 2, stop); return eval_fold(toks, after, stop, vars, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, i: Int, stop: Int, vars: List<Var>) {{ let first = eval_factor(toks, i, vars); let acc = eval_term(toks, i + 1, stop, vars, first); let after = skip_term(toks, i + 1, stop); return eval_fold(toks, after, stop, vars, acc) }}; fn run() {{ let vars = list(); vars.push(Var {{ nstart: 0, nlen: 1, value: 5 }}); let toks = list(); toks.push(Token {{ kind: 1, value: 0, nstart: 0, nlen: 1 }}); toks.push(Token {{ kind: 4, value: 0, nstart: 0, nlen: 0 }}); toks.push(Token {{ kind: 0, value: 3, nstart: 0, nlen: 0 }}); return eval_expr(toks, 0, 3, vars) }}; return run();" 8
# x + y * 4 with x=2, y=3 -> 14 (variable operands + precedence)
check "struct Token {{ kind, value, nstart, nlen }}; struct Var {{ nstart, nlen, value }}; fn lookup(vars: List<Var>, q: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if v.nstart == q {{ return v.value }}; i = i + 1 }}; return 0 }}; fn eval_factor(toks: List<Token>, i: Int, vars: List<Var>) {{ let t = toks[i]; if t.kind == 0 {{ return t.value }}; return lookup(vars, t.nstart) }}; fn eval_term(toks: List<Token>, i: Int, n: Int, vars: List<Var>, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rf = eval_factor(toks, i + 1, vars); return eval_term(toks, i + 2, n, vars, acc * rf) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, stop: Int, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let op = toks[i]; if op.kind == 4 {{ let rf = eval_factor(toks, i + 1, vars); let term = eval_term(toks, i + 2, stop, vars, rf); let after = skip_term(toks, i + 2, stop); return eval_fold(toks, after, stop, vars, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, i: Int, stop: Int, vars: List<Var>) {{ let first = eval_factor(toks, i, vars); let acc = eval_term(toks, i + 1, stop, vars, first); let after = skip_term(toks, i + 1, stop); return eval_fold(toks, after, stop, vars, acc) }}; fn run() {{ let vars = list(); vars.push(Var {{ nstart: 0, nlen: 1, value: 2 }}); vars.push(Var {{ nstart: 2, nlen: 1, value: 3 }}); let toks = list(); toks.push(Token {{ kind: 1, value: 0, nstart: 0, nlen: 1 }}); toks.push(Token {{ kind: 4, value: 0, nstart: 0, nlen: 0 }}); toks.push(Token {{ kind: 1, value: 0, nstart: 2, nlen: 1 }}); toks.push(Token {{ kind: 2, value: 0, nstart: 0, nlen: 0 }}); toks.push(Token {{ kind: 0, value: 4, nstart: 0, nlen: 0 }}); return eval_expr(toks, 0, 5, vars) }}; return run();" 14

# --- FP12bb: the COMPLETE arithmetic+variables compiler end-to-end ---
# fixpoint2.nl as one integrated program: run_program(src) tokenizes a multi-let
# program (kw_let/kw_return keyword recognition + idents + nums + + * = ;), builds
# a List<Var> symbol table from the `let` bindings (variables may reference earlier
# variables), evaluates each expr with precedence, and returns the `return` value.
# The smallest complete self-host compiler for a language WITH variables, run end
# to end. (Composes FP12z symbol table + FP12aa variable eval + keyword tokens.)
FP12BB_C='fn is_alpha(c: Int) {{ if c >= 97 and c <= 122 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; fn kw_let(src: Str, a: Int, alen: Int) {{ if alen != 3 {{ return 0 }}; if src[a] != 108 {{ return 0 }}; if src[a + 1] != 101 {{ return 0 }}; if src[a + 2] != 116 {{ return 0 }}; return 1 }}; fn kw_ret(src: Str, a: Int, alen: Int) {{ if alen != 6 {{ return 0 }}; if src[a] != 114 {{ return 0 }}; if src[a + 1] != 101 {{ return 0 }}; if src[a + 2] != 116 {{ return 0 }}; if src[a + 3] != 117 {{ return 0 }}; if src[a + 4] != 114 {{ return 0 }}; if src[a + 5] != 110 {{ return 0 }}; return 1 }}; struct Token {{ kind, value, nstart, nlen }}; struct Var {{ nstart, nlen, value }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c == 32 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v, nstart: st, nlen: i - st }}) }} else if is_alpha(c) == 1 {{ let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_alpha(src[i]) == 1 {{ i = i + 1 }} else {{ go = false }} }}; let ln = i - st; if kw_let(src, st, ln) == 1 {{ out.push(Token {{ kind: 7, value: 0, nstart: st, nlen: ln }}) }} else if kw_ret(src, st, ln) == 1 {{ out.push(Token {{ kind: 8, value: 0, nstart: st, nlen: ln }}) }} else {{ out.push(Token {{ kind: 1, value: 0, nstart: st, nlen: ln }}) }} }} else if c == 43 {{ out.push(Token {{ kind: 4, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 61 {{ out.push(Token {{ kind: 5, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 59 {{ out.push(Token {{ kind: 6, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn eval_factor(toks: List<Token>, i: Int, src: Str, vars: List<Var>) {{ let t = toks[i]; if t.kind == 0 {{ return t.value }}; if t.kind == 1 {{ return lookup(vars, src, t.nstart, t.nlen) }}; return 0 }}; fn eval_term(toks: List<Token>, i: Int, n: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rf = eval_factor(toks, i + 1, src, vars); return eval_term(toks, i + 2, n, src, vars, acc * rf) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, stop: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let op = toks[i]; if op.kind == 4 {{ let rf = eval_factor(toks, i + 1, src, vars); let term = eval_term(toks, i + 2, stop, src, vars, rf); let after = skip_term(toks, i + 2, stop); return eval_fold(toks, after, stop, src, vars, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, i: Int, stop: Int, src: Str, vars: List<Var>) {{ if i >= stop {{ return 0 }}; let first = eval_factor(toks, i, src, vars); let acc = eval_term(toks, i + 1, stop, src, vars, first); let after = skip_term(toks, i + 1, stop); return eval_fold(toks, after, stop, src, vars, acc) }}; fn find_semi(toks: List<Token>, i: Int, n: Int) {{ let mut j = i; while j < n {{ let t = toks[j]; if t.kind == 6 {{ return j }}; j = j + 1 }}; return n }}; fn run_program(src: Str) {{ let toks = list(); tokenize(src, toks); let n = toks.len; let vars = list(); let mut result = 0; let mut i = 0; while i < n {{ let t = toks[i]; if t.kind == 7 {{ let name = toks[i + 1]; let stop = find_semi(toks, i + 3, n); let val = eval_expr(toks, i + 3, stop, src, vars); vars.push(Var {{ nstart: name.nstart, nlen: name.nlen, value: val }}); i = stop + 1 }} else if t.kind == 8 {{ let stop = find_semi(toks, i + 1, n); result = eval_expr(toks, i + 1, stop, src, vars); i = stop + 1 }} else {{ i = i + 1 }} }}; return result }};'
# let x = 5; return x + 3   -> 8
check "$FP12BB_C fn run() {{ return run_program(\`let x = 5; return x + 3;\`) }}; return run();" 8
# let a = 4; let b = 6; return a * b   -> 24 (two variables, multiplication)
check "$FP12BB_C fn run() {{ return run_program(\`let a = 4; let b = 6; return a * b;\`) }}; return run();" 24
# let x = 2; let y = x + 1; return x + y * 4  -> 14 (y references x; precedence)
check "$FP12BB_C fn run() {{ return run_program(\`let x = 2; let y = x + 1; return x + y * 4;\`) }}; return run();" 14

# --- FP12cc: functions + recursion (fixpoint3.nl, the 3rd tier) ---
# fixpoint3.nl adds multi-char function definitions/calls with recursion: a
# List<Fn> function table looked up by find_fn, and eval_call which binds the
# arguments into a FRESH per-call List<Var> scope and recursively evaluates the
# body (so a function can call itself). Verified: the function table + call
# dispatch + fresh-scope binding + recursion-with-fresh-scope.
# find_fn: locate a function in the List<Fn> table by name
check "struct Fn {{ nstart, nlen, p1s, p1l, bstart, bend }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn find_fn(fns: List<Fn>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = fns.len; while i < m {{ let f = fns[i]; if name_eq(src, f.nstart, f.nlen, qs, ql) == 1 {{ return i }}; i = i + 1 }}; return 0 - 1 }}; fn run() {{ let fns = list(); fns.push(Fn {{ nstart: 0, nlen: 6, p1s: 0, p1l: 0, bstart: 0, bend: 0 }}); fns.push(Fn {{ nstart: 7, nlen: 6, p1s: 0, p1l: 0, bstart: 0, bend: 0 }}); let src = \`double triple\`; return find_fn(fns, src, 7, 6) }}; return run();" 1
# eval_call: look up the fn, bind the arg into a fresh callee scope, eval via lookup
check "struct Fn {{ nstart, nlen, p1s, p1l, bstart, bend }}; struct Var {{ nstart, nlen, value }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 }}; fn find_fn(fns: List<Fn>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = fns.len; while i < m {{ let f = fns[i]; if name_eq(src, f.nstart, f.nlen, qs, ql) == 1 {{ return i }}; i = i + 1 }}; return 0 - 1 }}; fn eval_call(fns: List<Fn>, src: Str, qs: Int, ql: Int, arg: Int) {{ let idx = find_fn(fns, src, qs, ql); if idx < 0 {{ return 0 }}; let f = fns[idx]; let callee = list(); callee.push(Var {{ nstart: f.p1s, nlen: f.p1l, value: arg }}); return lookup(callee, src, f.p1s, f.p1l) * 2 }}; fn run() {{ let fns = list(); fns.push(Fn {{ nstart: 0, nlen: 6, p1s: 7, p1l: 1, bstart: 0, bend: 0 }}); let src = \`double x\`; return eval_call(fns, src, 0, 6, 5) }}; return run();" 10
# recursive eval with a fresh List<Var> scope per call -- factorial(5) = 120
check "struct Var {{ nstart, nlen, value }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 }}; fn eval_fac(src: Str, ns: Int, nl: Int, n: Int) {{ let scope = list(); scope.push(Var {{ nstart: ns, nlen: nl, value: n }}); let v = lookup(scope, src, ns, nl); if v <= 1 {{ return 1 }}; return v * eval_fac(src, ns, nl, v - 1) }}; fn run() {{ let src = \`n\`; return eval_fac(src, 0, 1, 5) }}; return run();" 120

# --- FP12dd: the COMPLETE function-language compiler end-to-end (fixpoint3.nl tier) ---
# run_program(src) tokenizes a function-language program (fn/return keywords +
# idents + nums + + * ( ) { } ;), scans the `fn` definitions into a List<Fn>
# function table (build_fns), finds the top-level `return <call>;`, and evaluates
# the call -- binding the argument into a fresh callee List<Var> scope and
# evaluating the body. The 3rd self-host tier (a function language) end to end.
FP12DD_C='fn is_alpha(c: Int) {{ if c >= 97 and c <= 122 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; fn kw_fn(src: Str, a: Int, alen: Int) {{ if alen != 2 {{ return 0 }}; if src[a] != 102 {{ return 0 }}; if src[a + 1] != 110 {{ return 0 }}; return 1 }}; fn kw_ret(src: Str, a: Int, alen: Int) {{ if alen != 6 {{ return 0 }}; if src[a] != 114 {{ return 0 }}; if src[a + 1] != 101 {{ return 0 }}; if src[a + 2] != 116 {{ return 0 }}; if src[a + 3] != 117 {{ return 0 }}; if src[a + 4] != 114 {{ return 0 }}; if src[a + 5] != 110 {{ return 0 }}; return 1 }}; struct Token {{ kind, value, nstart, nlen }}; struct Var {{ nstart, nlen, value }}; struct Fn {{ nstart, nlen, p1s, p1l, bstart, bend }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 }}; fn find_fn(fns: List<Fn>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = fns.len; while i < m {{ let f = fns[i]; if name_eq(src, f.nstart, f.nlen, qs, ql) == 1 {{ return i }}; i = i + 1 }}; return 0 - 1 }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c == 32 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v, nstart: st, nlen: i - st }}) }} else if is_alpha(c) == 1 {{ let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_alpha(src[i]) == 1 {{ i = i + 1 }} else {{ go = false }} }}; let ln = i - st; if kw_fn(src, st, ln) == 1 {{ out.push(Token {{ kind: 13, value: 0, nstart: st, nlen: ln }}) }} else if kw_ret(src, st, ln) == 1 {{ out.push(Token {{ kind: 8, value: 0, nstart: st, nlen: ln }}) }} else {{ out.push(Token {{ kind: 1, value: 0, nstart: st, nlen: ln }}) }} }} else if c == 43 {{ out.push(Token {{ kind: 4, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 40 {{ out.push(Token {{ kind: 9, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 41 {{ out.push(Token {{ kind: 10, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 123 {{ out.push(Token {{ kind: 11, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 125 {{ out.push(Token {{ kind: 12, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 59 {{ out.push(Token {{ kind: 6, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn build_fns(toks: List<Token>, n: Int, fns: List<Fn>) {{ let mut i = 0; while i < n {{ let t = toks[i]; if t.kind == 13 {{ let nt = toks[i + 1]; let p1 = toks[i + 3]; let bstart = i + 6; let mut be = bstart; let mut go = true; while go {{ if be >= n {{ go = false }} else {{ let bt = toks[be]; if bt.kind == 12 {{ go = false }} else {{ be = be + 1 }} }} }}; fns.push(Fn {{ nstart: nt.nstart, nlen: nt.nlen, p1s: p1.nstart, p1l: p1.nlen, bstart: bstart, bend: be }}); i = be + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn eval_factor(toks: List<Token>, i: Int, src: Str, vars: List<Var>) {{ let t = toks[i]; if t.kind == 0 {{ return t.value }}; if t.kind == 1 {{ return lookup(vars, src, t.nstart, t.nlen) }}; return 0 }}; fn eval_term(toks: List<Token>, i: Int, n: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rf = eval_factor(toks, i + 1, src, vars); return eval_term(toks, i + 2, n, src, vars, acc * rf) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, stop: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let op = toks[i]; if op.kind == 4 {{ let rf = eval_factor(toks, i + 1, src, vars); let term = eval_term(toks, i + 2, stop, src, vars, rf); let after = skip_term(toks, i + 2, stop); return eval_fold(toks, after, stop, src, vars, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, i: Int, stop: Int, src: Str, vars: List<Var>) {{ if i >= stop {{ return 0 }}; let first = eval_factor(toks, i, src, vars); let acc = eval_term(toks, i + 1, stop, src, vars, first); let after = skip_term(toks, i + 1, stop); return eval_fold(toks, after, stop, src, vars, acc) }}; fn find_semi(toks: List<Token>, i: Int, n: Int) {{ let mut j = i; while j < n {{ let t = toks[j]; if t.kind == 6 {{ return j }}; j = j + 1 }}; return n }}; fn eval_call(toks: List<Token>, fns: List<Fn>, src: Str, i: Int, vars: List<Var>) {{ let nt = toks[i]; let idx = find_fn(fns, src, nt.nstart, nt.nlen); if idx < 0 {{ return 0 }}; let f = fns[idx]; let astop = find_semi(toks, i + 2, 999); let arg = eval_expr(toks, i + 2, astop, src, vars); let callee = list(); callee.push(Var {{ nstart: f.p1s, nlen: f.p1l, value: arg }}); return eval_expr(toks, f.bstart + 1, f.bend, src, callee) }}; fn run_program(src: Str) {{ let toks = list(); tokenize(src, toks); let n = toks.len; let fns = list(); build_fns(toks, n, fns); let empty = list(); let mut result = 0; let mut i = 0; while i < n {{ let t = toks[i]; if t.kind == 13 {{ let mut be = i + 6; let mut go = true; while go {{ if be >= n {{ go = false }} else {{ let bt = toks[be]; if bt.kind == 12 {{ go = false }} else {{ be = be + 1 }} }} }}; i = be + 1 }} else if t.kind == 8 {{ result = eval_call(toks, fns, src, i + 1, empty); let stop = find_semi(toks, i + 1, n); i = stop + 1 }} else {{ i = i + 1 }} }}; return result }};'
# fn double(x) { return x * 2 } return double(21);  -> 42
check "$FP12DD_C fn run() {{ return run_program(\`fn double(x) {{ return x * 2 }} return double(21);\`) }}; return run();" 42
# fn sq(x) { return x * x } return sq(7);  -> 49
check "$FP12DD_C fn run() {{ return run_program(\`fn sq(x) {{ return x * x }} return sq(7);\`) }}; return run();" 49
# two-function table, calling the second function -> 42 (build_fns scans both; find_fn dispatches)
check "$FP12DD_C fn run() {{ return run_program(\`fn double(x) {{ return x * 2 }} fn triple(y) {{ return y * 3 }} return triple(14);\`) }}; return run();" 42

# --- FP12ee: recursive function language (if-expression + recursion) ---
# The 3rd self-host tier WITH recursion: a function body is
# return-if-cond-then-a-else-b, and eval_value evaluates the if-expression
# (splitting on then/else, comparing, picking a branch); eval_call recurses
# through the body with a fresh scope each call. Compiles source-level
# recursive functions (factorial, sum) end to end.
FP12EE_C='fn is_alpha(c: Int) {{ if c >= 97 and c <= 122 {{ return 1 }}; return 0 }}; fn is_digit(c: Int) {{ if c >= 48 and c <= 57 {{ return 1 }}; return 0 }}; fn kw_fn(src: Str, a: Int, alen: Int) {{ if alen != 2 {{ return 0 }}; if src[a] != 102 {{ return 0 }}; if src[a + 1] != 110 {{ return 0 }}; return 1 }}; fn kw_ret(src: Str, a: Int, alen: Int) {{ if alen != 6 {{ return 0 }}; if src[a] != 114 {{ return 0 }}; return 1 }}; fn kw_if(src: Str, a: Int, alen: Int) {{ if alen != 2 {{ return 0 }}; if src[a] != 105 {{ return 0 }}; if src[a + 1] != 102 {{ return 0 }}; return 1 }}; fn kw_then(src: Str, a: Int, alen: Int) {{ if alen != 4 {{ return 0 }}; if src[a] != 116 {{ return 0 }}; if src[a + 1] != 104 {{ return 0 }}; return 1 }}; fn kw_else(src: Str, a: Int, alen: Int) {{ if alen != 4 {{ return 0 }}; if src[a] != 101 {{ return 0 }}; if src[a + 1] != 108 {{ return 0 }}; return 1 }}; struct Token {{ kind, value, nstart, nlen }}; struct Var {{ nstart, nlen, value }}; struct Fn {{ nstart, nlen, p1s, p1l, bstart, bend }}; fn name_eq(src: Str, a: Int, alen: Int, b: Int, blen: Int) {{ if alen != blen {{ return 0 }}; let mut k = 0; while k < alen {{ if src[a + k] != src[b + k] {{ return 0 }}; k = k + 1 }}; return 1 }}; fn lookup(vars: List<Var>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = vars.len; while i < m {{ let v = vars[i]; if name_eq(src, v.nstart, v.nlen, qs, ql) == 1 {{ return v.value }}; i = i + 1 }}; return 0 }}; fn find_fn(fns: List<Fn>, src: Str, qs: Int, ql: Int) {{ let mut i = 0; let m = fns.len; while i < m {{ let f = fns[i]; if name_eq(src, f.nstart, f.nlen, qs, ql) == 1 {{ return i }}; i = i + 1 }}; return 0 - 1 }}; fn find_kind(toks: List<Token>, i: Int, stop: Int, k: Int) {{ let mut j = i; while j < stop {{ let t = toks[j]; if t.kind == k {{ return j }}; j = j + 1 }}; return stop }}; fn tokenize(src: Str, out: List<Token>) {{ let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c == 32 {{ i = i + 1 }} else if is_digit(c) == 1 {{ let mut v = 0; let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_digit(src[i]) == 1 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; out.push(Token {{ kind: 0, value: v, nstart: st, nlen: i - st }}) }} else if is_alpha(c) == 1 {{ let st = i; let mut go = true; while go {{ if i >= n {{ go = false }} else if is_alpha(src[i]) == 1 {{ i = i + 1 }} else {{ go = false }} }}; let ln = i - st; if kw_fn(src, st, ln) == 1 {{ out.push(Token {{ kind: 13, value: 0, nstart: st, nlen: ln }}) }} else if kw_ret(src, st, ln) == 1 {{ out.push(Token {{ kind: 8, value: 0, nstart: st, nlen: ln }}) }} else if kw_if(src, st, ln) == 1 {{ out.push(Token {{ kind: 15, value: 0, nstart: st, nlen: ln }}) }} else if kw_then(src, st, ln) == 1 {{ out.push(Token {{ kind: 16, value: 0, nstart: st, nlen: ln }}) }} else if kw_else(src, st, ln) == 1 {{ out.push(Token {{ kind: 17, value: 0, nstart: st, nlen: ln }}) }} else {{ out.push(Token {{ kind: 1, value: 0, nstart: st, nlen: ln }}) }} }} else if c == 43 {{ out.push(Token {{ kind: 4, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 42 {{ out.push(Token {{ kind: 2, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 45 {{ out.push(Token {{ kind: 3, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 40 {{ out.push(Token {{ kind: 9, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 41 {{ out.push(Token {{ kind: 10, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 123 {{ out.push(Token {{ kind: 11, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 125 {{ out.push(Token {{ kind: 12, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 59 {{ out.push(Token {{ kind: 6, value: 0, nstart: 0, nlen: 0 }}); i = i + 1 }} else if c == 60 {{ if src[i + 1] == 61 {{ out.push(Token {{ kind: 29, value: 0, nstart: 0, nlen: 0 }}); i = i + 2 }} else {{ i = i + 1 }} }} else {{ i = i + 1 }} }}; return 0 }}; fn build_fns(toks: List<Token>, n: Int, fns: List<Fn>) {{ let mut i = 0; while i < n {{ let t = toks[i]; if t.kind == 13 {{ let nt = toks[i + 1]; let p1 = toks[i + 3]; let bstart = i + 6; let mut be = bstart; let mut go = true; while go {{ if be >= n {{ go = false }} else {{ let bt = toks[be]; if bt.kind == 12 {{ go = false }} else {{ be = be + 1 }} }} }}; fns.push(Fn {{ nstart: nt.nstart, nlen: nt.nlen, p1s: p1.nstart, p1l: p1.nlen, bstart: bstart, bend: be }}); i = be + 1 }} else {{ i = i + 1 }} }}; return 0 }}; fn arg_end(toks: List<Token>, i: Int, stop: Int) {{ let mut j = i; let mut depth = 0; while j < stop {{ let t = toks[j]; if t.kind == 9 {{ depth = depth + 1 }} else if t.kind == 10 {{ if depth == 0 {{ return j }}; depth = depth - 1 }}; j = j + 1 }}; return stop }}; fn skip_factor(toks: List<Token>, i: Int, stop: Int) {{ let t = toks[i]; if t.kind == 1 {{ let nx = toks[i + 1]; if nx.kind == 9 {{ let e = arg_end(toks, i + 2, stop); return e + 1 }} }}; return i + 1 }}; fn eval_factor(toks: List<Token>, fns: List<Fn>, i: Int, src: Str, vars: List<Var>) {{ let t = toks[i]; if t.kind == 0 {{ return t.value }}; if t.kind == 1 {{ let nx = toks[i + 1]; if nx.kind == 9 {{ return eval_call(toks, fns, src, i, vars) }}; return lookup(vars, src, t.nstart, t.nlen) }}; return 0 }}; fn eval_term(toks: List<Token>, fns: List<Fn>, i: Int, stop: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rf = eval_factor(toks, fns, i + 1, src, vars); let after = skip_factor(toks, i + 1, stop); return eval_term(toks, fns, after, stop, src, vars, acc * rf) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, stop: Int) {{ if i >= stop {{ return stop }}; let t = toks[i]; if t.kind == 2 {{ let after = skip_factor(toks, i + 1, stop); return skip_term(toks, after, stop) }}; return i }}; fn eval_fold(toks: List<Token>, fns: List<Fn>, i: Int, stop: Int, src: Str, vars: List<Var>, acc: Int) {{ if i >= stop {{ return acc }}; let op = toks[i]; if op.kind == 4 {{ let rf = eval_factor(toks, fns, i + 1, src, vars); let af = skip_factor(toks, i + 1, stop); let term = eval_term(toks, fns, i + 1, stop, src, vars, rf); let after = skip_term(toks, af, stop); return eval_fold(toks, fns, after, stop, src, vars, acc + term) }}; if op.kind == 3 {{ let rf = eval_factor(toks, fns, i + 1, src, vars); let af = skip_factor(toks, i + 1, stop); let term = eval_term(toks, fns, i + 1, stop, src, vars, rf); let after = skip_term(toks, af, stop); return eval_fold(toks, fns, after, stop, src, vars, acc - term) }}; return acc }}; fn eval_expr(toks: List<Token>, fns: List<Fn>, i: Int, stop: Int, src: Str, vars: List<Var>) {{ if i >= stop {{ return 0 }}; let first = eval_factor(toks, fns, i, src, vars); let af = skip_factor(toks, i, stop); let acc = eval_term(toks, fns, af, stop, src, vars, first); let after = skip_term(toks, af, stop); return eval_fold(toks, fns, after, stop, src, vars, acc) }}; fn eval_value(toks: List<Token>, fns: List<Fn>, src: Str, vars: List<Var>, i: Int, stop: Int) {{ let t = toks[i]; if t.kind == 15 {{ let then_pos = find_kind(toks, i + 1, stop, 16); let else_pos = find_kind(toks, then_pos + 1, stop, 17); let cmp = find_kind(toks, i + 1, then_pos, 29); let lhs = eval_expr(toks, fns, i + 1, cmp, src, vars); let rhs = eval_expr(toks, fns, cmp + 1, then_pos, src, vars); let mut cond = 0; if lhs <= rhs {{ cond = 1 }}; if cond == 1 {{ return eval_value(toks, fns, src, vars, then_pos + 1, else_pos) }}; return eval_value(toks, fns, src, vars, else_pos + 1, stop) }}; return eval_expr(toks, fns, i, stop, src, vars) }}; fn find_semi(toks: List<Token>, i: Int, n: Int) {{ let mut j = i; while j < n {{ let t = toks[j]; if t.kind == 6 {{ return j }}; j = j + 1 }}; return n }}; fn eval_call(toks: List<Token>, fns: List<Fn>, src: Str, i: Int, vars: List<Var>) {{ let nt = toks[i]; let idx = find_fn(fns, src, nt.nstart, nt.nlen); if idx < 0 {{ return 0 }}; let f = fns[idx]; let aend = arg_end(toks, i + 2, 999); let arg = eval_expr(toks, fns, i + 2, aend, src, vars); let callee = list(); callee.push(Var {{ nstart: f.p1s, nlen: f.p1l, value: arg }}); return eval_value(toks, fns, src, callee, f.bstart + 1, f.bend) }}; fn run_program(src: Str) {{ let toks = list(); tokenize(src, toks); let n = toks.len; let fns = list(); build_fns(toks, n, fns); let empty = list(); let mut result = 0; let mut i = 0; while i < n {{ let t = toks[i]; if t.kind == 13 {{ let mut be = i + 6; let mut go = true; while go {{ if be >= n {{ go = false }} else {{ let bt = toks[be]; if bt.kind == 12 {{ go = false }} else {{ be = be + 1 }} }} }}; i = be + 1 }} else if t.kind == 8 {{ result = eval_call(toks, fns, src, i + 1, empty); let stop = find_semi(toks, i + 1, n); i = stop + 1 }} else {{ i = i + 1 }} }}; return result }};'
check "$FP12EE_C fn run() {{ return run_program(\`fn fac(n) {{ return if n <= 1 then 1 else n * fac(n - 1) }} return fac(5);\`) }}; return run();" 120
check "$FP12EE_C fn run() {{ return run_program(\`fn fac(n) {{ return if n <= 1 then 1 else n * fac(n - 1) }} return fac(4);\`) }}; return run();" 24
check "$FP12EE_C fn run() {{ return run_program(\`fn sum(n) {{ return if n <= 0 then 0 else n + sum(n - 1) }} return sum(10);\`) }}; return run();" 55

# --- FP12ff: `.len` on a List-of-structs (was misread as a struct-field GEP) ---
# A List-of-structs var carries the element struct type in its slot's sty, so
# `toks.len` wrongly entered the struct-field-read branch -> field_index("len")
# = -1 -> GEP [nf x i64] index -1 = garbage. Fix: the struct-field branch now
# requires a scalar struct var (is_arr != 2); a List var's `.X` is always `.len`.
# (Values kept <= 255 to avoid 8-bit exit-code truncation.)
# toks.len * 10 + toks[0].value + toks[2].value + toks[4].value = 50 + 12+3+4 = 69
check "struct Token {{ kind, value }}; fn run() {{ let toks = list(); toks.push(Token {{ kind: 0, value: 12 }}); toks.push(Token {{ kind: 1, value: 0 }}); toks.push(Token {{ kind: 0, value: 3 }}); toks.push(Token {{ kind: 1, value: 0 }}); toks.push(Token {{ kind: 0, value: 4 }}); return toks.len * 10 + toks[0].value + toks[2].value + toks[4].value }}; return run();" 69
# toks.len * 30 (List-of-structs length in a multiplicative term) = 150
check "struct Token {{ kind, value }}; fn run() {{ let toks = list(); toks.push(Token {{ kind: 0, value: 1 }}); toks.push(Token {{ kind: 0, value: 2 }}); toks.push(Token {{ kind: 0, value: 3 }}); toks.push(Token {{ kind: 0, value: 4 }}); toks.push(Token {{ kind: 0, value: 5 }}); return toks.len * 30 }}; return run();" 150

# --- FP12gg: if/else-if chains where ALL branches return (dead merge block) ---
# An `if {return} else if {return} else {return}` left an empty trailing merge
# block (no statement follows -> no terminator -> invalid IR: "expected
# instruction opcode"). Fix: block_returns() detects when both branches return,
# so the if-handler marks the dead merge block with `unreachable`.
# 2-way all-return
check "fn f(c: Int) {{ if c == 0 {{ return 7 }} else {{ return 9 }} }}; fn run() {{ return f(0) + f(5) }}; return run();" 16
# 3-way all-return (else-if + else, every branch returns)
check "fn f(c: Int) {{ if c == 0 {{ return 1 }} else if c == 1 {{ return 10 }} else {{ return 100 }} }}; fn run() {{ return f(0) + f(1) + f(2) }}; return run();" 111
# 4-way all-return (two else-ifs + else)
check "fn f(c: Int) {{ if c == 0 {{ return 1 }} else if c == 1 {{ return 10 }} else if c == 2 {{ return 50 }} else {{ return 100 }} }}; fn run() {{ return f(0) + f(1) + f(2) + f(3) }}; return run();" 161
# regression: same chain with assignment (merge IS reachable -- must still work)
check "fn f(c: Int) {{ let mut r = 0; if c == 0 {{ r = 1 }} else if c == 1 {{ r = 10 }} else if c == 2 {{ r = 50 }} else {{ r = 100 }}; return r }}; fn run() {{ return f(0) + f(1) + f(2) + f(3) }}; return run();" 161

# --- FP12hh: `-> List` direct return (fixpoint.nl's original tokenize shape) ---
# `fn build() -> List<T> { let xs = list(); ...; return xs }` compiles via a
# hidden out-param: build gets a trailing `i64* %a<npar>`, `return xs` copies xs's
# buffer into it, and the caller `let ys = build()` allocates ys's buffer and
# passes it. Restores fixpoint.nl's `fn tokenize(src) -> List<Token>` (gap #4).
# scalar List return + index
check "fn build() -> List<Int> {{ let xs = list(); xs.push(10); xs.push(20); xs.push(12); return xs }}; fn run() {{ let ys = build(); return ys[0] + ys[1] + ys[2] }}; return run();" 42
# returned list length survives the copy
check "fn build() -> List<Int> {{ let xs = list(); xs.push(1); xs.push(2); xs.push(3); xs.push(4); return xs }}; fn run() {{ let ys = build(); return ys.len * 10 + ys[3] }}; return run();" 44
# List-of-structs return + field read
check "struct Tok {{ kind, val }}; fn build() -> List<Tok> {{ let xs = list(); xs.push(Tok {{ kind: 1, val: 30 }}); xs.push(Tok {{ kind: 2, val: 12 }}); return xs }}; fn run() {{ let ys = build(); return ys[0].val + ys[1].val }}; return run();" 42
# retlist with an argument
check "fn make(n: Int) -> List<Int> {{ let xs = list(); xs.push(n); xs.push(n + 1); return xs }}; fn run() {{ let ys = make(20); return ys[0] + ys[1] }}; return run();" 41
# fixpoint.nl's original shape: tokenize(src) -> List<Token>, then consume
check "struct Token {{ kind, value }}; fn tokenize(src: Str) -> List<Token> {{ let toks = list(); let mut i = 0; while i < src.len() {{ let c = src[i]; if c >= 48 and c <= 57 {{ toks.push(Token {{ kind: 1, value: c - 48 }}) }} else {{ toks.push(Token {{ kind: 2, value: 0 }}) }}; i = i + 1 }}; return toks }}; fn run() {{ let toks = tokenize(\`1a2a3\`); let mut s = 0; let mut j = 0; while j < toks.len {{ if toks[j].kind == 1 {{ s = s + toks[j].value }}; j = j + 1 }}; return s }}; return run();" 6

# --- FP12ii: fixpoint.nl's ORIGINAL shape end-to-end (tokenize -> List<Token>) ---
# With `-> List` direct return working, the whole arithmetic compiler runs in
# fixpoint.nl's verbatim shape: tokenize(src) RETURNS a List<Token> (not an
# out-param), and the recursive evaluator consumes it -- no workaround rewrite.
check "struct Token {{ kind, value }}; fn tokenize(src: Str) -> List<Token> {{ let toks = list(); let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c >= 48 and c <= 57 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if src[i] >= 48 and src[i] <= 57 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; toks.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ toks.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ toks.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return toks }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_fold(toks, after, n, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_fold(toks, after, n, acc) }}; fn run() {{ let toks = tokenize(\`12 + 3 * 4\`); return eval_expr(toks, toks.len) }}; return run();" 24
check "struct Token {{ kind, value }}; fn tokenize(src: Str) -> List<Token> {{ let toks = list(); let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c >= 48 and c <= 57 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if src[i] >= 48 and src[i] <= 57 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; toks.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ toks.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ toks.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return toks }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_fold(toks, after, n, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_fold(toks, after, n, acc) }}; fn run() {{ let toks = tokenize(\`2 + 3 * 4\`); return eval_expr(toks, toks.len) }}; return run();" 14
check "struct Token {{ kind, value }}; fn tokenize(src: Str) -> List<Token> {{ let toks = list(); let n = src.len(); let mut i = 0; while i < n {{ let c = src[i]; if c >= 48 and c <= 57 {{ let mut v = 0; let mut go = true; while go {{ if i >= n {{ go = false }} else if src[i] >= 48 and src[i] <= 57 {{ v = v * 10 + (src[i] - 48); i = i + 1 }} else {{ go = false }} }}; toks.push(Token {{ kind: 0, value: v }}) }} else if c == 43 {{ toks.push(Token {{ kind: 1, value: 0 }}); i = i + 1 }} else if c == 42 {{ toks.push(Token {{ kind: 2, value: 0 }}); i = i + 1 }} else {{ i = i + 1 }} }}; return toks }}; fn eval_term(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let t = toks[i]; if t.kind == 2 {{ let rhs = toks[i + 1]; return eval_term(toks, i + 2, n, acc * rhs.value) }}; return acc }}; fn skip_term(toks: List<Token>, i: Int, n: Int) {{ if i >= n {{ return n }}; let t = toks[i]; if t.kind == 2 {{ return skip_term(toks, i + 2, n) }}; return i }}; fn eval_fold(toks: List<Token>, i: Int, n: Int, acc: Int) {{ if i >= n {{ return acc }}; let op = toks[i]; if op.kind == 1 {{ let rs = i + 1; let rf = toks[rs]; let term = eval_term(toks, rs + 1, n, rf.value); let after = skip_term(toks, rs + 1, n); return eval_fold(toks, after, n, acc + term) }}; return acc }}; fn eval_expr(toks: List<Token>, n: Int) {{ if n <= 0 {{ return 0 }}; let first = toks[0]; let acc = eval_term(toks, 1, n, first.value); let after = skip_term(toks, 1, n); return eval_fold(toks, after, n, acc) }}; fn run() {{ let toks = tokenize(\`10 + 20 + 30\`); return eval_expr(toks, toks.len) }}; return run();" 60

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

# --- FP12jj: `for <var> in lo..hi { body }` (.. exclusive, ..= inclusive) ---
# The last general-nl loop construct: tokenized (for=34, in=35, ..=36, ..==37)
# and desugared in gen_stmts to an induction-variable while-loop on the var's
# slot. Covers exclusive/inclusive bounds, body arithmetic, function-param hi
# bound, push into a List, if-in-body, and nested for (inner depends on outer).
check "let mut s = 0; for i in 0..5 {{ s = s + i }}; return s;" 10
check "let mut s = 0; for i in 0..=5 {{ s = s + i }}; return s;" 15
check "let mut s = 0; for i in 1..4 {{ s = s + i }}; return s;" 6
check "let mut s = 0; for i in 0..3 {{ s = s + i * 2 }}; return s;" 6
check "fn sumto(n) {{ let mut s = 0; for i in 0..n {{ s = s + i }}; return s }}; return sumto(5);" 10
check "let mut c = 0; for i in 0..3 {{ for j in 0..3 {{ c = c + 1 }} }}; return c;" 9
check "let mut s = 0; for i in 0..6 {{ if i > 2 {{ s = s + i }} }}; return s;" 12
check "let xs = list(); for i in 0..5 {{ xs.push(i * 10) }}; let mut s = 0; let mut j = 0; while j < xs.len {{ s = s + xs[j]; j = j + 1 }}; return s;" 100
check "let mut s = 0; for i in 1..=3 {{ for j in 1..=i {{ s = s + 1 }} }}; return s;" 6
# Sanity: a for-loop emits an induction-variable loop (store init, loop label,
# icmp slt/sle against the bound, add 1 increment) -- desugared, not a builtin.
tmp="$(mktemp -d)"
PROG="let mut s = 0; for i in 0..5 {{ s = s + i }}; return s;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q "br label %loop" "$tmp/out.ll" && grep -q "icmp slt i64" "$tmp/out.ll" && grep -q "add i64" "$tmp/out.ll"; then
  echo "  PASS for-loop desugars to induction-variable loop (store/loop/icmp slt/add)";
else echo "  FAIL did not emit for-loop induction codegen"; cat "$tmp/out.ll"; fail=1; fi

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

# --- FP12kk: print(...) with interpolation (the self-host codegen emit stage) ---
# The transpiler rewrites print->puts; a brace-bearing literal routes to a printf
# call against a @.fmt<nstart> format global ({ident} -> %d/%s, % -> %%,
# trailing \n).
# Disambiguation: a lone `{` (e.g. the trailing brace of `define ... {`) is literal;
# only a valid `{ident}` interpolates. Interpolation braces are written {{var}} so
# the outer transpiler unescapes them to single braces reaching compile(). Note:
# $(...) in check_out strips the trailing newline, so single-line wants omit it.
check_out "print(\`hello world\`); return 0;" "hello world"
check_out "let v = 42; print(\`ret i64 {{v}}\`); return 0;" "ret i64 42"
check_out "let c = 3; let lv = 7; print(\`%t{{c}} = add i64 {{lv}}\`); return 0;" "%t3 = add i64 7"
check_out "let a = 1; let b = 2; let cc = 3; print(\`{{a}} {{b}} {{cc}}\`); return 0;" "1 2 3"
check_out "let op_s = \`mul\`; print(\`op={{op_s}}\`); return 0;" "op=mul"
check_out "fn emit(op_s: Str, value: Int) {{ print(\`  ret {{op_s}} {{value}}\`); return 0 }}; return emit(\`i64\`, 42);" "  ret i64 42"
check_out "let c = 3; let op_s = \`add\`; print(\`%t{{c}} = {{op_s}} i64 2, 4\`); return 0;" "%t3 = add i64 2, 4"
check_out "let mut i = 0; while i < 3 {{ print(\`line {{i}}\`); i = i + 1 }}; return 0;" "$(printf 'line 0\nline 1\nline 2')"
# the actual fixpoint.nl emit_ir codegen tail: a LONE literal `{` then an
# interpolation, proving literal-vs-interp disambiguation end to end.
check_out "fn emit(value) {{ print(\`define i64 @main() {{\`); print(\`  ret i64 {{value}}\`); print(\`}}\`); return 0 }}; return emit(99);" "$(printf 'define i64 @main() {\n  ret i64 99\n}')"
# Sanity: the interpolation path emits a printf call against a @.fmt format global.
tmp="$(mktemp -d)"
PROG="let v = 7; print(\`x={{v}}\`); return 0;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q '@.fmt' "$tmp/out.ll" && grep -q 'call i32 (i8\*, ...) @printf' "$tmp/out.ll"; then
  echo "  PASS print interpolation emits @.fmt global + printf call";
else echo "  FAIL did not emit printf interpolation"; cat "$tmp/out.ll"; fail=1; fi

# Sanity: string interpolation uses a %s format directive and an i8* vararg.
tmp="$(mktemp -d)"
PROG="let op_s = \`mul\`; print(\`op={{op_s}}\`); return 0;" python3 - "$SRC" "$tmp/c.nl" <<'PY'
import os, re, sys
src = open(sys.argv[1]).read()
src = re.sub(r'compile\("(?:[^"\\]|\\.)*"\)', 'compile("' + os.environ["PROG"] + '")', src, count=1)
open(sys.argv[2], "w").write(src)
PY
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1
"$tmp/c" > "$tmp/out.ll"
if grep -q 'c"op=%s\\0A\\00"' "$tmp/out.ll" && grep -q 'call i32 (i8\*, .*i8\* %t' "$tmp/out.ll"; then
  echo "  PASS print string interpolation emits %s + i8* vararg";
else echo "  FAIL did not emit string printf interpolation"; cat "$tmp/out.ll"; fail=1; fi

# --- FP12mm: source-file bootstrap smoke ---
# Real source files define their own `fn main()`. In that case fixpoint_full must
# not also synthesize a wrapper @main, or clang rejects duplicate definitions.
check "fn main() {{ return 42 }}" 42

# Compile the actual compiler/self/fixpoint.nl source file, normalized to the
# current compact self-host subset by tools/embed_self_source.py. The generated
# program is itself a tiny compiler; when run it emits LLVM IR with `ret i64 24`.
tmp="$(mktemp -d)"
python3 "$HERE/tools/embed_self_source.py" "$SRC" "$HERE/compiler/self/fixpoint.nl" "$tmp/c.nl" \
  || { echo "  FAIL source-file bootstrap: embed"; fail=1; }
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 \
  || { echo "  FAIL source-file bootstrap: compiler build"; fail=1; }
"$tmp/c" > "$tmp/source_compiler.ll"
main_count="$(grep -c '^define i64 @main()' "$tmp/source_compiler.ll" || true)"
if [ "$main_count" = "1" ]; then
  echo "  PASS source-file bootstrap emits a single @main";
else
  echo "  FAIL source-file bootstrap main count=$main_count"; cat "$tmp/source_compiler.ll"; fail=1
fi
clang -Wno-override-module -o "$tmp/source_compiler" "$tmp/source_compiler.ll" 2>/dev/null \
  || { echo "  FAIL source-file bootstrap: generated compiler IR invalid"; cat "$tmp/source_compiler.ll"; fail=1; }
"$tmp/source_compiler" > "$tmp/emitted.ll"
if grep -q 'ret i64 24' "$tmp/emitted.ll"; then
  echo "  PASS source-file bootstrap fixpoint.nl emits ret i64 24";
else
  echo "  FAIL source-file bootstrap emitted IR missing ret i64 24"; cat "$tmp/emitted.ll"; fail=1
fi
clang -Wno-override-module -o "$tmp/emitted_bin" "$tmp/emitted.ll" 2>/dev/null \
  || { echo "  FAIL source-file bootstrap: emitted IR invalid"; cat "$tmp/emitted.ll"; fail=1; }
"$tmp/emitted_bin"; got=$?
if [ "$got" = "24" ]; then
  echo "  PASS source-file bootstrap fixpoint.nl emitted IR runs (=24)";
else
  echo "  FAIL source-file bootstrap emitted binary got=$got want=24"; fail=1
fi

# Compile the actual compiler/self/fixpoint2.nl source file. This is the
# arithmetic+variables tier and exercises the real 10-param word_is helper.
tmp="$(mktemp -d)"
python3 "$HERE/tools/embed_self_source.py" "$SRC" "$HERE/compiler/self/fixpoint2.nl" "$tmp/c.nl" \
  || { echo "  FAIL source-file bootstrap fixpoint2.nl: embed"; fail=1; }
python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>/dev/null
( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$tmp/c.vais" -o "$tmp/c" ) >/dev/null 2>&1 \
  || { echo "  FAIL source-file bootstrap fixpoint2.nl: compiler build"; fail=1; }
"$tmp/c" > "$tmp/source_compiler.ll"
main_count="$(grep -c '^define i64 @main()' "$tmp/source_compiler.ll" || true)"
if [ "$main_count" = "1" ]; then
  echo "  PASS source-file bootstrap fixpoint2.nl emits a single @main";
else
  echo "  FAIL source-file bootstrap fixpoint2.nl main count=$main_count"; cat "$tmp/source_compiler.ll"; fail=1
fi
clang -Wno-override-module -o "$tmp/source_compiler" "$tmp/source_compiler.ll" 2>/dev/null \
  || { echo "  FAIL source-file bootstrap fixpoint2.nl: generated compiler IR invalid"; cat "$tmp/source_compiler.ll"; fail=1; }
"$tmp/source_compiler" > "$tmp/emitted.ll"
if grep -q 'ret i64 50' "$tmp/emitted.ll"; then
  echo "  PASS source-file bootstrap fixpoint2.nl emits ret i64 50";
else
  echo "  FAIL source-file bootstrap fixpoint2.nl emitted IR missing ret i64 50"; cat "$tmp/emitted.ll"; fail=1
fi
clang -Wno-override-module -o "$tmp/emitted_bin" "$tmp/emitted.ll" 2>/dev/null \
  || { echo "  FAIL source-file bootstrap fixpoint2.nl: emitted IR invalid"; cat "$tmp/emitted.ll"; fail=1; }
"$tmp/emitted_bin"; got=$?
if [ "$got" = "50" ]; then
  echo "  PASS source-file bootstrap fixpoint2.nl emitted IR runs (=50)";
else
  echo "  FAIL source-file bootstrap fixpoint2.nl emitted binary got=$got want=50"; fail=1
fi

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
