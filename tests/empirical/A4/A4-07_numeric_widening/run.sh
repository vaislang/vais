#!/usr/bin/env bash
# A4-07 Numeric widening — empirical fixture runner.
#
# Surface (per master-plan.toml [[order.A4.runtime_silent]] entry id="A4-07"):
#   site:    unification.rs:346 (vais-types/src/inference/unification.rs)
#   probe:   take_i64(small_i32) where small_i32: i32
#   expected (per design pending decision): explicit `as i64` required
#   actual   (current behavior):  type-checks, runs, returns 42 — runtime
#                                  correct, design pending.
#
# A4-07 is the only A4 entry where runtime semantics are CORRECT (no value
# corruption).  The classification as A4 reflects the AI-native design
# principle that implicit widening is an implicit behavior in user-facing
# semantics, even when the runtime value is right.  Master Plan v16 marks
# this as "design pending decision" — removal may be controversial because
# the runtime is right and the ergonomics cost is high.
#
# This runner pins the v1 evidence: type-check passes, run produces 42.
# Post-Step-13 the surface may be promoted to Controlled rather than removed.
#
# Exit codes:
#   0 — surface still has the v1-documented behavior.
#   1 — runtime exit code does not match expected.txt — surface DRIFTED.
#   2 — fixture itself broken.

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

if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )

if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce $WORK/probe" >&2
  exit 2
fi

ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

EXPECTED_EXIT="$(cat "$DIR/expected.txt" | tr -d '[:space:]')"

if [[ "$ACTUAL_EXIT" != "$EXPECTED_EXIT" ]]; then
  echo "DRIFT: A4-07 exit code changed." >&2
  echo "  expected: $EXPECTED_EXIT" >&2
  echo "  actual:   $ACTUAL_EXIT" >&2
  exit 1
fi

echo "A4-07 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (runtime correct, design pending)."
