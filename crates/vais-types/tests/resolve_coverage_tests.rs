//! Coverage tests for vais-types/src/resolve.rs
//!
//! Targets: type resolution, name resolution, scope-based type lookup,
//! struct/enum/function name resolution, generic type resolution.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) -> TypeChecker {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
    tc
}

// ============================================================================
// Named type resolution
// ============================================================================

#[test]
fn test_resolve_i64() {
    check_ok("fn f(x: i64) -> i64 = x");
}

#[test]
fn test_resolve_i32() {
    check_ok("fn f(x: i32) -> i32 = x");
}

#[test]
fn test_resolve_i16() {
    check_ok("fn f(x: i16) -> i16 = x");
}

#[test]
fn test_resolve_i8() {
    check_ok("fn f(x: i8) -> i8 = x");
}

#[test]
fn test_resolve_f64() {
    check_ok("fn f(x: f64) -> f64 = x");
}

#[test]
fn test_resolve_f32() {
    check_ok("fn f(x: f32) -> f32 = x");
}

#[test]
fn test_resolve_bool() {
    check_ok("fn f(x: bool) -> bool = x");
}

#[test]
fn test_resolve_str() {
    check_ok("fn f(x: str) -> str = x");
}

#[test]
fn test_resolve_u8() {
    check_ok("fn f(x: u8) -> u8 = x");
}

#[test]
fn test_resolve_u16() {
    check_ok("fn f(x: u16) -> u16 = x");
}

#[test]
fn test_resolve_u32() {
    check_ok("fn f(x: u32) -> u32 = x");
}

#[test]
fn test_resolve_u64() {
    check_ok("fn f(x: u64) -> u64 = x");
}

#[test]
fn test_resolve_i128() {
    check_ok("fn f(x: i128) -> i128 = x");
}

#[test]
fn test_resolve_u128() {
    check_ok("fn f(x: u128) -> u128 = x");
}

// ============================================================================
// Struct type resolution
// ============================================================================

#[test]
fn test_resolve_struct_type() {
    check_ok(
        r#"
        struct Point { x: i64, y: i64 }
        fn f(p: Point) -> i64 = p.x
    "#,
    );
}

#[test]
fn test_resolve_struct_field_types() {
    check_ok(
        r#"
        struct Mixed {
            a: i64,
            b: f64,
            c: bool,
            d: str
        }
        fn f() -> i64 {
            m := Mixed { a: 1, b: 2.0, c: true, d: "hi" }
            m.a
        }
    "#,
    );
}

#[test]
fn test_resolve_nested_struct() {
    check_ok(
        r#"
        struct Inner { val: i64 }
        struct Outer { inner: Inner }
        fn f() -> i64 {
            o := Outer { inner: Inner { val: 42 } }
            o.inner.val
        }
    "#,
    );
}

// ============================================================================
// Enum type resolution
// ============================================================================

#[test]
fn test_resolve_enum_type() {
    check_ok(
        r#"
        enum Direction { North, South, East, West }
        fn f() -> i64 {
            d := North
            0
        }
    "#,
    );
}

#[test]
fn test_resolve_enum_with_data() {
    check_ok(
        r#"
        enum Shape { Circle(i64), Rectangle(i64, i64) }
        fn f() -> i64 {
            s := Circle(5)
            0
        }
    "#,
    );
}

// ============================================================================
// Type alias resolution
// ============================================================================

#[test]
fn test_resolve_type_alias() {
    check_ok(
        r#"
        type Number = i64
        fn add(a: Number, b: Number) -> Number = a + b
    "#,
    );
}

#[test]
fn test_resolve_type_alias_chain() {
    check_ok(
        r#"
        type Num = i64
        type Count = Num
        fn f(x: Count) -> Count = x + 1
    "#,
    );
}

// ============================================================================
// Array type resolution
// ============================================================================

#[test]
fn test_resolve_array_type() {
    check_ok(
        r#"
        fn f() -> i64 {
            arr := [1, 2, 3]
            arr[0]
        }
    "#,
    );
}

// ============================================================================
// Tuple type resolution
// ============================================================================

#[test]
fn test_resolve_tuple_type() {
    check_ok(
        r#"
        fn f() -> i64 {
            t := (1, 2, 3)
            0
        }
    "#,
    );
}

// ============================================================================
// Function type resolution
// ============================================================================

#[test]
fn test_resolve_function_return_types() {
    check_ok(
        r#"
        fn returns_i64() -> i64 = 42
        fn returns_bool() -> bool = true
        fn returns_str() -> str = "hello"
        fn returns_f64() -> f64 = 3.14
    "#,
    );
}

// ============================================================================
// Generic type resolution
// ============================================================================

#[test]
fn test_resolve_generic_function() {
    check_ok(
        r#"
        fn id<T>(x: T) -> T = x
        fn test() -> i64 = id(42)
    "#,
    );
}

#[test]
fn test_resolve_generic_struct() {
    check_ok(
        r#"
        struct Wrapper<T> { value: T }
        fn test() -> i64 {
            w := Wrapper { value: 42 }
            w.value
        }
    "#,
    );
}

// ============================================================================
// Scope resolution
// ============================================================================

#[test]
fn test_resolve_variable_in_scope() {
    check_ok(
        r#"
        fn f() -> i64 {
            x := 10
            y := x + 1
            y
        }
    "#,
    );
}

#[test]
fn test_resolve_shadowed_variable() {
    check_ok(
        r#"
        fn f() -> i64 {
            x := 10
            x := 20
            x
        }
    "#,
    );
}

#[test]
fn test_resolve_block_scope() {
    check_ok(
        r#"
        fn f() -> i64 {
            x := {
                a := 10
                b := 20
                a + b
            }
            x
        }
    "#,
    );
}

// ============================================================================
// Multiple file-level declarations
// ============================================================================

#[test]
fn test_resolve_forward_reference() {
    check_ok(
        r#"
        fn test() -> i64 = helper(5)
        fn helper(x: i64) -> i64 = x * 2
    "#,
    );
}

#[test]
fn test_resolve_struct_used_in_function() {
    check_ok(
        r#"
        fn make_point() -> Point = Point { x: 0, y: 0 }
        struct Point { x: i64, y: i64 }
    "#,
    );
}

// ============================================================================
// Complex resolution scenarios
// ============================================================================

#[test]
fn test_resolve_method_on_struct() {
    check_ok(
        r#"
        struct Pair { a: i64, b: i64 }
        impl Pair {
            fn sum(self) -> i64 = self.a + self.b
        }
        fn test() -> i64 {
            p := Pair { a: 3, b: 4 }
            p.sum()
        }
    "#,
    );
}

#[test]
fn test_resolve_trait_method_on_struct() {
    check_ok(
        r#"
        trait Sized {
            fn size(self) -> i64
        }
        struct Box { width: i64 }
        impl Box: Sized {
            fn size(self) -> i64 = self.width
        }
        fn test() -> i64 {
            b := Box { width: 10 }
            b.size()
        }
    "#,
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_resolve_empty_struct() {
    check_ok(
        r#"
        struct Empty {}
        fn test() -> i64 {
            e := Empty {}
            0
        }
    "#,
    );
}

#[test]
fn test_resolve_recursive_struct_use() {
    check_ok(
        r#"
        struct Node { value: i64 }
        fn test() -> i64 {
            n := Node { value: 42 }
            n.value
        }
    "#,
    );
}
