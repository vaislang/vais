#!/usr/bin/env bash
# Value-correctness test runner (P7b: compile != correct).
# For each examples/*.$VAIS_TEST_EXT whose first line is `# expect: N`, transpile -> Vais ->
# compile -> run, and check the exit code equals N (mod 256).
#
# Usage:  scripts/test.sh          # run all annotated examples
#         scripts/test.sh c4       # run one (by basename)
# Exit 0 iff all pass. This is the safety net for all later work.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
TRANSPILER="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
SOURCE_ROOT="${VAIS_TEST_ROOT:-$HERE}"
SOURCE_EXT="${VAIS_TEST_EXT:-vais}"
source "$HERE/scripts/legacy-vaisc-env.sh"

run_one() {
    local src="$1"
    local name; name="$(basename "${src%.$SOURCE_EXT}")"
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
    if ! legacy_vaisc_build "$vais" -o "$bin" >"$tmp/build" 2>&1; then
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
    if [ -f "$SOURCE_ROOT/examples/$1.$SOURCE_EXT" ]; then files=("$SOURCE_ROOT/examples/$1.$SOURCE_EXT");
    else files=("$SOURCE_ROOT/compiler/self/$1.$SOURCE_EXT"); fi
else
    # value-correctness corpus = examples/ + self-host modules (compiler/self/)
    files=("$SOURCE_ROOT"/examples/*."$SOURCE_EXT" "$SOURCE_ROOT"/compiler/self/*."$SOURCE_EXT")
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
