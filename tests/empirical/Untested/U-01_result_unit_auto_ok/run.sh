#!/usr/bin/env bash
# Untested-01 → empirically RECLASSIFIED to Rejected
# Site: unification.rs:366 (Result ↔ Unit auto Ok/Some wrap)
#
# Master-plan v16 listed this surface as Untested / classification
# deferred (treat as A4 candidate by default). Empirical probe shows
# the type checker DOES reject — E001 'expected Result<i64,str>, found ()'.
# Recommendation: master-plan.toml should reclassify this entry from
# Untested to Rejected. See STEP7_FINDINGS for the reclassification log.
#
# assertion_kind = "check_fails"

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

CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: Untested-01 vaisc check now succeeds — Result/Unit auto-wrap" >&2
  echo "  surface may have been added. Investigate and reclassify." >&2
  exit 1
fi

REQUIRED=(
  "E001"
  "expected Result"
  "found ()"
)

for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: Untested-01 vaisc check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "Untested-01 OK: vaisc check exits ${CHECK_EXIT} with E001 — Unit → Result is rejected. RECLASSIFY to Rejected in master-plan."
