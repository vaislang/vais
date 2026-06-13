#!/usr/bin/env bash
# NV-C1 front-contract gate for the New Vais `vaisc` command.
#
# The day-1 native front is intentionally narrow: Int functions, let/let mut,
# integer arithmetic/comparisons, return, if/else, while, and plain function
# calls. The first IO slice also accepts print/putchar. Broader language
# features stay on the Legacy bootstrap path until their native slices land.
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

expect_reject "for_loop" "for.*not in the New Vais native day-1" "while" <<'SRC'
fn main() -> Int {
    let mut s = 0
    for i in 0..7 {
        s = s + i
    }
    return s
}
SRC

expect_reject "struct_decl" "struct declarations" "Legacy bootstrap" <<'SRC'
struct Pair { a, b }

fn main() -> Int {
    let p = Pair { a: 20, b: 22 }
    return p.a + p.b
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

expect_reject "list_literal" "list/array literals" "scalar Int" <<'SRC'
fn main() -> Int {
    let xs = [20, 22]
    return xs[0] + xs[1]
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
    echo "RESULT: New Vais vaisc NV-C1 front contract OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
