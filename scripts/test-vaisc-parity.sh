#!/usr/bin/env bash
# NV-C4 parity gate for the New Vais `vaisc` native path.
#
# The manifest records which examples are native-supported, bootstrap-only, or
# tracked. Native-supported entries must match both their `# expect:` value and
# the Legacy bootstrap oracle. Bootstrap-only entries must remain Legacy-green
# and be rejected by the native front. Tracked entries are allowed to fail
# natively, but the script fails if one starts passing so the manifest is updated.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
LEGACY_BUILD="$HERE/scripts/build.sh"
MANIFEST="${1:-$HERE/tools/vaisc-parity.tsv}"
SOURCE_ROOT="${VAISC_PARITY_ROOT:-$HERE}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

native_pass=0
bootstrap_pass=0
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

run_legacy() {
    local src="$1"
    local key="$2"
    local bin="$tmp/$key.legacy.bin"
    local log="$tmp/$key.legacy.build.log"
    if ! bash "$LEGACY_BUILD" "$src" -o "$bin" >"$log" 2>&1; then
        echo "build-fail:$(head -1 "$log" | cut -c1-100)"
        return 0
    fi
    "$bin" >"$tmp/$key.legacy.out" 2>"$tmp/$key.legacy.err"
    echo "run:$?"
}

native_front_rejects() {
    local src="$1"
    local key="$2"
    if "$VAISC" emit-ir "$src" -o "$tmp/$key.reject.ll" >"$tmp/$key.reject.out" 2>"$tmp/$key.reject.err"; then
        return 1
    fi
    grep -q "help:" "$tmp/$key.reject.err"
}

check_native_supported() {
    local src="$1"
    local expect="$2"
    local key="$3"
    local want=$((expect % 256))
    local native_result legacy_result
    native_result="$(run_native "$src" "$key")"
    legacy_result="$(run_legacy "$src" "$key")"
    if [ "$native_result" = "run:$want" ] && [ "$legacy_result" = "run:$want" ]; then
        echo "  PASS native-supported $src (= $want)"
        native_pass=$((native_pass + 1))
    else
        echo "  FAIL native-supported $src: native=$native_result legacy=$legacy_result want=run:$want"
        fail=1
    fi
}

check_bootstrap_only() {
    local src="$1"
    local expect="$2"
    local key="$3"
    local want=$((expect % 256))
    local legacy_result
    legacy_result="$(run_legacy "$src" "$key")"
    if [ "$legacy_result" != "run:$want" ]; then
        echo "  FAIL bootstrap-only $src: legacy=$legacy_result want=run:$want"
        fail=1
        return
    fi
    if native_front_rejects "$src" "$key"; then
        echo "  PASS bootstrap-only $src remains Legacy-green and native-rejected"
        bootstrap_pass=$((bootstrap_pass + 1))
    else
        echo "  FAIL bootstrap-only $src: native front no longer rejects; update parity manifest"
        fail=1
    fi
}

check_tracked() {
    local src="$1"
    local expect="$2"
    local key="$3"
    local want=$((expect % 256))
    local native_result legacy_result
    legacy_result="$(run_legacy "$src" "$key")"
    if [ "$legacy_result" != "run:$want" ]; then
        echo "  FAIL tracked $src: legacy=$legacy_result want=run:$want"
        fail=1
        return
    fi
    native_result="$(run_native "$src" "$key")"
    if [ "$native_result" = "run:$want" ]; then
        echo "  FAIL tracked $src now passes natively; promote it to native-supported"
        fail=1
    else
        echo "  PASS tracked $src stays recorded (native=$native_result)"
        tracked_pass=$((tracked_pass + 1))
    fi
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
        bootstrap-only) check_bootstrap_only "$src" "$expect" "$key" ;;
        tracked) check_tracked "$src" "$expect" "$key" ;;
        *)
            echo "  FAIL $rel: unknown parity status '$status'"
            fail=1
            ;;
    esac
done <"$MANIFEST"

echo ""
if [ "$fail" -eq 0 ]; then
    echo "RESULT: New Vais vaisc NV-C4 parity gate OK (native=$native_pass bootstrap=$bootstrap_pass tracked=$tracked_pass)"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
