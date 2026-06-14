#!/usr/bin/env bash
# Value-correctness test runner (P7b: compile != correct).
# For each release-subset `.vais` file whose first line is `# expect: N`,
# compile -> run, and check the exit code equals N (mod 256).
#
# Usage:  scripts/test.sh          # run release-subset annotated examples
#         scripts/test.sh c4       # run one (by basename)
# Exit 0 iff all pass. This is the safety net for all later work.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE_ROOT="${VAIS_TEST_ROOT:-$HERE}"
source "$HERE/scripts/vais-build-env.sh"

run_one() {
    local src="$1"
    local name; name="$(basename "${src%.vais}")"
    local expect; expect="$(sed -n '1s/^# expect:[[:space:]]*\([0-9]*\).*/\1/p' "$src")"
    [ -z "$expect" ] && return 2   # not annotated
    local tmp; tmp="$(mktemp -d)"
    local bin="$tmp/$name.bin"
    if ! vais_build "$src" -o "$bin" >"$tmp/build" 2>&1; then
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
    if [ -f "$SOURCE_ROOT/examples/$1.vais" ]; then files=("$SOURCE_ROOT/examples/$1.vais");
    else files=("$SOURCE_ROOT/compiler/self/$1.vais"); fi
else
    files=()
    while IFS= read -r rel; do
        files+=("$SOURCE_ROOT/$rel")
    done < <(awk -F '\t' '$1 !~ /^($|#)/ && $2 == "native-supported" { print $1 }' "$HERE/tools/vaisc-parity.tsv")
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
