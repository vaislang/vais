#!/usr/bin/env bash
# NV-C2 direct-emitter gate for the Vais `vaisc` command.
#
# The direct engine is intentionally smaller than the full engine in this
# slice: it compiles Int helpers, locals, calls, simple struct locals, and
# control flow through the native direct path without the Python fallback.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
fail=0

tmp="$(mktemp -d)"
src="$tmp/nv_c2_direct.vais"
cat > "$src" <<'SRC'
fn main() -> Int {
    let a = 6
    let b = 7
    let c = 8
    return (a * b) + (c / 4) - 2
}
SRC

if "$VAISC" emit-ir "$src" \
    --engine direct -o "$tmp/direct.ll" \
    >"$tmp/direct-emit.out" 2>"$tmp/direct-emit.err"; then
    main_count="$(grep -c '^define i64 @main()' "$tmp/direct.ll" || true)"
    if [ "$main_count" = "1" ] &&
        grep -q ' mul .*i64 ' "$tmp/direct.ll" &&
        grep -q ' sdiv .*i64 ' "$tmp/direct.ll" &&
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

helper_src="$tmp/direct_helper_control.vais"
cat > "$helper_src" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    let mut i = 0
    let mut acc = 0
    while i < 6 {
        if i > 2 {
            acc = add(acc, i)
        }
        i = i + 1
    }
    return acc + 30
}
SRC

if "$VAISC" emit-ir "$helper_src" \
    --engine direct -o "$tmp/helper.ll" \
    >"$tmp/helper.out" 2>"$tmp/helper.err" &&
    grep -q '^define i64 @add' "$tmp/helper.ll" &&
    grep -q 'call i64 @add' "$tmp/helper.ll" &&
    grep -q ' br ' "$tmp/helper.ll"; then
    "$VAISC" run "$helper_src" --engine direct >"$tmp/helper-run.out" 2>"$tmp/helper-run.err"
    helper_run=$?
    if [ "$helper_run" = "42" ]; then
        echo "  PASS direct helper calls, locals, if, and while run (=42)"
    else
        echo "  FAIL direct helper/control got=$helper_run want=42"
        cat "$tmp/helper-run.err"
        fail=1
    fi
else
    echo "  FAIL direct helper/control emission"
    cat "$tmp/helper.err"
    fail=1
fi

struct_src="$tmp/direct_struct.vais"
cat > "$struct_src" <<'SRC'
struct Box {
    value: Int,
    bonus: Int,
}

fn inc(n: Int) -> Int {
    return n + 1
}

fn main() -> Int {
    let b = Box { value: 39, bonus: inc(1) }
    b.value = b.value + b.bonus + 1
    return b.value
}
SRC

if "$VAISC" emit-ir "$struct_src" \
    --engine direct -o "$tmp/struct.ll" \
    >"$tmp/struct.out" 2>"$tmp/struct.err" &&
    grep -q '%struct.Box = type' "$tmp/struct.ll" &&
    grep -q 'getelementptr .*%struct.Box' "$tmp/struct.ll"; then
    "$VAISC" run "$struct_src" --engine direct >"$tmp/struct-run.out" 2>"$tmp/struct-run.err"
    struct_run=$?
    if [ "$struct_run" = "42" ]; then
        echo "  PASS direct struct local literal, field read, and field write run (=42)"
    else
        echo "  FAIL direct struct got=$struct_run want=42"
        cat "$tmp/struct-run.err"
        fail=1
    fi
else
    echo "  FAIL direct struct emission"
    cat "$tmp/struct.err"
    fail=1
fi

fakebin="$tmp/fake-python"
mkdir -p "$fakebin"
cat > "$fakebin/python3" <<'PY'
#!/usr/bin/env sh
exit 99
PY
chmod +x "$fakebin/python3"
PATH="$fakebin:$PATH" "$VAISC" run "$helper_src" --engine direct >"$tmp/no-python.out" 2>"$tmp/no-python.err"
no_python_run=$?
if [ "$no_python_run" = "42" ]; then
    echo "  PASS direct engine does not invoke python3"
else
    echo "  FAIL direct engine used python3 or returned $no_python_run"
    cat "$tmp/no-python.err"
    fail=1
fi

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C2 direct emitter OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
