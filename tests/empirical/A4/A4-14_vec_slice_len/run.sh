#!/usr/bin/env bash
# A4-14 — Vec<T> ↔ &[T] permissive (.len() path) silent corruption.
#
# Master-plan v36 currently lists this as Controlled; empirical evidence
# (this fixture) shows runtime corruption (exit ≠ 3). v37 reclassifies
# Controlled → A4-14.
#
# Probe asserts the specified behavior: implicit &Vec<T> where &[T] is
# expected must compile to an actual slice fat value and return the Vec
# length. Set VAIS_REJECT_A4_14=1 only for strict migration audits.

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

# Stage 1: vaisc check must accept the specified coercion.
CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: A4-14 vaisc check rejects the specified &Vec<i64> -> &[i64] coercion:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

# Stage 2: build must succeed and runtime must return the well-typed length.
"$VAISC" build "$WORK/probe.vais" -o "$WORK/probe_bin" --force-rebuild >/dev/null 2>&1
RUN_EXIT=0
"$WORK/probe_bin" >/dev/null 2>&1 && RUN_EXIT=0 || RUN_EXIT=$?
if [[ "$RUN_EXIT" != "3" ]]; then
  echo "DRIFT: A4-14 runtime returned ${RUN_EXIT}; expected Vec length 3." >&2
  exit 1
fi

echo "A4-14 OK: &Vec<i64> is materialized as &[i64] and runtime returns 3."
