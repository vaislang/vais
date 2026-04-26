#!/usr/bin/env bash
# vais-server-regression.sh
#
# vais-server 빌드를 compiler regression suite로 실행한다.
# Pillar 2.3 (Phase Ω) 산출물 — vaisdb-regression.sh와 동일 패턴.
#
# Why: vais-server 빌드 깨짐이 compiler "green" 상태를 통과해 머지되는 패턴 차단.
#      compiler PR이 vais-server를 깨면 즉시 발견 + 차단됨.
#
# Usage:
#   ./scripts/vais-server-regression.sh                    # default: test_shutdown only
#   ./scripts/vais-server-regression.sh test_shutdown      # specific test
#   ./scripts/vais-server-regression.sh --all              # wave 1: 2 tests
#   VERBOSE=1 ./scripts/vais-server-regression.sh          # show full clang output
#
# Note: vais-web 통합은 별도 (Rust workspace로 cargo test 패턴, 본 script scope 외).

set -uo pipefail

# ─── Config ──────────────────────────────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_ROOT="${SERVER_ROOT:-$REPO_ROOT/../lang/packages/vais-server}"
VAISC="${VAISC:-$HOME/.cargo/bin/vaisc}"
STD_PATH="${STD_PATH:-/tmp/vais-lib/std}"

# Known-failure baselines (2026-04-26, iter 86 실측, ADR 0001 §1 R3).
# These clang errors are tracked as TEMP-SITE-FIX(adr-0001) — to be resolved
# by Pillar 1.x cross-cutting fixes (Vec_push undefined, var-to-llvm typing).
WAVE1_TESTS=(test_shutdown test_http)
WAVE1_BASELINES=(1 1)
WAVE1_PATHS=(
    "tests/core/test_shutdown.vais"
    "tests/integration/test_http.vais"
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

if [[ ! -d "$SERVER_ROOT" ]]; then
    echo "❌ vais-server root not found at $SERVER_ROOT"
    echo "   Set SERVER_ROOT env var to the vais-server package directory"
    exit 2
fi

if [[ ! -e "$STD_PATH" ]]; then
    echo "ℹ Creating $STD_PATH symlink → $REPO_ROOT/std"
    mkdir -p "$(dirname "$STD_PATH")"
    ln -sf "$REPO_ROOT/std" "$STD_PATH"
fi

# ─── Cache nuke (ADR 0001 §3 학습) ───────────────────────────────────────────

echo "▶ Nuking vais-server caches and /tmp test artifacts..."
find "$SERVER_ROOT" -name ".vais-cache" -type d -exec rm -rf {} + 2>/dev/null || true
for t in "${WAVE1_TESTS[@]}"; do
    rm -rf /tmp/${t}* 2>/dev/null || true
done

# ─── Target selection ────────────────────────────────────────────────────────

TARGET_ARG="${1:-test_shutdown}"

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

cd "$SERVER_ROOT"

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
    if ! VAIS_DEP_PATHS="$(pwd)/src:$STD_PATH" VAIS_STD_PATH="$STD_PATH" \
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

if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
    echo ""
    echo "❌ REGRESSIONS:"
    for t in "${FAILED_TESTS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi

if [[ ${#IMPROVED_TESTS[@]} -gt 0 ]]; then
    echo ""
    echo "🎉 IMPROVEMENTS:"
    for t in "${IMPROVED_TESTS[@]}"; do
        echo "  - $t"
    done
    echo ""
    echo "Action required:"
    echo "  1. Verify improvements are intentional (not flaky)"
    echo "  2. Update WAVE1_BASELINES in scripts/vais-server-regression.sh"
    echo "  3. Update ROADMAP to mark which task closed the gaps"
fi

if [[ ${#FAILED_TESTS[@]} -eq 0 && ${#IMPROVED_TESTS[@]} -eq 0 ]]; then
    echo ""
    echo "✓ vais-server wave 1 baseline holds (all ${#TESTS[@]} tests)"
fi

exit 0
