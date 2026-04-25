#!/bin/bash
# Vais language conformance test runner.
# For each .vais file under tests/lang/<category>/:
#   1. compile to LLVM IR via vaisc
#   2. link IR with clang
#   3. execute the binary
# A test PASSES if all three steps succeed AND the binary exits 0.
# A test FAILS otherwise — reason is captured.

set -u
cd "$(dirname "$0")"

VAISC="${VAISC:-$HOME/.cargo/bin/vaisc}"
RUNTIME_C="../../selfhost/runtime.c"
SYNC_RUNTIME_C="../../std/sync_runtime.c"

# Build runtime objects once.
RUNTIME_O="/tmp/vais_lang_test_runtime.o"
SYNC_O="/tmp/vais_lang_test_sync.o"
if [ ! -f "$RUNTIME_O" ] || [ "$RUNTIME_C" -nt "$RUNTIME_O" ]; then
  clang -c "$RUNTIME_C" -o "$RUNTIME_O" 2>/dev/null
fi
if [ ! -f "$SYNC_O" ] || [ "$SYNC_RUNTIME_C" -nt "$SYNC_O" ]; then
  clang -c "$SYNC_RUNTIME_C" -o "$SYNC_O" 2>/dev/null
fi

# Stdlib symlink expected at /tmp/vais-lib/std (matches vaisdb convention).
export VAIS_DEP_PATHS="/tmp/vais-lib/std"
export VAIS_STD_PATH="/tmp/vais-lib/std"

PASS=0
FAIL=0
XFAIL=0
FAIL_LIST=()
XFAIL_LIST=()

for category in [0-9][0-9]_*/; do
  for test_file in "$category"*.vais; do
    [ -f "$test_file" ] || continue

    # Files starting with `# XFAIL:` are expected-to-fail regressions —
    # they document an open compiler bug. Skip from PASS/FAIL counts
    # but report so the bug is visible.
    if head -1 "$test_file" | grep -q "^# XFAIL:"; then
      reason=$(head -1 "$test_file" | sed 's/^# XFAIL: *//')
      printf "%-60s XFAIL (%s)\n" "$test_file" "$reason"
      XFAIL=$((XFAIL+1))
      XFAIL_LIST+=("$test_file: $reason")
      continue
    fi

    base=$(basename "$test_file" .vais)
    work_dir=$(mktemp -d /tmp/vais_lang_${base}.XXXXXX)
    bin_out="$work_dir/${base}"
    log="$work_dir/${base}.log"

    # Clean global emit area so previous runs don't leak into this test.
    rm -f /tmp/${base}*.ll 2>/dev/null
    # Step 1: compile (vaisc emits to /tmp/<base>*.ll based on -o basename)
    ll_out="/tmp/${base}.ll"
    if ! "$VAISC" build "$test_file" --emit-ir -o "$ll_out" --force-rebuild >"$log" 2>&1; then
      printf "%-60s FAIL (compile)\n" "$test_file"
      FAIL=$((FAIL+1))
      FAIL_LIST+=("$test_file: compile")
      rm -rf "$work_dir"
      continue
    fi

    # Step 2: link (vaisc may emit multiple .ll files with the basename prefix)
    ll_files=$(ls /tmp/${base}*.ll 2>/dev/null)
    if [ -z "$ll_files" ]; then ll_files="$ll_out"; fi
    if ! clang -O0 -o "$bin_out" $ll_files "$RUNTIME_O" "$SYNC_O" -lm >>"$log" 2>&1; then
      printf "%-60s FAIL (link)\n" "$test_file"
      FAIL=$((FAIL+1))
      FAIL_LIST+=("$test_file: link")
      rm -rf "$work_dir"
      continue
    fi

    # Step 3: execute
    if ! "$bin_out" >>"$log" 2>&1; then
      printf "%-60s FAIL (run)\n" "$test_file"
      FAIL=$((FAIL+1))
      FAIL_LIST+=("$test_file: run (logs in $log)")
      # keep work_dir for inspection on failure
      continue
    fi

    printf "%-60s PASS\n" "$test_file"
    PASS=$((PASS+1))
    rm -rf "$work_dir"
  done
done

echo
echo "=========================================="
echo "Vais language conformance: $PASS passed, $FAIL failed, $XFAIL xfail"
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
