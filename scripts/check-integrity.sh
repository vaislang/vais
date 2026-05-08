#!/usr/bin/env bash
# check-integrity.sh — Vais compiler integrity gate
#
# Usage:
#   ./scripts/check-integrity.sh
#
# Environment overrides (defaults shown; baseline locked 2026-05-03):
#   INTEGRITY_STD_MIN=82                   minimum std_files pass count
#   INTEGRITY_VAISDB_MIN=261               minimum vaisdb_files pass count
#   INTEGRITY_HTTP_CLIENT_RUNTIME_MIN=1    minimum http_client runtime smoke
#   INTEGRITY_VAISDB_RUNTIME_MIN=28        minimum vaisdb runtime smoke
#   INTEGRITY_SERVER_RUNTIME_MIN=13        minimum vais-server runtime smoke
#   INTEGRITY_WEB_RUNTIME_MIN=23           minimum vais-web runtime smoke
#   INTEGRITY_BACKEND_PHASE158_MIN=18      minimum phase158 backend smoke
#   INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN=2   minimum cross_package_schema gate (positive + negative)
#
# Strict-default imports (Step 11 root fix, loop 29, 2026-05-08):
#   This script does NOT export VAIS_STRICT_IMPORTS — the compiler is
#   default-strict since simple.rs:209-249 inverted the Err arm. Every
#   `vaisc check` / `vaisc build` invocation downstream of this script
#   (cargo test on integrity / ecosystem / runtime smoke gates) inherits
#   strict imports. Legacy harness opt-out is `VAIS_STRICT_IMPORTS=0`.
#   See LESSONS L-002 (no silent failure) + WORKLOG loop 29.
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
INTEGRITY_STD_MIN="${INTEGRITY_STD_MIN:-82}"
# Baseline lock 2026-05-03: codex review identified that the prior
# threshold of 219 (a Phase Ω P1.4 iter 114 floor) no longer reflects the
# current certified baseline. With the package-codegen path now stable at
# 261/261, allowing a silent drop to 219 violates the "안정성 100% / no
# silent regression" north star. Lock to the actual current baseline so
# any reduction below 261 trips the gate.
INTEGRITY_VAISDB_MIN="${INTEGRITY_VAISDB_MIN:-261}"

# Runtime smoke baseline lock 2026-05-03: per-runtime minima.
# Previously the script trusted `cargo test` exit=0 only, which would not
# catch a silent reduction in pass count if the suite count itself shrank.
# These minima are the current promoted gate counts as of 2026-05-03.
INTEGRITY_HTTP_CLIENT_RUNTIME_MIN="${INTEGRITY_HTTP_CLIENT_RUNTIME_MIN:-1}"
INTEGRITY_VAISDB_RUNTIME_MIN="${INTEGRITY_VAISDB_RUNTIME_MIN:-28}"
INTEGRITY_SERVER_RUNTIME_MIN="${INTEGRITY_SERVER_RUNTIME_MIN:-13}"
INTEGRITY_WEB_RUNTIME_MIN="${INTEGRITY_WEB_RUNTIME_MIN:-23}"
INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN="${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN:-2}"
INTEGRITY_BACKEND_PHASE158_MIN="${INTEGRITY_BACKEND_PHASE158_MIN:-18}"

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
# Run Core, MIR, and codegen quarantine gates
# ---------------------------------------------------------------------------
CORE_LOG="/tmp/core-certify.log"
echo "check-integrity: running Core v0 certification gate..."

CORE_EXIT=0
bash scripts/core-certify.sh 2>&1 | tee "${CORE_LOG}" || CORE_EXIT=$?

MIR_LOG="/tmp/mir-validation.log"
echo "check-integrity: running MIR strict semantic subset gate..."

MIR_EXIT=0
(
    # Use explicit short-circuiting instead of relying on `set -e` inside a
    # piped subshell; otherwise an early cargo failure can be logged but not
    # carried into the aggregate gate status.
    cargo test -p vais-mir --test lower_strict_tests --release &&
        cargo test -p vais-mir --test core_strict_fixtures --release &&
        cargo test -p vais-mir --test interpreter_tests --release &&
        cargo test -p vais-mir --test validation_tests --release
) 2>&1 | tee "${MIR_LOG}" || MIR_EXIT=$?

CODEGEN_INVARIANT_LOG="/tmp/codegen-invariants.log"
echo "check-integrity: running codegen invariant quarantine gate..."

CODEGEN_INVARIANT_EXIT=0
(
    cargo test -p vais-codegen --test ret_invariant_test --release &&
        cargo test -p vais-codegen --test index_invariant_test --release &&
        cargo test -p vais-codegen --test call_arg_invariant_test --release
) 2>&1 | tee "${CODEGEN_INVARIANT_LOG}" || CODEGEN_INVARIANT_EXIT=$?

UNSAFE_AUDIT_LOG="/tmp/unsafe-audit.log"
echo "check-integrity: running unsafe documentation audit..."

UNSAFE_AUDIT_EXIT=0
bash scripts/unsafe-audit.sh 2>&1 | tee "${UNSAFE_AUDIT_LOG}" || UNSAFE_AUDIT_EXIT=$?

# ---------------------------------------------------------------------------
# Run ecosystem integrity tests
# ---------------------------------------------------------------------------
INTEGRITY_LOG="/tmp/integrity.log"
echo "check-integrity: running ecosystem integrity matrix..."

# Phase Ω P1.4 iter 114: pre-clean every `.vais-cache` directory under
# the std and vaisdb source trees so each run starts from the same
# state. Without this, a `.vais-cache` populated by a previous run can
# satisfy `--force-rebuild`'s metadata pass with a stale entry on one
# run and a fresh entry on the next, even when the source code is
# byte-identical. The 217–223 vaisdb pass-count drift documented in
# ROADMAP iter 109/113 retro had two contributors: (a) the shared
# `/tmp/__ok.ll` race fixed in `crates/vaisc/tests/integrity.rs` this
# iter, and (b) leftover cache state across runs, fixed here.
echo "check-integrity: cleaning .vais-cache before run..."
find "${REPO_ROOT}/std" -type d -name ".vais-cache" -exec rm -rf {} + 2>/dev/null || true
VAISDB_SRC="${REPO_ROOT}/../lang/packages/vaisdb"
if [ -d "${VAISDB_SRC}" ]; then
    find "${VAISDB_SRC}" -type d -name ".vais-cache" -exec rm -rf {} + 2>/dev/null || true
fi

# `--test-threads=1` serializes the integrity tests so two concurrent
# `vaisc build` invocations within the same package directory cannot
# race on the shared `.vais-cache`. The integrity suite is the only
# place we serialize; the broader test runs stay parallel.
INTEGRITY_EXIT=0
cargo test -p vaisc --test integrity --release -- --nocapture --test-threads=1 2>&1 | tee "${INTEGRITY_LOG}" || INTEGRITY_EXIT=$?

# ---------------------------------------------------------------------------
# Run phase158 e2e tests
# ---------------------------------------------------------------------------
PHASE158_LOG="/tmp/phase158.log"
echo "check-integrity: running phase158 e2e tests..."

PHASE158_EXIT=0
cargo test -p vaisc --test e2e --release phase158 2>&1 | tee "${PHASE158_LOG}" || PHASE158_EXIT=$?

# ---------------------------------------------------------------------------
# Run std/http_client runtime smoke tests
# ---------------------------------------------------------------------------
HTTP_CLIENT_RUNTIME_LOG="/tmp/http-client-runtime-smoke.log"
echo "check-integrity: running std/http_client runtime smoke tests..."

HTTP_CLIENT_RUNTIME_EXIT=0
cargo test -p vaisc --test e2e --release phase_http_client_runtime -- --nocapture --test-threads=1 2>&1 | tee "${HTTP_CLIENT_RUNTIME_LOG}" || HTTP_CLIENT_RUNTIME_EXIT=$?

# ---------------------------------------------------------------------------
# Run VaisDB runtime smoke tests
# ---------------------------------------------------------------------------
VAISDB_RUNTIME_LOG="/tmp/vaisdb-runtime-smoke.log"
echo "check-integrity: running VaisDB runtime smoke tests..."

VAISDB_RUNTIME_EXIT=0
# Runtime smoke fixtures compile and execute child binaries. Run them serially
# and without libtest stdout capture so child-process diagnostics and cache
# cleanup order are deterministic inside the aggregate integrity gate.
cargo test -p vaisc --test e2e --release phase_vaisdb_runtime_smoke -- --nocapture --test-threads=1 2>&1 | tee "${VAISDB_RUNTIME_LOG}" || VAISDB_RUNTIME_EXIT=$?

# ---------------------------------------------------------------------------
# Run vais-server runtime smoke tests
# ---------------------------------------------------------------------------
SERVER_RUNTIME_LOG="/tmp/vais-server-runtime-smoke.log"
echo "check-integrity: running vais-server runtime smoke tests..."

SERVER_RUNTIME_EXIT=0
cargo test -p vaisc --test e2e --release phase_vais_server_runtime_smoke -- --nocapture --test-threads=1 2>&1 | tee "${SERVER_RUNTIME_LOG}" || SERVER_RUNTIME_EXIT=$?

# ---------------------------------------------------------------------------
# Run vais-web runtime smoke tests
# ---------------------------------------------------------------------------
WEB_RUNTIME_LOG="/tmp/vais-web-runtime-smoke.log"
WEB_RUNTIME_DIR="${REPO_ROOT}/../lang/packages/vais-web/packages/kit"
echo "check-integrity: running vais-web runtime smoke tests..."

WEB_RUNTIME_EXIT=0
if [ ! -d "${WEB_RUNTIME_DIR}" ]; then
    echo "vais-web runtime smoke directory missing: ${WEB_RUNTIME_DIR}" | tee "${WEB_RUNTIME_LOG}"
    WEB_RUNTIME_EXIT=1
else
    (
        cd "${WEB_RUNTIME_DIR}"
        NPM_TOKEN="${NPM_TOKEN:-}" pnpm exec vitest run \
            __tests__/e2e/vais-server-bridge.test.ts \
            __tests__/e2e/vais-web-route-hydration.test.ts \
            __tests__/e2e/vais-web-adapter-runtime.test.ts \
            __tests__/e2e/vais-web-node-live.test.ts \
            __tests__/e2e/vais-web-cloud-adapter-runtime.test.ts \
            __tests__/e2e/vais-web-browser-bundle-runtime.test.ts \
            __tests__/e2e/vais-web-real-browser-runtime.test.ts \
            __tests__/e2e/vais-web-platform-output-runtime.test.ts \
            __tests__/e2e/vais-web-production-bundle-runtime.test.ts \
            __tests__/e2e/vais-web-file-routing-production-runtime.test.ts \
            __tests__/e2e/vais-web-cross-browser-hydration-runtime.test.ts \
            __tests__/e2e/vais-web-ssr-data-production-runtime.test.ts \
            __tests__/e2e/vais-web-server-action-production-runtime.test.ts \
            __tests__/e2e/vais-web-server-action-auth-rate-production-runtime.test.ts \
            __tests__/e2e/vais-web-server-action-file-upload-production-runtime.test.ts \
            __tests__/e2e/vais-web-cloudflare-miniflare-runtime.test.ts \
            __tests__/e2e/vais-web-cloudflare-live-deploy.test.ts
    ) 2>&1 | tee "${WEB_RUNTIME_LOG}" || WEB_RUNTIME_EXIT=$?
fi

# ---------------------------------------------------------------------------
# Run cross_package_schema validation gate (Master Plan v16 Step 14 stage 0)
# ---------------------------------------------------------------------------
# The gate proves that a typed change to a shared schema.vais propagates
# to .vais consumers (vaisdb-style + vais-server-style) AND to .ts
# consumers via the generated .d.ts. Source of truth:
# compiler/tests/empirical/cross_package_schema/. See also Step 8 stage 5
# (commit 676e92fb) and design doc compiler/docs/design/cross-package-schema.md.

CROSS_PKG_SCHEMA_LOG="/tmp/vais-cross-package-schema.log"
CROSS_PKG_SCHEMA_EXIT=0
CROSS_PKG_SCHEMA_PASSED=0
CROSS_PKG_SCHEMA_TOTAL=2  # positive + negative
CROSS_PKG_SCHEMA_GATE="${REPO_ROOT}/tests/empirical/cross_package_schema/tests/gate.sh"

if [ -x "${CROSS_PKG_SCHEMA_GATE}" ]; then
    echo "check-integrity: running cross_package_schema gate (positive + negative)..."
    : > "${CROSS_PKG_SCHEMA_LOG}"
    POS_EXIT=0
    bash "${CROSS_PKG_SCHEMA_GATE}" positive >> "${CROSS_PKG_SCHEMA_LOG}" 2>&1 || POS_EXIT=$?
    NEG_EXIT=0
    bash "${CROSS_PKG_SCHEMA_GATE}" negative >> "${CROSS_PKG_SCHEMA_LOG}" 2>&1 || NEG_EXIT=$?
    if [ "${POS_EXIT}" -eq 0 ]; then
        CROSS_PKG_SCHEMA_PASSED=$((CROSS_PKG_SCHEMA_PASSED + 1))
    fi
    if [ "${NEG_EXIT}" -eq 0 ]; then
        CROSS_PKG_SCHEMA_PASSED=$((CROSS_PKG_SCHEMA_PASSED + 1))
    fi
    if [ "${POS_EXIT}" -ne 0 ] || [ "${NEG_EXIT}" -ne 0 ]; then
        CROSS_PKG_SCHEMA_EXIT=1
    fi
else
    echo "check-integrity: cross_package_schema gate.sh missing at ${CROSS_PKG_SCHEMA_GATE}" | tee -a "${CROSS_PKG_SCHEMA_LOG}"
    CROSS_PKG_SCHEMA_EXIT=1
fi

# ---------------------------------------------------------------------------
# Parse summary lines from logs
# ---------------------------------------------------------------------------
CORE_SUMMARY="$(grep "CORE_CERTIFICATION pass=" "${CORE_LOG}" 2>/dev/null | tail -1 || true)"
CORE_SUMMARY="${CORE_SUMMARY:-CORE_CERTIFICATION pass=? fail=? total=?}"

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

# Parse cargo test passed count from log
parse_cargo_passed() {
    local log="$1"
    # Look for "test result: ok. N passed" line in cargo test output
    # Anchor match from the start of the number to avoid matching digits in other fields
    grep "test result:" "${log}" 2>/dev/null \
        | tail -1 \
        | sed 's/test result: [^.]*\. \([0-9][0-9]*\) passed.*/\1/' \
        || true
}

parse_cargo_running() {
    local log="$1"
    grep "^running [0-9][0-9]* test" "${log}" 2>/dev/null \
        | tail -1 \
        | sed 's/running \([0-9][0-9]*\) test.*/\1/' \
        || true
}

parse_vitest_passed() {
    local log="$1"
    grep "Tests" "${log}" 2>/dev/null \
        | grep "passed" \
        | tail -1 \
        | sed 's/.*Tests[[:space:]]*\([0-9][0-9]*\) passed.*/\1/' \
        || true
}

parse_vitest_total() {
    # Matches both shapes:
    #   "Tests  N passed (N)"
    #   "Tests  N passed | M skipped (TOTAL)"
    # Returns the parenthesised total. Falls back via caller's :- when sed
    # finds no match (e.g. all-skipped runs that print no `passed` token).
    local log="$1"
    grep "Tests" "${log}" 2>/dev/null \
        | grep "passed" \
        | tail -1 \
        | sed 's/.*(\([0-9][0-9]*\)).*/\1/' \
        || true
}

PHASE158_PASSED="$(parse_cargo_passed "${PHASE158_LOG}")"
PHASE158_PASSED="${PHASE158_PASSED:-0}"
PHASE158_TOTAL="$(parse_cargo_running "${PHASE158_LOG}")"
PHASE158_TOTAL="${PHASE158_TOTAL:-18}"
HTTP_CLIENT_RUNTIME_PASSED="$(parse_cargo_passed "${HTTP_CLIENT_RUNTIME_LOG}")"
HTTP_CLIENT_RUNTIME_PASSED="${HTTP_CLIENT_RUNTIME_PASSED:-0}"
HTTP_CLIENT_RUNTIME_TOTAL="$(parse_cargo_running "${HTTP_CLIENT_RUNTIME_LOG}")"
HTTP_CLIENT_RUNTIME_TOTAL="${HTTP_CLIENT_RUNTIME_TOTAL:-${HTTP_CLIENT_RUNTIME_PASSED}}"
HTTP_CLIENT_RUNTIME_TOTAL="${HTTP_CLIENT_RUNTIME_TOTAL:-0}"
VAISDB_RUNTIME_PASSED="$(parse_cargo_passed "${VAISDB_RUNTIME_LOG}")"
VAISDB_RUNTIME_PASSED="${VAISDB_RUNTIME_PASSED:-0}"
VAISDB_RUNTIME_TOTAL="$(parse_cargo_running "${VAISDB_RUNTIME_LOG}")"
VAISDB_RUNTIME_TOTAL="${VAISDB_RUNTIME_TOTAL:-${VAISDB_RUNTIME_PASSED}}"
VAISDB_RUNTIME_TOTAL="${VAISDB_RUNTIME_TOTAL:-0}"
SERVER_RUNTIME_PASSED="$(parse_cargo_passed "${SERVER_RUNTIME_LOG}")"
SERVER_RUNTIME_PASSED="${SERVER_RUNTIME_PASSED:-0}"
SERVER_RUNTIME_TOTAL="$(parse_cargo_running "${SERVER_RUNTIME_LOG}")"
SERVER_RUNTIME_TOTAL="${SERVER_RUNTIME_TOTAL:-${SERVER_RUNTIME_PASSED}}"
SERVER_RUNTIME_TOTAL="${SERVER_RUNTIME_TOTAL:-0}"
WEB_RUNTIME_PASSED="$(parse_vitest_passed "${WEB_RUNTIME_LOG}")"
WEB_RUNTIME_PASSED="${WEB_RUNTIME_PASSED:-0}"
WEB_RUNTIME_TOTAL="$(parse_vitest_total "${WEB_RUNTIME_LOG}")"
WEB_RUNTIME_TOTAL="${WEB_RUNTIME_TOTAL:-${WEB_RUNTIME_PASSED}}"
WEB_RUNTIME_TOTAL="${WEB_RUNTIME_TOTAL:-0}"

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

# Runtime smoke baseline checks (2026-05-03 lock).
# These complement the `cargo test exit=0` checks below: even if the suite
# exits 0, a silent reduction in promoted-test count is flagged.
if [ -n "${HTTP_CLIENT_RUNTIME_PASSED}" ] && [ "${HTTP_CLIENT_RUNTIME_PASSED}" -lt "${INTEGRITY_HTTP_CLIENT_RUNTIME_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: http_client_runtime baseline=${INTEGRITY_HTTP_CLIENT_RUNTIME_MIN} current=${HTTP_CLIENT_RUNTIME_PASSED}/${HTTP_CLIENT_RUNTIME_TOTAL}\n"
fi
if [ -n "${VAISDB_RUNTIME_PASSED}" ] && [ "${VAISDB_RUNTIME_PASSED}" -lt "${INTEGRITY_VAISDB_RUNTIME_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: vaisdb_runtime baseline=${INTEGRITY_VAISDB_RUNTIME_MIN} current=${VAISDB_RUNTIME_PASSED}/${VAISDB_RUNTIME_TOTAL}\n"
fi
if [ -n "${SERVER_RUNTIME_PASSED}" ] && [ "${SERVER_RUNTIME_PASSED}" -lt "${INTEGRITY_SERVER_RUNTIME_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: vais_server_runtime baseline=${INTEGRITY_SERVER_RUNTIME_MIN} current=${SERVER_RUNTIME_PASSED}/${SERVER_RUNTIME_TOTAL}\n"
fi
if [ -n "${WEB_RUNTIME_PASSED}" ] && [ "${WEB_RUNTIME_PASSED}" -lt "${INTEGRITY_WEB_RUNTIME_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: vais_web_runtime baseline=${INTEGRITY_WEB_RUNTIME_MIN} current=${WEB_RUNTIME_PASSED}/${WEB_RUNTIME_TOTAL}\n"
fi
if [ -n "${PHASE158_PASSED}" ] && [ "${PHASE158_PASSED}" -lt "${INTEGRITY_BACKEND_PHASE158_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: backend_phase158 baseline=${INTEGRITY_BACKEND_PHASE158_MIN} current=${PHASE158_PASSED}/${PHASE158_TOTAL}\n"
fi
if [ "${CROSS_PKG_SCHEMA_PASSED}" -lt "${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN}" ]; then
    REGRESSION=1
    REGRESSION_MSG="${REGRESSION_MSG}  REGRESSION: cross_package_schema baseline=${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN} current=${CROSS_PKG_SCHEMA_PASSED}/${CROSS_PKG_SCHEMA_TOTAL}\n"
fi

# ---------------------------------------------------------------------------
# Final result
# ---------------------------------------------------------------------------
OVERALL_EXIT=0

if [ "${INTEGRITY_EXIT}" -ne 0 ]; then
    echo ""
    echo "ECOSYSTEM MATRIX FAILED: cargo test integrity exited ${INTEGRITY_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${CORE_EXIT}" -ne 0 ]; then
    echo ""
    echo "CORE FAILED: core-certify exited ${CORE_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${MIR_EXIT}" -ne 0 ]; then
    echo ""
    echo "MIR FAILED: structural validation exited ${MIR_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${CODEGEN_INVARIANT_EXIT}" -ne 0 ]; then
    echo ""
    echo "CODEGEN INVARIANTS FAILED: quarantine gate exited ${CODEGEN_INVARIANT_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${UNSAFE_AUDIT_EXIT}" -ne 0 ]; then
    echo ""
    echo "UNSAFE AUDIT FAILED: unsafe documentation gate exited ${UNSAFE_AUDIT_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${PHASE158_EXIT}" -ne 0 ]; then
    echo ""
    echo "BACKEND REGRESSION FAILED: cargo test phase158 exited ${PHASE158_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${HTTP_CLIENT_RUNTIME_EXIT}" -ne 0 ]; then
    echo ""
    echo "HTTP CLIENT RUNTIME SMOKE FAILED: cargo test phase_http_client_runtime exited ${HTTP_CLIENT_RUNTIME_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${VAISDB_RUNTIME_EXIT}" -ne 0 ]; then
    echo ""
    echo "VAISDB RUNTIME SMOKE FAILED: cargo test phase_vaisdb_runtime_smoke exited ${VAISDB_RUNTIME_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${SERVER_RUNTIME_EXIT}" -ne 0 ]; then
    echo ""
    echo "SERVER RUNTIME SMOKE FAILED: cargo test phase_vais_server_runtime_smoke exited ${SERVER_RUNTIME_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${WEB_RUNTIME_EXIT}" -ne 0 ]; then
    echo ""
    echo "WEB RUNTIME SMOKE FAILED: vitest vais-web runtime smoke exited ${WEB_RUNTIME_EXIT}"
    OVERALL_EXIT=1
fi

if [ "${CROSS_PKG_SCHEMA_EXIT}" -ne 0 ]; then
    echo ""
    echo "CROSS PACKAGE SCHEMA GATE FAILED: gate.sh exited ${CROSS_PKG_SCHEMA_EXIT}"
    cat "${CROSS_PKG_SCHEMA_LOG}" 2>/dev/null | tail -20
    OVERALL_EXIT=1
fi

if [ "${REGRESSION}" -ne 0 ]; then
    echo ""
    echo "ECOSYSTEM REGRESSION FAILED: threshold regression detected"
    printf "%b" "${REGRESSION_MSG}"
    OVERALL_EXIT=1
fi

print_gate_summary() {
    echo ""

    if [ "${CORE_EXIT}" -eq 0 ]; then
        echo "CORE OK: ${CORE_SUMMARY}"
    else
        echo "CORE FAIL: exit=${CORE_EXIT}"
    fi

    if [ "${MIR_EXIT}" -eq 0 ]; then
        echo "MIR OK: lower_strict_tests + core_strict_fixtures + interpreter_tests + validation_tests"
    else
        echo "MIR FAIL: exit=${MIR_EXIT}"
    fi

    if [ "${CODEGEN_INVARIANT_EXIT}" -eq 0 ]; then
        echo "CODEGEN OK: ret/index/call-arg invariant tests"
    else
        echo "CODEGEN FAIL: exit=${CODEGEN_INVARIANT_EXIT}"
    fi

    if [ "${UNSAFE_AUDIT_EXIT}" -eq 0 ]; then
        echo "UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0"
    else
        echo "UNSAFE AUDIT FAIL: exit=${UNSAFE_AUDIT_EXIT}"
    fi

    if [ "${INTEGRITY_EXIT}" -eq 0 ] && [ "${REGRESSION}" -eq 0 ]; then
        echo "ECOSYSTEM OK: syntax=${SYNTAX_TOTAL}/? stages=${STAGES_TOTAL}/? std=${STD_PASS}/${STD_TOTAL} vaisdb=${VAISDB_PASS}/${VAISDB_TOTAL}"
    else
        echo "ECOSYSTEM FAIL: syntax=${SYNTAX_TOTAL}/? stages=${STAGES_TOTAL}/? std=${STD_PASS}/${STD_TOTAL} vaisdb=${VAISDB_PASS}/${VAISDB_TOTAL}"
    fi

    if [ "${PHASE158_EXIT}" -eq 0 ]; then
        echo "BACKEND OK: phase158=${PHASE158_PASSED}/${PHASE158_TOTAL}"
    else
        echo "BACKEND FAIL: exit=${PHASE158_EXIT} phase158=${PHASE158_PASSED}/${PHASE158_TOTAL}"
    fi

    if [ "${HTTP_CLIENT_RUNTIME_EXIT}" -eq 0 ]; then
        echo "HTTP CLIENT RUNTIME OK: smoke=${HTTP_CLIENT_RUNTIME_PASSED}/${HTTP_CLIENT_RUNTIME_TOTAL}"
    else
        echo "HTTP CLIENT RUNTIME FAIL: exit=${HTTP_CLIENT_RUNTIME_EXIT} smoke=${HTTP_CLIENT_RUNTIME_PASSED}/${HTTP_CLIENT_RUNTIME_TOTAL}"
    fi

    if [ "${VAISDB_RUNTIME_EXIT}" -eq 0 ]; then
        echo "VAISDB RUNTIME OK: smoke=${VAISDB_RUNTIME_PASSED}/${VAISDB_RUNTIME_TOTAL}"
    else
        echo "VAISDB RUNTIME FAIL: exit=${VAISDB_RUNTIME_EXIT} smoke=${VAISDB_RUNTIME_PASSED}/${VAISDB_RUNTIME_TOTAL}"
    fi

    if [ "${SERVER_RUNTIME_EXIT}" -eq 0 ]; then
        echo "SERVER RUNTIME OK: smoke=${SERVER_RUNTIME_PASSED}/${SERVER_RUNTIME_TOTAL}"
    else
        echo "SERVER RUNTIME FAIL: exit=${SERVER_RUNTIME_EXIT} smoke=${SERVER_RUNTIME_PASSED}/${SERVER_RUNTIME_TOTAL}"
    fi

    if [ "${WEB_RUNTIME_EXIT}" -eq 0 ]; then
        echo "WEB RUNTIME OK: smoke=${WEB_RUNTIME_PASSED}/${WEB_RUNTIME_TOTAL}"
    else
        echo "WEB RUNTIME FAIL: exit=${WEB_RUNTIME_EXIT} smoke=${WEB_RUNTIME_PASSED}/${WEB_RUNTIME_TOTAL}"
    fi

    if [ "${CROSS_PKG_SCHEMA_EXIT}" -eq 0 ]; then
        echo "CROSS PACKAGE SCHEMA OK: gate=${CROSS_PKG_SCHEMA_PASSED}/${CROSS_PKG_SCHEMA_TOTAL}"
    else
        echo "CROSS PACKAGE SCHEMA FAIL: exit=${CROSS_PKG_SCHEMA_EXIT} gate=${CROSS_PKG_SCHEMA_PASSED}/${CROSS_PKG_SCHEMA_TOTAL}"
    fi
}

if [ "${OVERALL_EXIT}" -eq 0 ]; then
    print_gate_summary
    echo "INTEGRITY OK: core=ok mir=ok codegen=ok unsafe_audit=ok ecosystem=ok backend=ok http_client_runtime=ok vaisdb_runtime=ok server_runtime=ok web_runtime=ok cross_package_schema=ok"
    exit 0
else
    print_gate_summary
    echo "INTEGRITY FAILED: one or more gates failed"
    exit 1
fi
