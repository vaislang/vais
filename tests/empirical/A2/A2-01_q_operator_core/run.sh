#!/usr/bin/env bash
# A2-01 — `?` operator on Result/Option (Core-typed, single-module).
# Master Plan v16 Order Step 9 (A2 promotions).
# Predicate: see compiler/docs/certification/A2_SUBSETS.md §A2-01.
#
# Two-probe runner:
#   probe_pos.vais  must compile + run + exit 43.
#   probe_neg.vais  uses `?` in a function whose return type is plain
#                    i64 (predicate violation). Currently the type
#                    checker accepts this and the failure surfaces at
#                    clang IR generation. That itself is a finding —
#                    A2-NEG-DRIFT — but it does prove the use is
#                    rejected somewhere in the build pipeline. Future
#                    iteration should tighten this to a type-check
#                    rejection.

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

# Currently type-check passes (A2-NEG-DRIFT finding). Build must fail.
BUILD_OUT="$( cd "$WORK" && "$VAISC" probe_neg.vais 2>&1 || true )"
BUILD_EXIT=0
( cd "$WORK" && "$VAISC" probe_neg.vais >/dev/null 2>&1 ) || BUILD_EXIT=$?

if [[ "$BUILD_EXIT" == "0" ]]; then
  echo "DRIFT: A2-01 negative probe built successfully — predicate" >&2
  echo "  violation should be rejected somewhere in the pipeline." >&2
  exit 1
fi

# Confirm clang failure (current behaviour). When the type checker is
# tightened to reject this earlier, change the assertion to look for
# `vaisc check` exit non-zero with a stable error code.
if ! grep -qE "clang compilation failed|error" <<< "$BUILD_OUT"; then
  echo "DRIFT: A2-01 negative build failed but stderr unfamiliar:" >&2
  echo "$BUILD_OUT" >&2
  exit 1
fi

echo "A2-01 OK: positive exits 43; negative is rejected (currently at clang stage — see A2_SUBSETS.md A2-NEG-DRIFT)."
