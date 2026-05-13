#!/usr/bin/env bash
# check-integrity.sh - Vais main-scoped integrity gate.
#
# This runner aggregates the integrity checks that are currently reproducible
# from the main compiler tree plus the explicitly documented local workspace
# schema gates. It is not the full DB/server/web runtime aggregate gate; those
# promoted runtime counts remain integration evidence until their fixtures and
# package gates are ported to main.
#
# Usage:
#   bash scripts/check-integrity.sh
#
# Environment:
#   VAISC=/path/to/vaisc                 compiler binary for schema gates
#   TSC=/path/to/tsc                     TypeScript compiler for schema gates
#   INTEGRITY_SKIP_BROWSER=1             skip browser playground smoke
#   INTEGRITY_SKIP_SCHEMA=1              skip cross-package schema gates
#   INTEGRITY_FULL_RUST=1                also run workspace clippy + tests
#   INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN=15
#   INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN=9

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN="${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN:-15}"
INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN="${INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN:-9}"

STATUS=0
SUMMARY=()

log() {
  printf 'check-integrity: %s\n' "$*"
}

fail_gate() {
  local name="$1"
  local detail="$2"
  STATUS=1
  SUMMARY+=("${name}=fail")
  printf 'check-integrity: FAIL: %s: %s\n' "${name}" "${detail}" >&2
}

pass_gate() {
  local name="$1"
  local detail="${2:-ok}"
  SUMMARY+=("${name}=${detail}")
  printf 'check-integrity: OK: %s: %s\n' "${name}" "${detail}"
}

run_gate() {
  local name="$1"
  shift
  log "running ${name}..."
  if "$@"; then
    pass_gate "${name}"
  else
    fail_gate "${name}" "command exited non-zero"
  fi
}

run_browser_compiler_gate() {
  log "running browser_compiler build..."
  if ! npm --prefix playground run browser-compiler:build; then
    fail_gate "browser_compiler" "wasm build failed"
    return
  fi

  log "running browser_compiler check..."
  if node scripts/check-browser-compiler-gate.mjs; then
    pass_gate "browser_compiler"
  else
    fail_gate "browser_compiler" "browser compiler smoke failed"
  fi
}

parse_assertions() {
  local log_file="$1"
  local fallback_total="$2"
  local line
  line="$(grep -E 'GATE ASSERTIONS: [0-9]+/[0-9]+' "${log_file}" | tail -n 1 || true)"
  if [[ "${line}" =~ GATE[[:space:]]ASSERTIONS:[[:space:]]([0-9]+)/([0-9]+) ]]; then
    printf '%s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  else
    printf '0 %s\n' "${fallback_total}"
  fi
}

ensure_vaisc() {
  if [[ -n "${VAISC:-}" ]]; then
    if [[ -x "${VAISC}" ]]; then
      return 0
    fi
    fail_gate "vaisc_binary" "VAISC is set but not executable: ${VAISC}"
    return 1
  fi

  local debug_vaisc="${REPO_ROOT}/target/debug/vaisc"
  if [[ ! -x "${debug_vaisc}" ]]; then
    log "building vaisc for schema gates..."
    if ! cargo build --locked -p vaisc; then
      fail_gate "vaisc_binary" "cargo build -p vaisc failed"
      return 1
    fi
  fi
  VAISC="${debug_vaisc}"
  export VAISC
}

run_cross_package_schema_gate() {
  local gate="${REPO_ROOT}/tests/empirical/cross_package_schema/tests/gate.sh"
  local log_file="/tmp/vais-cross-package-schema-main.log"
  local positive_passed positive_total negative_passed negative_total passed total

  : > "${log_file}"
  if ! bash "${gate}" positive >> "${log_file}" 2>&1; then
    fail_gate "cross_package_schema" "positive gate failed; see ${log_file}"
    return
  fi
  read -r positive_passed positive_total < <(parse_assertions "${log_file}" 11)

  : > "${log_file}.negative"
  if ! bash "${gate}" negative >> "${log_file}.negative" 2>&1; then
    cat "${log_file}.negative" >> "${log_file}"
    fail_gate "cross_package_schema" "negative gate failed; see ${log_file}"
    return
  fi
  read -r negative_passed negative_total < <(parse_assertions "${log_file}.negative" 4)
  cat "${log_file}.negative" >> "${log_file}"

  passed=$((positive_passed + negative_passed))
  total=$((positive_total + negative_total))
  if [[ "${passed}" -lt "${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN}" ]]; then
    fail_gate "cross_package_schema" "${passed}/${total} below ${INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN}; see ${log_file}"
  else
    pass_gate "cross_package_schema" "${passed}/${total}"
  fi
}

run_multi_domain_product_gate() {
  local gate="${REPO_ROOT}/tests/product/multi_domain_schema/tests/gate.sh"
  local log_file="/tmp/vais-multi-domain-product-main.log"
  local passed total

  if ! bash "${gate}" > "${log_file}" 2>&1; then
    fail_gate "multi_domain_product" "gate failed; see ${log_file}"
    return
  fi

  read -r passed total < <(parse_assertions "${log_file}" 9)
  if [[ "${passed}" -lt "${INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN}" ]]; then
    fail_gate "multi_domain_product" "${passed}/${total} below ${INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN}; see ${log_file}"
  else
    pass_gate "multi_domain_product" "${passed}/${total}"
  fi
}

run_gate "public_claims" node scripts/check-public-claims.mjs
run_gate "playground_mode_contract" node scripts/check-playground-mode-contract.mjs

if [[ "${INTEGRITY_SKIP_BROWSER:-0}" != "1" ]]; then
  run_browser_compiler_gate
fi

run_gate "rustfmt" cargo fmt --check
run_gate "emit_ts_tests" cargo test --locked -p vaisc --test emit_ts_skeleton --test emit_ts_exhaustiveness

if [[ "${INTEGRITY_FULL_RUST:-0}" == "1" ]]; then
  run_gate "workspace_clippy" cargo clippy --workspace --exclude vais-python --exclude vais-node -- -D warnings
  run_gate "workspace_tests" cargo test --workspace --exclude vais-python --exclude vais-node
fi

if [[ "${INTEGRITY_SKIP_SCHEMA:-0}" != "1" ]]; then
  if ensure_vaisc; then
    run_cross_package_schema_gate
    run_multi_domain_product_gate
  fi
fi

printf '\n'
if [[ "${STATUS}" -eq 0 ]]; then
  if [[ "${INTEGRITY_SKIP_BROWSER:-0}" == "1" ]]; then
    BROWSER_SUMMARY="browser=skipped"
  else
    BROWSER_SUMMARY="browser=ok"
  fi
  if [[ "${INTEGRITY_SKIP_SCHEMA:-0}" == "1" ]]; then
    SCHEMA_SUMMARY="schema=skipped"
  else
    SCHEMA_SUMMARY="schema=ok"
  fi
  printf 'INTEGRITY OK: scope=main public_claims=ok playground=ok %s emit_ts=ok %s' \
    "${BROWSER_SUMMARY}" "${SCHEMA_SUMMARY}"
  if [[ "${INTEGRITY_FULL_RUST:-0}" == "1" ]]; then
    printf ' rust_ci=ok'
  fi
  printf '\n'
  printf 'INTEGRITY SUMMARY: %s\n' "${SUMMARY[*]}"
  exit 0
fi

printf 'INTEGRITY FAILED: one or more main-scoped gates failed\n' >&2
printf 'INTEGRITY SUMMARY: %s\n' "${SUMMARY[*]}" >&2
exit 1
