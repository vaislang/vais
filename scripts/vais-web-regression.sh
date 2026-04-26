#!/usr/bin/env bash
# vais-web-regression.sh
#
# vais-web (Rust workspace) cargo test를 compiler regression suite로 실행한다.
# Pillar 2.3 wave 2 (Phase Ω) 산출물.
#
# Why: vais-web은 vais 컴파일러 crates (vais-codegen-js, vais-parser, vais-ast)에
#      의존하므로 compiler 변경이 vais-web을 깨면 즉시 발견 + 차단됨.
#
# vais-web 자체는 Rust workspace (vaisx-compiler/parser/wasm) — .vais source 없음.
# 따라서 vaisc build 패턴이 아닌 cargo test 패턴 사용.
#
# Usage:
#   ./scripts/vais-web-regression.sh                # full workspace test
#   VERBOSE=1 ./scripts/vais-web-regression.sh      # show full output

set -uo pipefail

# ─── Config ──────────────────────────────────────────────────────────────────

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_ROOT="${WEB_ROOT:-$REPO_ROOT/../lang/packages/vais-web}"

# Known-pass baseline (2026-04-26, iter 87 실측, ADR 0001 §1 R3).
# vais-web cargo test --workspace 합계 PASS count.
# Task 완료 시 해당 카운트 갱신.
KNOWN_PASS_COUNT="${KNOWN_PASS_COUNT:-271}"

# ─── Pre-flight ──────────────────────────────────────────────────────────────

if [[ ! -d "$WEB_ROOT" ]]; then
    echo "❌ vais-web root not found at $WEB_ROOT"
    echo "   Set WEB_ROOT env var to the vais-web package directory"
    exit 2
fi

if [[ ! -f "$WEB_ROOT/Cargo.toml" ]]; then
    echo "❌ vais-web is not a Cargo workspace at $WEB_ROOT"
    exit 2
fi

# ─── Run cargo test ──────────────────────────────────────────────────────────

cd "$WEB_ROOT"

echo "▶ Running cargo test --workspace --no-fail-fast on vais-web..."
echo "  baseline (KNOWN_PASS_COUNT): $KNOWN_PASS_COUNT"

TEST_LOG=$(mktemp)
if ! cargo test --workspace --no-fail-fast > "$TEST_LOG" 2>&1; then
    echo "❌ cargo test exited non-zero — see $TEST_LOG"
    [[ "${VERBOSE:-0}" == "1" ]] && cat "$TEST_LOG"
    grep -E "FAILED|^error" "$TEST_LOG" | head -10
    exit 1
fi

[[ "${VERBOSE:-0}" == "1" ]] && cat "$TEST_LOG"

# Aggregate passed / failed across all binaries
PASSED=$(grep "test result" "$TEST_LOG" | awk '{ for(i=1;i<=NF;i++) if($i=="passed;") {gsub(",","",$(i-1)); s+=$(i-1)} } END { print s+0 }')
FAILED=$(grep "test result" "$TEST_LOG" | awk '{ for(i=1;i<=NF;i++) if($i=="failed;") {gsub(",","",$(i-1)); s+=$(i-1)} } END { print s+0 }')
IGNORED=$(grep "test result" "$TEST_LOG" | awk '{ for(i=1;i<=NF;i++) if($i=="ignored;") {gsub(",","",$(i-1)); s+=$(i-1)} } END { print s+0 }')

echo ""
echo "═══ Summary ═══"
echo "  Total passed:  $PASSED"
echo "  Total failed:  $FAILED"
echo "  Total ignored: $IGNORED"
echo "  Baseline pass: $KNOWN_PASS_COUNT"

# ─── Verdict ────────────────────────────────────────────────────────────────

if [[ "$FAILED" -gt 0 ]]; then
    echo ""
    echo "❌ REGRESSION: $FAILED test(s) failed"
    grep -E "FAILED" "$TEST_LOG" | head -20
    exit 1
fi

if [[ "$PASSED" -lt "$KNOWN_PASS_COUNT" ]]; then
    echo ""
    echo "❌ REGRESSION: passed count $PASSED < baseline $KNOWN_PASS_COUNT (delta -$((KNOWN_PASS_COUNT - PASSED)))"
    exit 1
fi

if [[ "$PASSED" -gt "$KNOWN_PASS_COUNT" ]]; then
    echo ""
    echo "🎉 IMPROVEMENT: passed count $PASSED > baseline $KNOWN_PASS_COUNT (delta +$((PASSED - KNOWN_PASS_COUNT)))"
    echo "Action required:"
    echo "  1. Verify improvements are intentional (not test additions)"
    echo "  2. Update KNOWN_PASS_COUNT in scripts/vais-web-regression.sh"
fi

echo ""
echo "✓ vais-web baseline holds ($PASSED >= $KNOWN_PASS_COUNT)"
exit 0
