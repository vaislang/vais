#!/usr/bin/env bash
# vaisdb-regression.sh
#
# vaisdb 빌드를 compiler regression suite로 실행한다.
# Pillar 2 (Phase Ω) 산출물 — ADR 0001 §1 R3 (same-class audit) 자동화의 일부.
#
# Why: vaisdb 빌드 깨짐이 compiler "green" 상태를 통과해 머지되는 패턴 차단.
#      compiler PR이 vaisdb를 깨면 즉시 발견 + 차단됨.
#
# Usage:
#   ./scripts/vaisdb-regression.sh                    # default: test_btree only (backward compat)
#   ./scripts/vaisdb-regression.sh test_btree         # specific test
#   ./scripts/vaisdb-regression.sh --all              # wave 1: 5 tests
#   VERBOSE=1 ./scripts/vaisdb-regression.sh          # show full clang output

set -uo pipefail

# ─── Config ──────────────────────────────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VAISDB_ROOT="${VAISDB_ROOT:-$REPO_ROOT/../lang/packages/vaisdb}"
VAISC="${VAISC:-$HOME/.cargo/bin/vaisc}"
STD_PATH="${STD_PATH:-/tmp/vais-lib/std}"
MIGRATED_VAISDB_ROOT=""

# Known-failure baselines (2026-04-26, iter 78 실측).
# bash 3.2 (macOS 기본) 은 associative array 미지원 — parallel arrays + lookup 함수로 우회.
# These clang errors are tracked as TEMP-SITE-FIX(adr-0001) — to be resolved
# by Task #6 (stmt.rs Vec→fat-ptr ret) and Task #7 (slice indexing bitcast).
WAVE1_TESTS=(test_btree test_wal test_buffer_pool test_graph test_migration)
# test_btree/test_wal: upstream vais-lang latest snapshot still carries known
# LLVM lowering gaps (TEMP-SITE-FIX(adr-0001)); Ubuntu clang 17 reports 1/2.
# test_buffer_pool: legacy source canonicalization plus build package-root
# import resolution lowered the clang-error baseline to zero.
# test_graph: 2 → 1 (Phase Ω P1.2 iter 90+91, commits 7fcdd285+27f6b260).
# Vec.push + HashMap.insert update_var_type fixes resolved 1+ Vec<T> indexing
# erasures in graph.vais. Flaky between 0 and 1 in --all context (CLAUDE
# known-issue: 제네릭 인스턴스 process leak); standalone is 0 or 1.
# Conservative baseline 1 to avoid CI false-positive on flaky 0 measurements.
WAVE1_BASELINES=(1 2 0 1 3)
WAVE1_PATHS=(
    "tests/storage/test_btree.vais"
    "tests/storage/test_wal.vais"
    "tests/storage/test_buffer_pool.vais"
    "tests/graph/test_graph.vais"
    "tests/sql/test_migration.vais"
)

# Lookup helpers (bash 3.2 compatible).
get_baseline() {
    local name="$1"
    local i=0
    for t in "${WAVE1_TESTS[@]}"; do
        if [[ "$t" == "$name" ]]; then
            echo "${WAVE1_BASELINES[$i]}"
            return 0
        fi
        i=$((i + 1))
    done
    echo ""
}

get_path() {
    local name="$1"
    local i=0
    for t in "${WAVE1_TESTS[@]}"; do
        if [[ "$t" == "$name" ]]; then
            echo "${WAVE1_PATHS[$i]}"
            return 0
        fi
        i=$((i + 1))
    done
    echo ""
}

is_known_test() {
    local name="$1"
    local found=""
    found="$(get_baseline "$name")"
    [[ -n "$found" ]]
}

# ─── Pre-flight ──────────────────────────────────────────────────────────────

if [[ ! -x "$VAISC" ]]; then
    echo "❌ vaisc not found at $VAISC"
    echo "   Set VAISC env var or install via 'cargo install --path crates/vaisc'"
    exit 2
fi

if [[ ! -d "$VAISDB_ROOT" ]]; then
    echo "❌ vaisdb root not found at $VAISDB_ROOT"
    echo "   Set VAISDB_ROOT env var to the vaisdb package directory"
    exit 2
fi

if [[ ! -e "$STD_PATH" ]]; then
    echo "ℹ Creating $STD_PATH symlink → $REPO_ROOT/std"
    mkdir -p "$(dirname "$STD_PATH")"
    ln -sf "$REPO_ROOT/std" "$STD_PATH"
fi

cleanup_migrated_vaisdb_root() {
    if [[ -n "${MIGRATED_VAISDB_ROOT:-}" && -d "$MIGRATED_VAISDB_ROOT" ]]; then
        rm -rf "$MIGRATED_VAISDB_ROOT"
    fi
}

canonicalize_vais_sources() {
    local root="$1"
    local failed=0
    local file=""

    while IFS= read -r -d '' file; do
        if ! "$VAISC" fmt "$file" --to=multi >/dev/null 2>&1; then
            echo "❌ canonical syntax migration failed for $file" >&2
            failed=1
        fi
    done < <(find "$root" -name "*.vais" -type f -print0)

    return "$failed"
}

normalize_legacy_test_assertions() {
    local root="$1"
    local file=""

    while IFS= read -r -d '' file; do
        if ! perl -0pi -e '
            s{\bassert_eq\(([^;\n]*?),\s*("(?:[^"\\]|\\.)*")\)}{assert_str_eq($1, $2)}g;
            if (/\bassert_str_eq\(/) {
                s!U std/test\.\{([^}]*)\};!{
                    my $items = $1;
                    $items =~ /\bassert_str_eq\b/
                        ? "U std/test.{$items};"
                        : "U std/test.{assert_str_eq, $items};";
                }!gex;
            }
            s{
                ^([ \t]*(?:_\s*=>\s*)?assert_(?:true|false))\(([^;\n]*?)\)([ \t]*[,;]?[ \t]*(?:\#.*)?$)
            }{
                my ($head, $expr, $tail) = ($1, $2, $3);
                $expr =~ /\bas\s+i64\b/ ? "$head($expr)$tail" : "$head(($expr) as i64)$tail";
            }gmex;
        ' "$file"; then
            echo "❌ legacy test assertion migration failed for $file" >&2
            return 1
        fi
    done < <(find "$root" -name "*.vais" -type f -print0)
}

echo "▶ Preparing canonical VaisDB source snapshot..."
MIGRATED_VAISDB_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/vaisdb-regression.XXXXXX")"
trap cleanup_migrated_vaisdb_root EXIT
cp -R "$VAISDB_ROOT"/. "$MIGRATED_VAISDB_ROOT"/
if ! canonicalize_vais_sources "$MIGRATED_VAISDB_ROOT"; then
    exit 1
fi
if ! normalize_legacy_test_assertions "$MIGRATED_VAISDB_ROOT"; then
    exit 1
fi
VAISDB_ROOT="$MIGRATED_VAISDB_ROOT"

# ─── Cache nuke (ADR 0001 §3, vaisdb iter 73 학습) ───────────────────────────
# vaisc cache (.vais-cache)가 specialization 결과를 보존하여 fix 효과를 가린다.
# clean rebuild 의무.

echo "▶ Nuking vaisdb caches and /tmp test artifacts..."
find "$VAISDB_ROOT" -name ".vais-cache" -type d -exec rm -rf {} + 2>/dev/null || true
for t in "${WAVE1_TESTS[@]}"; do
    rm -rf /tmp/${t}* 2>/dev/null || true
done

# ─── Target selection ────────────────────────────────────────────────────────

TARGET_ARG="${1:-test_btree}"

if [[ "$TARGET_ARG" == "--all" ]]; then
    TESTS=("${WAVE1_TESTS[@]}")
elif is_known_test "$TARGET_ARG"; then
    TESTS=("$TARGET_ARG")
else
    echo "❌ Unknown test: $TARGET_ARG"
    echo "   Available: ${WAVE1_TESTS[*]}"
    echo "   Or: --all"
    exit 2
fi

# ─── Per-test build + link + verdict ─────────────────────────────────────────

cd "$VAISDB_ROOT"

TOTAL_ERRORS=0
TOTAL_BASELINE=0
FAILED_TESTS=()
IMPROVED_TESTS=()

for TARGET_TEST in "${TESTS[@]}"; do
    BASELINE="$(get_baseline "$TARGET_TEST")"
    TEST_PATH="$(get_path "$TARGET_TEST")"
    TOTAL_BASELINE=$((TOTAL_BASELINE + BASELINE))

    echo ""
    echo "═══ $TARGET_TEST ═══"
    echo "▶ Building $TARGET_TEST.vais (baseline: $BASELINE errors)..."

    BUILD_LOG=$(mktemp)
    if ! VAIS_DEP_PATHS="$(pwd)/src:$(pwd):$STD_PATH" VAIS_STD_PATH="$STD_PATH" \
        "$VAISC" build "$TEST_PATH" \
        --emit-ir -o "/tmp/$TARGET_TEST.ll" --force-rebuild \
        > "$BUILD_LOG" 2>&1; then
        echo "❌ vaisc build FAILED — see $BUILD_LOG"
        tail -30 "$BUILD_LOG"
        FAILED_TESTS+=("$TARGET_TEST (build fail)")
        continue
    fi

    [[ "${VERBOSE:-0}" == "1" ]] && cat "$BUILD_LOG"

    EMITTED_IR_COUNT=$(ls /tmp/${TARGET_TEST}_*.ll 2>/dev/null | wc -l | tr -d ' ')
    echo "  ✓ vaisc emitted $EMITTED_IR_COUNT IR files"

    LINK_LOG=$(mktemp)
    clang -O0 -o "/tmp/${TARGET_TEST}_bin" /tmp/${TARGET_TEST}_*.ll -lm \
        > "$LINK_LOG" 2>&1

    CLANG_ERRORS=$(grep -c "^[^:]*\.ll:[0-9]*:[0-9]*: error:" "$LINK_LOG" 2>/dev/null || echo 0)
    CLANG_ERRORS=${CLANG_ERRORS//[^0-9]/}
    [[ -z "$CLANG_ERRORS" ]] && CLANG_ERRORS=0
    TOTAL_ERRORS=$((TOTAL_ERRORS + CLANG_ERRORS))

    echo "  clang errors: $CLANG_ERRORS (baseline: $BASELINE)"

    if [[ "$CLANG_ERRORS" -gt "$BASELINE" ]]; then
        echo "  ❌ REGRESSION ($CLANG_ERRORS > $BASELINE)"
        grep -E "^[^:]*\.ll:[0-9]*:[0-9]*: error:" "$LINK_LOG" | head -10
        FAILED_TESTS+=("$TARGET_TEST ($CLANG_ERRORS > $BASELINE)")
    elif [[ "$CLANG_ERRORS" -lt "$BASELINE" ]]; then
        echo "  🎉 IMPROVEMENT ($CLANG_ERRORS < $BASELINE)"
        IMPROVED_TESTS+=("$TARGET_TEST: $CLANG_ERRORS (was $BASELINE)")
    else
        echo "  ✓ baseline holds ($CLANG_ERRORS = $BASELINE)"
    fi
done

# ─── Global summary ──────────────────────────────────────────────────────────

echo ""
echo "═══ Summary ═══"
echo "  Total errors:   $TOTAL_ERRORS"
echo "  Total baseline: $TOTAL_BASELINE"
echo "  Tests run:      ${#TESTS[@]}"

if [[ "${#FAILED_TESTS[@]}" -gt 0 ]]; then
    echo ""
    echo "❌ REGRESSION DETECTED in:"
    for t in "${FAILED_TESTS[@]}"; do echo "  - $t"; done
    echo ""
    echo "ADR 0001 §1 R3: same-class audit failed."
    echo "  → fix the new error sites OR document them as TEMP-SITE-FIX(adr-0001)"
    echo "  → update WAVE1_BASELINES in this script if the new baseline is intentional"
    exit 1
fi

if [[ "${#IMPROVED_TESTS[@]}" -gt 0 ]]; then
    echo ""
    echo "🎉 IMPROVEMENTS:"
    for t in "${IMPROVED_TESTS[@]}"; do echo "  - $t"; done
    echo ""
    echo "Action required:"
    echo "  1. Verify improvements are intentional (not flaky)"
    echo "  2. Update WAVE1_BASELINES in scripts/vaisdb-regression.sh"
    echo "  3. Update ROADMAP to mark which task closed the gaps"
    exit 0
fi

echo ""
echo "✓ vaisdb wave 1 baseline holds (all ${#TESTS[@]} tests)"
exit 0
