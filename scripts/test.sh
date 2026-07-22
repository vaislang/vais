#!/usr/bin/env bash
# Value-correctness test runner (P7b: compile != correct).
#
# Usage:  scripts/test.sh          # run release-subset annotated examples
#         scripts/test.sh c4       # run one (by basename)
#
# The value-corpus logic is implemented in tools/vais_value_check.vais. This
# shell file is the temp-dir bootstrap plus the shard fan-out: entries are
# stateless-hash sharded across VAIS_VALUE_SHARDS parallel workers (default
# 8; 1 keeps the serial path), and the per-shard RESULT counters are summed
# into the single canonical RESULT line.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE_ROOT="${VAIS_TEST_ROOT:-$HERE}"
MANIFEST="$HERE/tools/vaisc-parity.tsv"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

if [ "$#" -ge 1 ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_value_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work" "$1"
    exit $?
fi

shards="${VAIS_VALUE_SHARDS:-8}"
if [ "$shards" -le 1 ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_value_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work"
    exit $?
fi

pids=()
for ((i = 0; i < shards; i++)); do
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_value_check.vais" -- \
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

echo
cat "$tmp"/shard*.log | awk '
    /^RESULT: pass=/ {
        for (f = 1; f <= NF; f++) {
            if ($f ~ /^pass=/) { sub("pass=", "", $f); pass += $f }
            if ($f ~ /^fail=/) { sub("fail=", "", $f); failn += $f }
            if ($f ~ /^skip=/) { sub("skip=", "", $f); skip += $f }
        }
    }
    END { printf "RESULT: pass=%d fail=%d skip=%d\n", pass, failn, skip }
'
exit "$fail"
