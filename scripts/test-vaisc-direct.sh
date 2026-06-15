#!/usr/bin/env bash
# NV-C2 direct-emitter gate for the Vais `vaisc` command.
#
# The direct engine is intentionally smaller than the full engine in this
# slice: it compiles Int helpers, locals, calls, simple struct locals and
# struct ABI helpers, plus control flow through the native direct path without
# the Python fallback.
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

abi_src="$tmp/direct_struct_abi.vais"
cat > "$abi_src" <<'SRC'
struct Pair {
    a: Int,
    b: Int,
}

fn make(x: Int) -> Pair {
    return Pair { a: x, b: x + 1 }
}

fn id(p: Pair) -> Pair {
    return p
}

fn sum(p: Pair) -> Int {
    return p.a + p.b
}

fn main() -> Int {
    let p = id(make(20))
    return sum(p) + sum(Pair { a: 0, b: 1 })
}
SRC

if "$VAISC" emit-ir "$abi_src" \
    --engine direct -o "$tmp/abi.ll" \
    >"$tmp/abi.out" 2>"$tmp/abi.err" &&
    grep -q '%struct.Pair = type' "$tmp/abi.ll" &&
    grep -q 'define .*@make' "$tmp/abi.ll" &&
    grep -q 'define .*@sum' "$tmp/abi.ll"; then
    "$VAISC" run "$abi_src" --engine direct >"$tmp/abi-run.out" 2>"$tmp/abi-run.err"
    abi_run=$?
    if [ "$abi_run" = "42" ]; then
        echo "  PASS direct struct parameter and return ABI runs (=42)"
    else
        echo "  FAIL direct struct ABI got=$abi_run want=42"
        cat "$tmp/abi-run.err"
        fail=1
    fi
else
    echo "  FAIL direct struct ABI emission"
    cat "$tmp/abi.err"
    fail=1
fi

list_src="$tmp/direct_list_int.vais"
cat > "$list_src" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let ys = [1, 2, xs.len()]
    return xs.sum() - xs.len() - xs[1] + ys[2] + ys.sum() - 4
}
SRC

if "$VAISC" emit-ir "$list_src" \
    --engine direct -o "$tmp/list.ll" \
    >"$tmp/list.out" 2>"$tmp/list.err" &&
    grep -q '__vais_list_int_sum' "$tmp/list.ll" &&
    grep -q 'getelementptr .*DirectListInt' "$tmp/list.ll"; then
    "$VAISC" run "$list_src" --engine direct >"$tmp/list-run.out" 2>"$tmp/list-run.err"
    list_run=$?
    if [ "$list_run" = "42" ]; then
        echo "  PASS direct local List<Int> push, len, index, literal, and sum run (=42)"
    else
        echo "  FAIL direct List<Int> got=$list_run want=42"
        cat "$tmp/list-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Int> emission"
    cat "$tmp/list.err"
    fail=1
fi

list_struct_src="$tmp/direct_list_struct_local.vais"
cat > "$list_struct_src" <<'SRC'
struct Box {
    value: Int,
}

fn main() -> Int {
    let xs: List<Box> = []
    xs.push(Box { value: 10 })
    xs.push(Box { value: 30 })
    let ys: List<Box> = [Box { value: 1 }]
    let zs: List<Box> = list()
    zs.push(Box { value: 2 })
    return xs[0].value + xs[1].value + xs.len() + ys[0].value + zs[0].value - 3
}
SRC

if "$VAISC" emit-ir "$list_struct_src" \
    --engine direct -o "$tmp/list-struct.ll" \
    >"$tmp/list-struct.out" 2>"$tmp/list-struct.err" &&
    grep -q 'DirectList_Box' "$tmp/list-struct.ll" &&
    grep -q 'getelementptr .*struct.Box' "$tmp/list-struct.ll"; then
    "$VAISC" run "$list_struct_src" --engine direct >"$tmp/list-struct-run.out" 2>"$tmp/list-struct-run.err"
    list_struct_run=$?
    if [ "$list_struct_run" = "42" ]; then
        echo "  PASS direct local List<Struct> push, len, index, and field read run (=42)"
    else
        echo "  FAIL direct List<Struct> got=$list_struct_run want=42"
        cat "$tmp/list-struct-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Struct> emission"
    cat "$tmp/list-struct.err"
    fail=1
fi

list_abi_src="$tmp/direct_list_int_abi.vais"
cat > "$list_abi_src" <<'SRC'
fn make(a: Int, b: Int, c: Int) -> List<Int> {
    let xs: List<Int> = []
    xs.push(a)
    xs.push(b)
    xs.push(c)
    return xs
}

fn pass(xs: List<Int>) -> List<Int> {
    return xs
}

fn score(xs: List<Int>) -> Int {
    return xs.sum() + xs.len() + xs[1]
}

fn main() -> Int {
    let xs = make(10, 20, 30)
    let ys: List<Int> = pass(xs)
    return score(ys) - 41
}
SRC

if "$VAISC" emit-ir "$list_abi_src" \
    --engine direct -o "$tmp/list-abi.ll" \
    >"$tmp/list-abi.out" 2>"$tmp/list-abi.err" &&
    grep -q 'define .*@make' "$tmp/list-abi.ll" &&
    grep -q 'define .*@score' "$tmp/list-abi.ll" &&
    grep -q 'call .*@score' "$tmp/list-abi.ll"; then
    "$VAISC" run "$list_abi_src" --engine direct >"$tmp/list-abi-run.out" 2>"$tmp/list-abi-run.err"
    list_abi_run=$?
    if [ "$list_abi_run" = "42" ]; then
        echo "  PASS direct List<Int> parameter and return ABI runs (=42)"
    else
        echo "  FAIL direct List<Int> ABI got=$list_abi_run want=42"
        cat "$tmp/list-abi-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Int> ABI emission"
    cat "$tmp/list-abi.err"
    fail=1
fi

list_out_src="$tmp/direct_list_int_out_param.vais"
cat > "$list_out_src" <<'SRC'
fn fill(out: List<Int>, n: Int) -> Int {
    out.push(n)
    out.push(n + 2)
    return out.len()
}

fn main() -> Int {
    let xs: List<Int> = []
    let count = fill(xs, 20)
    return count * 10 + xs[1]
}
SRC

if "$VAISC" emit-ir "$list_out_src" \
    --engine direct -o "$tmp/list-out.ll" \
    >"$tmp/list-out.out" 2>"$tmp/list-out.err" &&
    grep -q 'define .*@fill' "$tmp/list-out.ll" &&
    grep -q 'call .*@fill' "$tmp/list-out.ll"; then
    "$VAISC" run "$list_out_src" --engine direct >"$tmp/list-out-run.out" 2>"$tmp/list-out-run.err"
    list_out_run=$?
    if [ "$list_out_run" = "42" ]; then
        echo "  PASS direct List<Int> parameter push mutates caller list (=42)"
    else
        echo "  FAIL direct List<Int> out-param got=$list_out_run want=42"
        cat "$tmp/list-out-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Int> out-param emission"
    cat "$tmp/list-out.err"
    fail=1
fi

list_inline_src="$tmp/direct_list_int_inline_values.vais"
cat > "$list_inline_src" <<'SRC'
fn make(a: Int) -> List<Int> {
    return [a, a + 2]
}

fn empty() -> List<Int> {
    return []
}

fn score(xs: List<Int>) -> Int {
    xs.push(10)
    return xs.sum() + xs.len()
}

fn main() -> Int {
    let xs = make(20)
    let ys = empty()
    return score([5, 5]) + score(list()) + xs[1] + ys.len() - 14
}
SRC

if "$VAISC" emit-ir "$list_inline_src" \
    --engine direct -o "$tmp/list-inline.ll" \
    >"$tmp/list-inline.out" 2>"$tmp/list-inline.err" &&
    grep -q 'define .*@make' "$tmp/list-inline.ll" &&
    grep -q 'call .*@score' "$tmp/list-inline.ll"; then
    "$VAISC" run "$list_inline_src" --engine direct >"$tmp/list-inline-run.out" 2>"$tmp/list-inline-run.err"
    list_inline_run=$?
    if [ "$list_inline_run" = "42" ]; then
        echo "  PASS direct inline List<Int> call and return values run (=42)"
    else
        echo "  FAIL direct inline List<Int> got=$list_inline_run want=42"
        cat "$tmp/list-inline-run.err"
        fail=1
    fi
else
    echo "  FAIL direct inline List<Int> emission"
    cat "$tmp/list-inline.err"
    fail=1
fi

list_hoist_src="$tmp/direct_list_int_return_call_args.vais"
cat > "$list_hoist_src" <<'SRC'
fn make(a: Int) -> List<Int> {
    return [a, a + 1]
}

fn pass(xs: List<Int>) -> List<Int> {
    xs.push(1)
    return xs
}

fn score(xs: List<Int>) -> Int {
    xs.push(10)
    return xs.sum() + xs.len()
}

fn main() -> Int {
    let a = score(make(10))
    let b = score(pass(make(5)))
    let xs: List<Int> = [score(make(1))]
    xs.push(score(make(2)))
    a = score(make(0))
    return a + b + xs.len()
}
SRC

if "$VAISC" emit-ir "$list_hoist_src" \
    --engine direct -o "$tmp/list-hoist.ll" \
    >"$tmp/list-hoist.out" 2>"$tmp/list-hoist.err" &&
    grep -q 'call void @make' "$tmp/list-hoist.ll" &&
    grep -q 'call void @pass' "$tmp/list-hoist.ll" &&
    grep -q 'call .*@score' "$tmp/list-hoist.ll"; then
    "$VAISC" run "$list_hoist_src" --engine direct >"$tmp/list-hoist-run.out" 2>"$tmp/list-hoist-run.err"
    list_hoist_run=$?
    if [ "$list_hoist_run" = "42" ]; then
        echo "  PASS direct List<Int> return-call arguments hoist and run (=42)"
    else
        echo "  FAIL direct List<Int> hoist got=$list_hoist_run want=42"
        cat "$tmp/list-hoist-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Int> hoist emission"
    cat "$tmp/list-hoist.err"
    fail=1
fi

list_while_hoist_src="$tmp/direct_list_int_while_hoist.vais"
cat > "$list_while_hoist_src" <<'SRC'
fn make(n: Int) -> List<Int> {
    return [n]
}

fn score(xs: List<Int>) -> Int {
    xs.push(1)
    return xs.sum()
}

fn main() -> Int {
    let mut i = 0
    let mut total = 0
    while score(make(i)) < 4 {
        total = total + score(make(i))
        i = i + 1
    }
    return total * 6 + i * 2
}
SRC

if "$VAISC" emit-ir "$list_while_hoist_src" \
    --engine direct -o "$tmp/list-while-hoist.ll" \
    >"$tmp/list-while-hoist.out" 2>"$tmp/list-while-hoist.err" &&
    grep -q 'br label' "$tmp/list-while-hoist.ll" &&
    grep -q 'call void @make' "$tmp/list-while-hoist.ll" &&
    grep -q 'call .*@score' "$tmp/list-while-hoist.ll"; then
    "$VAISC" run "$list_while_hoist_src" --engine direct >"$tmp/list-while-hoist-run.out" 2>"$tmp/list-while-hoist-run.err"
    list_while_hoist_run=$?
    if [ "$list_while_hoist_run" = "42" ]; then
        echo "  PASS direct List<Int> while-condition returned arguments hoist per iteration (=42)"
    else
        echo "  FAIL direct List<Int> while hoist got=$list_while_hoist_run want=42"
        cat "$tmp/list-while-hoist-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Int> while hoist emission"
    cat "$tmp/list-while-hoist.err"
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
