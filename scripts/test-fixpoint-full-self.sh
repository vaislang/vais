#!/usr/bin/env bash
# Long self-host gate for compiler/self/fixpoint_full.vais.
#
# This verifies the full-source path, not just snippet-level codegen:
#   reference fixpoint_full -> generated first-generation compiler IR -> clang/run.
# It also checks that first-generation compilers can consume file-sized embedded
# sources again by retargeting their default compile("...") program to the real
# compiler/self/fixpoint*.vais sources, including fixpoint_full.vais itself.
#
# The five probes are independent (each embeds and builds its own
# first-generation compiler), so they run as parallel phase workers; only the
# final stage1/stage2 comparison consumes two workers' outputs. Set
# VAIS_SELFHOST_PHASES=serial for the original single-process behavior.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

if [ "${VAIS_SELFHOST_PHASES:-parallel}" = "serial" ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_self_check.vais" -- "$HERE" "$tmp"
    exit $?
fi

phases=(self_probe retarget_fixpoint retarget_fixpoint2 retarget_fixpoint3 retarget_fixpoint_full)
pids=()
for phase in "${phases[@]}"; do
    "$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_self_check.vais" -- \
        "$HERE" "$tmp/w_$phase" "$phase" > "$tmp/$phase.log" 2>&1 &
    pids+=($!)
done

fail=0
for i in "${!phases[@]}"; do
    if ! wait "${pids[$i]}"; then
        fail=1
    fi
done

for phase in "${phases[@]}"; do
    grep -v '^RESULT:' "$tmp/$phase.log" | sed '/^$/d' || true
done

if [ "$fail" -eq 0 ]; then
    stage1="$tmp/w_self_probe/self_probe/source_compiler.ll"
    stage2="$tmp/w_retarget_fixpoint_full/retarget_fixpoint_full/emitted.ll"
    if ! "$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_self_check.vais" -- \
        "$HERE" "$tmp/w_compare" "$stage1" "$stage2" > "$tmp/compare.log" 2>&1; then
        fail=1
    fi
    grep -v '^RESULT:' "$tmp/compare.log" | sed '/^$/d' || true
fi

if [ "$fail" -ne 0 ]; then
    echo "RESULT: FAILURES"
    exit 1
fi
echo "RESULT: fixpoint_full full-source self-host gate OK"
