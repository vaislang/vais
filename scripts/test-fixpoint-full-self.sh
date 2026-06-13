#!/usr/bin/env bash
# Long self-host gate for compiler/self/fixpoint_full.vais.
#
# This verifies the full-source path, not just snippet-level codegen:
#   seed fixpoint_full -> generated first-generation compiler IR -> clang/run.
# It also checks that first-generation compilers can consume file-sized embedded
# sources again by retargeting their default compile("...") program to the real
# compiler/self/fixpoint*.vais sources, including fixpoint_full.vais itself.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAIS_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais-legacy/compiler}"
source "$HERE/scripts/legacy-vaisc-env.sh"
TR="$HERE/compiler/transpiler/legacy_vais_bootstrap.py"
SRC="$HERE/compiler/self/fixpoint_full.vais"
EMBED="$HERE/tools/embed_self_source.py"
NORM_IR="$HERE/tools/normalize_stage_ir.py"
fail=0
last_source_compiler_ll=""
last_emitted_ll=""

compare_stage_ir() {
  local left="$1" right="$2" label="$3" tmp bytes
  tmp="$(mktemp -d)"
  if [ -z "$left" ] || [ -z "$right" ]; then
    echo "  FAIL $label: missing stage IR path"; fail=1; return
  fi
  python3 "$NORM_IR" "$left" "$tmp/left.ll" \
    || { echo "  FAIL $label: normalize left"; fail=1; return; }
  python3 "$NORM_IR" "$right" "$tmp/right.ll" \
    || { echo "  FAIL $label: normalize right"; fail=1; return; }
  if cmp -s "$tmp/left.ll" "$tmp/right.ll"; then
    bytes="$(wc -c < "$tmp/left.ll" | tr -d ' ')"
    echo "  PASS $label normalized stage IR matches ($bytes bytes)";
  else
    echo "  FAIL $label normalized stage IR differs"
    diff -u "$tmp/left.ll" "$tmp/right.ll" | sed -n '1,160p'
    fail=1
  fi
}

run_full_probe() {
  local source="$1" label="$2" want="$3" mode="${4:-program}" tmp
  tmp="$(mktemp -d)"

  python3 "$EMBED" "$SRC" "$source" "$tmp/c.nl" \
    || { echo "  FAIL $label: embed"; fail=1; return; }
  python3 "$TR" "$tmp/c.nl" > "$tmp/c.vais" 2>"$tmp/transpile.err" \
    || { echo "  FAIL $label: transpile"; cat "$tmp/transpile.err"; fail=1; return; }
  legacy_vaisc_build "$tmp/c.vais" -o "$tmp/c" >"$tmp/build.log" 2>&1 \
    || { echo "  FAIL $label: compiler build"; cat "$tmp/build.log"; fail=1; return; }

  "$tmp/c" > "$tmp/source_compiler.ll"
  last_source_compiler_ll="$tmp/source_compiler.ll"
  local main_count neg_gep_count ir_bytes
  main_count="$(grep -c '^define i64 @main()' "$tmp/source_compiler.ll" || true)"
  neg_gep_count="$(grep -c 'i64 -[0-9]' "$tmp/source_compiler.ll" || true)"
  ir_bytes="$(wc -c < "$tmp/source_compiler.ll" | tr -d ' ')"
  if [ "$main_count" = "1" ]; then
    echo "  PASS $label emits one @main ($ir_bytes bytes)";
  else
    echo "  FAIL $label main count=$main_count"; fail=1
  fi
  if [ "$neg_gep_count" = "0" ]; then
    echo "  PASS $label emits no negative GEP indexes";
  else
    echo "  FAIL $label negative GEP count=$neg_gep_count"; fail=1
  fi

  clang -Wno-override-module -o "$tmp/source_compiler" "$tmp/source_compiler.ll" 2>"$tmp/clang1.err" \
    || { echo "  FAIL $label: generated compiler IR invalid"; cat "$tmp/clang1.err"; fail=1; return; }
  "$tmp/source_compiler" > "$tmp/emitted.ll"
  local source_rc=$?
  if [ "$source_rc" = "0" ]; then
    echo "  PASS $label generated compiler runs";
  else
    echo "  FAIL $label generated compiler exit=$source_rc"; fail=1
    echo "  emitted IR prefix:"
    head -n 120 "$tmp/emitted.ll"
    return
  fi
  clang -Wno-override-module -o "$tmp/emitted_bin" "$tmp/emitted.ll" 2>"$tmp/clang2.err" \
    || { echo "  FAIL $label: emitted IR invalid"; cat "$tmp/clang2.err"; fail=1; return; }
  last_emitted_ll="$tmp/emitted.ll"

  if [ "$mode" = "compiler" ]; then
    "$tmp/emitted_bin" > "$tmp/final.ll"
    local compiler_rc=$?
    if [ "$compiler_rc" = "0" ]; then
      echo "  PASS $label emitted compiler runs";
    else
      echo "  FAIL $label emitted compiler exit=$compiler_rc"; head -n 120 "$tmp/final.ll"; fail=1; return
    fi
    if grep -q "ret i64 $want" "$tmp/final.ll"; then
      echo "  PASS $label final IR emits ret i64 $want";
    else
      echo "  FAIL $label final IR missing ret i64 $want"; head -n 120 "$tmp/final.ll"; fail=1; return
    fi
    clang -Wno-override-module -o "$tmp/final_bin" "$tmp/final.ll" 2>"$tmp/clang3.err" \
      || { echo "  FAIL $label: final IR invalid"; cat "$tmp/clang3.err"; fail=1; return; }
    "$tmp/final_bin"; local got=$?
    if [ "$got" = "$want" ]; then
      echo "  PASS $label final binary runs (=$got)";
    else
      echo "  FAIL $label final binary got=$got want=$want"; fail=1
    fi
  else
    if grep -q "ret i64 $want" "$tmp/emitted.ll"; then
      echo "  PASS $label emits ret i64 $want";
    else
      echo "  FAIL $label emitted IR missing ret i64 $want"; head -n 120 "$tmp/emitted.ll"; fail=1
    fi
    "$tmp/emitted_bin"; local got=$?
    if [ "$got" = "$want" ]; then
      echo "  PASS $label emitted binary runs (=$got)";
    else
      echo "  FAIL $label emitted binary got=$got want=$want"; fail=1
    fi
  fi
}

run_full_probe "$SRC" "full-source fixpoint_full.vais self probe" 42
stage1_self_ir="$last_source_compiler_ll"

run_retarget_probe() {
  local target="$1" want="$2" label="$3" tmp_variant
  tmp_variant="$(mktemp -d)"
  python3 "$EMBED" "$SRC" "$target" "$tmp_variant/retargeted_fixpoint_full.vais" \
    || { echo "  FAIL full-source retarget: embed $(basename "$target") into compiler"; fail=1; return; }
  if [ "$fail" -eq 0 ]; then
    run_full_probe "$tmp_variant/retargeted_fixpoint_full.vais" "$label" "$want" compiler
  fi
}

run_retarget_probe "$HERE/compiler/self/fixpoint.vais" 24 "first-generation compiler consumes fixpoint.vais"
run_retarget_probe "$HERE/compiler/self/fixpoint2.vais" 50 "first-generation compiler consumes fixpoint2.vais"
run_retarget_probe "$HERE/compiler/self/fixpoint3.vais" 120 "first-generation compiler consumes fixpoint3.vais"
run_retarget_probe "$SRC" 42 "first-generation compiler consumes fixpoint_full.vais"
stage2_self_ir="$last_emitted_ll"

if [ "$fail" -eq 0 ]; then
  compare_stage_ir "$stage1_self_ir" "$stage2_self_ir" "fixpoint_full stage1/stage2 compiler output"
fi

[ "$fail" -eq 0 ] && echo "RESULT: fixpoint_full full-source self-host gate OK" || echo "RESULT: FAILURES"
exit $fail
