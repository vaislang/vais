#!/usr/bin/env bash
# A4-06 Integer truthy — empirical fixture runner.
#
# Surface (per master-plan.toml [[order.A4.runtime_silent]] entry id="A4-06"):
#   site:    control_flow.rs:188,243,273,396 (vais-types/src/checker_expr/control_flow.rs)
#   probe:   I x { 100 } EL { 200 } where x: i64
#   expected (correct semantics): type-check rejects implicit i64-as-bool
#   actual   (current behavior):  type-checks, runs, takes the truthy branch
#                                  for non-zero i64, exits 100.
#
# Per L-002 design (no implicit behavior in user-facing semantics): integer
# values must require explicit `!= 0` to be used as booleans.  Currently
# the type checker silently treats any non-zero i64 as `true` — the runtime
# observable is "correct" in the obvious sense but the design is violated.
#
# This runner pins the CURRENT (defective by design) behavior.  When A4-06
# is removed in Step 13, the compiler will reject the probe at type-check
# time and this fixture migrates to a negative form.
#
# Exit codes from this runner:
#   0 — surface still has the v1-documented behavior.
#   1 — runtime exit code does not match expected.txt — surface DRIFTED.
#   2 — fixture itself broken.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
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

if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks." >&2
  echo "  This may mean A4-06 has been removed.  Update the fixture per" >&2
  echo "  compiler/tests/empirical/A4/README.md migration guidance." >&2
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
  echo "DRIFT: A4-06 exit code changed." >&2
  echo "  expected: $EXPECTED_EXIT (per expected.txt)" >&2
  echo "  actual:   $ACTUAL_EXIT" >&2
  exit 1
fi

echo "A4-06 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (matches expected — design violated, runtime stable)."
