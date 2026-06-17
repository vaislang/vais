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

expect_direct_trap() {
    local label="$1" src="$2"
    "$VAISC" run "$src" --engine direct >"$tmp/$label.out" 2>"$tmp/$label.err"
    local rc=$?
    if [ "$rc" -ne 0 ]; then
        echo "  PASS direct List bounds trap: $label"
    else
        echo "  FAIL direct List bounds trap did not fire: $label"
        fail=1
    fi
}

expect_direct_reject() {
    local label="$1" src="$2" needle="$3" help="$4"
    "$VAISC" emit-ir "$src" --engine direct -o "$tmp/$label.ll" >"$tmp/$label.out" 2>"$tmp/$label.err"
    local rc=$?
    if [ "$rc" -ne 0 ] &&
        grep -q "$needle" "$tmp/$label.err" &&
        grep -q "$help" "$tmp/$label.err"; then
        echo "  PASS direct rejects $label"
    else
        echo "  FAIL direct reject $label rc=$rc"
        cat "$tmp/$label.err"
        fail=1
    fi
}

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

direct_import_src="$tmp/direct_import.vais"
cat > "$direct_import_src" <<'SRC'
import math.add

fn main() -> Int {
    return 42
}
SRC
expect_direct_reject "import" "$direct_import_src" "direct native emitter does not support imports" "use the full engine for local imports"

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
    let empty: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let ys = [1, 2, xs.len()]
    let tail = xs.last()
    let popped = xs.pop()
    return xs.sum() - xs.len() - xs[1] + ys[2] + ys.sum() + 25 + empty.is_empty() - xs.is_empty() - 1 + tail - 30 + popped - 30
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
        echo "  PASS direct local List<Int> push, len, is_empty, last, pop, index, literal, and sum run (=42)"
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

str_src="$tmp/direct_str_slice.vais"
cat > "$str_src" <<'SRC'
fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
}

fn same(a: Str, b: Str) -> Bool {
    return a == b
}

fn byte_at(s: Str, i: Int) -> Int {
    return s[i]
}

fn count_digits(s: Str) -> Int {
    let mut i = 0
    let mut n = 0
    while i < s.len() {
        if is_digit(s[i]) {
            n = n + 1
        }
        i = i + 1
    }
    return n
}

fn main() -> Int {
    let a: Str = "a1b2"
    let b = `a1b2`
    if a != b {
        return 0
    }
    return count_digits(a) * 20 + byte_at(b, 0) - 96 + same("ok", `ok`)
}
SRC

if "$VAISC" emit-ir "$str_src" \
    --engine direct -o "$tmp/str.ll" \
    >"$tmp/str.out" 2>"$tmp/str.err" &&
    grep -q '__vais_str_len' "$tmp/str.ll" &&
    grep -q '__vais_str_byte' "$tmp/str.ll" &&
    grep -q '__vais_str_eq' "$tmp/str.ll"; then
    "$VAISC" run "$str_src" --engine direct >"$tmp/str-run.out" 2>"$tmp/str-run.err"
    str_run=$?
    if [ "$str_run" = "42" ]; then
        echo "  PASS direct Str len, index, equality, Bool helper, and byte classification run (=42)"
    else
        echo "  FAIL direct Str slice got=$str_run want=42"
        cat "$tmp/str-run.err"
        fail=1
    fi
else
    echo "  FAIL direct Str slice emission"
    cat "$tmp/str.err"
    fail=1
fi

parse_src="$tmp/direct_parse_helpers.vais"
cat > "$parse_src" <<'SRC'
fn main() -> Int {
    let s: Str = "16x"
    let a = parse_uint("19")
    let b = parse_uint(s)
    let c = parse_int("-5")
    let d = parse_int(`12z`)
    return a + b + c + d
}
SRC

if "$VAISC" emit-ir "$parse_src" \
    --engine direct -o "$tmp/parse.ll" \
    >"$tmp/parse.out" 2>"$tmp/parse.err" &&
    grep -q '__vais_parse_uint' "$tmp/parse.ll" &&
    grep -q '__vais_parse_int' "$tmp/parse.ll"; then
    "$VAISC" run "$parse_src" --engine direct >"$tmp/parse-run.out" 2>"$tmp/parse-run.err"
    parse_run=$?
    if [ "$parse_run" = "42" ]; then
        echo "  PASS direct parse_uint/parse_int Str helpers run (=42)"
    else
        echo "  FAIL direct parse helpers got=$parse_run want=42"
        cat "$tmp/parse-run.err"
        fail=1
    fi
else
    echo "  FAIL direct parse helper emission"
    cat "$tmp/parse.err"
    fail=1
fi

map_src="$tmp/direct_map_int_int.vais"
cat > "$map_src" <<'SRC'
fn main() -> Int {
    let scores: Map<Int, Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    return scores.get(4, 0) + scores.get(5, 0 - 1) + scores.contains(4) + scores.contains(5) + scores.len()
}
SRC

if "$VAISC" emit-ir "$map_src" \
    --engine direct -o "$tmp/map.ll" \
    >"$tmp/map.out" 2>"$tmp/map.err" &&
    grep -q '__vais_map_int_int_insert' "$tmp/map.ll" &&
    grep -q '__vais_map_int_int_get' "$tmp/map.ll" &&
    grep -q '__vais_map_int_int_contains' "$tmp/map.ll" &&
    grep -q '__vais_map_int_int_len' "$tmp/map.ll"; then
    "$VAISC" run "$map_src" --engine direct >"$tmp/map-run.out" 2>"$tmp/map-run.err"
    map_run=$?
    if [ "$map_run" = "42" ]; then
        echo "  PASS direct local Map<Int,Int> insert/get/contains/len run (=42)"
    else
        echo "  FAIL direct Map<Int,Int> got=$map_run want=42"
        cat "$tmp/map-run.err"
        fail=1
    fi
else
    echo "  FAIL direct Map<Int,Int> emission"
    cat "$tmp/map.err"
    fail=1
fi

list_struct_src="$tmp/direct_list_struct_local.vais"
cat > "$list_struct_src" <<'SRC'
struct Box {
    value: Int,
}

fn main() -> Int {
    let xs: List<Box> = []
    let empty_boxes: List<Box> = []
    xs.push(Box { value: 10 })
    xs.push(Box { value: 30 })
    let ys: List<Box> = [Box { value: 1 }]
    let zs: List<Box> = list()
    zs.push(Box { value: 2 })
    let tail = xs.last()
    let popped = xs.pop()
    return xs[0].value + xs.len() + ys[0].value + zs[0].value + 28 + empty_boxes.is_empty() - xs.is_empty() - 1 + tail.value - 30 + popped.value - 30
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
        echo "  PASS direct local List<Struct> push, len, is_empty, last, pop, index, and field read run (=42)"
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

list_bad_index_src="$tmp/direct_list_bad_index.vais"
cat > "$list_bad_index_src" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(1)
    return xs[1]
}
SRC
expect_direct_trap "bad-index" "$list_bad_index_src"

list_bad_last_src="$tmp/direct_list_bad_last.vais"
cat > "$list_bad_last_src" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    return xs.last()
}
SRC
expect_direct_trap "bad-last" "$list_bad_last_src"

list_bad_pop_src="$tmp/direct_list_bad_pop.vais"
cat > "$list_bad_pop_src" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    return xs.pop()
}
SRC
expect_direct_trap "bad-pop" "$list_bad_pop_src"

list_struct_abi_src="$tmp/direct_list_struct_abi.vais"
cat > "$list_struct_abi_src" <<'SRC'
struct Box {
    value: Int,
}

fn make(v: Int) -> List<Box> {
    return [Box { value: v }, Box { value: v + 1 }]
}

fn empty_box() -> List<Box> {
    return []
}

fn append(out: List<Box>, v: Int) -> Int {
    out.push(Box { value: v })
    return out.len()
}

fn score(xs: List<Box>) -> Int {
    return xs[0].value + xs.len()
}

fn main() -> Int {
    let xs: List<Box> = []
    let n = append(xs, 20)
    let ys: List<Box> = make(21)
    let zs: List<Box> = empty_box()
    zs.push(Box { value: 1 })
    let inline_score = score([Box { value: 17 }])
    let returned_score = score(make(2))
    let mut i = 0
    let mut total = 0
    while score(make(i)) < 5 {
        total = total + score(make(i))
        i = i + 1
    }
    return xs[0].value + ys[1].value + n + zs[0].value + inline_score + returned_score + total - 33
}
SRC

if "$VAISC" emit-ir "$list_struct_abi_src" \
    --engine direct -o "$tmp/list-struct-abi.ll" \
    >"$tmp/list-struct-abi.out" 2>"$tmp/list-struct-abi.err" &&
    grep -q '%struct.DirectList_Box = type' "$tmp/list-struct-abi.ll" &&
    grep -q 'sret(%struct.DirectList_Box)' "$tmp/list-struct-abi.ll" &&
    grep -q 'define i64 @append(ptr' "$tmp/list-struct-abi.ll" &&
    grep -q 'define i64 @score(ptr' "$tmp/list-struct-abi.ll"; then
    "$VAISC" run "$list_struct_abi_src" --engine direct >"$tmp/list-struct-abi-run.out" 2>"$tmp/list-struct-abi-run.err"
    list_struct_abi_run=$?
    if [ "$list_struct_abi_run" = "42" ]; then
        echo "  PASS direct List<Struct> parameter, return, inline, and hoisted arguments run (=42)"
    else
        echo "  FAIL direct List<Struct> ABI got=$list_struct_abi_run want=42"
        cat "$tmp/list-struct-abi-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Struct> ABI emission"
    cat "$tmp/list-struct-abi.err"
    fail=1
fi

list_struct_field_write_src="$tmp/direct_list_struct_field_write.vais"
cat > "$list_struct_field_write_src" <<'SRC'
struct Box {
    value: Int,
}

fn bump(xs: List<Box>, i: Int, v: Int) -> Int {
    xs[i].value = v
    return xs[i].value
}

fn main() -> Int {
    let xs: List<Box> = [Box { value: 1 }, Box { value: 2 }]
    xs[0].value = 10
    let a = bump(xs, 1, 30)
    let b = Box { value: 3 }
    b.value = xs[0].value + a + 2
    return b.value
}
SRC

if "$VAISC" emit-ir "$list_struct_field_write_src" \
    --engine direct -o "$tmp/list-struct-field-write.ll" \
    >"$tmp/list-struct-field-write.out" 2>"$tmp/list-struct-field-write.err" &&
    grep -q '%struct.DirectList_Box = type' "$tmp/list-struct-field-write.ll" &&
    grep -q 'store i64 10' "$tmp/list-struct-field-write.ll" &&
    grep -q 'define i64 @bump(ptr' "$tmp/list-struct-field-write.ll"; then
    "$VAISC" run "$list_struct_field_write_src" --engine direct >"$tmp/list-struct-field-write-run.out" 2>"$tmp/list-struct-field-write-run.err"
    list_struct_field_write_run=$?
    if [ "$list_struct_field_write_run" = "42" ]; then
        echo "  PASS direct List<Struct> indexed field assignment runs (=42)"
    else
        echo "  FAIL direct List<Struct> field assignment got=$list_struct_field_write_run want=42"
        cat "$tmp/list-struct-field-write-run.err"
        fail=1
    fi
else
    echo "  FAIL direct List<Struct> field assignment emission"
    cat "$tmp/list-struct-field-write.err"
    fail=1
fi

list_element_assignment_src="$tmp/direct_list_element_assignment.vais"
cat > "$list_element_assignment_src" <<'SRC'
struct Box {
    value: Int,
}

fn set_box(xs: List<Box>, i: Int, v: Int) -> Int {
    xs[i] = Box { value: v }
    return xs[i].value
}

fn set_int(xs: List<Int>, i: Int, v: Int) -> Int {
    xs[i] = v
    return xs[i]
}

fn main() -> Int {
    let boxes: List<Box> = [Box { value: 1 }, Box { value: 2 }]
    boxes[0] = Box { value: 10 }
    boxes[1] = boxes[0]
    let a = set_box(boxes, 1, 20)
    let ints: List<Int> = [1, 2]
    ints[0] = 5
    ints[1] = ints[0]
    let b = set_int(ints, 1, 7)
    return boxes[0].value + boxes[1].value + a + ints[0] + ints[1] + b - 27
}
SRC

if "$VAISC" emit-ir "$list_element_assignment_src" \
    --engine direct -o "$tmp/list-element-assignment.ll" \
    >"$tmp/list-element-assignment.out" 2>"$tmp/list-element-assignment.err" &&
    grep -q '%struct.DirectList_Box = type' "$tmp/list-element-assignment.ll" &&
    grep -q 'define i64 @set_box(ptr' "$tmp/list-element-assignment.ll" &&
    grep -q 'llvm.memcpy' "$tmp/list-element-assignment.ll" &&
    grep -q 'store i64 5' "$tmp/list-element-assignment.ll"; then
    "$VAISC" run "$list_element_assignment_src" --engine direct >"$tmp/list-element-assignment-run.out" 2>"$tmp/list-element-assignment-run.err"
    list_element_assignment_run=$?
    if [ "$list_element_assignment_run" = "42" ]; then
        echo "  PASS direct List<Int> and List<Struct> element assignments run (=42)"
    else
        echo "  FAIL direct list element assignment got=$list_element_assignment_run want=42"
        cat "$tmp/list-element-assignment-run.err"
        fail=1
    fi
else
    echo "  FAIL direct list element assignment emission"
    cat "$tmp/list-element-assignment.err"
    fail=1
fi

list_assignment_src="$tmp/direct_list_assignment.vais"
cat > "$list_assignment_src" <<'SRC'
struct Box {
    value: Int,
}

fn make(v: Int) -> List<Box> {
    return [Box { value: v }, Box { value: v + 1 }]
}

fn replace(out: List<Box>, v: Int) -> Int {
    out = [Box { value: v }, Box { value: v + 1 }]
    out = make(v + 2)
    return out.len()
}

fn main() -> Int {
    let xs: List<Box> = []
    xs = []
    xs.push(Box { value: 5 })
    xs = list()
    xs.push(Box { value: 7 })
    xs = make(20)
    let n = replace(xs, 30)
    let ints: List<Int> = []
    ints = [1, 2]
    ints = list()
    ints.push(4)
    ints = [5, 6]
    return xs[0].value + xs[1].value + n + ints.sum() - 36
}
SRC

if "$VAISC" emit-ir "$list_assignment_src" \
    --engine direct -o "$tmp/list-assignment.ll" \
    >"$tmp/list-assignment.out" 2>"$tmp/list-assignment.err" &&
    grep -q '%struct.DirectList_Box = type' "$tmp/list-assignment.ll" &&
    grep -q 'sret(%struct.DirectList_Box)' "$tmp/list-assignment.ll"; then
    "$VAISC" run "$list_assignment_src" --engine direct >"$tmp/list-assignment-run.out" 2>"$tmp/list-assignment-run.err"
    list_assignment_run=$?
    if [ "$list_assignment_run" = "42" ]; then
        echo "  PASS direct List<Int> and List<Struct> assignments run (=42)"
    else
        echo "  FAIL direct list assignment got=$list_assignment_run want=42"
        cat "$tmp/list-assignment-run.err"
        fail=1
    fi
else
    echo "  FAIL direct list assignment emission"
    cat "$tmp/list-assignment.err"
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
    let tail = xs.pop()
    return xs.sum() + xs.len() + xs[1] + tail + 1
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

list_if_hoist_src="$tmp/direct_list_if_hoist.vais"
cat > "$list_if_hoist_src" <<'SRC'
struct Box {
    value: Int,
}

fn make_int(n: Int) -> List<Int> {
    return [n, n + 1]
}

fn score_int(xs: List<Int>) -> Int {
    return xs.sum()
}

fn make_box(n: Int) -> List<Box> {
    return [Box { value: n }, Box { value: n + 1 }]
}

fn score_box(xs: List<Box>) -> Int {
    return xs[0].value + xs[1].value
}

fn main() -> Int {
    let mut total = 0
    if score_int(make_int(20)) == 41 {
        total = total + 20
    }
    if score_box(make_box(10)) == 21 {
        total = total + 22
    }
    return total
}
SRC

if "$VAISC" emit-ir "$list_if_hoist_src" \
    --engine direct -o "$tmp/list-if-hoist.ll" \
    >"$tmp/list-if-hoist.out" 2>"$tmp/list-if-hoist.err" &&
    grep -q 'call void @make_int' "$tmp/list-if-hoist.ll" &&
    grep -q 'call .*@score_int' "$tmp/list-if-hoist.ll" &&
    grep -q 'call void @make_box' "$tmp/list-if-hoist.ll" &&
    grep -q 'call .*@score_box' "$tmp/list-if-hoist.ll"; then
    "$VAISC" run "$list_if_hoist_src" --engine direct >"$tmp/list-if-hoist-run.out" 2>"$tmp/list-if-hoist-run.err"
    list_if_hoist_run=$?
    if [ "$list_if_hoist_run" = "42" ]; then
        echo "  PASS direct List<Int> and List<Struct> if-condition returned arguments hoist (=42)"
    else
        echo "  FAIL direct list if-condition hoist got=$list_if_hoist_run want=42"
        cat "$tmp/list-if-hoist-run.err"
        fail=1
    fi
else
    echo "  FAIL direct list if-condition hoist emission"
    cat "$tmp/list-if-hoist.err"
    fail=1
fi

list_else_if_hoist_src="$tmp/direct_list_else_if_hoist.vais"
cat > "$list_else_if_hoist_src" <<'SRC'
struct Box {
    value: Int,
}

fn make_int(n: Int) -> List<Int> {
    return [n, n + 1]
}

fn score_int(xs: List<Int>) -> Int {
    return xs.sum()
}

fn make_box(n: Int) -> List<Box> {
    return [Box { value: n }, Box { value: n + 1 }]
}

fn score_box(xs: List<Box>) -> Int {
    return xs[0].value + xs[1].value
}

fn main() -> Int {
    let mut total = 0
    if score_int([1, 1]) == 99 {
        total = total + 1
    } else if score_int(make_int(20)) == 41 {
        total = total + 20
    }
    if score_box([Box { value: 1 }, Box { value: 1 }]) == 99 {
        total = total + 1
    } else if score_box(make_box(10)) == 21 {
        total = total + 22
    }
    return total
}
SRC

if "$VAISC" emit-ir "$list_else_if_hoist_src" \
    --engine direct -o "$tmp/list-else-if-hoist.ll" \
    >"$tmp/list-else-if-hoist.out" 2>"$tmp/list-else-if-hoist.err" &&
    grep -q 'call void @make_int' "$tmp/list-else-if-hoist.ll" &&
    grep -q 'call .*@score_int' "$tmp/list-else-if-hoist.ll" &&
    grep -q 'call void @make_box' "$tmp/list-else-if-hoist.ll" &&
    grep -q 'call .*@score_box' "$tmp/list-else-if-hoist.ll"; then
    "$VAISC" run "$list_else_if_hoist_src" --engine direct >"$tmp/list-else-if-hoist-run.out" 2>"$tmp/list-else-if-hoist-run.err"
    list_else_if_hoist_run=$?
    if [ "$list_else_if_hoist_run" = "42" ]; then
        echo "  PASS direct List<Int> and List<Struct> else-if returned arguments lower (=42)"
    else
        echo "  FAIL direct list else-if returned arguments got=$list_else_if_hoist_run want=42"
        cat "$tmp/list-else-if-hoist-run.err"
        fail=1
    fi
else
    echo "  FAIL direct list else-if returned-argument emission"
    cat "$tmp/list-else-if-hoist.err"
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
