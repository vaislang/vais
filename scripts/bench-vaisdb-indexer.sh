#!/usr/bin/env bash
# Repeatable local timing harness for the first Vais-authored VaisDB indexer.
#
# This is a developer baseline, not a release assertion. It measures the
# compile+run path for the direct and default engines and still verifies that
# each iteration exits with the expected value result.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLE="$ROOT/examples/e295_vaisdb_indexer_prototype.vais"
ITERATIONS="${1:-${VAISDB_BENCH_ITERATIONS:-5}}"

case "$ITERATIONS" in
    ''|*[!0-9]*)
        echo "usage: bash scripts/bench-vaisdb-indexer.sh [positive-iterations]" >&2
        exit 2
        ;;
esac

if [ "$ITERATIONS" -le 0 ]; then
    echo "usage: bash scripts/bench-vaisdb-indexer.sh [positive-iterations]" >&2
    exit 2
fi

bench_engine() {
    local label="$1"
    local engine="$2"

    printf '\n==> %s engine, %s iterations\n' "$label" "$ITERATIONS"
    /usr/bin/time -p bash -c '
        set -euo pipefail
        root="$1"
        example="$2"
        engine="$3"
        iterations="$4"
        i=1
        while [ "$i" -le "$iterations" ]; do
            set +e
            if [ "$engine" = "direct" ]; then
                "$root/scripts/vaisc" run "$example" --engine direct >/dev/null
            else
                "$root/scripts/vaisc" run "$example" >/dev/null
            fi
            got=$?
            set -e
            if [ "$got" -ne 42 ]; then
                printf "FAIL %s iteration %s: got=%s expect=42\n" "$engine" "$i" "$got" >&2
                exit 1
            fi
            i=$((i + 1))
        done
    ' bench-loop "$ROOT" "$EXAMPLE" "$engine" "$ITERATIONS"
}

echo "VaisDB indexer prototype compile+run baseline"
echo "example: $EXAMPLE"
bench_engine "direct" "direct"
bench_engine "default" "default"

echo
echo "RESULT: VaisDB indexer benchmark completed"
