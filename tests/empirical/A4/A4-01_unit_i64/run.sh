#!/usr/bin/env bash
# A4-01 Unit ↔ i64 — empirical fixture runner.
#
# Surface (per master-plan.toml [[order.A4.runtime_silent]] entry id="A4-01"):
#   site:    unification.rs:361 (vais-types/src/inference/unification.rs)
#   probe:   x: i64 = void_fn()  where void_fn returns ()
#   expected (correct semantics): emission rejected at type-check
#   actual   (current behavior):  type-checks, runs, exits with the LLVM
#                                  default for the void-channel value
#                                  (currently observed: 96 on macOS arm64
#                                  release build).
#
# This runner pins the CURRENT (defective) behavior.  When A4-01 is removed
# in Step 13, the compiler will reject the probe at type-check time and
# this runner's expected behavior changes.  At that time, replace this
# fixture with a NEGATIVE fixture (see compiler/tests/empirical/A4/
# README.md for migration guidance).
#
# Exit codes from this runner:
#   0 — surface still has the v1-documented behavior (A4-01 not yet removed).
#   1 — runtime exit code does not match expected.txt — surface DRIFTED.
#       Investigate before assuming this is a fix.
#   2 — fixture itself broken (compiler missing, probe fails to compile).

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
# fixture path: compiler/tests/empirical/A4/A4-01_unit_i64/
# ../../../../ = compiler/ (the compiler git repo root).
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_BROKEN: vaisc not found at $VAISC" >&2
  echo "  Build with: cd compiler && cargo build --release --bin vaisc" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$DIR/probe.vais" "$WORK/probe.vais"

# Type-check must pass (current behavior — this is what A4-01 documents).
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks." >&2
  echo "  This may mean A4-01 has been removed.  Update the fixture per" >&2
  echo "  compiler/tests/empirical/A4/README.md migration guidance." >&2
  exit 1
fi

# Compile to a binary in $WORK.
( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )

if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce $WORK/probe" >&2
  exit 2
fi

# Run and capture exit code (probe does not write stdout).
ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

EXPECTED_EXIT="$(cat "$DIR/expected.txt" | tr -d '[:space:]')"

if [[ "$ACTUAL_EXIT" != "$EXPECTED_EXIT" ]]; then
  echo "DRIFT: A4-01 exit code changed." >&2
  echo "  expected: $EXPECTED_EXIT (per expected.txt)" >&2
  echo "  actual:   $ACTUAL_EXIT" >&2
  echo "  Surface behavior has shifted.  Review before assuming this is a" >&2
  echo "  fix or a regression." >&2
  exit 1
fi

echo "A4-01 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (matches expected)."
