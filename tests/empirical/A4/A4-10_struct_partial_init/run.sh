#!/usr/bin/env bash
# A4-10 — Struct partial-init silent acceptance.
# Site: TBD (struct constructor type-check; site identification deferred).
# Discovered during Step 8 stage 5 negative-gate construction;
# STEP7_FINDINGS F-15. NOT yet in master-plan.toml [[phase.A4.runtime_silent]];
# Step 7 next iteration is responsible for adding it.
#
# Surface: a struct constructor that omits required fields type-checks
# successfully. The runtime zero-initializes the missing fields, so
# accessing them later returns 0 (or "" / null / etc. for non-numeric
# fields), masking the missing-field error.
#
# Probe: User struct with 4 fields; constructor only provides 3.
#        F returns u.age — would be 99 if age were explicitly set,
#        but is 0 in the partial init case.
#
# assertion_kind = "exit_not"
# Forbidden set: [99] — the value a well-typed (full-init) probe
# would have returned. Defective rule yields 0 (or whatever the
# zero-init produces). 99 was chosen because it is unlikely to be
# the runtime's spurious value (0 is the typical zero-init).

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

# Phase 1: type-check must currently PASS (this IS the surface).
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks." >&2
  echo "  This is GOOD news — A4-10 may have been fixed." >&2
  echo "  Migrate fixture to negative form per A4 README." >&2
  exit 1
fi

# Phase 2: full build must succeed.
( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce binary" >&2
  exit 2
fi

# Phase 3: runtime exit must NOT be in forbidden_set.
ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

FORBIDDEN=(99)
for f in "${FORBIDDEN[@]}"; do
  if [[ "$ACTUAL_EXIT" == "$f" ]]; then
    echo "DRIFT: A4-10 exit landed on forbidden value $f." >&2
    echo "  This means the surface is no longer firing — runtime returned" >&2
    echo "  the value the WELL-TYPED program would have returned." >&2
    echo "  Either A4-10 has been fixed (constructor now requires all" >&2
    echo "  fields), or the probe collided coincidentally." >&2
    exit 1
  fi
done

echo "A4-10 OK: probe type-checks, builds, runs, exits ${ACTUAL_EXIT} (≠ forbidden $(IFS=,; echo "${FORBIDDEN[*]}") — partial-init silent acceptance confirmed)."
