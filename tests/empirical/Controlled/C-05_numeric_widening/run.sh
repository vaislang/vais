#!/usr/bin/env bash
# Controlled-05 — Numeric widening (runtime correctness).
# Site: unification.rs:346
# Classification: Controlled at runtime (NOT A4 at runtime — runtime is
# correct). Note overlap: A4-07 (compiler/tests/empirical/A4/A4-07_numeric_widening)
# documents the SAME site as A4-runtime-silent because the DESIGN is
# violated even though the RUNTIME is correct. Both fixtures coexist
# intentionally — Controlled-05 proves runtime correctness, A4-07 proves
# the design violation is empirically present (not just hypothesized).
#
# Probe: i32 silently widens to i64 at function call.

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
  echo "FIXTURE_BROKEN: vaisc did not produce binary" >&2
  exit 2
fi

ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

EXPECTED="$(cat "$DIR/expected.txt" | tr -d '[:space:]')"

if [[ "$ACTUAL_EXIT" != "$EXPECTED" ]]; then
  echo "DRIFT: Controlled-05 exit changed." >&2
  echo "  expected: $EXPECTED  actual: $ACTUAL_EXIT" >&2
  exit 1
fi

echo "Controlled-05 OK: i32→i64 widening, take_i64(16)+1 = ${ACTUAL_EXIT} (runtime correct)."
