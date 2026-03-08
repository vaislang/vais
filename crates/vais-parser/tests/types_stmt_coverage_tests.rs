//! Coverage tests for parser types.rs and stmt.rs
//!
//! Targets: types.rs (generic params, type parsing, const expr, trait bounds),
//! stmt.rs (let patterns, assignments, control flow statements)

use vais_ast::*;
use vais_parser::parse;

fn parse_ok(source: &str) -> Module {
    parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {:?}", source, e))
}

fn parse_err(source: &str) {
    assert!(
        parse(source).is_err(),
        "Expected parse error for: {}",
        source
    );
}

// ============================================================================
// types.rs — parse_generics: type parameter
// ============================================================================

#[test]
fn test_parse_generic_single() {
    let module = parse_ok("F id<T>(x: T) -> T = x");
    assert!(!module.items.is_empty());
    if let Item::Function(f) = &module.items[0].node {
        assert_eq!(f.generics.len(), 1);
    }
}

#[test]
fn test_parse_generic_multiple() {
    let module = parse_ok("F pair<A, B>(a: A, b: B) -> A = a");
    if let Item::Function(f) = &module.items[0].node {
        assert_eq!(f.generics.len(), 2);
    }
}

// ============================================================================
// types.rs — parse_generics: bounded type parameter
// ============================================================================

#[test]
fn test_parse_generic_with_bound() {
    let module = parse_ok(
        r#"
        W Display { F show(self) -> str }
        F print_it<T: Display>(x: T) -> i64 = 0
    "#,
    );
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_generic_multi_bound() {
    let module = parse_ok(
        r#"
        W A { F a(self) -> i64 }
        W B { F b(self) -> i64 }
        F both<T: A + B>(x: T) -> i64 = 0
    "#,
    );
    assert!(module.items.len() >= 3);
}

// ============================================================================
// types.rs — parse_generics: const generic
// ============================================================================

#[test]
fn test_parse_const_generic() {
    let module = parse_ok("F fixed<const N: u64>() -> i64 = 0");
    if let Item::Function(f) = &module.items[0].node {
        assert_eq!(f.generics.len(), 1);
    }
}

// ============================================================================
// types.rs — parse_generics: lifetime parameter
// ============================================================================

#[test]
fn test_parse_lifetime_generic() {
    let module = parse_ok("F borrow<'a>(x: &'a i64) -> i64 = 0");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_type: primitive types
// ============================================================================

#[test]
fn test_parse_type_i64() {
    let module = parse_ok("F test(x: i64) -> i64 = x");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_bool() {
    let module = parse_ok("F test(x: bool) -> bool = x");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_f64() {
    let module = parse_ok("F test(x: f64) -> f64 = x");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_str() {
    let module = parse_ok("F test(x: str) -> str = x");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_type: compound types
// ============================================================================

#[test]
fn test_parse_type_array() {
    let module = parse_ok("F test(arr: [i64; 5]) -> i64 = 0");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_tuple() {
    let module = parse_ok("F test(t: (i64, bool)) -> i64 = 0");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_reference() {
    let module = parse_ok("F test(x: &i64) -> i64 = 0");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_mut_reference() {
    let module = parse_ok("F test(x: &mut i64) -> i64 = 0");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_option() {
    let module = parse_ok("F test(x: Option<i64>) -> i64 = 0");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_type_result() {
    let module = parse_ok("F test(x: Result<i64, str>) -> i64 = 0");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_type: function type
// ============================================================================

#[test]
fn test_parse_type_pointer() {
    let module = parse_ok("F test(x: &i64) -> i64 = 0");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_type: nested generics (Vec<Vec<i64>>)
// ============================================================================

#[test]
fn test_parse_nested_generic() {
    let module = parse_ok("F test(x: Vec<Vec<i64>>) -> i64 = 0");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_struct_fields
// ============================================================================

#[test]
fn test_parse_struct_fields() {
    let module = parse_ok(
        r#"
        S Point { x: i64, y: i64, z: i64 }
    "#,
    );
    if let Item::Struct(s) = &module.items[0].node {
        assert_eq!(s.fields.len(), 3);
    }
}

#[test]
fn test_parse_struct_generic() {
    let module = parse_ok(
        r#"
        S Pair<A, B> { first: A, second: B }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — where clause
// ============================================================================

#[test]
fn test_parse_where_clause() {
    let module = parse_ok(
        r#"
        W Printable { F show(self) -> i64 }
        F print_all<T>(items: T) -> i64 where T: Printable = 0
    "#,
    );
    assert!(module.items.len() >= 2);
}

// ============================================================================
// stmt.rs — let binding
// ============================================================================

#[test]
fn test_parse_let_simple() {
    let module = parse_ok("F test() -> i64 { x := 42\n R x }");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_let_mutable() {
    let module = parse_ok("F test() -> i64 { x := mut 0\n x = 1\n R x }");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_let_with_expr() {
    let module = parse_ok("F test() -> i64 { x := 21 * 2\n R x }");
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — return statement
// ============================================================================

#[test]
fn test_parse_return() {
    let module = parse_ok("F test() -> i64 { R 42 }");
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — expression statement
// ============================================================================

#[test]
fn test_parse_expr_stmt() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            println("hello")
            R 0
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — assignment
// ============================================================================

#[test]
fn test_parse_assignment() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            R x
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — compound assignment
// ============================================================================

#[test]
fn test_parse_compound_add() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x += 10
            R x
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_compound_all_ops() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            x := mut 100
            x += 1
            x -= 2
            x *= 3
            x /= 4
            x %= 5
            x &= 6
            x |= 7
            x ^= 8
            x <<= 1
            x >>= 1
            R x
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — if/else
// ============================================================================

#[test]
fn test_parse_if() {
    let module = parse_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 { R 1 }
            R 0
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_if_else() {
    let module = parse_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 { R 1 } E { R 0 }
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — match statement
// ============================================================================

#[test]
fn test_parse_match() {
    let module = parse_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 => 10,
                2 => 20,
                _ => 0
            }
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — loop
// ============================================================================

#[test]
fn test_parse_infinite_loop() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L {
                x += 1
                I x > 10 { B }
            }
            R x
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_for_loop() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i:0..10 {
                sum += i
            }
            R sum
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — break and continue
// ============================================================================

#[test]
fn test_parse_break() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            L {
                B
            }
            R 0
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_continue() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i:0..10 {
                I i % 2 == 0 { C }
                sum += i
            }
            R sum
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// stmt.rs — defer
// ============================================================================

#[test]
fn test_parse_defer() {
    let module = parse_ok(
        r#"
        F test() -> i64 {
            D println("cleanup")
            R 42
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — type alias
// ============================================================================

#[test]
fn test_parse_type_alias() {
    let module = parse_ok("T Num = i64");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — trait declaration
// ============================================================================

#[test]
fn test_parse_trait() {
    let module = parse_ok(
        r#"
        W Comparable {
            F compare(self, other: i64) -> i64
        }
    "#,
    );
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — impl block
// ============================================================================

#[test]
fn test_parse_impl() {
    let module = parse_ok(
        r#"
        S Point { x: i64, y: i64 }
        X Point {
            F new(x: i64, y: i64) -> Point = Point { x: x, y: y }
        }
    "#,
    );
    assert!(module.items.len() >= 2);
}

// ============================================================================
// types.rs — enum declaration
// ============================================================================

#[test]
fn test_parse_enum_unit() {
    let module = parse_ok("E Color { Red, Green, Blue }");
    if let Item::Enum(e) = &module.items[0].node {
        assert_eq!(e.variants.len(), 3);
    }
}

#[test]
fn test_parse_enum_with_payload() {
    let module = parse_ok("E Shape { Circle(i64), Rect(i64, i64) }");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — const expression parsing
// ============================================================================

#[test]
fn test_parse_const_expr() {
    let module = parse_ok("G MAX: i64 = 100");
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_const_expr_negative() {
    let module = parse_ok("G MIN: i64 = -100");
    assert!(!module.items.is_empty());
}

// ============================================================================
// types.rs — parse_params (function parameters)
// ============================================================================

#[test]
fn test_parse_params_none() {
    let module = parse_ok("F test() -> i64 = 0");
    if let Item::Function(f) = &module.items[0].node {
        assert!(f.params.is_empty());
    }
}

#[test]
fn test_parse_params_single() {
    let module = parse_ok("F test(x: i64) -> i64 = x");
    if let Item::Function(f) = &module.items[0].node {
        assert_eq!(f.params.len(), 1);
    }
}

#[test]
fn test_parse_params_many() {
    let module = parse_ok("F test(a: i64, b: i64, c: i64, d: i64) -> i64 = a");
    if let Item::Function(f) = &module.items[0].node {
        assert_eq!(f.params.len(), 4);
    }
}

#[test]
fn test_parse_self_param() {
    let module = parse_ok(
        r#"
        S Foo { x: i64 }
        X Foo {
            F get(self) -> i64 = self.x
        }
    "#,
    );
    assert!(!module.items.is_empty());
}
