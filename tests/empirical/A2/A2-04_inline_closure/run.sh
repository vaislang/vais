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
#   probe_neg.vais  closure ESCAPES (returned from make_adder). Currently
#                    type-checks AND builds AND runs but produces a wrong
#                    runtime value (245 instead of 42). This is a new
#                    silent surface — STEP7_FINDINGS F-18 candidate.
#                    Runner asserts exit ≠ 42 (exit_not form).

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

# ── Negative probe — escaping closure produces wrong runtime ──────────────
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"
"$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 || {
    # GOOD news — type checker now rejects escape. Migrate fixture per
    # STEP7_FINDINGS F-18.
    echo "DRIFT: A2-04 negative now rejected at vaisc check — escape" >&2
    echo "  closures may have gained a stable diagnostic (good news;" >&2
    echo "  migrate fixture to check_fails)." >&2
    exit 1
}
( cd "$WORK" && "$VAISC" probe_neg.vais >/dev/null 2>&1 ) || {
    echo "DRIFT: A2-04 negative now fails at codegen — closures escape" >&2
    echo "  may have gained late-stage rejection (also good news;" >&2
    echo "  migrate fixture)." >&2
    exit 1
}
[[ -x "$WORK/probe_neg" ]] || { echo "FIXTURE_BROKEN: vaisc did not produce probe_neg" >&2; exit 2; }

NEG_EXIT=0
"$WORK/probe_neg" || NEG_EXIT=$?
# Forbidden: 42 — the value the well-typed escaping closure would return.
if [[ "$NEG_EXIT" == "42" ]]; then
    echo "DRIFT: A2-04 negative exit landed on 42 — escape closure may have" >&2
    echo "  been fixed, or runtime collided coincidentally with the correct" >&2
    echo "  value. Investigate." >&2
    exit 1
fi

echo "A2-04 OK: positive inline closure exits 42; negative escape closure exits ${NEG_EXIT} ≠ 42 (silent corruption — F-18 candidate)."
