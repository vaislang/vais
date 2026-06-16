#!/usr/bin/env bash
# NV-C1 front-contract gate for the Vais `vaisc` command.
#
# The native front is intentionally narrow: Int functions, let/let mut,
# integer arithmetic/comparisons, return, if/else, while, plain function calls,
# print/putchar, simple structs, payload-free enum/match, small Int-coded
# payload enum/match, single-Int closure return, and the first
# List push/len/is_empty/last/pop/index/sum slice.
# Broader language features stay on the full compiler path until their native
# slices land.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
fail=0

tmp="$(mktemp -d)"

accept="$tmp/front_accept.vais"
cat > "$accept" <<'SRC'
fn loop_sum(n: Int) -> Int {
    let mut s = 0
    let mut i = 0
    while i < n {
        s = s + 6
        i = i + 1
    }
    if s == 42 {
        return s
    } else {
        return 0
    }
}

fn main() -> Int {
    return loop_sum(7)
}
SRC

"$VAISC" run "$accept" >"$tmp/accept.out" 2>"$tmp/accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts day-1 Int/let/while/if/function-call subset"
else
    echo "  FAIL accepts day-1 subset got=$got want=42"
    cat "$tmp/accept.err"
    fail=1
fi

io_accept="$tmp/front_io_accept.vais"
cat > "$io_accept" <<'SRC'
fn main() -> Int {
    let x = 42
    print("the answer is {x}")
    putchar(33)
    return 0
}
SRC

"$VAISC" run "$io_accept" >"$tmp/io_accept.out" 2>"$tmp/io_accept.err"
got=$?
io_out="$(cat "$tmp/io_accept.out")"
io_want="$(printf 'the answer is 42\n!')"
if [ "$got" = "0" ] && [ "$io_out" = "$io_want" ]; then
    echo "  PASS accepts print interpolation and putchar IO slice"
else
    echo "  FAIL accepts IO slice got=$got stdout=[$io_out]"
    cat "$tmp/io_accept.err"
    fail=1
fi

struct_accept="$tmp/front_struct_accept.vais"
cat > "$struct_accept" <<'SRC'
struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.value
}
SRC

"$VAISC" run "$struct_accept" >"$tmp/struct_accept.out" 2>"$tmp/struct_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts struct literal and field access slice"
else
    echo "  FAIL accepts struct slice got=$got want=42"
    cat "$tmp/struct_accept.err"
    fail=1
fi

enum_accept="$tmp/front_enum_accept.vais"
cat > "$enum_accept" <<'SRC'
enum Color { Red, Green, Blue }

fn color_number(c: Color) -> Int {
    match c {
        Color.Red => return 1,
        Color.Green => return 2,
        Color.Blue => return 3,
    }
}

fn main() -> Int {
    let c = Color.Green
    return color_number(c)
}
SRC

"$VAISC" run "$enum_accept" >"$tmp/enum_accept.out" 2>"$tmp/enum_accept.err"
got=$?
if [ "$got" = "2" ]; then
    echo "  PASS accepts payload-free enum/match slice"
else
    echo "  FAIL accepts enum/match slice got=$got want=2"
    cat "$tmp/enum_accept.err"
    fail=1
fi

payload_enum_accept="$tmp/front_payload_enum_accept.vais"
cat > "$payload_enum_accept" <<'SRC'
enum Node { Lit(Int), Add(Node, Node), Mul(Node, Node) }

fn eval(n: Node) -> Int {
    match n {
        Lit(v) => return v,
        Add(a, b) => return eval(a) + eval(b),
        Mul(a, b) => return eval(a) * eval(b),
    }
}

fn main() -> Int {
    return eval(Add(Lit(12), Lit(2)))
}
SRC

"$VAISC" run "$payload_enum_accept" >"$tmp/payload_enum_accept.out" 2>"$tmp/payload_enum_accept.err"
got=$?
if [ "$got" = "14" ]; then
    echo "  PASS accepts small payload enum/match slice"
else
    echo "  FAIL accepts payload enum/match slice got=$got want=14"
    cat "$tmp/payload_enum_accept.err"
    fail=1
fi

closure_accept="$tmp/front_closure_accept.vais"
cat > "$closure_accept" <<'SRC'
fn adder(n: Int) -> fn(Int) -> Int {
    return |x| x + n
}

fn main() -> Int {
    let add3 = adder(3)
    return add3(4)
}
SRC

"$VAISC" run "$closure_accept" >"$tmp/closure_accept.out" 2>"$tmp/closure_accept.err"
got=$?
if [ "$got" = "7" ]; then
    echo "  PASS accepts single-Int closure return slice"
else
    echo "  FAIL accepts closure return slice got=$got want=7"
    cat "$tmp/closure_accept.err"
    fail=1
fi

list_accept="$tmp/front_list_accept.vais"
cat > "$list_accept" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    let empty: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let popped = xs.pop()
    return xs.sum() - xs.len() - xs[1] - xs[0] + 25 + empty.is_empty() - xs.is_empty() - 1 + xs.last() - 20 + popped - 30
}
SRC

"$VAISC" run "$list_accept" >"$tmp/list_accept.out" 2>"$tmp/list_accept.err"
got=$?
if [ "$got" = "23" ]; then
    echo "  PASS accepts List push/len/is_empty/last/pop/index/sum slice"
else
    echo "  FAIL accepts List slice got=$got want=23"
    cat "$tmp/list_accept.err"
    fail=1
fi

expect_reject() {
    local name="$1"
    local needle="$2"
    local help_needle="$3"
    local src="$tmp/$name.vais"
    cat > "$src"
    "$VAISC" emit-ir "$src" -o "$tmp/$name.ll" >"$tmp/$name.out" 2>"$tmp/$name.err"
    local rc=$?
    if [ "$rc" = "0" ]; then
        echo "  FAIL rejects $name: command unexpectedly succeeded"
        fail=1
        return
    fi
    if grep -q "$needle" "$tmp/$name.err" && grep -q "help:" "$tmp/$name.err" && grep -q "$help_needle" "$tmp/$name.err"; then
        echo "  PASS rejects $name with P4 help"
    else
        echo "  FAIL rejects $name: missing expected diagnostic"
        cat "$tmp/$name.err"
        fail=1
    fi
}

expect_reject "bad_main_signature" "requires .*fn main() -> Int" "write the entrypoint" <<'SRC'
fn main() {
    return 42
}
SRC

expect_reject "helper_missing_return_type" "helper functions must return" "fn name" <<'SRC'
fn add(a: Int, b: Int) {
    return a + b
}

fn main() -> Int {
    return add(20, 22)
}
SRC

expect_reject "for_loop" "for.*not in the Vais native day-1" "while" <<'SRC'
fn main() -> Int {
    let mut s = 0
    for i in 0..7 {
        s = s + i
    }
    return s
}
SRC

expect_reject "payload_enum_struct" "enum declarations beyond payload-free tags" "broader payload enums" <<'SRC'
struct Pt { x: Int }
enum Node { Lit(Pt) }

fn main() -> Int {
    return 0
}
SRC

expect_reject "match_expr_body" "match.*simple enum return arms" "payload match code" <<'SRC'
enum Color { Red, Green }

fn main() -> Int {
    let c = Color.Green
    match c {
        Color.Red => 1,
        Color.Green => 2,
    }
}
SRC

expect_reject "closure_literal" "closures beyond the single-Int closure-return" "broader closure cases" <<'SRC'
fn main() -> Int {
    let add1 = |x| x + 1
    return add1(2)
}
SRC

expect_reject "rust_and" "logical AND uses the word" "replace .*&&.*and" <<'SRC'
fn main() -> Int {
    if 1 == 1 && 2 == 2 {
        return 42
    }
    return 0
}
SRC

expect_reject "unsupported_method" "method calls beyond push/len/is_empty/last/pop/sum" "plain function call" <<'SRC'
fn main() -> Int {
    let xs = [20, 22]
    return xs.clear()
}
SRC

expect_reject "string_type" "only Int scalar typing" "string, char, and bool" <<'SRC'
fn len(s: Str) -> Int {
    return 42
}

fn main() -> Int {
    return len(`x`)
}
SRC

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C1 front contract OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
