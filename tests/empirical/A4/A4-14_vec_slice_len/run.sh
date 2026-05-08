#!/usr/bin/env bash
# A4-14 — Vec<T> ↔ &[T] permissive (.len() path) silent corruption.
#
# Master-plan v36 currently lists this as Controlled; empirical evidence
# (this fixture) shows runtime corruption (exit ≠ 3). v37 reclassifies
# Controlled → A4-14.
#
# Probe asserts CURRENT BEHAVIOR: vaisc check passes, build succeeds,
# runtime exit is NOT 3.

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

# Stage 1: vaisc check must succeed (silent surface).
CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 && CHECK_EXIT=0 || CHECK_EXIT=$?
if [[ "$CHECK_EXIT" != "0" ]]; then
  echo "DRIFT: A4-14 vaisc check now FAILS — surface may have been hard-blocked." >&2
  echo "  Update master-plan (Controlled → A4-14 → Removed) and flip this fixture" >&2
  echo "  to check_fails with E001/Vec/Slice patterns." >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

# Stage 2: build must succeed (no late-codegen reject).
BUILD_OUTPUT="$( "$VAISC" build "$WORK/probe.vais" -o "$WORK/probe_bin" 2>&1 || true )"
if [[ ! -x "$WORK/probe_bin" ]]; then
  echo "DRIFT: A4-14 build did not produce probe_bin:" >&2
  echo "$BUILD_OUTPUT" >&2
  exit 1
fi

# Stage 3: runtime exit must NOT be 3.
RUN_EXIT=0
"$WORK/probe_bin" >/dev/null 2>&1 && RUN_EXIT=0 || RUN_EXIT=$?
if [[ "$RUN_EXIT" == "3" ]]; then
  echo "DRIFT: A4-14 runtime returned 3 (the well-typed result) — Vec ↔ Slice surface" >&2
  echo "  may have been tightened or test infra changed. Investigate." >&2
  exit 1
fi

echo "A4-14 OK: Vec<i64> ↔ &[i64] silently accepted; runtime exit=${RUN_EXIT} (≠3)."
