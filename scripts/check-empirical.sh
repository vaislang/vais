#!/usr/bin/env bash
# check-empirical.sh — run every empirical fixture under
# compiler/tests/empirical/ and report pass / fail / skip totals.
#
# Per Master Plan v16 Order Step 7 §"CI integration: a future
# `compiler/scripts/check-empirical.sh` walks every fixture..." and
# Step 11 (A3 quarantine fixtures use the same runner contract).
#
# Each fixture directory must contain a `run.sh` executable. Exit
# semantics:
#   0  fixture passes (surface still has documented behaviour)
#   1  fixture DRIFT (behaviour changed; investigate before assuming fix)
#   2  FIXTURE_BROKEN (toolchain missing, probe malformed, etc.)
#
# This script collects all three classes and reports them, returning
# 0 only when EVERY fixture exited 0.
#
# Usage:
#   bash compiler/scripts/check-empirical.sh [class]
#     class: optional, one of A4 / A3 / A2 / A1 / Controlled / Rejected / Untested.
#            If omitted, runs all classes.

set -u

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/.." && pwd)"
EMPIRICAL="$COMPILER_ROOT/tests/empirical"

if [[ ! -d "$EMPIRICAL" ]]; then
  echo "ERROR: $EMPIRICAL does not exist" >&2
  exit 2
fi

CLASS_FILTER="${1:-}"
if [[ -n "$CLASS_FILTER" ]]; then
  case "$CLASS_FILTER" in
    A4|A3|A2|A1|Controlled|Rejected|Untested) ;;
    *) echo "Usage: $0 [A4|A3|A2|A1|Controlled|Rejected|Untested]" >&2; exit 2 ;;
  esac
fi

PASS=0
DRIFT=0
BROKEN=0
SKIPPED=0
FIXTURES=()
RESULTS=()

run_fixture() {
  local rs="$1"
  local fdir
  fdir="$(dirname "$rs")"
  local fid
  fid="$(basename "$fdir")"

  if [[ ! -x "$rs" ]]; then
    SKIPPED=$((SKIPPED + 1))
    RESULTS+=("SKIP $fid (run.sh not executable)")
    return
  fi
  local out
  out="$(bash "$rs" 2>&1 || true)"
  local rc
  bash "$rs" >/dev/null 2>&1
  rc=$?
  case "$rc" in
    0) PASS=$((PASS + 1));   RESULTS+=("PASS  $fid") ;;
    1) DRIFT=$((DRIFT + 1)); RESULTS+=("DRIFT $fid: $(echo "$out" | tail -1)") ;;
    2) BROKEN=$((BROKEN + 1)); RESULTS+=("BROKEN $fid: $(echo "$out" | tail -1)") ;;
    *) BROKEN=$((BROKEN + 1)); RESULTS+=("UNKNOWN($rc) $fid") ;;
  esac
}

for class_dir in "$EMPIRICAL"/*/; do
  [[ -d "$class_dir" ]] || continue
  class_name="$(basename "$class_dir")"
  case "$class_name" in
    A4|A3|A2|A1|Controlled|Rejected|Untested) ;;
    cross_package_schema) continue ;;  # has its own gate.sh, run separately
    *) continue ;;
  esac
  if [[ -n "$CLASS_FILTER" && "$class_name" != "$CLASS_FILTER" ]]; then
    continue
  fi
  for fixture in "$class_dir"*/; do
    [[ -d "$fixture" ]] || continue
    [[ -f "$fixture/run.sh" ]] || continue
    run_fixture "$fixture/run.sh"
    FIXTURES+=("$fixture")
  done
done

# Report.
for line in "${RESULTS[@]}"; do
  echo "$line"
done
echo ""
TOTAL=${#FIXTURES[@]}
echo "EMPIRICAL FIXTURES: $PASS pass / $DRIFT drift / $BROKEN broken / $SKIPPED skipped (total $TOTAL)"

if [[ $DRIFT -gt 0 || $BROKEN -gt 0 ]]; then
  exit 1
fi
exit 0
