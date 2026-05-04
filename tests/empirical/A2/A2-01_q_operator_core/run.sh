#!/usr/bin/env bash
# A2-01 — `?` operator on Result/Option (Core-typed, single-module).
# Master Plan v16 Order Step 9 (A2 promotions).
# Predicate: see compiler/docs/certification/A2_SUBSETS.md §A2-01.
#
# Two-probe runner:
#   probe_pos.vais  must compile + run + exit 43.
#   probe_neg.vais  uses `?` in a function whose return type is plain
#                    i64 (predicate violation). Type checker rejects
#                    at vaisc check with E001 since the A4-11 fix
#                    landed (commit 204a6a03 / Step 13 stage 1).
#                    A2-NEG-DRIFT is RESOLVED.

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

# ── Positive probe ────────────────────────────────────────────────────────
cp "$DIR/probe_pos.vais" "$WORK/probe_pos.vais"

if ! "$VAISC" check "$WORK/probe_pos.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe_pos no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe_pos.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos binary" >&2
  exit 2
fi

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "43" ]]; then
  echo "DRIFT: A2-01 positive exit=${POS_EXIT}, expected 43" >&2
  exit 1
fi

# ── Negative probe ────────────────────────────────────────────────────────
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"

# A2-NEG-DRIFT was resolved when Step 13 A4-11 (`?` in non-Result function)
# landed strict default. The negative probe is now caught at vaisc check
# rather than at clang.
NEG_OUT="$( "$VAISC" check "$WORK/probe_neg.vais" 2>&1 || true )"
NEG_EXIT=0
"$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 || NEG_EXIT=$?

if [[ "$NEG_EXIT" == "0" ]]; then
  echo "DRIFT: A2-01 negative probe now type-checks — A4-11 may have regressed." >&2
  exit 1
fi

if ! grep -qE "Result|Option|expected" <<< "$NEG_OUT"; then
  echo "DRIFT: A2-01 negative check failed but stderr unfamiliar:" >&2
  echo "$NEG_OUT" >&2
  exit 1
fi

echo "A2-01 OK: positive exits 43; negative rejected at vaisc check (A2-NEG-DRIFT resolved via A4-11)."
