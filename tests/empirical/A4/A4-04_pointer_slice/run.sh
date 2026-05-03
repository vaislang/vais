#!/usr/bin/env bash
# A4-04 Pointer<T> ↔ Slice<T>/SliceMut<T> — empirical fixture runner (exit_not).
# Site: unification.rs:417 (Phase 162)
# assertion_kind = "exit_not"
# Forbidden set: [4] — well-typed take_slice over a 4-element slice would return len=4.

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
  echo "DRIFT: probe no longer type-checks (A4-04 may be removed)." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce binary" >&2
  exit 2
fi

ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

FORBIDDEN=(4)
for f in "${FORBIDDEN[@]}"; do
  if [[ "$ACTUAL_EXIT" == "$f" ]]; then
    echo "DRIFT: A4-04 exit $f is in forbidden_set — surface no longer firing." >&2
    exit 1
  fi
done

echo "A4-04 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (≠ forbidden $(IFS=,; echo "${FORBIDDEN[*]}") — silent corruption confirmed)."
