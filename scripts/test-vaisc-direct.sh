#!/usr/bin/env bash
# NV-C2 direct-emitter gate for the Vais `vaisc` command.
#
# The direct engine is intentionally smaller than the full engine in this
# slice: it compiles one `fn main() -> Int` with a single Int return expression
# straight to LLVM IR through the minimal native emitter.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
fail=0

tmp="$(mktemp -d)"
src="$tmp/nv_c2_direct.vais"
cat > "$src" <<'SRC'
fn main() -> Int {
    return (6 * 7) + (8 / 4) - 2
}
SRC

if "$VAISC" emit-ir "$src" \
    --engine direct -o "$tmp/direct.ll" \
    >"$tmp/direct-emit.out" 2>"$tmp/direct-emit.err"; then
    main_count="$(grep -c '^define i64 @main()' "$tmp/direct.ll" || true)"
    if [ "$main_count" = "1" ] &&
        grep -q ' mul i64 ' "$tmp/direct.ll" &&
        grep -q ' sdiv i64 ' "$tmp/direct.ll" &&
        grep -q ' ret i64 ' "$tmp/direct.ll"; then
        echo "  PASS direct emit-ir emits arithmetic @main"
    else
        echo "  FAIL direct emit-ir shape"
        cat "$tmp/direct.ll"
        fail=1
    fi
else
    echo "  FAIL direct emit-ir"
    cat "$tmp/direct-emit.err"
    fail=1
fi

if clang -Wno-override-module -o "$tmp/direct_from_ir" "$tmp/direct.ll" >"$tmp/clang.log" 2>&1; then
    "$tmp/direct_from_ir"
    direct_from_ir=$?
    if [ "$direct_from_ir" = "42" ]; then
        echo "  PASS direct LLVM IR builds/runs (=42)"
    else
        echo "  FAIL direct LLVM IR got=$direct_from_ir want=42"
        fail=1
    fi
else
    echo "  FAIL direct LLVM IR does not build"
    cat "$tmp/clang.log"
    fail=1
fi

if "$VAISC" build "$src" \
    --engine direct -o "$tmp/direct_bin" \
    >"$tmp/direct-build.out" 2>"$tmp/direct-build.err"; then
    "$tmp/direct_bin"
    direct_build=$?
    if [ "$direct_build" = "42" ]; then
        echo "  PASS direct build binary runs (=42)"
    else
        echo "  FAIL direct build got=$direct_build want=42"
        fail=1
    fi
else
    echo "  FAIL direct build"
    cat "$tmp/direct-build.err"
    fail=1
fi

"$VAISC" run "$src" \
    --engine direct \
    >"$tmp/direct-run.out" 2>"$tmp/direct-run.err"
direct_run=$?
if [ "$direct_run" = "42" ]; then
    echo "  PASS direct run exits 42"
else
    echo "  FAIL direct run got=$direct_run want=42"
    cat "$tmp/direct-run.err"
    fail=1
fi

"$VAISC" run "$src" >"$tmp/full-run.out" 2>"$tmp/full-run.err"
full_run=$?
if [ "$full_run" = "42" ]; then
    echo "  PASS full engine matches direct result (=42)"
else
    echo "  FAIL full engine got=$full_run want=42"
    cat "$tmp/full-run.err"
    fail=1
fi

helper_src="$tmp/direct_helper_reject.vais"
cat > "$helper_src" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    return add(20, 22)
}
SRC

"$VAISC" emit-ir "$helper_src" \
    --engine direct -o "$tmp/helper.ll" \
    >"$tmp/helper.out" 2>"$tmp/helper.err"
helper_rc=$?
if [ "$helper_rc" != "0" ] &&
    grep -q "only a single .*fn main" "$tmp/helper.err" &&
    grep -q "help:" "$tmp/helper.err"; then
    echo "  PASS direct emitter rejects helper functions with P4 help"
else
    echo "  FAIL direct emitter helper rejection"
    cat "$tmp/helper.err"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C2 direct emitter OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
