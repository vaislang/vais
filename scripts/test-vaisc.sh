#!/usr/bin/env bash
# NV-C0 smoke gate for the Vais `vaisc` command contract.
#
# This intentionally uses a `.vais` source path and the user-facing Vais command.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
fail=0

tmp="$(mktemp -d)"
src="$tmp/nv_c0_smoke.vais"
cat > "$src" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    let x = add(20, 22)
    return x
}
SRC

if "$VAISC" emit-ir "$src" -o "$tmp/native.ll" >"$tmp/emit.out" 2>"$tmp/emit.err"; then
    main_count="$(grep -c '^define i64 @main()' "$tmp/native.ll" || true)"
    add_count="$(grep -c '^define i64 @add' "$tmp/native.ll" || true)"
    if [ "$main_count" = "1" ] && [ "$add_count" = "1" ]; then
        echo "  PASS vaisc emit-ir emits @add and one @main"
    else
        echo "  FAIL vaisc emit-ir shape add=$add_count main=$main_count"
        fail=1
    fi
else
    echo "  FAIL vaisc emit-ir"
    cat "$tmp/emit.err"
    fail=1
fi

if clang -Wno-override-module -o "$tmp/native_from_ir" "$tmp/native.ll" >"$tmp/clang.log" 2>&1; then
    "$tmp/native_from_ir"
    native_from_ir=$?
    if [ "$native_from_ir" = "42" ]; then
        echo "  PASS emitted LLVM IR builds/runs (=42)"
    else
        echo "  FAIL emitted LLVM IR got=$native_from_ir want=42"
        fail=1
    fi
else
    echo "  FAIL emitted LLVM IR does not build"
    cat "$tmp/clang.log"
    fail=1
fi

if "$VAISC" build "$src" -o "$tmp/native_bin" --ir-out "$tmp/native_build.ll" >"$tmp/build.out" 2>"$tmp/build.err"; then
    "$tmp/native_bin"
    native_build=$?
    if [ "$native_build" = "42" ]; then
        echo "  PASS vaisc build binary runs (=42)"
    else
        echo "  FAIL vaisc build got=$native_build want=42"
        fail=1
    fi
else
    echo "  FAIL vaisc build"
    cat "$tmp/build.err"
    fail=1
fi

"$VAISC" run "$src" >"$tmp/run.out" 2>"$tmp/run.err"
native_run=$?
if [ "$native_run" = "42" ]; then
    echo "  PASS vaisc run exits 42"
else
    echo "  FAIL vaisc run got=$native_run want=42"
    cat "$tmp/run.err"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C0 smoke OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
