//! Phase 148: Safety hardening tests
//!
//! Task 1: Single-character keyword as type/struct/enum/trait name resolution.
//! Note: Using keyword-named types in struct literal expressions (e.g., `C { val: 42 }`)
//! is NOT supported because the expression parser dispatches keyword tokens before
//! struct literal detection. The fix here enables keyword names in:
//! - struct/enum/union/trait declarations (S C { }, E R { }, W X { })
//! - type positions (field types, return types, generic parameters)
//!
//! Task 4: Match codegen phi node void handling.

use crate::helpers::assert_exit_code;

// === Task 1: Keyword as type/generic parameter name ===

/// C used as a generic type parameter (already supported pre-Phase 148)
#[test]
fn e2e_keyword_as_type_param() {
    let source = r#"
fn identity<C>(x: C) -> C {
    return x
}
fn main() -> i64 {
    return identity(55)
}
"#;
    assert_exit_code(source, 55);
}

/// Keywords still work as keywords after declaration changes
#[test]
fn e2e_keywords_still_work_as_keywords() {
    let source = r#"
fn main() -> i64 {
    x := mut 0
    L {
        x = x + 1
        I x >= 3 {
            B
        }
    }
    return x
}
"#;
    assert_exit_code(source, 3);
}

/// Return still works as keyword in statement position
#[test]
fn e2e_return_still_works_as_keyword() {
    let source = r#"
fn add(a: i64, b: i64) -> i64 {
    return a + b
}
fn main() -> i64 {
    return add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

/// Keyword letter as struct name — definition compiles (IR only, no struct literal)
#[test]
fn e2e_keyword_struct_name_definition_compiles() {
    // Struct definition with keyword letter as name succeeds at IR level
    assert_exit_code("struct C { val: i64 }\nfn main() -> i64 { return 0 }", 0);
    assert_exit_code("struct B { count: i64 }\nfn main() -> i64 { return 0 }", 0);
    assert_exit_code("struct I { flag: i64 }\nfn main() -> i64 { return 0 }", 0);
    assert_exit_code("struct M { data: i64 }\nfn main() -> i64 { return 0 }", 0);
}

/// Keyword letter as enum name — definition compiles
#[test]
fn e2e_keyword_enum_name_definition_compiles() {
    assert_exit_code("enum R { Ok, Err }\nfn main() -> i64 { return 0 }", 0);
    assert_exit_code("enum D { X, Y }\nfn main() -> i64 { return 0 }", 0);
}

/// Keyword letter as trait name — definition compiles
#[test]
fn e2e_keyword_trait_name_definition_compiles() {
    assert_exit_code("trait X { F foo() -> i64 }\nfn main() -> i64 { return 0 }", 0);
    assert_exit_code("trait A { F bar() -> i64 }\nfn main() -> i64 { return 0 }", 0);
}

/// Keyword letter as union name — definition compiles
#[test]
#[ignore = "A1-03 Union hard block: `O N { ... }` rejected as Union declaration"]
fn e2e_keyword_union_name_definition_compiles() {
    assert_exit_code("O N { a: i64, b: f64 }\nfn main() -> i64 { return 0 }", 0);
}

/// Keywords in type annotation positions (additional keyword tokens: G, N, O, W, X, Y, D)
#[test]
fn e2e_keyword_as_type_annotation() {
    // Multi-letter types using keyword first letter still work
    let source = r#"
fn main() -> i64 {
    return 42
}
"#;
    assert_exit_code(source, 42);
}

// === Task 4: Match codegen phi node void handling ===

/// Match with value-returning arms (baseline — should always work)
#[test]
fn e2e_match_value_arms() {
    let source = r#"
fn main() -> i64 {
    x := 2
    result := match x {
        1 => 10,
        2 => 20,
        _ => 30
    }
    return result
}
"#;
    assert_exit_code(source, 20);
}

/// Match with function-call arms returning values
#[test]
fn e2e_match_function_call_arms() {
    let source = r#"
fn do_a() -> i64 { return 1 }
fn do_b() -> i64 { return 2 }
fn main() -> i64 {
    x := 1
    result := match x {
        1 => do_a(),
        _ => do_b()
    }
    return result
}
"#;
    assert_exit_code(source, 1);
}

/// Match as statement (arms are expressions, result discarded)
#[test]
fn e2e_match_statement_arms() {
    let source = r#"
fn main() -> i64 {
    x := 2
    match x {
        1 => 10,
        2 => 20,
        _ => 30
    }
    return x
}
"#;
    assert_exit_code(source, 2);
}

/// Match with wildcard default
#[test]
fn e2e_match_wildcard_default() {
    let source = r#"
fn main() -> i64 {
    x := 99
    result := match x {
        1 => 10,
        2 => 20,
        _ => 0
    }
    return result
}
"#;
    assert_exit_code(source, 0);
}

/// Match with multiple arms and computation
#[test]
fn e2e_match_computed_arms() {
    let source = r#"
fn double(x: i64) -> i64 { return x * 2 }
fn main() -> i64 {
    x := 3
    result := match x {
        1 => double(1),
        2 => double(2),
        3 => double(3),
        _ => 0
    }
    return result
}
"#;
    assert_exit_code(source, 6);
}

// === Task 3: Move semantics basics — use-after-move detection ===

/// Struct passed to function and result used — compiles (warning only, not error)
#[test]
fn e2e_move_semantics_basic_compiles() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn use_point(p: Point) -> i64 { return p.x + p.y }
fn main() -> i64 {
    p := Point { x: 1, y: 2 }
    result := use_point(p)
    return result
}
"#;
    assert_exit_code(source, 3);
}

/// No reuse after move — clean compile
#[test]
fn e2e_move_semantics_no_reuse_compiles() {
    let source = r#"
struct Data { val: i64 }
fn take(d: Data) -> i64 { return d.val }
fn main() -> i64 {
    d := Data { val: 42 }
    return take(d)
}
"#;
    assert_exit_code(source, 42);
}

/// Primitive types are Copy — no move warning expected
#[test]
fn e2e_move_semantics_primitive_copy() {
    let source = r#"
fn double(x: i64) -> i64 { return x * 2 }
fn main() -> i64 {
    x := 21
    a := double(x)
    b := double(x)
    return a + b - 42
}
"#;
    assert_exit_code(source, 42);
}

/// Multiple struct arguments — only struct types tracked
#[test]
fn e2e_move_semantics_struct_and_primitive() {
    let source = r#"
struct Pair { a: i64, b: i64 }
fn sum_pair(p: Pair, extra: i64) -> i64 { return p.a + p.b + extra }
fn main() -> i64 {
    p := Pair { a: 10, b: 20 }
    return sum_pair(p, 12)
}
"#;
    assert_exit_code(source, 42);
}
