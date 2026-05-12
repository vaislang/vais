#!/usr/bin/env bash
# A4-03 Auto-deref &T ↔ T — empirical fixture runner (check_fails).
#
# Surface: &i64 silently unified with i64 in function call (auto-deref
# without explicit deref operator).
# Site: unification.rs:754
# assertion_kind = "check_fails"

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
cp "$DIR/positive.vais" "$WORK/positive.vais"

CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: A4-03 check silently accepted implicit &T -> T:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! grep -qF "E001" <<< "$CHECK_OUTPUT"; then
  echo "DRIFT: A4-03 check failed without stable E001 diagnostic:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! grep -qF "&i64" <<< "$CHECK_OUTPUT"; then
  echo "DRIFT: A4-03 check failed without &i64 in diagnostic:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! VAIS_REJECT_A4_03=0 "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: A4-03 legacy opt-out no longer reproduces the old accept path." >&2
  exit 1
fi

if ! "$VAISC" check "$WORK/positive.vais" >/dev/null 2>&1; then
  echo "FIXTURE_BROKEN: explicit deref positive no longer type-checks." >&2
  "$VAISC" check "$WORK/positive.vais" >&2 || true
  exit 2
fi

( cd "$WORK" && "$VAISC" positive.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/positive" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce positive binary" >&2
  exit 2
fi

POSITIVE_EXIT=0
"$WORK/positive" || POSITIVE_EXIT=$?
if [[ "$POSITIVE_EXIT" != "42" ]]; then
  echo "FIXTURE_BROKEN: explicit deref positive exited $POSITIVE_EXIT, expected 42." >&2
  exit 2
fi

echo "A4-03 OK: default check rejects implicit &T -> T; legacy opt-out accepts; explicit deref exits 42."
