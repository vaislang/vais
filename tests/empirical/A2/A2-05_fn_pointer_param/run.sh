#!/usr/bin/env bash
# A2-05 — fn-pointer parameters in std API (bounded to existing std use).
# Master Plan v17 Order Step 9 (A2 promotions).
# Predicate: see compiler/docs/certification/A2_SUBSETS.md §A2-05 (TBD).
#
# Two-probe runner:
#   probe_pos.vais  multi-impl dispatch via fn-pointer parameter must
#                   compile + run + exit 50 (= 20 + 30). Multi-impl
#                   verified to NOT silently constant-fold to first
#                   impl (unlike A2-03 dyn dispatch — see F-23).
#   probe_neg.vais  passes an i64 literal where fn(i64) -> i64 expected.
#                   Type checker rejects at vaisc check with E001.

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
  echo "DRIFT: A2-05 positive probe no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe_pos.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos binary" >&2
  exit 2
fi

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "50" ]]; then
  echo "DRIFT: A2-05 positive exit=${POS_EXIT}, expected 50 (20 + 30 from double(10) + triple(10)). If you see 40 (= 20+20) or 60 (= 30+30), fn-pointer dispatch may have regressed to A2-03-style impl-selection bug — see STEP7_FINDINGS F-23." >&2
  exit 1
fi

# ── Negative probe ────────────────────────────────────────────────────────
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"

NEG_OUT="$( "$VAISC" check "$WORK/probe_neg.vais" 2>&1 || true )"
NEG_EXIT=0
"$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 || NEG_EXIT=$?

if [[ "$NEG_EXIT" == "0" ]]; then
  echo "DRIFT: A2-05 negative probe now type-checks — fn-pointer parameter type rejection may have regressed." >&2
  exit 1
fi

if ! grep -qE "fn\(.*\)|expected" <<< "$NEG_OUT"; then
  echo "DRIFT: A2-05 negative check failed but stderr unfamiliar:" >&2
  echo "$NEG_OUT" >&2
  exit 1
fi

echo "A2-05 OK: positive multi-impl fn-pointer dispatch exits 50; negative i64-as-fn-pointer rejected at vaisc check."
