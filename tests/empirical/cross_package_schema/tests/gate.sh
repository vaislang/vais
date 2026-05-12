#!/usr/bin/env bash
# Cross-package schema validation gate (Master Plan v16 Order Step 8 stage 5).
#
# Per design v6 §Validation gate semantics. Single entrypoint with one
# argument: `positive` or `negative`.
#
# Three exit codes:
#   0 — Gate PASS (all assertions held)
#   1 — Gate FAIL (an assertion did not hold the expected way; schema
#       propagation invariant violated)
#   2 — FIXTURE_DRIFT (the fixture itself is broken: pristine baseline
#       does not type-check, mutation primitive produced no change,
#       or required toolchain is missing)
#
# Module-import note: Vais user-space module imports across separate
# files do not currently resolve outside std/. The gate works around
# this by concatenating `schema/user.vais` with each consumer file
# at gate time (`cat schema/user.vais consumer.vais > combined.vais`)
# before running `vaisc check`. The semantic intent is preserved: a
# consumer references the schema's User type and its `email` field;
# a schema mutation that breaks the consumer must surface as a check
# failure.

set -uo pipefail

# ── Argument parsing ─────────────────────────────────────────────────────
if [[ $# -ne 1 || ( "$1" != "positive" && "$1" != "negative" ) ]]; then
  echo "Usage: $0 positive|negative" >&2
  exit 2
fi
MODE="$1"
case "$MODE" in
  positive) ASSERTION_TOTAL=9 ;;
  negative) ASSERTION_TOTAL=4 ;;
esac

# ── Path resolution ──────────────────────────────────────────────────────
DIR="$(cd "$(dirname "$0")" && pwd)"
FIX="$(cd "$DIR/.." && pwd)"
COMPILER_ROOT="$(cd "$FIX/../../.." && pwd)"
WORKSPACE_ROOT="$(cd "$COMPILER_ROOT/.." && pwd)"

VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
TSC="${TSC:-${WORKSPACE_ROOT}/lang/packages/vais-web/node_modules/.pnpm/node_modules/.bin/tsc}"

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_DRIFT: vaisc not found at $VAISC" >&2
  echo "  Build with: cd compiler && cargo build --release --bin vaisc" >&2
  exit 2
fi
if [[ ! -x "$TSC" ]]; then
  echo "FIXTURE_DRIFT: tsc not found at $TSC" >&2
  echo "  Install via: cd lang/packages/vais-web && pnpm install" >&2
  exit 2
fi

SCHEMA="$FIX/schema/user.vais"
PRISTINE="$FIX/schema/.user.vais.pristine"
GEN="$FIX/gen/user.d.ts"

# ── Status tracking (cleanup must not mask failure) ──────────────────────
GATE_EXIT=0
record_fail() {
  local code="$1"
  if [[ "$GATE_EXIT" == "0" ]]; then
    GATE_EXIT="$code"
  fi
}

# Save pristine schema once.
cp "$SCHEMA" "$PRISTINE"

cleanup() {
  cp "$PRISTINE" "$SCHEMA"
  rm -f "$PRISTINE" "$SCHEMA.bak"
  exit "$GATE_EXIT"
}
trap cleanup EXIT INT TERM

# ── Helpers ──────────────────────────────────────────────────────────────
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"; cleanup' EXIT INT TERM

# Compile a consumer by concatenating schema + consumer into a single
# .vais file in $WORK and running `vaisc check` on it.
check_consumer() {
  local consumer_path="$1"
  local consumer_basename
  consumer_basename="$(basename "$consumer_path" .vais)"
  local combined="$WORK/${consumer_basename}.combined.vais"
  cat "$SCHEMA" "$consumer_path" > "$combined"
  "$VAISC" check "$combined" >/dev/null 2>&1
  return $?
}

# TS check uses tsc against the generated .d.ts plus the consumer file.
check_ts_consumer() {
  local ts_consumer="$FIX/consumers/vais_web_consumer.ts"
  "$TSC" --noEmit --strict --target ES2020 --moduleResolution node \
    "$GEN" "$ts_consumer" >/dev/null 2>&1
  return $?
}

# Cross-platform sed -i (BSD + GNU compatible via .bak extension).
sed_inplace() {
  local pattern="$1"
  local file="$2"
  sed -i.bak "$pattern" "$file"
  rm -f "${file}.bak"
}

# Confirm a mutation actually changed the file vs the pristine copy.
assert_diff() {
  local label="$1"
  if diff -q "$SCHEMA" "$PRISTINE" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: $label — mutation produced no change to $SCHEMA" >&2
    record_fail 2
    return 1
  fi
  return 0
}

# ── Phase 0 (positive) — pre-change baseline ─────────────────────────────
phase0_baseline() {
  echo ">>> Phase 0: pre-change baseline (4 assertions, FIXTURE_DRIFT on any fail)"

  if ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: phase0.1 emit-ts failed on pristine schema" >&2
    record_fail 2
    return 1
  fi
  echo "  [0.1] emit-ts on pristine schema       OK"

  if ! check_consumer "$FIX/consumers/vaisdb_table.vais"; then
    echo "FIXTURE_DRIFT: phase0.2 vaisdb_table consumer fails on pristine" >&2
    record_fail 2
    return 1
  fi
  echo "  [0.2] vaisc check vaisdb_table         OK"

  if ! check_consumer "$FIX/consumers/vais_server_api.vais"; then
    echo "FIXTURE_DRIFT: phase0.3 vais_server_api consumer fails on pristine" >&2
    record_fail 2
    return 1
  fi
  echo "  [0.3] vaisc check vais_server_api      OK"

  if ! check_ts_consumer; then
    echo "FIXTURE_DRIFT: phase0.4 tsc on TS consumer fails against pristine .d.ts" >&2
    record_fail 2
    return 1
  fi
  echo "  [0.4] tsc on vais_web_consumer.ts      OK"
  return 0
}

# ── Phase 1 (positive) — schema mutation ─────────────────────────────────
phase1_mutate_email() {
  echo ">>> Phase 1: typed change (rename email → mail)"

  sed_inplace 's/email: str/mail: str/' "$SCHEMA"
  if ! assert_diff "phase1.1 rename email → mail"; then
    return 1
  fi
  echo "  [1.1] sed mutation produced real diff   OK"

  if ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: phase1.2 emit-ts failed on mutated schema" >&2
    record_fail 2
    return 1
  fi
  echo "  [1.2] emit-ts on mutated schema         OK (well-typed change)"
  return 0
}

# ── Phase 2 (positive) — propagation (must FAIL) ─────────────────────────
phase2_propagation() {
  echo ">>> Phase 2: propagation (3 assertions, Gate FAIL if any succeeds)"

  if check_consumer "$FIX/consumers/vaisdb_table.vais"; then
    echo "GATE FAIL: phase2.1 vaisdb_table still type-checks after rename — propagation broken" >&2
    record_fail 1
    return 1
  fi
  echo "  [2.1] vaisdb_table now fails           OK"

  if check_consumer "$FIX/consumers/vais_server_api.vais"; then
    echo "GATE FAIL: phase2.2 vais_server_api still type-checks after rename" >&2
    record_fail 1
    return 1
  fi
  echo "  [2.2] vais_server_api now fails        OK"

  if check_ts_consumer; then
    echo "GATE FAIL: phase2.3 TS consumer still compiles after .d.ts regenerated" >&2
    record_fail 1
    return 1
  fi
  echo "  [2.3] vais_web_consumer.ts now fails   OK"
  return 0
}

# ── Negative gate ────────────────────────────────────────────────────────
# Note: original design called for required-field-addition propagation, but
# Vais struct constructors currently accept partial init silently (missing
# fields are not flagged at type-check). That is itself a separate A4-level
# finding — see compiler/tests/empirical/A4/STEP7_FINDINGS.md.
# Stage 5 negative gate uses a TYPE-CHANGE scenario instead: schema flips
# `email: str` to `email: i64`. Consumers that read `u.email` as `str`
# (`F handler_email(u: User) -> str { R u.email }`) now fail with E001
# 'expected str, found i64'.
run_negative() {
  echo ">>> Negative gate: field-type change propagates"

  if ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: negative phase0 emit-ts failed on pristine" >&2
    record_fail 2
    return 1
  fi

  # Flip email field type: str → i64.
  sed_inplace 's/email: str/email: i64/' "$SCHEMA"
  if ! assert_diff "negative.1 email type change"; then
    return 1
  fi
  echo "  [N.1] type-change mutation produced diff OK"

  if ! "$VAISC" check "$SCHEMA" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: negative.2 mutated schema does not type-check" >&2
    record_fail 2
    return 1
  fi
  echo "  [N.2] mutated schema still type-checks  OK"

  if ! "$VAISC" emit-ts "$SCHEMA" --output "$GEN" >/dev/null 2>&1; then
    echo "FIXTURE_DRIFT: negative.3 emit-ts failed on mutated schema" >&2
    record_fail 2
    return 1
  fi
  echo "  [N.3] emit-ts on mutated schema         OK"

  # Consumer `lookup_email(u) -> str { R u.email }` must now fail because
  # the field is now i64 not str.
  if check_consumer "$FIX/consumers/vaisdb_table.vais"; then
    echo "GATE FAIL: negative.4 vaisdb_table still type-checks — type-change did not propagate" >&2
    record_fail 1
    return 1
  fi
  echo "  [N.4] vaisdb_table now fails           OK"
  return 0
}

# ── Main dispatch ────────────────────────────────────────────────────────
case "$MODE" in
  positive)
    phase0_baseline && phase1_mutate_email && phase2_propagation
    ;;
  negative)
    run_negative
    ;;
esac

if [[ "$GATE_EXIT" == "0" ]]; then
  echo ""
  echo "GATE ASSERTIONS: ${ASSERTION_TOTAL}/${ASSERTION_TOTAL}"
  echo "GATE PASS ($MODE): all assertions held."
else
  echo ""
  echo "GATE ASSERTIONS: 0/${ASSERTION_TOTAL}"
fi
# Cleanup trap fires now and exits with $GATE_EXIT.
