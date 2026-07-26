#!/usr/bin/env bash
# VaisDB scale gate (VaisDB P4): budget-checked ingest and ranking over a
# deterministic synthetic corpus, so index-scale regressions surface in the
# ladder as budget breaches instead of drifting silently. Budgets sit far
# above the docs/design/VAISDB-SCALE-BASELINE.md figures (374 real docs
# ingest in ~1s), so only real regressions fire.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/vaisdb-scale.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

"$ROOT/scripts/vaisc" package "$ROOT/examples/e337_vaisdb_cli_package" -o "$tmp/vaisdb-dist" >/dev/null
"$ROOT/scripts/vaisc" package "$ROOT/examples/e350_vaisbench_package" -o "$tmp/vaisbench-dist" >/dev/null
vdb="$tmp/vaisdb-dist/bin/vaisdb"
bench="$tmp/vaisbench-dist/bin/vaisbench"

# 60 documents, 203 unique terms each (overlapping numeric ranges plus a
# shared anchor line) -> 12,180 postings, well past the old map contract.
corpus="$tmp/corpus"
mkdir -p "$corpus"
for d in $(seq 0 59); do
    { seq -s ' ' $((d * 40)) $((d * 40 + 199)); printf 'shared anchor terms\n'; } > "$corpus/doc$d.txt"
done

idx="$tmp/idx"
echo "== vaisdb scale: ingest-dir budget =="
"$bench" -b 20000 1 "$vdb" ingest-dir "$idx" "$corpus"

echo "== vaisdb scale: rank budget (read path) =="
"$bench" -b 5000 3 "$vdb" rank "$idx" "absent_zz missing_zz" 5 >/dev/null
echo "rank budget held"

# Correctness spot checks outside the bench; the CLI carries scores and
# counts in its exit codes, so capture instead of piping under pipefail.
set +e
"$vdb" rank "$idx" "anchor shared" 3 >/dev/null 2>&1
rank_score=$?
stats_out=$("$vdb" stats "$idx" 2>/dev/null)
set -e
if [ "$rank_score" -ne 2 ]; then
    echo "FAIL: expected top score 2, got $rank_score"
    exit 1
fi
if [ "$stats_out" != "docs=60 terms=12180" ]; then
    echo "FAIL: unexpected stats: $stats_out"
    exit 1
fi
echo "RESULT: VaisDB scale gate OK"
