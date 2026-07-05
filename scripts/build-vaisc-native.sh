#!/usr/bin/env bash
# Build the native Vais compiler driver.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
CLANG="${CLANG:-clang}"
OUT="${1:-$HERE/build/vaisc}"
BUILD_DIR="$(dirname "$OUT")"
CORE="$HERE/compiler/self/vaisc_core.ll"
DRIVER="$HERE/tools/vaisc_native.c"
CORE_NATIVE="$BUILD_DIR/vaisc_core_native.$(basename "$OUT").$$.ll"

mkdir -p "$BUILD_DIR"
trap 'rm -f "$CORE_NATIVE"' EXIT

main_count="$(grep -Ec '^define i64 @main\(\)( #[0-9]+)? \{$' "$CORE" || true)"
if [ "$main_count" != "1" ]; then
    echo "error: expected one @main in $CORE, found $main_count" >&2
    exit 1
fi

awk '
    BEGIN { replaced = 0 }
    /^define i64 @main\(\)( #[0-9]+)? \{$/ && replaced == 0 {
        sub("@main", "@vais_selftest_main")
        print
        replaced = 1
        next
    }
    { print }
    END {
        if (replaced != 1) {
            exit 2
        }
    }
' "$CORE" > "$CORE_NATIVE"

if [ "$(uname -s)" = "Darwin" ]; then
    "$CLANG" -Wno-override-module -O2 -Wl,-stack_size,0x4000000 -o "$OUT" "$CORE_NATIVE" "$DRIVER"
else
    "$CLANG" -Wno-override-module -O2 -o "$OUT" "$CORE_NATIVE" "$DRIVER"
fi
echo "built: $OUT"
