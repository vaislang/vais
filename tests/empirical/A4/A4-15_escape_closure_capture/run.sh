#!/usr/bin/env bash
# A4-15 — Escape closure captured-environment loss (master-plan v37 add).
#
# Closure returned from a function and called later loses its captured
# environment because the capture frame lives on the caller's stack and
# is freed at return. STEP7_FINDINGS F-18 (2026-05-04) first observed.
#
# Probe asserts CURRENT BEHAVIOR: vaisc check passes, build succeeds,
# runtime exit ≠ 42 (the well-typed result).

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
  echo "DRIFT: A4-15 vaisc check now FAILS — escape closure may be type-check-rejected." >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

# Stage 2: build must succeed.
BUILD_OUTPUT="$( "$VAISC" build "$WORK/probe.vais" -o "$WORK/probe_bin" 2>&1 || true )"
if [[ ! -x "$WORK/probe_bin" ]]; then
  echo "DRIFT: A4-15 build did not produce probe_bin:" >&2
  echo "$BUILD_OUTPUT" >&2
  exit 1
fi

# Stage 3: runtime exit must NOT be 42.
RUN_EXIT=0
"$WORK/probe_bin" >/dev/null 2>&1 && RUN_EXIT=0 || RUN_EXIT=$?
if [[ "$RUN_EXIT" == "42" ]]; then
  echo "DRIFT: A4-15 runtime returned 42 (well-typed result) — escape closure may have" >&2
  echo "  been correctly fixed or build happened to land capture in right stack slot." >&2
  echo "  Investigate; flip assertion_kind from exit_not to exact_exit if surface fixed." >&2
  exit 1
fi

echo "A4-15 OK: escape closure silently accepted; runtime exit=${RUN_EXIT} (≠42)."
