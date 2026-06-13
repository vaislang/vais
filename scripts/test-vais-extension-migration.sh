#!/usr/bin/env bash
# NV-M3 compatibility gate: prove the checked-in .vais corpus can still be
# mirrored as transitional .nl without changing value-correctness or native
# parity behavior.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mirror="$tmp/root"
manifest="$tmp/vaisc-parity-nl.tsv"
mkdir -p "$mirror/examples" "$mirror/compiler/self"

while IFS= read -r -d '' src; do
    rel="${src#$HERE/}"
    dst="$mirror/${rel%.vais}.nl"
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
done < <(find "$HERE/examples" "$HERE/compiler/self" -name '*.vais' -print0)

while IFS=$'\t' read -r rel status note; do
    case "$rel" in
        ""|\#*) printf '%s\n' "$rel" >>"$manifest"; continue ;;
    esac
    if [[ "$rel" == *.vais ]]; then
        rel="${rel%.vais}.nl"
    fi
    printf '%s\t%s\t%s\n' "$rel" "$status" "$note" >>"$manifest"
done <"$HERE/tools/vaisc-parity.tsv"

fail=0

if VAIS_TEST_ROOT="$mirror" VAIS_TEST_EXT=nl bash "$HERE/scripts/test.sh"; then
    echo "  PASS .nl mirrored value corpus"
else
    echo "  FAIL .nl mirrored value corpus"
    fail=1
fi

if VAISC_PARITY_ROOT="$mirror" \
   VAISC_SELF_HOST_TRUST_ROOTS="$mirror" \
   bash "$HERE/scripts/test-vaisc-parity.sh" "$manifest"; then
    echo "  PASS .nl mirrored native parity corpus"
else
    echo "  FAIL .nl mirrored native parity corpus"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: New Vais transitional .nl extension compatibility gate OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
