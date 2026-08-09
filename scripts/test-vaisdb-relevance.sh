#!/usr/bin/env bash
# Deterministic vaisdb relevance gate: a role-controlled mini corpus
# (definition-site doc, common-token heavy user, partial match, noise
# flood, multibyte doc) locks the CURRENT ranking behavior of the OR
# and -all modes — exact top rows, tie bands, and an integer MRR100
# floor over the query set. Ranking changes must consciously update
# these numbers, and the companion vaisdb-relevance-report.sh shows the
# live-repo effect of the same change.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

fail=0
expect() {
    local label="$1"
    local want="$2"
    shift 2
    set +e
    "$@" >/dev/null 2>&1
    local got=$?
    set -e
    if [ "$got" -eq "$want" ]; then
        printf '  PASS %s (= %s)\n' "$label" "$got"
    else
        printf '  FAIL %s: got=%s expect=%s\n' "$label" "$got" "$want"
        fail=1
    fi
    return 0
}

echo "VaisDB relevance gate"
dist="$tmp/vaisdb-dist"
"$ROOT/scripts/vaisc" package "$ROOT/examples/e337_vaisdb_cli_package" -o "$dist" >/dev/null || {
    echo "  FAIL vaisdb package build"
    echo "RESULT: FAILURES"
    exit 1
}
VDB="$dist/bin/vaisdb"

docs="$tmp/docs"
idx="$tmp/idx"
mkdir -p "$docs"
printf 'alpha beta gamma glue\n' > "$docs/defsite.txt"
printf 'alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha beta beta gamma filler\n' > "$docs/heavy.txt"
printf 'alpha beta only here\n' > "$docs/partial.txt"
printf 'alpha filler1 pad\n' > "$docs/noise1.txt"
printf 'alpha filler2 pad\n' > "$docs/noise2.txt"
printf 'alpha filler3 pad\n' > "$docs/noise3.txt"
printf '김치 검색 엔진\n' > "$docs/hangul.txt"
expect "relevance corpus builds" 0 /bin/sh -c "'$VDB' reindex '$idx' '$docs' | grep -qx 'reindexed added=7 updated=0 removed=0 skipped=0'"

# Locked current behavior (captured 2026-08-08, rarity-weighted -all):
# N=7; alpha df=6 rarity=1, beta df=3 rarity=2, gamma df=2 rarity=2.
expect "-all top row is the saturated heavy user" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 -all | grep -q '^1\. heavy=14 '"
expect "-all second row is the definition-site doc" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 -all | grep -q '^2\. defsite=5 '"
expect "-all drops the gamma-less partial doc" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 -all | grep -q 'partial' && exit 1; exit 0"
expect "-all drops the noise flood" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 -all | grep -q 'noise' && exit 1; exit 0"
expect "rare-term tie band scores equally" 0 /bin/sh -c "out=\$('$VDB' search '$idx' gamma 7 -all); echo \"\$out\" | grep -q '^[12]\. defsite=2 ' && echo \"\$out\" | grep -q '^[12]\. heavy=2 '"
expect "two-term -all keeps the partial doc" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta' 7 -all | grep -q '^[23]\. partial=3 '"
expect "OR keeps noise recall" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 | grep -q '^[456]\. noise'"
expect "OR top row keeps the plain sum" 0 /bin/sh -c "'$VDB' search '$idx' 'alpha beta gamma' 7 | grep -q '^1\. heavy=15 '"
expect "multibyte query ranks the hangul doc" 0 /bin/sh -c "'$VDB' search '$idx' '검색' 7 -all | grep -q '^1\. hangul=3 '"

# Integer MRR100 over the expected-doc query set; rank comes from the
# printed row number. A ranking change moves this floor consciously.
rr_total=0
rr_count=0
add_query() {
    local query="$1"
    local id="$2"
    local rank
    # search exits with its top score, so the substitution absorbs it.
    rank="$("$VDB" search "$idx" "$query" 7 -all 2>/dev/null | grep "\. ${id}=" | head -1 | cut -d. -f1)" || true
    rr_count=$((rr_count + 1))
    if [ -n "$rank" ] && [ "$rank" -ge 1 ] 2>/dev/null; then
        rr_total=$((rr_total + 100 / rank))
    fi
}
add_query "alpha beta gamma" defsite
add_query "gamma" defsite
add_query "alpha beta" partial
add_query "검색" hangul
mrr100=$((rr_total / rr_count))
echo "  METRIC MRR100=$mrr100 (queries=$rr_count)"
if [ "$mrr100" -eq 70 ]; then
    printf '  PASS MRR100 floor holds (= %s)\n' "$mrr100"
else
    printf '  FAIL MRR100 floor: got=%s expect=70\n' "$mrr100"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: VaisDB relevance gate OK"
    exit 0
fi
echo "RESULT: FAILURES"
exit 1
