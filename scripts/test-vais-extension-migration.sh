#!/usr/bin/env bash
# NV-M2 migration gate: prove the current .nl corpus can be mirrored as .vais
# without changing value-correctness or native parity behavior.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mirror="$tmp/root"
manifest="$tmp/vaisc-parity-vais.tsv"
mkdir -p "$mirror/examples" "$mirror/compiler/self"

while IFS= read -r -d '' src; do
    rel="${src#$HERE/}"
    dst="$mirror/${rel%.nl}.vais"
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
done < <(find "$HERE/examples" "$HERE/compiler/self" -name '*.nl' -print0)

while IFS=$'\t' read -r rel status note; do
    case "$rel" in
        ""|\#*) printf '%s\n' "$rel" >>"$manifest"; continue ;;
    esac
    if [[ "$rel" == *.nl ]]; then
        rel="${rel%.nl}.vais"
    fi
    printf '%s\t%s\t%s\n' "$rel" "$status" "$note" >>"$manifest"
done <"$HERE/tools/vaisc-parity.tsv"

fail=0

if VAIS_TEST_ROOT="$mirror" VAIS_TEST_EXT=vais bash "$HERE/scripts/test.sh"; then
    echo "  PASS .vais mirrored value corpus"
else
    echo "  FAIL .vais mirrored value corpus"
    fail=1
fi

if VAISC_PARITY_ROOT="$mirror" \
   VAISC_SELF_HOST_TRUST_ROOTS="$mirror" \
   bash "$HERE/scripts/test-vaisc-parity.sh" "$manifest"; then
    echo "  PASS .vais mirrored native parity corpus"
else
    echo "  FAIL .vais mirrored native parity corpus"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: New Vais .vais extension migration gate OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
