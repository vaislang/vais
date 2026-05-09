//! Parser coverage tests
//!
//! Targets uncovered lines in parser (1,028 uncovered, 74.3%)
//! Focus: lib.rs error recovery, item/macros.rs, types.rs advanced parsing

use vais_ast::*;
use vais_parser::parse;

fn parse_ok(source: &str) -> Module {
    parse(source).unwrap_or_else(|e| panic!("Parse failed for: {}\nErr: {:?}", source, e))
}

fn _parse_err(source: &str) {
    assert!(
        parse(source).is_err(),
        "Expected parse error for: {}",
        source
    );
}

// ============================================================================
// Macro declarations
// ============================================================================

#[test]
fn test_parse_macro_rules_basic() {
    let source = r#"
        macro my_macro! {
            () => { 42 }
        }
    "#;
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_macro_rules_with_args() {
    let source = r#"
        macro add! {
            ($a:expr, $b:expr) => { $a + $b }
        }
    "#;
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_macro_invocation() {
    let source = "fn test() -> i64 = my_macro!(1, 2, 3)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Type parsing coverage
// ============================================================================

#[test]
fn test_parse_optional_type() {
    let source = "fn test(x: i64?) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_result_type() {
    let source = "fn test() -> Result<i64, str> = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(f.ret_type.is_some());
    }
}

#[test]
fn test_parse_array_type() {
    let source = "fn test(x: [i64]) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_tuple_type() {
    let source = "fn test(x: (i64, bool)) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_map_type() {
    let source = "fn test(x: HashMap<str, i64>) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_nested_generic_type() {
    let source = "fn test(x: Vec<Option<i64>>) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_fn_type() {
    let source = "fn test(f: fn(i64) -> bool) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_ref_type() {
    let source = "fn test(x: &i64) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_ref_mut_type() {
    let source = "fn test(x: &mut i64) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

#[test]
fn test_parse_slice_type() {
    let source = "fn test(x: &[i64]) -> i64 = 0";
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.params.is_empty());
    }
}

// ============================================================================
// Advanced declarations
// ============================================================================

#[test]
fn test_parse_type_alias() {
    let source = "type Num = i64";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
    assert!(matches!(&module.items[0].node, Item::TypeAlias(..)));
}

#[test]
fn test_parse_global() {
    let source = "G counter: i64 = 0";
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_const() {
    let source = "C MAX: i64 = 100";
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_extern_block() {
    let source = r#"
        N {
            fn malloc(size: i64) -> i64
            fn free(ptr: i64) -> i64
        }
    "#;
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_use_statement() {
    let source = "use std.io";
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

// ============================================================================
// Struct/Enum/Trait/Impl
// ============================================================================

#[test]
fn test_parse_struct_with_methods() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        impl Point {
            fn new(x: i64, y: i64) -> Point {
                return Point { x: x, y: y }
            }
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_enum_with_variants() {
    let source = r#"
        E Color {
            Red,
            Green,
            Blue,
            Custom(i64)
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
    if let Item::Enum(e) = &module.items[0].node {
        assert_eq!(e.variants.len(), 4);
    }
}

#[test]
fn test_parse_trait() {
    let source = r#"
        trait Printable {
            fn print(self) -> i64
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_trait_impl() {
    let source = r#"
        trait Printable {
            fn print(self) -> i64
        }
        struct Point { x: i64 }
        impl Point: Printable {
            fn print(self) -> i64 = self.x
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 3);
}

// ============================================================================
// Expressions
// ============================================================================

#[test]
fn test_parse_match_expression() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                0 => 10,
                1 => 20,
                _ => 30
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_if_else_chain() {
    let source = r#"
        fn test(x: i64) -> i64 {
            I x > 10 {
                return 1
            } E I x > 5 {
                return 2
            } E {
                return 3
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_loop_with_range() {
    let source = r#"
        fn test() -> i64 {
            sum := 0
            L i:0..10 {
                sum = sum + i
            }
            return sum
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_lambda() {
    let source = "fn test() -> i64 { f := |x| x * 2 \n R f(5) }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pipe_operator() {
    let source = "fn test(x: i64) -> i64 = x |> abs";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_ternary() {
    let source = "fn test(x: i64) -> i64 = x > 0 ? x : 0 - x";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_self_recursion() {
    let source = "fn factorial(n: i64) -> i64 = n <= 1 ? 1 : n * @(n - 1)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_string_interpolation() {
    let source = r#"fn test(name: str) -> str = ~"Hello, {name}!""#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_struct_literal() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        fn test() -> Point = Point { x: 1, y: 2 }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_defer() {
    let source = r#"
        fn test() -> i64 {
            D free(ptr)
            return 0
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_comptime() {
    let source = "fn test() -> i64 = comptime { 4 * 8 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Where clause
// ============================================================================

#[test]
fn test_parse_where_clause() {
    let source = r#"
        fn print_it<T>(x: T) -> i64 where T: Printable {
            return x.print()
        }
    "#;
    let module = parse_ok(source);
    if let Item::Function(f) = &module.items[0].node {
        assert!(!f.where_clause.is_empty());
    }
}

// ============================================================================
// Attributes
// ============================================================================

#[test]
fn test_parse_cfg_attribute() {
    let source = r#"
        #[cfg(target_os = "linux")]
        fn linux_only() -> i64 = 0
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_wasm_attribute() {
    let source = r#"
        #[wasm_export("add")]
        fn add(a: i64, b: i64) -> i64 = a + b
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Pattern matching
// ============================================================================

#[test]
fn test_parse_pattern_wildcard() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pattern_literal() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                42 => 1,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_or_pattern() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                1 | 2 | 3 => 10,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_guard_pattern() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                n I n > 0 => n,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Union
// ============================================================================

#[test]
fn test_parse_union() {
    let source = r#"
        O IntOrFloat {
            i: i64,
            f: f64
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Multiple items
// ============================================================================

#[test]
fn test_parse_multiple_functions() {
    let source = r#"
        fn add(a: i64, b: i64) -> i64 = a + b
        fn sub(a: i64, b: i64) -> i64 = a - b
        fn mul(a: i64, b: i64) -> i64 = a * b
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 3);
}

#[test]
fn test_parse_pub_function() {
    let source = "pub fn public_fn() -> i64 = 42";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
    if let Item::Function(f) = &module.items[0].node {
        assert!(f.is_pub);
    }
}

// ============================================================================
// Error recovery
// ============================================================================

#[test]
fn test_parse_empty_source() {
    let module = parse_ok("");
    assert!(module.items.is_empty());
}

#[test]
fn test_parse_comments_only() {
    let source = "# This is a comment\n# Another comment";
    let module = parse_ok(source);
    assert!(module.items.is_empty());
}
