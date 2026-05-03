#!/usr/bin/env bash
# A4-05 Array → Pointer decay — empirical fixture runner (exit_not).
# Site: unification.rs:424
# assertion_kind = "exit_not"
# Forbidden set: [42] — well-typed take_i64(arr[0]) where arr[0]=42 returns 42.
# Defective rule passes the array's stack address (cast as *i64) instead.

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
  echo "DRIFT: probe no longer type-checks (A4-05 may be removed)." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce binary" >&2
  exit 2
fi

ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

FORBIDDEN=(42)
for f in "${FORBIDDEN[@]}"; do
  if [[ "$ACTUAL_EXIT" == "$f" ]]; then
    echo "DRIFT: A4-05 exit $f is in forbidden_set — surface no longer firing." >&2
    exit 1
  fi
done

echo "A4-05 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (≠ forbidden $(IFS=,; echo "${FORBIDDEN[*]}") — silent corruption confirmed)."
