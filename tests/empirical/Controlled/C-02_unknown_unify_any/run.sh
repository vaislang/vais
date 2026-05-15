#!/usr/bin/env bash
# Controlled-02 — Unknown unify-any (generic inference variable).
# Site: unification.rs:220 (vais-types/src/inference/unification.rs)
# Classification: Controlled (NOT A4) — generic inference is correct.
# Probe: identity function with type parameter T, called with i64 literal.

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
  echo "DRIFT: Controlled-02 exit changed." >&2
  echo "  expected: $EXPECTED  actual: $ACTUAL_EXIT" >&2
  exit 1
fi

echo "Controlled-02 OK: id<T> generic inference, runtime returns ${ACTUAL_EXIT} (correct)."
