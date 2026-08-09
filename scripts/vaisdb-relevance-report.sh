#!/usr/bin/env bash
# Advisory relevance report over the live whole-repo index: a curated
# query set with long-lived expected documents, reporting each
# expectation's rank in OR and -all mode plus hits@1 / hits@3 / MRR100
# per mode. No assertions — the numbers land in
# docs/RELEVANCE-BASELINE.md by hand when a ranking change ships, and
# the deterministic regression layer lives in
# scripts/test-vaisdb-relevance.sh.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/build/vaisdb-repo-dist"
IDX="$ROOT/build/repo-docs-index"

bash "$ROOT/scripts/vaisdb-repo.sh" >/dev/null
VDB="$DIST/bin/vaisdb"

# query|expected-doc-id (stable, long-lived files)
CASES=(
    "token_shard_at|examples/e337_vaisdb_cli_package/src/vaisdb/index"
    "str_slice_raw|tools/vaisc_native"
    "rename atomic|examples/e391_fs_rename_atomic_swap"
    "fixpoint|compiler/self/fixpoint_full"
    "is_word_byte|examples/e337_vaisdb_cli_package/src/vaisdb/index"
    "front contract|tools/vaisc_front_check"
    "shell sort desugar|tools/vaisc_native"
    "release gates|scripts/test-release-gates"
)

report_mode() {
    local mode_label="$1"
    local mode_flag="$2"
    local hits1=0
    local hits3=0
    local rr_total=0
    local n=0
    echo "== mode: $mode_label =="
    for case in "${CASES[@]}"; do
        local query="${case%%|*}"
        local want="${case##*|}"
        local rank
        if [ -n "$mode_flag" ]; then
            rank="$("$VDB" search "$IDX" "$query" 10 "$mode_flag" 2>/dev/null | grep "\. ${want}=" | head -1 | cut -d. -f1)" || true
        else
            rank="$("$VDB" search "$IDX" "$query" 10 2>/dev/null | grep "\. ${want}=" | head -1 | cut -d. -f1)" || true
        fi
        n=$((n + 1))
        if [ -n "$rank" ] && [ "$rank" -ge 1 ] 2>/dev/null; then
            printf '  rank=%-3s %-22s -> %s\n' "$rank" "$query" "$want"
            rr_total=$((rr_total + 100 / rank))
            if [ "$rank" -le 1 ]; then hits1=$((hits1 + 1)); fi
            if [ "$rank" -le 3 ]; then hits3=$((hits3 + 1)); fi
        else
            printf '  rank=--  %-22s -> %s (not in top 10)\n' "$query" "$want"
        fi
    done
    printf '  %s: hits@1=%s/%s hits@3=%s/%s MRR100=%s\n' "$mode_label" "$hits1" "$n" "$hits3" "$n" "$((rr_total / n))"
}

report_mode "OR" ""
report_mode "-all" "-all"
echo "RESULT: vaisdb relevance report done (advisory, no assertions)"
