#!/usr/bin/env bash
# check-integrity.sh — Vais compiler integrity gate (Phase 0.4)
#
# Usage:
#   ./scripts/check-integrity.sh
#
# Environment overrides (defaults shown):
#   INTEGRITY_STD_MIN=37      minimum std_files pass count
#   INTEGRITY_VAISDB_MIN=176  minimum vaisdb_files pass count
#
# Exit codes:
#   0  all gates pass
#   1  regression detected OR a test suite exited non-zero

set -euo pipefail

# ---------------------------------------------------------------------------
# Locate repo root
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# ---------------------------------------------------------------------------
# Baseline thresholds (override via env)
# ---------------------------------------------------------------------------
INTEGRITY_STD_MIN="${INTEGRITY_STD_MIN:-58}"
INTEGRITY_VAISDB_MIN="${INTEGRITY_VAISDB_MIN:-178}"

# ---------------------------------------------------------------------------
# Ensure /tmp/vais-lib/std symlink exists
# ---------------------------------------------------------------------------
STD_LINK="/tmp/vais-lib/std"
STD_SRC="${REPO_ROOT}/std"

if [ ! -d "/tmp/vais-lib" ]; then
    mkdir -p "/tmp/vais-lib"
fi

# Create or refresh symlink only if needed
if [ -L "${STD_LINK}" ]; then
    CURRENT_TARGET="$(readlink "${STD_LINK}")"
    if [ "${CURRENT_TARGET}" != "${STD_SRC}" ]; then
        rm -f "${STD_LINK}"
        ln -s "${STD_SRC}" "${STD_LINK}"
    fi
elif [ -e "${STD_LINK}" ]; then
    # It's a real dir/file — remove and re-create
    rm -rf "${STD_LINK}"
    ln -s "${STD_SRC}" "${STD_LINK}"
else
    ln -s "${STD_SRC}" "${STD_LINK}"
fi

echo "check-integrity: std symlink ${STD_LINK} -> ${STD_SRC}"

# ---------------------------------------------------------------------------
# Run integrity tests
# ---------------------------------------------------------------------------
INTEGRITY_LOG="/tmp/integrity.log"
echo "check-integrity: running integrity tests..."

INTEGRITY_EXIT=0
cargo test -p vaisc --test integrity --release -- --nocapture 2>&1 | tee "${INTEGRITY_LOG}" || INTEGRITY_EXIT=$?

# ---------------------------------------------------------------------------
# Run phase158 e2e tests
# ---------------------------------------------------------------------------
PHASE158_LOG="/tmp/phase158.log"
echo "check-integrity: running phase158 e2e tests..."

PHASE158_EXIT=0
cargo test -p vaisc --test e2e --release phase158 2>&1 | tee "${PHASE158_LOG}" || PHASE158_EXIT=$?

# ---------------------------------------------------------------------------
# Parse INTEGRITY summary lines from log
# ---------------------------------------------------------------------------
# Lines look like:
#   INTEGRITY std_files pass=37 fail=45 total=82
#   INTEGRITY vaisdb_files pass=177 fail=84 total=261
#   INTEGRITY compiler_syntax pass=? fail=? total=30
#   INTEGRITY compiler_stages pass=? fail=? total=14

parse_pass() {
    local category="$1"
    local log="$2"
    # Extract pass=N for the given category; use last match in case of duplicates
    grep "INTEGRITY ${category} pass=" "${log}" 2>/dev/null \
        | tail -1 \
        | sed 's/.*pass=\([0-9?][0-9]*\).*/\1/'
}

parse_total() {
    local category="$1"
    local log="$2"
    grep "INTEGRITY ${category} pass=" "${log}" 2>/dev/null \
        | tail -1 \
        | sed 's/.*total=\([0-9][0-9]*\).*/\1/'
}

STD_PASS="$(parse_pass std_files "${INTEGRITY_LOG}")"
STD_TOTAL="$(parse_total std_files "${INTEGRITY_LOG}")"
VAISDB_PASS="$(parse_pass vaisdb_files "${INTEGRITY_LOG}")"
VAISDB_TOTAL="$(parse_total vaisdb_files "${INTEGRITY_LOG}")"

# Fallback values if parsing fails
STD_PASS="${STD_PASS:-0}"
STD_TOTAL="${STD_TOTAL:-82}"
VAISDB_PASS="${VAISDB_PASS:-0}"
VAISDB_TOTAL="${VAISDB_TOTAL:-261}"

# compiler_syntax and compiler_stages have pass=? so we just capture total
SYNTAX_TOTAL="$(parse_total compiler_syntax "${INTEGRITY_LOG}")"
STAGES_TOTAL="$(parse_total compiler_stages "${INTEGRITY_LOG}")"
SYNTAX_TOTAL="${SYNTAX_TOTAL:-30}"
STAGES_TOTAL="${STAGES_TOTAL:-14}"

# Parse phase158 result from log
parse_phase158_passed() {
    local log="$1"
    # Look for "test result: ok. N passed" line in cargo test output
    # Anchor match from the start of the number to avoid matching digits in other fields
    grep "test result:" "${log}" 2>/dev/null \
        | tail -1 \
        | sed 's/test result: [^.]*\. \([0-9][0-9]*\) passed.*/\1/'
}

PHASE158_PASSED="$(parse_phase158_passed "${PHASE158_LOG}")"
PHASE158_PASSED="${PHASE158_PASSED:-0}"

# ---------------------------------------------------------------------------
# Regression checks
# ---------------------------------------------------------------------------
REGRESSION=0
REGRESSION_MSG=""

# std_files threshold check
if [ "${STD_PASS}" != "?" ] && [ -n "${STD_PASS}" ]; then
    if [ "${STD_PASS}" -lt "${INTEGRITY_STD_MIN}" ]; then
        REGRESSION=1
        REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: std_files baseline=${INTEGRITY_STD_MIN} current=${STD_PASS}/${STD_TOTAL} (delta=$((STD_PASS - INTEGRITY_STD_MIN)))\n"
    fi
fi

# vaisdb_files threshold check
if [ "${VAISDB_PASS}" != "?" ] && [ -n "${VAISDB_PASS}" ]; then
    if [ "${VAISDB_PASS}" -lt "${INTEGRITY_VAISDB_MIN}" ]; then
        REGRESSION=1
        REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: vaisdb_files baseline=${INTEGRITY_VAISDB_MIN} current=${VAISDB_PASS}/${VAISDB_TOTAL} (delta=$((VAISDB_PASS - INTEGRITY_VAISDB_MIN)))\n"
    fi
fi

# ---------------------------------------------------------------------------
# Final result
# ---------------------------------------------------------------------------
OVERALL_EXIT=0

if [ "${INTEGRITY_EXIT}" -ne 0 ]; then
    echo ""
    echo "INTEGRITY FAILED: cargo test integrity exited ${INTEGRITY_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${PHASE158_EXIT}" -ne 0 ]; then
    echo ""
    echo "INTEGRITY FAILED: cargo test phase158 exited ${PHASE158_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${REGRESSION}" -ne 0 ]; then
    echo ""
    echo "INTEGRITY FAILED: regression detected"
    printf "%b" "${REGRESSION_MSG}"
    OVERALL_EXIT=1
fi

if [ "${OVERALL_EXIT}" -eq 0 ]; then
    echo ""
    echo "INTEGRITY OK: syntax=${SYNTAX_TOTAL}/? stages=${STAGES_TOTAL}/? std=${STD_PASS}/${STD_TOTAL} vaisdb=${VAISDB_PASS}/${VAISDB_TOTAL} phase158=${PHASE158_PASSED}/18"
    exit 0
else
    exit 1
fi
