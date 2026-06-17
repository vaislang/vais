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

str_accept="$tmp/front_str_accept.vais"
cat > "$str_accept" <<'SRC'
fn is_digit(c: Int) -> Bool {
    return c >= 48 and c <= 57
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
    return count_digits("a1b2") * 21
}
SRC

"$VAISC" run "$str_accept" >"$tmp/str_accept.out" 2>"$tmp/str_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts Str parameter, Bool helper, len, index, and byte classification slice"
else
    echo "  FAIL accepts Str slice got=$got want=42"
    cat "$tmp/str_accept.err"
    fail=1
fi

parse_accept="$tmp/front_parse_accept.vais"
cat > "$parse_accept" <<'SRC'
fn main() -> Int {
    let s: Str = "16x"
    return parse_uint("19") + parse_uint(s) + parse_int("-5") + parse_int(`12z`)
}
SRC

"$VAISC" run "$parse_accept" >"$tmp/parse_accept.out" 2>"$tmp/parse_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts named parse_uint/parse_int Str helpers"
else
    echo "  FAIL accepts parse helpers got=$got want=42"
    cat "$tmp/parse_accept.err"
    fail=1
fi

module_root="$tmp/module_basic"
mkdir -p "$module_root/math"
cat > "$module_root/math/add.vais" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}
SRC
cat > "$module_root/main.vais" <<'SRC'
import math.add

fn main() -> Int {
    return add(20, 22)
}
SRC

"$VAISC" run "$module_root/main.vais" >"$tmp/module_accept.out" 2>"$tmp/module_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts local import multi-file full slice"
else
    echo "  FAIL accepts local import multi-file got=$got want=42"
    cat "$tmp/module_accept.err"
    fail=1
fi

VAISC_FORCE_PYTHON=1 "$VAISC" run "$module_root/main.vais" >"$tmp/module_py_accept.out" 2>"$tmp/module_py_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS Python fallback accepts local import multi-file slice"
else
    echo "  FAIL Python fallback local import got=$got want=42"
    cat "$tmp/module_py_accept.err"
    fail=1
fi

package_root="$tmp/package_basic"
mkdir -p "$package_root/src/math"
cat > "$package_root/vais.toml" <<'SRC'
name = "package_basic"
version = "0.1.0"
source = "src"
SRC
cat > "$package_root/src/math/add.vais" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}
SRC
cat > "$package_root/src/main.vais" <<'SRC'
import math.add

fn main() -> Int {
    return add(20, 22)
}
SRC

"$VAISC" run "$package_root/src/main.vais" >"$tmp/package_accept.out" 2>"$tmp/package_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts package manifest source root full slice"
else
    echo "  FAIL accepts package manifest source root got=$got want=42"
    cat "$tmp/package_accept.err"
    fail=1
fi

VAISC_FORCE_PYTHON=1 "$VAISC" run "$package_root/src/main.vais" >"$tmp/package_py_accept.out" 2>"$tmp/package_py_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS Python fallback accepts package manifest source root"
else
    echo "  FAIL Python fallback package manifest got=$got want=42"
    cat "$tmp/package_py_accept.err"
    fail=1
fi

dependency_root="$tmp/dependency_basic"
mkdir -p "$dependency_root/app/src" "$dependency_root/mathlib/src/arith"
cat > "$dependency_root/app/vais.toml" <<'SRC'
name = "dependency_app"
version = "0.1.0"
source = "src"

[dependencies]
mathlib = "../mathlib"
SRC
cat > "$dependency_root/mathlib/vais.toml" <<'SRC'
name = "mathlib"
version = "0.1.0"
source = "src"
SRC
cat > "$dependency_root/mathlib/src/arith/add.vais" <<'SRC'
fn add(a: Int, b: Int) -> Int {
    return a + b
}
SRC
cat > "$dependency_root/mathlib/src/public.vais" <<'SRC'
import arith.add
SRC
cat > "$dependency_root/app/src/main.vais" <<'SRC'
import mathlib.public

fn main() -> Int {
    return add(20, 22)
}
SRC

"$VAISC" run "$dependency_root/app/src/main.vais" >"$tmp/dependency_accept.out" 2>"$tmp/dependency_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts local dependency package paths"
else
    echo "  FAIL accepts local dependency package paths got=$got want=42"
    cat "$tmp/dependency_accept.err"
    fail=1
fi

VAISC_FORCE_PYTHON=1 "$VAISC" run "$dependency_root/app/src/main.vais" >"$tmp/dependency_py_accept.out" 2>"$tmp/dependency_py_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS Python fallback accepts local dependency package paths"
else
    echo "  FAIL Python fallback local dependency package paths got=$got want=42"
    cat "$tmp/dependency_py_accept.err"
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

expect_reject_path() {
    local label="$1"
    local src="$2"
    local needle="$3"
    local help_needle="$4"
    "$VAISC" emit-ir "$src" -o "$tmp/$label.ll" >"$tmp/$label.out" 2>"$tmp/$label.err"
    local rc=$?
    if [ "$rc" = "0" ]; then
        echo "  FAIL rejects $label: command unexpectedly succeeded"
        fail=1
        return
    fi
    if grep -q "$needle" "$tmp/$label.err" && grep -q "help:" "$tmp/$label.err" && grep -q "$help_needle" "$tmp/$label.err"; then
        echo "  PASS rejects $label with P4 help"
    else
        echo "  FAIL rejects $label: missing expected diagnostic"
        cat "$tmp/$label.err"
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

map_accept="$tmp/front_map_accept.vais"
cat > "$map_accept" <<'SRC'
fn main() -> Int {
    let scores: Map<Int, Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    return scores.get(4, 0) + scores.get(5, 0 - 1) + scores.contains(4) + scores.contains(5) + scores.len()
}
SRC

"$VAISC" run "$map_accept" >"$tmp/map_accept.out" 2>"$tmp/map_accept.err"
got=$?
if [ "$got" = "42" ]; then
    echo "  PASS accepts local Map<Int,Int> full slice"
else
    echo "  FAIL accepts local Map<Int,Int> got=$got want=42"
    cat "$tmp/map_accept.err"
    fail=1
fi

expect_reject "map_generic_not_verified" "only local Map<Int,Int> values are verified" "generic key/value forms" <<'SRC'
fn main() -> Int {
    let scores: Map<Str, Int> = {}
    return 0
}
SRC

expect_reject "module_not_implemented" "module and package declarations are not implemented" "module names are derived from file paths" <<'SRC'
module math.add

fn main() -> Int {
    return 42
}
SRC

expect_reject "package_not_implemented" "module and package declarations are not implemented" "module names are derived from file paths" <<'SRC'
package demo

fn main() -> Int {
    return 42
}
SRC

expect_reject "missing_import" "import path not found" "expected local module file" <<'SRC'
import math.missing

fn main() -> Int {
    return 0
}
SRC

bad_manifest_root="$tmp/bad_manifest"
mkdir -p "$bad_manifest_root/src"
cat > "$bad_manifest_root/vais.toml" <<'SRC'
name = "bad_manifest"
version = "0.1.0"
SRC
cat > "$bad_manifest_root/src/main.vais" <<'SRC'
fn main() -> Int {
    return 42
}
SRC
expect_reject_path "bad_manifest" "$bad_manifest_root/src/main.vais" "package manifest is missing required key" "write .*source"

bad_source_root="$tmp/bad_source_manifest"
mkdir -p "$bad_source_root/src"
cat > "$bad_source_root/vais.toml" <<'SRC'
name = "bad_source"
version = "0.1.0"
source = "../src"
SRC
cat > "$bad_source_root/src/main.vais" <<'SRC'
fn main() -> Int {
    return 42
}
SRC
expect_reject_path "bad_source_manifest" "$bad_source_root/src/main.vais" "package manifest source must be a local relative path" "absolute paths and .*\\.\\."

missing_dep_root="$tmp/missing_dependency_manifest"
mkdir -p "$missing_dep_root/src"
cat > "$missing_dep_root/vais.toml" <<'SRC'
name = "missing_dep"
version = "0.1.0"
source = "src"

[dependencies]
missing = "../missing"
SRC
cat > "$missing_dep_root/src/main.vais" <<'SRC'
fn main() -> Int {
    return 42
}
SRC
expect_reject_path "missing_dependency_manifest" "$missing_dep_root/src/main.vais" "local dependency manifest not found" "expected local dependency manifest"

bad_dep_path_root="$tmp/bad_dependency_path"
mkdir -p "$bad_dep_path_root/src"
cat > "$bad_dep_path_root/vais.toml" <<'SRC'
name = "bad_dep_path"
version = "0.1.0"
source = "src"

[dependencies]
web = "https://example.com/pkg"
SRC
cat > "$bad_dep_path_root/src/main.vais" <<'SRC'
fn main() -> Int {
    return 42
}
SRC
expect_reject_path "bad_dependency_path" "$bad_dep_path_root/src/main.vais" "local dependency path must be a relative local path" "absolute paths, URLs"

dependency_cycle_root="$tmp/dependency_cycle"
mkdir -p "$dependency_cycle_root/a/src" "$dependency_cycle_root/b/src"
cat > "$dependency_cycle_root/a/vais.toml" <<'SRC'
name = "cycle_a"
version = "0.1.0"
source = "src"

[dependencies]
b = "../b"
SRC
cat > "$dependency_cycle_root/b/vais.toml" <<'SRC'
name = "cycle_b"
version = "0.1.0"
source = "src"

[dependencies]
a = "../a"
SRC
cat > "$dependency_cycle_root/a/src/main.vais" <<'SRC'
fn main() -> Int {
    return 42
}
SRC
expect_reject_path "dependency_cycle" "$dependency_cycle_root/a/src/main.vais" "local dependency cycle detected" "remove one dependency"

dup_root="$tmp/duplicate_import"
mkdir -p "$dup_root/a" "$dup_root/b"
cat > "$dup_root/a/one.vais" <<'SRC'
fn helper() -> Int {
    return 1
}
SRC
cat > "$dup_root/b/two.vais" <<'SRC'
fn helper() -> Int {
    return 2
}
SRC
expect_reject "duplicate_import/main" "duplicate top-level symbol" "first definition" <<'SRC'
import a.one
import b.two

fn main() -> Int {
    return helper()
}
SRC

cycle_root="$tmp/cycle_import"
mkdir -p "$cycle_root"
cat > "$cycle_root/a.vais" <<'SRC'
import b

fn a() -> Int {
    return 1
}
SRC
cat > "$cycle_root/b.vais" <<'SRC'
import a

fn b() -> Int {
    return 2
}
SRC
expect_reject "cycle_import/main" "import cycle detected" "remove one import from the cycle" <<'SRC'
import a

fn main() -> Int {
    return a()
}
SRC

expect_reject "string_type" "helper parameters must use verified scalar types" "Int.*Str.*Bool" <<'SRC'
fn len(s: String) -> Int {
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
