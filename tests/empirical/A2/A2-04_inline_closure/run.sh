#!/usr/bin/env bash
# A2-04 — Closures (no escape, inline-only).
# Master Plan v16 Order Step 9 (A2 promotions).
# Predicate (in A2_SUBSETS.md §A2-04 — pending):
#   1. Closure literal `|args| body` is passed directly to a function
#      parameter typed `|args| -> ret`.
#   2. The closure does not escape past the inline call (i.e. it is
#      not stored in a variable, returned, or captured into a struct).
#   3. Captured environment values are i64 / f64 / bool / str (Core types).
#
# Two-probe runner:
#   probe_pos.vais  inline closure literal — must compile + run + exit 42.
#   probe_neg.vais  closure ESCAPES (returned from make_adder). This is outside
#                    the A2-04 predicate and is now hard-blocked by the A4-15
#                    type-checker detector with E001 + escape closure + A4-15.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
[[ -x "$VAISC" ]] || { echo "FIXTURE_BROKEN: vaisc not found" >&2; exit 2; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT

# ── Positive probe ────────────────────────────────────────────────────────
cp "$DIR/probe_pos.vais" "$WORK/probe_pos.vais"
"$VAISC" check "$WORK/probe_pos.vais" >/dev/null 2>&1 || {
    echo "DRIFT: A2-04 positive probe no longer type-checks." >&2
    exit 1
}
( cd "$WORK" && "$VAISC" probe_pos.vais >/dev/null 2>&1 )
[[ -x "$WORK/probe_pos" ]] || { echo "FIXTURE_BROKEN: vaisc did not produce probe_pos" >&2; exit 2; }

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "42" ]]; then
    echo "DRIFT: A2-04 positive exit=${POS_EXIT}, expected 42" >&2
    exit 1
fi

# ── Negative probe — escaping closure rejects at type check ───────────────
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"
CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe_neg.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 && CHECK_EXIT=0 || CHECK_EXIT=$?
if [[ "$CHECK_EXIT" == "0" ]]; then
    echo "DRIFT: A2-04 negative now type-checks — escape closure detector may have regressed." >&2
    exit 1
fi

REQUIRED=("E001" "escape closure" "A4-15")
for pat in "${REQUIRED[@]}"; do
    if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
        echo "DRIFT: A2-04 negative rejected but stderr lacks '$pat':" >&2
        echo "$CHECK_OUTPUT" >&2
        exit 1
    fi
done

echo "A2-04 OK: positive inline closure exits 42; negative escape closure rejects at vaisc check with E001 + A4-15."
