#!/usr/bin/env bash
# End-to-end check for the STRING code generator (compiler/self/fixpoint_str.nl):
# compiles string literals + byte indexing + length into real LLVM IR. This is
# the capability the nl compiler needs to tokenize its own SOURCE: a string
# literal becomes a global [N x i8] constant; the variable is an i8* to its
# element 0; `s[i]` is getelementptr i8 + load i8 + zext i64; `s.len()` is the
# compile-time length N. Combined with `while`/assignment this scans a source
# string byte by byte — the core of a tokenizer.
#
# Verified by compiling the emitted IR with clang and checking the runtime
# value (exit codes are 8-bit, so test values stay <= 255).
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
TR="$HERE/compiler/transpiler/nl2vais.py"
SRC="$HERE/compiler/self/fixpoint_str.nl"
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

# byte index + compile-time length: s[1]='B'(66) + len 3 = 69
check "let s = \`ABC\`; return s[1] + s.len();" 69
# sum every byte of a string via a while scan: 'A'+'B'+'C' = 198 (tokenizer scan shape)
check "let s = \`ABC\`; let mut i = 0; let mut acc = 0; while i < s.len() {{ acc = acc + s[i]; i = i + 1 }}; return acc;" 198
# two string vars: length of each tracked independently. 'X'88+'Z'90+2+1 = 181
check "let a = \`XY\`; let b = \`Z\`; return a[0] + b[0] + a.len() + b.len();" 181
# length of a keyword-shaped literal (the tokenizer measures token lengths)
check "let s = \`return\`; return s.len();" 6
# index near both ends of a longer literal: 't'116 + 'e'101 = 217
check "let s = \`tokenize\`; return s[0] + s[7];" 217
# scan one string in a loop, then index a second (build+consume over two sources)
# 'a'97+'b'98 = 195, + 'X'88 = 283 -> 283 & 0xFF = 27 (8-bit exit), so subtract to fit
check "let a = \`ab\`; let b = \`XYZ\`; let mut i = 0; let mut acc = 0; while i < a.len() {{ acc = acc + a[i]; i = i + 1 }}; return acc + b[0] - 100;" 183
#   195 + 88 - 100 = 183

# Sanity: the emitted IR has a string global, an i8* alloca init'd to its
# element-0 pointer, and a byte load via GEP i8 + zext.
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
if grep -q 'private constant \[' "$tmp/out.ll" && grep -q 'alloca i8\*' "$tmp/out.ll" && grep -q 'getelementptr i8, i8\*' "$tmp/out.ll" && grep -q 'load i8,' "$tmp/out.ll" && grep -q 'zext i8' "$tmp/out.ll"; then
  echo "  PASS emits string global + i8* alloca + byte load (GEP i8 / load i8 / zext) [string codegen]";
else echo "  FAIL did not emit string codegen"; cat "$tmp/out.ll"; fail=1; fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint string codegen (literal global + s[i] byte load + s.len()) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
