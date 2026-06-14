#!/usr/bin/env bash
# NV-C4 parity gate for the Vais `vaisc` native path.
#
# The manifest records the release subset. Entries marked native-supported must
# match their `# expect:` value through scripts/vaisc.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
MANIFEST="${1:-$HERE/tools/vaisc-parity.tsv}"
SOURCE_ROOT="${VAISC_PARITY_ROOT:-$HERE}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

native_pass=0
full_pass=0
tracked_pass=0
fail=0

sanitize() {
    printf '%s' "$1" | tr '/.' '__'
}

expect_for() {
    sed -n '1s/^# expect:[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$1"
}

run_native() {
    local src="$1"
    local key="$2"
    local bin="$tmp/$key.native.bin"
    local ir="$tmp/$key.native.ll"
    local log="$tmp/$key.native.build.log"
    if ! "$VAISC" build "$src" -o "$bin" --ir-out "$ir" >"$log" 2>&1; then
        echo "build-fail:$(head -1 "$log" | cut -c1-100)"
        return 0
    fi
    "$bin" >"$tmp/$key.native.out" 2>"$tmp/$key.native.err"
    echo "run:$?"
}

check_native_supported() {
    local src="$1"
    local expect="$2"
    local key="$3"
    local want=$((expect % 256))
    local native_result
    native_result="$(run_native "$src" "$key")"
    if [ "$native_result" = "run:$want" ]; then
        echo "  PASS native-supported $src (= $want)"
        native_pass=$((native_pass + 1))
    else
        echo "  FAIL native-supported $src: native=$native_result want=run:$want"
        fail=1
    fi
}

check_full_only() {
    local src="$1"
    echo "  SKIP full-only $src"
    full_pass=$((full_pass + 1))
}

check_tracked() {
    local src="$1"
    echo "  SKIP tracked $src"
    tracked_pass=$((tracked_pass + 1))
}

if [ ! -f "$MANIFEST" ]; then
    echo "error: parity manifest not found: $MANIFEST" >&2
    exit 1
fi

while IFS=$'\t' read -r rel status note; do
    case "$rel" in
        ""|\#*) continue ;;
    esac
    src="$SOURCE_ROOT/$rel"
    if [ ! -f "$src" ]; then
        echo "  FAIL manifest path missing: $rel"
        fail=1
        continue
    fi
    expect="$(expect_for "$src")"
    if [ -z "$expect" ]; then
        echo "  FAIL manifest path has no # expect: $rel"
        fail=1
        continue
    fi
    key="$(sanitize "$rel")"
    case "$status" in
        native-supported) check_native_supported "$src" "$expect" "$key" ;;
        full-only) check_full_only "$src" "$expect" "$key" ;;
        tracked) check_tracked "$src" "$expect" "$key" ;;
        *)
            echo "  FAIL $rel: unknown parity status '$status'"
            fail=1
            ;;
    esac
done <"$MANIFEST"

echo ""
if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C4 parity gate OK (native=$native_pass full=$full_pass tracked=$tracked_pass)"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
