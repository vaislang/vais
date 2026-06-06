#!/usr/bin/env bash
# Value-correctness test runner (P7b: compile != correct).
# For each examples/*.nl whose first line is `# expect: N`, transpile -> Vais ->
# compile -> run, and check the exit code equals N (mod 256).
#
# Usage:  scripts/test.sh          # run all annotated examples
#         scripts/test.sh c4       # run one (by basename)
# Exit 0 iff all pass. This is the safety net for all later work.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
TRANSPILER="$HERE/compiler/transpiler/nl2vais.py"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"

run_one() {
    local src="$1"
    local name; name="$(basename "${src%.nl}")"
    local expect; expect="$(sed -n '1s/^# expect:[[:space:]]*\([0-9]*\).*/\1/p' "$src")"
    [ -z "$expect" ] && return 2   # not annotated
    local tmp; tmp="$(mktemp -d)"
    local vais="$tmp/$name.vais"
    if ! python3 "$TRANSPILER" "$src" > "$vais" 2>"$tmp/warn"; then
        echo "  FAIL $name: transpile error"; return 1
    fi
    if [ -s "$tmp/warn" ]; then
        echo "  SKIP $name: $(head -1 "$tmp/warn" | sed 's/.*UNSUPPORTED//' | cut -c1-40)"; return 3
    fi
    local bin="$tmp/$name.bin"
    if ! ( cd "$VAIS_ROOT" && rm -rf /tmp/.vais-cache && vaisc build "$vais" -o "$bin" ) >"$tmp/build" 2>&1; then
        echo "  FAIL $name: build error ($(grep -m1 'error' "$tmp/build" | cut -c1-40))"; return 1
    fi
    "$bin"; local got=$?
    local em=$(( expect % 256 ))
    if [ "$got" = "$em" ]; then
        echo "  PASS $name (=$got)"; return 0
    else
        echo "  FAIL $name: got=$got expect=$em"; return 1
    fi
}

pass=0; fail=0; skip=0
if [ $# -ge 1 ]; then
    # allow a basename from either examples/ or compiler/self/
    if [ -f "$HERE/examples/$1.nl" ]; then files=("$HERE/examples/$1.nl");
    else files=("$HERE/compiler/self/$1.nl"); fi
else
    # value-correctness corpus = examples/ + self-host modules (compiler/self/)
    files=("$HERE"/examples/*.nl "$HERE"/compiler/self/*.nl)
fi
for f in "${files[@]}"; do
    [ -f "$f" ] || continue
    run_one "$f"; rc=$?
    case $rc in
        0) pass=$((pass+1));;
        1) fail=$((fail+1));;
        3) skip=$((skip+1));;
    esac
done
echo ""
echo "RESULT: pass=$pass fail=$fail skip=$skip"
[ "$fail" -eq 0 ]
