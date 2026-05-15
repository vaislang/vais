#!/usr/bin/env bash
# A4-15 — Escape closure captured-environment loss (HARD-BLOCKED 2026-05-08).
#
# TC-level escape detector at vais-types/checker_expr/stmts.rs::Stmt::Return
# + vais-types/checker_fn.rs::check_function trailing-expression branch.
# Probe rejects with E001 mentioning 'escape closure' + 'A4-15'.
#
# Opt-out via VAIS_REJECT_A4_15=0 restores legacy silent accept (for legacy
# harness only; this fixture asserts the default-strict behavior).

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_BROKEN: vaisc not found at $VAISC" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$DIR/probe.vais" "$WORK/probe.vais"

# Stage 1: vaisc check must FAIL with E001 + escape closure + A4-15.
CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 && CHECK_EXIT=0 || CHECK_EXIT=$?
if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: A4-15 vaisc check now SUCCEEDS — escape closure detector may have" >&2
  echo "  been disabled. Restore default-strict behavior (VAIS_REJECT_A4_15 != '0')." >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

REQUIRED=("E001" "escape closure" "A4-15")
for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: A4-15 check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "A4-15 OK: escape closure rejected by TC detector with E001 + A4-15 marker."
