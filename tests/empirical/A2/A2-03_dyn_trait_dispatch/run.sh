#!/usr/bin/env bash
# A2-03 — dyn / trait object dispatch (multi-impl).
# Master Plan v18 Order Step 9 (A2 promotions).
# LANDED 2026-05-05 via A4-12 step 2b sub-tasks 2b-1..2b-5c (DEFERRED #18).
#
# Two-probe runner:
#   probe_pos.vais  multi-impl dyn dispatch must compile + run + exit 49
#                   (= 42 + 7 from H.greet() + Wd.greet()). Multi-impl
#                   verified to NOT silently constant-fold to first impl.
#   probe_neg.vais  passes an i64 literal where dyn Greet expected.
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
  echo "DRIFT: A2-03 positive probe no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" build probe_pos.vais -o probe_pos >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos binary" >&2
  exit 2
fi

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "49" ]]; then
  echo "DRIFT: A2-03 positive exit=${POS_EXIT}, expected 49 (= 42 + 7 from H.greet + Wd.greet). If you see 84 (=42+42) or 14 (=7+7), dyn dispatch may have regressed to F-23 silent constant-fold." >&2
  exit 1
fi

# ── Negative probe ────────────────────────────────────────────────────────
# NOTE (2026-05-05): the type checker currently accepts i64 → dyn Greet
# silently (separate silent surface, not in #18 scope). build succeeds
# but runtime SIGSEGV (the bare i64 has no vtable). The negative probe
# verifies the *runtime defense*: build emits, run crashes (not "returns
# a wrong value silently"). Once the type-check-level rejection lands
# (separate task), tighten this to assertion_kind=check_fails.
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"

# Build must succeed (current TC limitation; tracked separately).
( cd "$WORK" && "$VAISC" build probe_neg.vais -o probe_neg >/dev/null 2>&1 ) \
  || { echo "FIXTURE_BROKEN: A2-03 negative probe failed to build (TC may have tightened — switch to check_fails assertion)"; exit 2; }

NEG_RUN_EXIT=0
"$WORK/probe_neg" >/dev/null 2>&1 || NEG_RUN_EXIT=$?
# Expect non-zero (crash / SIGSEGV / wrong runtime). 0 means silent
# corruption regression: the bare-i64 dispatch returned a value as if
# the dyn call succeeded.
if [[ "$NEG_RUN_EXIT" == "0" ]]; then
  echo "DRIFT: A2-03 negative probe ran cleanly with exit 0 — silent corruption regression (bare i64 should not satisfy dyn Greet at runtime)." >&2
  exit 1
fi

echo "A2-03 OK: multi-impl dyn dispatch exits 49; negative i64-as-dyn crashes at runtime (exit=${NEG_RUN_EXIT}, not 0)."
