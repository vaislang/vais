#!/usr/bin/env bash
# A4-13 — Box<T> ↔ T silent accept at call site (master-plan v23 → v24).
#
# Probe asserts the hardened behavior: direct Box<T> where T is expected must
# fail with a stable type mismatch. Set VAIS_REJECT_A4_13=0 only for legacy
# drift investigation; default compiler behavior must stay strict.

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

CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: A4-13 vaisc check silently accepted Box<i64> where i64 is expected:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! grep -qF "E001" <<< "$CHECK_OUTPUT"; then
  echo "DRIFT: A4-13 check failed without stable E001 diagnostic:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

if ! grep -qF "Box<i64>" <<< "$CHECK_OUTPUT"; then
  echo "DRIFT: A4-13 check failed without Box<i64> in diagnostic:" >&2
  echo "$CHECK_OUTPUT" >&2
  exit 1
fi

echo "A4-13 OK: vaisc check rejects direct Box<i64> where i64 is expected."
