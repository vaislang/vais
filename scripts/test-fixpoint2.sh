#!/usr/bin/env bash
# End-to-end check for the multi-char-identifier fixpoint compiler
# (compiler/self/fixpoint2.vais): tokenizes `let <name> = <expr>; ... return
# <expr>` into a List<Token> (identifiers carry their NAME as a source range),
# evaluates with a List<Var> symbol table looked up by source-name comparison,
# and emits LLVM IR. Resolves the single-letter-identifier limitation.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais-legacy/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint2.vais"
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
  legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >/dev/null 2>&1 \
    || { echo "  FAIL '$prog': build"; fail=1; return; }
  "$tmp/c" > "$tmp/out.ll"
  clang -Wno-override-module -o "$tmp/bin" "$tmp/out.ll" 2>/dev/null \
    || { echo "  FAIL '$prog': IR invalid"; fail=1; return; }
  "$tmp/bin"; local got=$?
  if [ "$got" = "$want" ]; then echo "  PASS '$prog' -> $got";
  else echo "  FAIL '$prog': got=$got want=$want"; fail=1; fi
}

# single multi-char var
check "let x = 5; return x;" 5
# real-world variable names
check "let total = 10; let count = total * 4; return total + count;" 50
check "let width = 4; let height = 5; return width * height;" 20
# multiple distinct names
check "let foo = 3; let bar = 7; return foo + bar;" 10
# dependency chain
check "let base = 10; let doubled = base * 2; let result = doubled + base; return result;" 30
# prefix-sharing names must NOT collide (foo vs food)
check "let foo = 1; let food = 100; return food;" 100
check "let foo = 1; let food = 100; return foo;" 1
# precedence + left-assoc with named vars
check "let a = 2; let b = 3; let c = a * b; return c + 1;" 7
check "let n = 20; return n - 5 - 3;" 12

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint2 compiler (multi-char identifiers) end-to-end OK" || echo "RESULT: FAILURES"
exit $fail
