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
#   ./scripts/vaisdb-regression.sh                    # default: test_btree only
#   ./scripts/vaisdb-regression.sh test_btree         # specific test
#   ./scripts/vaisdb-regression.sh --all              # all vaisdb storage tests
#   VERBOSE=1 ./scripts/vaisdb-regression.sh          # show full clang output

set -uo pipefail

# ─── Config ──────────────────────────────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VAISDB_ROOT="${VAISDB_ROOT:-$REPO_ROOT/../lang/packages/vaisdb}"
VAISC="${VAISC:-$HOME/.cargo/bin/vaisc}"
STD_PATH="${STD_PATH:-/tmp/vais-lib/std}"

# Known-failure baseline (2026-04-26).
# These clang errors are tracked as TEMP-SITE-FIX(adr-0001) — to be resolved
# by Task #6 (stmt.rs Vec→fat-ptr ret) and Task #7 (slice indexing bitcast).
KNOWN_FAILURE_COUNT="${KNOWN_FAILURE_COUNT:-2}"
KNOWN_FAILURE_PATTERNS=(
    "test_btree_node.ll:.*ret.*i8\*.*i64.*defined with type 'ptr' but expected"
    "test_btree_key.ll:.*defined with type 'ptr' but expected.*i8\*.*i64"
)

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

# ─── Cache nuke (ADR 0001 §3, vaisdb iter 73 학습) ───────────────────────────
# vaisc cache (.vais-cache)가 specialization 결과를 보존하여 fix 효과를 가린다.
# clean rebuild 의무.

echo "▶ Nuking vaisdb caches and /tmp test artifacts..."
find "$VAISDB_ROOT" -name ".vais-cache" -type d -exec rm -rf {} + 2>/dev/null || true
rm -rf /tmp/test_btree* 2>/dev/null || true

# ─── Build ───────────────────────────────────────────────────────────────────

TARGET_TEST="${1:-test_btree}"

if [[ "$TARGET_TEST" == "--all" ]]; then
    echo "❌ --all not yet implemented (Pillar 2 follow-up)"
    exit 2
fi

TEST_FILE="$VAISDB_ROOT/tests/storage/$TARGET_TEST.vais"
if [[ ! -f "$TEST_FILE" ]]; then
    echo "❌ Test file not found: $TEST_FILE"
    exit 2
fi

echo "▶ Building $TARGET_TEST.vais (vaisc emit IR)..."
cd "$VAISDB_ROOT"

BUILD_LOG=$(mktemp)
if ! VAIS_DEP_PATHS="$(pwd)/src:$STD_PATH" VAIS_STD_PATH="$STD_PATH" \
    "$VAISC" build "tests/storage/$TARGET_TEST.vais" \
    --emit-ir -o "/tmp/$TARGET_TEST.ll" --force-rebuild \
    > "$BUILD_LOG" 2>&1; then
    echo "❌ vaisc build FAILED — see $BUILD_LOG"
    tail -30 "$BUILD_LOG"
    exit 1
fi

[[ "${VERBOSE:-0}" == "1" ]] && cat "$BUILD_LOG"

EMITTED_IR_COUNT=$(ls /tmp/${TARGET_TEST}_*.ll 2>/dev/null | wc -l | tr -d ' ')
echo "  ✓ vaisc emitted $EMITTED_IR_COUNT IR files"

# ─── Link (clang) ────────────────────────────────────────────────────────────

echo "▶ Linking IR with clang..."
LINK_LOG=$(mktemp)
clang -O0 -o "/tmp/${TARGET_TEST}_bin" /tmp/${TARGET_TEST}_*.ll -lm \
    > "$LINK_LOG" 2>&1
LINK_EXIT=$?

# Count clang errors (signature: "error: ...")
CLANG_ERRORS=$(grep -c "^[^:]*\.ll:[0-9]*:[0-9]*: error:" "$LINK_LOG" || true)

# ─── Verdict ─────────────────────────────────────────────────────────────────

echo ""
echo "▶ Result:"
echo "  clang exit code: $LINK_EXIT"
echo "  clang errors:    $CLANG_ERRORS (baseline known-failure: $KNOWN_FAILURE_COUNT)"

if [[ "$CLANG_ERRORS" -gt "$KNOWN_FAILURE_COUNT" ]]; then
    echo ""
    echo "❌ REGRESSION DETECTED — clang errors $CLANG_ERRORS > baseline $KNOWN_FAILURE_COUNT"
    echo ""
    echo "Errors:"
    grep -E "^[^:]*\.ll:[0-9]*:[0-9]*: error:" "$LINK_LOG" | head -20
    echo ""
    echo "ADR 0001 §1 R3: same-class audit failed."
    echo "  → fix the new error sites OR document them as TEMP-SITE-FIX(adr-0001)"
    echo "  → update KNOWN_FAILURE_COUNT in this script if the new baseline is intentional"
    exit 1
fi

if [[ "$CLANG_ERRORS" -lt "$KNOWN_FAILURE_COUNT" ]]; then
    echo ""
    echo "🎉 IMPROVEMENT — clang errors $CLANG_ERRORS < baseline $KNOWN_FAILURE_COUNT"
    echo ""
    echo "Action required:"
    echo "  1. Verify the improvement is intentional (not a flaky test)"
    echo "  2. Update KNOWN_FAILURE_COUNT=$CLANG_ERRORS in scripts/vaisdb-regression.sh"
    echo "  3. Update ROADMAP to mark which task closed the gap"
    exit 0
fi

echo ""
echo "✓ vaisdb regression baseline holds ($CLANG_ERRORS = $KNOWN_FAILURE_COUNT)"
echo ""
echo "Pending site fixes (TEMP-SITE-FIX, awaiting Task #6/#7):"
for pattern in "${KNOWN_FAILURE_PATTERNS[@]}"; do
    echo "  - $pattern"
done

exit 0
