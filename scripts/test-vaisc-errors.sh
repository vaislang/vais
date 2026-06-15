#!/usr/bin/env bash
# NV-C3 P4 diagnostic gate for the Vais `vaisc` native path.
#
# Checks that known Rust/Vais habit errors and direct-emitter parse errors carry
# source coordinates, the source line, a caret, `help:`, and a concrete `fix:`.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
VAISC="$HERE/scripts/vaisc"
fail=0

tmp="$(mktemp -d)"

expect_diag() {
    local name="$1"
    local mode="$2"
    local message_needle="$3"
    local fix_needle="$4"
    local src="$tmp/$name.vais"
    cat > "$src"

    if [ "$mode" = "direct" ]; then
        "$VAISC" emit-ir "$src" --engine direct -o "$tmp/$name.ll" >"$tmp/$name.out" 2>"$tmp/$name.err"
    else
        "$VAISC" emit-ir "$src" -o "$tmp/$name.ll" >"$tmp/$name.out" 2>"$tmp/$name.err"
    fi
    local rc=$?
    if [ "$rc" = "0" ]; then
        echo "  FAIL $name: command unexpectedly succeeded"
        fail=1
        return
    fi

    if grep -q "$message_needle" "$tmp/$name.err" &&
        grep -q "$src:" "$tmp/$name.err" &&
        grep -q "\\^" "$tmp/$name.err" &&
        grep -q "help:" "$tmp/$name.err" &&
        grep -q "fix:" "$tmp/$name.err" &&
        grep -q "$fix_needle" "$tmp/$name.err"; then
        echo "  PASS $name has P4 coordinate/help/fix diagnostic"
    else
        echo "  FAIL $name: missing expected P4 diagnostic"
        cat "$tmp/$name.err"
        fail=1
    fi
}

expect_diag "rust_and" "full" "logical AND uses the word" "and" <<'SRC'
fn main() -> Int {
    if 1 == 1 && 2 == 2 {
        return 42
    }
    return 0
}
SRC

expect_diag "rust_or" "full" "logical OR uses the word" "or" <<'SRC'
fn main() -> Int {
    if 1 == 0 || 2 == 2 {
        return 42
    }
    return 0
}
SRC

expect_diag "as_cast" "full" "Type(x).*not.*x as Type" "Int(1)" <<'SRC'
fn main() -> Int {
    return 1 as Int
}
SRC

expect_diag "colon_path" "full" "not .::" "Foo.Bar" <<'SRC'
fn main() -> Int {
    return Foo::Bar
}
SRC

expect_diag "rust_scalar_type" "full" "scalar types are capitalized" "fn id(x: Int)" <<'SRC'
fn id(x: i32) -> Int {
    return x
}

fn main() -> Int {
    return id(42)
}
SRC

expect_diag "turbofish_new" "full" "no turbofish constructor" "let xs = \\[]" <<'SRC'
fn main() -> Int {
    let xs = Vec<Int>::new()
    return 42
}
SRC

expect_diag "direct_identifier" "direct" "unknown Int identifier" "return 40 + 2" <<'SRC'
fn main() -> Int {
    return answer
}
SRC

expect_diag "direct_struct_field" "direct" "unknown struct field access" "return b.value" <<'SRC'
struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.missing
}
SRC

expect_diag "direct_list_struct_field_target" "direct" "assignment target is not a known List<Struct> field" "xs\\[0\\].value = 42" <<'SRC'
struct Box {
    value: Int,
}

fn main() -> Int {
    let xs: List<Box> = [Box { value: 1 }]
    xs[0].missing = 42
    return 0
}
SRC

expect_diag "direct_list_element_target" "direct" "assignment target is not a known List element" "xs\\[0\\] = value" <<'SRC'
fn main() -> Int {
    let x = 1
    x[0] = 42
    return 0
}
SRC

expect_diag "direct_list_constructor_expr" "direct" "return expression type does not match" "return 40 + 2" <<'SRC'
fn main() -> Int {
    return list()
}
SRC

expect_diag "direct_list_method" "direct" "supports List push, len, index, and List<Int> sum" "xs.push(value)" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    return xs.clear()
}
SRC

expect_diag "direct_list_value_expr" "direct" "return expression type does not match" "return 40 + 2" <<'SRC'
fn main() -> Int {
    let xs: List<Int> = []
    return xs
}
SRC

expect_diag "direct_list_arg_type" "direct" "function call argument type does not match" "pass a matching argument" <<'SRC'
fn id(n: Int) -> Int {
    return n
}

fn main() -> Int {
    let xs: List<Int> = []
    return id(xs)
}
SRC

expect_diag "direct_missing_return" "direct" "requires at least one .*return.* statement" "fn main() -> Int" <<'SRC'
fn main() -> Int {
    let x = 42
}
SRC

if [ "$fail" -eq 0 ]; then
    echo "RESULT: Vais vaisc NV-C3 diagnostics OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
