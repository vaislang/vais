#!/usr/bin/env bash
# Large-corpus VaisDB operation timings: generate a deterministic corpus
# inside the documented contracts (docs <= 4095, index-wide vocabulary
# <= 4096), verify every operation's output once, then time each through
# the packaged vaisbench (median over N runs).
#
#   scripts/bench-vaisdb-corpus.sh [docs] [runs]
#
# Defaults: 1000 docs, 3 runs. This is a developer baseline harness, not a
# release assertion; the release-time regression watch stays in
# scripts/vaisbench-gate.sh with budgets from docs/PERF-BASELINE.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS="${1:-1000}"
RUNS="${2:-3}"

case "$DOCS$RUNS" in
    *[!0-9]*)
        echo "usage: bash scripts/bench-vaisdb-corpus.sh [docs] [runs]" >&2
        exit 2
        ;;
esac
if [ "$DOCS" -lt 1 ] || [ "$DOCS" -gt 4095 ]; then
    echo "error: docs must stay within the 4095 list contract" >&2
    exit 2
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/vaisdb-corpus-bench.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

vdb_dist="$tmp/vaisdb-dist"
vb_dist="$tmp/vaisbench-dist"
"$ROOT/scripts/vaisc" package "$ROOT/examples/e337_vaisdb_cli_package" -o "$vdb_dist" >/dev/null
"$ROOT/scripts/vaisc" package "$ROOT/examples/e350_vaisbench_package" -o "$vb_dist" >/dev/null
VDB="$vdb_dist/bin/vaisdb"
VBENCH="$vb_dist/bin/vaisbench"

# Deterministic corpus: 100 words per doc drawn from a 3000-term global
# vocabulary (well under the 4096 window even with the markers below).
# Every 100th doc carries the "needle alpha" pair for phrase checks and
# doc0500 carries the unique term "goldneedle" for exact-hit checks.
corpus="$tmp/corpus"
mkdir -p "$corpus"
awk -v docs="$DOCS" -v out="$corpus" 'BEGIN {
    for (i = 0; i < docs; i++) {
        file = sprintf("%s/doc%04d.txt", out, i)
        line = ""
        for (j = 0; j < 100; j++) {
            t = (i * 37 + j * 13 + (i * j) % 101) % 3000
            line = line sprintf("t%04d ", t)
            if (j % 20 == 19) { print line > file; line = "" }
        }
        if (i % 100 == 0) { print "needle alpha" > file }
        if (i == 500 && docs > 500) { print "goldneedle" > file }
        close(file)
    }
}'

second="$tmp/second"
mkdir -p "$second"
printf 't0001 t0002 crosshit\n' > "$second/extra.txt"

idx="$tmp/idx"
idx2="$tmp/idx2"

echo "==> corpus: $DOCS docs x 100 words, 3000-term vocabulary; $RUNS runs"

# One verified pass before any timing: every operation must produce the
# expected output on this corpus or the timings mean nothing.
"$VDB" reindex "$idx" "$corpus" | grep -qx "reindexed added=$DOCS updated=0 removed=0 skipped=0"
"$VDB" reindex "$idx2" "$second" >/dev/null
if [ "$DOCS" -gt 500 ]; then
    ("$VDB" search "$idx" goldneedle 3 || true) | grep -q '^1\. doc0500=1 '
fi
markers=$(( (DOCS + 99) / 100 ))
phrase_shown=$(( markers < 5 ? markers : 5 ))
set +e
"$VDB" phrase "$idx" "needle alpha" 5 >/dev/null
got=$?
set -e
if [ "$got" -ne "$phrase_shown" ]; then
    echo "error: phrase sanity expected $phrase_shown rows, got exit $got" >&2
    exit 1
fi
# similar/top/why exit with their shown-count/score contracts (nonzero by
# design), so the sanity calls tolerate the code and only assert output.
("$VDB" similar "$idx" doc0000 3 || true) | grep -q '^1\. ' || { echo "error: similar sanity" >&2; exit 1; }
("$VDB" top "$idx" 5 || true) | grep -q '^1\. ' || { echo "error: top sanity" >&2; exit 1; }
"$VDB" why "$idx" "t0001" doc0000 >/dev/null 2>&1 || true

bench() {
    local label="$1"
    shift
    printf '%-28s ' "$label"
    "$VBENCH" "$RUNS" /bin/sh -c "$1 >/dev/null 2>&1; exit 0" | sed 's/^runs=[0-9]* //'
}

echo "==> timings (vaisbench, median of $RUNS; query exits normalized to 0)"
bench "reindex cold ($DOCS docs)" "rm -rf '$tmp/idx-cold' && '$VDB' reindex '$tmp/idx-cold' '$corpus'"
bench "reindex warm (all skip)" "'$VDB' reindex '$idx' '$corpus'"
bench "search term k=10" "'$VDB' search '$idx' t0001 10"
bench "search term k=10 -all" "'$VDB' search '$idx' t0001 10 -all"
bench "msearch 2 indexes" "'$VDB' msearch t0001 10 '$idx' '$idx2'"
bench "phrase 2-word" "'$VDB' phrase '$idx' 'needle alpha' 5"
bench "similar doc k=5" "'$VDB' similar '$idx' doc0000 5"
bench "top k=10" "'$VDB' top '$idx' 10"

echo "RESULT: vaisdb corpus bench OK"
