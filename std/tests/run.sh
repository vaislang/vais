#!/bin/bash
# Stdlib self-test runner.
# For each test_*.vais in this directory:
#   1. compile to LLVM IR via vaisc
#   2. link with clang + runtime objects
#   3. execute (must exit 0)

set -u
cd "$(dirname "$0")"

VAISC="${VAISC:-$HOME/.cargo/bin/vaisc}"
RUNTIME_C="../../selfhost/runtime.c"
SYNC_RUNTIME_C="../sync_runtime.c"

RUNTIME_O="/tmp/vais_std_test_runtime.o"
SYNC_O="/tmp/vais_std_test_sync.o"
if [ ! -f "$RUNTIME_O" ] || [ "$RUNTIME_C" -nt "$RUNTIME_O" ]; then
  clang -c "$RUNTIME_C" -o "$RUNTIME_O" 2>/dev/null
fi
if [ ! -f "$SYNC_O" ] || [ "$SYNC_RUNTIME_C" -nt "$SYNC_O" ]; then
  clang -c "$SYNC_RUNTIME_C" -o "$SYNC_O" 2>/dev/null
fi

export VAIS_DEP_PATHS="/tmp/vais-lib/std"
export VAIS_STD_PATH="/tmp/vais-lib/std"

PASS=0
FAIL=0
XFAIL=0
FAIL_LIST=()
XFAIL_LIST=()

# Report any xfail_*.vais files (skipped) so open compiler bugs stay visible.
for xfail_file in xfail_*.vais; do
  [ -f "$xfail_file" ] || continue
  reason=$(head -1 "$xfail_file" | sed 's/^# XFAIL: *//')
  printf "%-50s XFAIL (%s)\n" "$xfail_file" "$reason"
  XFAIL=$((XFAIL+1))
  XFAIL_LIST+=("$xfail_file: $reason")
done

for test_file in test_*.vais; do
  [ -f "$test_file" ] || continue

  base=$(basename "$test_file" .vais)
  work_dir=$(mktemp -d /tmp/vais_std_${base}.XXXXXX)
  bin_out="$work_dir/${base}"
  log="$work_dir/${base}.log"

  rm -f /tmp/${base}*.ll 2>/dev/null
  ll_out="/tmp/${base}.ll"

  if ! "$VAISC" build "$test_file" --emit-ir -o "$ll_out" --force-rebuild >"$log" 2>&1; then
    printf "%-50s FAIL (compile, log: %s)\n" "$test_file" "$log"
    FAIL=$((FAIL+1))
    FAIL_LIST+=("$test_file: compile")
    continue
  fi

  ll_files=$(ls /tmp/${base}*.ll 2>/dev/null)
  if [ -z "$ll_files" ]; then ll_files="$ll_out"; fi
  if ! clang -O0 -o "$bin_out" $ll_files "$RUNTIME_O" "$SYNC_O" -lm >>"$log" 2>&1; then
    printf "%-50s FAIL (link, log: %s)\n" "$test_file" "$log"
    FAIL=$((FAIL+1))
    FAIL_LIST+=("$test_file: link")
    continue
  fi

  if ! "$bin_out" >>"$log" 2>&1; then
    rc=$?
    printf "%-50s FAIL (run, exit=%d, log: %s)\n" "$test_file" "$rc" "$log"
    FAIL=$((FAIL+1))
    FAIL_LIST+=("$test_file: run exit=$rc")
    continue
  fi

  printf "%-50s PASS\n" "$test_file"
  PASS=$((PASS+1))
  rm -rf "$work_dir"
done

echo
echo "=========================================="
echo "Stdlib self-tests: $PASS passed, $FAIL failed, $XFAIL xfail"
echo "=========================================="

if [ "$XFAIL" -gt 0 ]; then
  echo "Expected failures (open compiler bugs):"
  for f in "${XFAIL_LIST[@]}"; do echo "  - $f"; done
  echo
fi

if [ "$FAIL" -gt 0 ]; then
  echo "Failures:"
  for f in "${FAIL_LIST[@]}"; do echo "  - $f"; done
  exit 1
fi
exit 0
