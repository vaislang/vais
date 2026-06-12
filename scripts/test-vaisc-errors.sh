#!/usr/bin/env bash
# NV-C3 P4 diagnostic gate for the New Vais `vaisc` native path.
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

expect_diag "rust_and" "bootstrap" "logical AND uses the word" "and" <<'SRC'
fn main() -> Int {
    if 1 == 1 && 2 == 2 {
        return 42
    }
    return 0
}
SRC

expect_diag "rust_or" "bootstrap" "logical OR uses the word" "or" <<'SRC'
fn main() -> Int {
    if 1 == 0 || 2 == 2 {
        return 42
    }
    return 0
}
SRC

expect_diag "as_cast" "bootstrap" "Type(x).*not.*x as Type" "Int(1)" <<'SRC'
fn main() -> Int {
    return 1 as Int
}
SRC

expect_diag "colon_path" "bootstrap" "not .::" "Foo.Bar" <<'SRC'
fn main() -> Int {
    return Foo::Bar
}
SRC

expect_diag "rust_scalar_type" "bootstrap" "scalar types are capitalized" "fn id(x: Int)" <<'SRC'
fn id(x: i32) -> Int {
    return x
}

fn main() -> Int {
    return id(42)
}
SRC

expect_diag "turbofish_new" "bootstrap" "no turbofish constructor" "let xs = \\[]" <<'SRC'
fn main() -> Int {
    let xs = Vec<Int>::new()
    return 42
}
SRC

expect_diag "direct_identifier" "direct" "literal Int expressions only" "return 40 + 2" <<'SRC'
fn main() -> Int {
    return answer
}
SRC

expect_diag "direct_missing_return" "direct" "single .*return.* statement" "fn main() -> Int" <<'SRC'
fn main() -> Int {
    let x = 42
}
SRC

if [ "$fail" -eq 0 ]; then
    echo "RESULT: New Vais vaisc NV-C3 diagnostics OK"
else
    echo "RESULT: FAILURES"
fi
exit "$fail"
