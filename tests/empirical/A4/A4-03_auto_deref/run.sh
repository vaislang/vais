#!/usr/bin/env bash
# A4-03 Auto-deref &T ↔ T — empirical fixture runner (exit_not).
#
# Surface: &i64 silently unified with i64 in function call (auto-deref
# without explicit deref operator).
# Site: unification.rs:570
# assertion_kind = "exit_not"
# Forbidden set: [42] — well-typed take_i64(*r) where val=42 returns 42.

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
  echo "DRIFT: probe no longer type-checks (A4-03 may be removed)." >&2
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
    echo "DRIFT: A4-03 exit $f is in forbidden_set — surface no longer firing." >&2
    exit 1
  fi
done

echo "A4-03 OK: probe type-checks, compiles, runs, exits ${ACTUAL_EXIT} (≠ forbidden $(IFS=,; echo "${FORBIDDEN[*]}") — silent corruption confirmed)."
