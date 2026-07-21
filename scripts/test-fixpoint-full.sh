#!/usr/bin/env bash
# Long full-codegen regression gate. The fixture orchestration lives in Vais so
# the remaining shell boundary is process setup, shard fan-out, and temp-dir
# cleanup. Cases are stateless-hash sharded across VAIS_FIXPOINT_SHARDS
# parallel workers (default 8); every shard runs the same tool with a
# (shard-index, shard-count) argument pair and skips cases outside its bucket,
# so coverage is identical to the serial run.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

shards="${VAIS_FIXPOINT_SHARDS:-8}"
if [ "$shards" -le 1 ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_codegen_check.vais" -- "$HERE" "$tmp"
    exit $?
fi

pids=()
for ((i = 0; i < shards; i++)); do
    mkdir -p "$tmp/shard$i"
    "$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_codegen_check.vais" -- \
        "$HERE" "$tmp/shard$i" "$i" "$shards" > "$tmp/shard$i.log" 2>&1 &
    pids+=($!)
done

fail=0
for ((i = 0; i < shards; i++)); do
    if ! wait "${pids[$i]}"; then
        fail=1
    fi
done

for ((i = 0; i < shards; i++)); do
    # RESULT lines are per-shard; print case output, keep one aggregate below.
    grep -v '^RESULT:' "$tmp/shard$i.log" || true
done

if [ "$fail" -ne 0 ]; then
    echo "RESULT: FAILURES"
    exit 1
fi
echo "RESULT: fixpoint full codegen (functions with imperative bodies) end-to-end OK"
