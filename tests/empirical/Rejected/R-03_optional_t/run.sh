#!/usr/bin/env bash
# Rejected-03 — Optional ↔ T (bare i64 passed where Option<i64> expected).
# Site: unification.rs:98 (Site 23, master-plan v16 v1 probe)
# Classification: REJECTED at type-check (NOT A4)
#
# Probe: pass i64 literal 42 to a function expecting Option<i64>.
#        Type checker rejects with E001 'expected Option<i64>, found i64'.
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
  echo "DRIFT: Rejected-03 vaisc check now succeeds — Optional surface" >&2
  echo "  may have been changed to auto-wrap. Investigate." >&2
  exit 1
fi

REQUIRED=(
  "E001"
  "expected Option<i64>"
  "found i64"
)

for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: Rejected-03 vaisc check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "Rejected-03 OK: vaisc check exits ${CHECK_EXIT} with E001 'expected Option<i64>, found i64' — bare i64 → Option<i64> is rejected as documented."
