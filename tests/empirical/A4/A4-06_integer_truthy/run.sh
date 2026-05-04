#!/usr/bin/env bash
# A4-06 Integer truthy — empirical fixture runner (Step 13 stage 1 LANDED).
#
# Original surface (master-plan.toml v16 §A4 runtime_silent A4-06):
#   site:    control_flow.rs:195,250,282,407 (4 sites in vais-types/src/
#            checker_expr/control_flow.rs).
#   defect:  Integer i64 in `I cond {}` / `EL I cond {}` / `cond ? a : b` /
#            `LW cond {}` was silently treated as bool — non-zero ⇒ truthy.
#            Per L-002 (no implicit behavior in user-facing semantics) the
#            type checker must require explicit `cond != 0` (or any bool
#            expression).
#
# Step 13 stage 1 status (compiler commit landing this fixture migration):
#   strict default ON. Legacy lenient mode opt-in via VAIS_REJECT_A4_06=0
#   escape hatch (kept for one cycle in case downstream finds a missed
#   migration site; will be removed once telemetry confirms no usage).
#
# Two-probe form:
#   probe_pos.vais  uses `I x != 0` — explicit form must keep working.
#                    Expected: vaisc check ok, binary exits 100.
#   probe_neg.vais  uses `I x` (the original integer-as-truthy form) —
#                    must be rejected at vaisc check with E001 type
#                    mismatch (expected bool, found i64 / vice versa).
#
# Exit codes from this runner:
#   0 — both probes behave per spec (positive ok, negative rejected).
#   1 — DRIFT: positive failed OR negative now accepted.
#   2 — fixture broken.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"

[[ -x "$VAISC" ]] || { echo "FIXTURE_BROKEN: vaisc not found at $VAISC" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ── Positive probe — explicit `!= 0` cond ─────────────────────────────────
cp "$DIR/probe_pos.vais" "$WORK/probe_pos.vais"
"$VAISC" check "$WORK/probe_pos.vais" >/dev/null 2>&1 || {
    echo "DRIFT: A4-06 positive probe (explicit != 0) no longer type-checks." >&2
    exit 1
}
( cd "$WORK" && "$VAISC" probe_pos.vais >/dev/null 2>&1 )
[[ -x "$WORK/probe_pos" ]] || { echo "FIXTURE_BROKEN: vaisc did not produce probe_pos" >&2; exit 2; }

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "100" ]]; then
    echo "DRIFT: A4-06 positive exit=${POS_EXIT}, expected 100" >&2
    exit 1
fi

# ── Negative probe — original integer-as-truthy form ──────────────────────
# Strict default rejects this at vaisc check. Legacy escape hatch
# VAIS_REJECT_A4_06=0 still accepts it (kept for one cycle).
cp "$DIR/probe_neg.vais" "$WORK/probe_neg.vais"
NEG_CHECK_EXIT=0
"$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 || NEG_CHECK_EXIT=$?
if [[ "$NEG_CHECK_EXIT" == "0" ]]; then
    echo "DRIFT: A4-06 negative probe now accepted at strict default." >&2
    echo "  Expected: vaisc check rejects integer-as-truthy with E001." >&2
    echo "  Strict default flip may have been reverted; investigate." >&2
    exit 1
fi

# Optional sanity: legacy escape hatch must still accept the negative form
# while the escape hatch exists.
LEGACY_CHECK_EXIT=0
VAIS_REJECT_A4_06=0 "$VAISC" check "$WORK/probe_neg.vais" >/dev/null 2>&1 || LEGACY_CHECK_EXIT=$?
if [[ "$LEGACY_CHECK_EXIT" != "0" ]]; then
    echo "INFO: A4-06 legacy escape hatch (VAIS_REJECT_A4_06=0) no longer accepts" >&2
    echo "  the negative probe. This is fine if the escape hatch was removed in a" >&2
    echo "  later cycle; otherwise update the fixture." >&2
    # Don't fail on this — escape-hatch removal is expected and benign.
fi

echo "A4-06 OK: positive (explicit != 0) exits 100; negative (integer-as-truthy) rejected at vaisc check."
