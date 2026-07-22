#!/usr/bin/env bash
# NV-C4 parity gate for the Vais `vaisc` native path.
#
# The manifest records the release subset. Entries marked native-supported must
# match their `# expect:` value through scripts/vaisc. The parity logic itself
# is implemented in tools/vais_parity_check.vais; this shell file is the
# temp-dir bootstrap plus the shard fan-out (VAIS_PARITY_SHARDS workers,
# default 8; 1 keeps the serial path) with the per-shard counters summed into
# the single canonical RESULT line.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="${1:-$HERE/tools/vaisc-parity.tsv}"
SOURCE_ROOT="${VAISC_PARITY_ROOT:-$HERE}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

shards="${VAIS_PARITY_SHARDS:-8}"
if [ "$shards" -le 1 ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_parity_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work"
    exit $?
fi

pids=()
for ((i = 0; i < shards; i++)); do
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_parity_check.vais" -- \
        "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work$i" "$i" "$shards" \
        > "$tmp/shard$i.log" 2>&1 &
    pids+=($!)
done

fail=0
for ((i = 0; i < shards; i++)); do
    if ! wait "${pids[$i]}"; then
        fail=1
    fi
done

for ((i = 0; i < shards; i++)); do
    grep -v '^RESULT:' "$tmp/shard$i.log" | sed '/^$/d' || true
done

if [ "$fail" -ne 0 ]; then
    echo "RESULT: FAILURES"
    exit 1
fi
cat "$tmp"/shard*.log | awk '
    /^RESULT: Vais vaisc NV-C4 parity gate OK/ {
        if (match($0, /native=[0-9]+/)) { n += substr($0, RSTART + 7, RLENGTH - 7) }
        if (match($0, /full=[0-9]+/)) { f += substr($0, RSTART + 5, RLENGTH - 5) }
        if (match($0, /tracked=[0-9]+/)) { t += substr($0, RSTART + 8, RLENGTH - 8) }
    }
    END { printf "RESULT: Vais vaisc NV-C4 parity gate OK (native=%d full=%d tracked=%d)\n", n, f, t }
'
