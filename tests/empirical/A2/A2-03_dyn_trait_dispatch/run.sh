#!/usr/bin/env bash
# A2-03 — dyn / trait object dispatch (multi-impl).
# Master Plan v18 Order Step 9 (A2 promotions).
# LANDED 2026-05-05 via A4-12 step 2b sub-tasks 2b-1..2b-5c (DEFERRED #18).
# DUAL-BACKEND COVERAGE 2026-05-05 via A4-12 step 2a-C sub-tasks 2a-C-1..2a-C-4 (DEFERRED #19).
#
# Two-probe runner, both backends:
#   probe_pos.vais  multi-impl dyn dispatch must compile + run + exit 49
#                   (= 42 + 7 from H.greet() + Wd.greet()). Multi-impl
#                   verified to NOT silently constant-fold to first impl.
#                   Verified on:
#                     - inkwell backend (default for `vaisc build`)
#                     - text-IR backend (VAIS_SINGLE_MODULE=1)
#                   Both must produce exit 49.
#   probe_neg.vais  passes an i64 literal where dyn Greet expected.
#                   `vaisc check` must reject with a stable E001 diagnostic
#                   before build/run; this prevents the historical runtime
#                   crash from a missing vtable.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
VAISC_FLAGS=(--no-update-check --timeout 0)

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_BROKEN: vaisc not found at $VAISC" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ── Positive probe — inkwell backend (default) ────────────────────────────
cp "$DIR/probe_pos.vais" "$WORK/probe_pos.vais"

if ! "$VAISC" "${VAISC_FLAGS[@]}" check "$WORK/probe_pos.vais" >/dev/null 2>&1; then
  echo "DRIFT: A2-03 positive probe no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" "${VAISC_FLAGS[@]}" build probe_pos.vais -o probe_pos_inkwell >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos_inkwell" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos_inkwell binary" >&2
  exit 2
fi

POS_EXIT_INKWELL=0
"$WORK/probe_pos_inkwell" || POS_EXIT_INKWELL=$?
if [[ "$POS_EXIT_INKWELL" != "49" ]]; then
  echo "DRIFT: A2-03 positive (inkwell) exit=${POS_EXIT_INKWELL}, expected 49 (= 42 + 7 from H.greet + Wd.greet). If you see 84 (=42+42) or 14 (=7+7), dyn dispatch may have regressed to F-23 silent constant-fold." >&2
  exit 1
fi

# ── Positive probe — text-IR backend ──────────────────────────────────────
# VAIS_SINGLE_MODULE=1 forces the text-IR codegen path. Verifies
# DEFERRED #19 (sorted_method_names + dispatch wiring) didn't drift.
( cd "$WORK" && VAIS_SINGLE_MODULE=1 "$VAISC" "${VAISC_FLAGS[@]}" build probe_pos.vais -o probe_pos_textir >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos_textir" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos_textir binary (text-IR backend)" >&2
  exit 2
fi

POS_EXIT_TEXTIR=0
"$WORK/probe_pos_textir" || POS_EXIT_TEXTIR=$?
if [[ "$POS_EXIT_TEXTIR" != "49" ]]; then
  echo "DRIFT: A2-03 positive (text-IR) exit=${POS_EXIT_TEXTIR}, expected 49. text-IR dispatch wiring (DEFERRED #19) may have regressed." >&2
  exit 1
fi

# ── Negative probe ────────────────────────────────────────────────────────
# W1-C/T-508: the type checker must reject i64 → dyn Greet before build/run.
# This replaces the historical runtime-crash guard with a product-readable
# diagnostic guard.
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"

NEG_CHECK_OUT="$WORK/probe_neg.check.out"
if "$VAISC" "${VAISC_FLAGS[@]}" check "$WORK/probe_neg.vais" >"$NEG_CHECK_OUT" 2>&1; then
  echo "DRIFT: A2-03 negative probe type-checked; bare i64 must not satisfy dyn Greet." >&2
  exit 1
fi

if ! grep -q 'error: error\[E001\] Type mismatch' "$NEG_CHECK_OUT"; then
  echo "DRIFT: A2-03 negative diagnostic missing top-level error[E001] envelope." >&2
  cat "$NEG_CHECK_OUT" >&2
  exit 1
fi

if ! grep -q 'expected dyn Greet' "$NEG_CHECK_OUT" || ! grep -q 'found i64' "$NEG_CHECK_OUT"; then
  echo "DRIFT: A2-03 negative diagnostic must mention expected dyn Greet and found i64." >&2
  cat "$NEG_CHECK_OUT" >&2
  exit 1
fi

if grep -q 'No errors found' "$NEG_CHECK_OUT"; then
  echo "DRIFT: A2-03 negative diagnostic reported check success text." >&2
  cat "$NEG_CHECK_OUT" >&2
  exit 1
fi

echo "A2-03 OK: multi-impl dyn dispatch exits 49 on inkwell + text-IR; negative i64-as-dyn rejects at check with error[E001]."
