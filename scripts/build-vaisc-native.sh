#!/usr/bin/env bash
# Build the Python-free native Vais compiler driver.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
CLANG="${CLANG:-clang}"
OUT="${1:-$HERE/build/vaisc}"
BUILD_DIR="$(dirname "$OUT")"
CORE="$HERE/compiler/self/vaisc_core.ll"
DRIVER="$HERE/tools/vaisc_native.c"
CORE_NATIVE="$BUILD_DIR/vaisc_core_native.ll"

mkdir -p "$BUILD_DIR"

main_count="$(grep -c '^define i64 @main() {$' "$CORE" || true)"
if [ "$main_count" != "1" ]; then
    echo "error: expected one @main in $CORE, found $main_count" >&2
    exit 1
fi

awk '
    BEGIN { replaced = 0 }
    /^define i64 @main\(\) \{$/ && replaced == 0 {
        print "define i64 @vais_selftest_main() {"
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

"$CLANG" -Wno-override-module -O2 -o "$OUT" "$CORE_NATIVE" "$DRIVER"
echo "built: $OUT"
