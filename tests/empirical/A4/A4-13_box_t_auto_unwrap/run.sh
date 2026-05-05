#!/usr/bin/env bash
# A4-13 — Box<T> ↔ T silent accept at call site (master-plan v23 → v24).
#
# Probe asserts CURRENT BEHAVIOR (silent accept). Master-plan v23
# classifies this surface as Rejected; STEP7_FINDINGS F-24 (2026-05-05)
# proves it is in fact silent. This fixture pins the empirical
# evidence so reclassification to A4-13 in master-plan v24 has a
# permanent referent.
#
# When the hard-block phase lands (Step 13 stage 1+ for A4-13), flip
# this fixture from check_succeeds → check_fails with E001/Box patterns.

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
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" != "0" ]]; then
  echo "DRIFT: A4-13 vaisc check now FAILS — silent surface may have" >&2
  echo "  been hard-blocked. Update master-plan and flip this fixture" >&2
  echo "  to check_fails with E001/Box patterns." >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! grep -qF "OK No errors found" <<< "$CHECK_OUTPUT"; then
  echo "DRIFT: A4-13 check exited 0 but stdout lacks 'OK No errors found':" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

echo "A4-13 OK: vaisc check silently accepts Box<i64> where i64 expected (call site)."
