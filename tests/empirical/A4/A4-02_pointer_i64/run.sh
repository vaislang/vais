#!/usr/bin/env bash
# A4-02 Pointer<T> ↔ i64 — empirical fixture runner.
#
# Surface (per master-plan.toml [[order.A4.runtime_silent]] entry id="A4-02"):
#   site:    unification.rs:410 (vais-types/src/inference/unification.rs)
#   probe:   take_i64(p) where p: *i64
#   expected (correct semantics): type-check rejects *i64 ↔ i64
#   actual   (current behavior):  type-checks, runs, returns a value
#                                  derived from `load i64` against the
#                                  pointer's stack address — not the
#                                  pointee value 42.
#
# assertion_kind = "exit_not" — the well-typed program (*p, dereferenced)
# would return 42. The defective unification rule passes the address
# itself (or the lower bits of one) as i64 to take_i64, yielding any
# non-42 value. Specific value depends on stack layout, OS, optimization;
# only "exit ≠ 42" is environment-stable.
#
# Exit codes from this runner:
#   0 — surface still has the v1-documented behavior (exit ≠ forbidden).
#   1 — runtime exit landed in forbidden_set — surface DRIFTED toward
#       the "correct" value, OR probe fails to type-check / build.
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

# Phase 1: type-check must currently PASS.
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks." >&2
  echo "  This is GOOD news — A4-02 may have been removed." >&2
  echo "  Migrate this fixture to a negative form per A4 README." >&2
  exit 1
fi

# Phase 2: full build must succeed.
( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce $WORK/probe" >&2
  exit 2
fi

# Phase 3: runtime exit must NOT be in forbidden_set.
ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

# Forbidden set: the value the well-typed program would have returned.
# A4-02: `take_i64(*p)` where p: *i64 = &val (val=42) → 42.
FORBIDDEN=(42)

for f in "${FORBIDDEN[@]}"; do
  if [[ "$ACTUAL_EXIT" == "$f" ]]; then
    echo "DRIFT: A4-02 exit landed on forbidden value $f." >&2
    echo "  This means the surface is no longer firing — runtime returned" >&2
    echo "  the value the WELL-TYPED program would have returned." >&2
    echo "  Either A4-02 has been fixed, or the probe is no longer" >&2
    echo "  exercising the surface.  Investigate before assuming a fix." >&2
    exit 1
  fi
done

echo "A4-02 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (≠ forbidden $(IFS=,; echo "${FORBIDDEN[*]}") — silent corruption confirmed)."
