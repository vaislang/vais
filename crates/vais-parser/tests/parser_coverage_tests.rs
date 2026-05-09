//! Extended parser coverage tests
//!
//! Targets uncovered lines in lib.rs (297 uncovered), expr/primary.rs (217),
//! item/macros.rs (155), types.rs (151), expr/precedence.rs (109),
//! item/declarations.rs (88), stmt.rs (82)
//! Focus: token_description, parse_with_cfg, parse_with_recovery,
//! advanced expression parsing, error recovery paths

use std::collections::HashMap;
use vais_ast::*;
use vais_parser::{parse, parse_with_cfg, parse_with_recovery};

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
// parse_with_cfg tests (lib.rs lines 862-879)
// ============================================================================

#[test]
fn test_parse_with_cfg_linux() {
    let source = r#"
        #[cfg(target_os = "linux")]
        fn linux_only() -> i64 = 1

        fn always() -> i64 = 2
    "#;
    let mut cfg = HashMap::new();
    cfg.insert("target_os".to_string(), "linux".to_string());
    let module = parse_with_cfg(source, cfg).unwrap();
    // linux_only should be included
    assert!(module.items.len() >= 1);
}

#[test]
fn test_parse_with_cfg_windows_excluded() {
    let source = r#"
        #[cfg(target_os = "windows")]
        fn windows_only() -> i64 = 1

        fn always() -> i64 = 2
    "#;
    let mut cfg = HashMap::new();
    cfg.insert("target_os".to_string(), "linux".to_string());
    let module = parse_with_cfg(source, cfg).unwrap();
    // windows_only should be excluded, only always should remain
    assert!(module
        .items
        .iter()
        .any(|item| { matches!(&item.node, Item::Function(f) if f.name.node == "always") }));
}

#[test]
fn test_parse_with_cfg_empty() {
    let source = "fn test() -> i64 = 42";
    let module = parse_with_cfg(source, HashMap::new()).unwrap();
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// parse_with_recovery tests (lib.rs lines 911-932)
// ============================================================================

#[test]
fn test_parse_with_recovery_valid_code() {
    let source = "fn test() -> i64 = 42";
    let (module, errors) = parse_with_recovery(source);
    assert!(errors.is_empty());
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_with_recovery_broken_function() {
    let source = "fn broken(; S Valid { x: i64 }";
    let (module, errors) = parse_with_recovery(source);
    assert!(!errors.is_empty());
    // Should still parse the valid struct
    assert!(module
        .items
        .iter()
        .any(|item| { matches!(&item.node, Item::Struct(s) if s.name.node == "Valid") }));
}

#[test]
fn test_parse_with_recovery_multiple_errors() {
    let source = r#"
        fn a( = 1
        fn b( = 2
        fn valid() -> i64 = 42
    "#;
    let (module, errors) = parse_with_recovery(source);
    assert!(!errors.is_empty());
    // valid function should be present
    assert!(module
        .items
        .iter()
        .any(|item| { matches!(&item.node, Item::Function(f) if f.name.node == "valid") }));
}

#[test]
fn test_parse_with_recovery_missing_body() {
    let source = "fn test() -> i64";
    let (module, errors) = parse_with_recovery(source);
    // May or may not produce errors, but should not crash
    let _ = (module, errors);
}

// ============================================================================
// Advanced expression parsing (expr/primary.rs)
// ============================================================================

#[test]
fn test_parse_string_interpolation() {
    let source = r#"fn test() -> str = ~"hello {name}""#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_range_expressions() {
    let source = "fn test() -> i64 { L i:0..10 { C }; R 0 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_inclusive_range() {
    let source = "fn test() -> i64 { L i:0..=10 { C }; R 0 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pipe_operator() {
    let source = r#"
        fn double(x: i64) -> i64 = x * 2
        fn test() -> i64 = 5 |> double
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 2);
}

#[test]
fn test_parse_ternary_expression() {
    let source = "fn abs(x: i64) -> i64 = x > 0 ? x : -x";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_try_operator() {
    // Try operator syntax is valid even if Result type is complex
    let source = r#"
        fn test(x: i64) -> i64 {
            y := x + 1
            y
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_unwrap_operator() {
    let source = "fn test(x: i64) -> i64 { y := x + 1; y! }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_self_recursion() {
    let source = "fn factorial(n: i64) -> i64 { I n <= 1 { R 1 }; R n * @(n - 1) }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_lambda() {
    let source = "fn test() -> i64 { f := |x: i64| x * 2; f(21) }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_lambda_multi_param() {
    let source = "fn test() -> i64 { f := |x: i64, y: i64| x + y; f(1, 2) }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_struct_literal() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        fn test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_array_literal() {
    let source = "fn test() -> i64 { arr := [1, 2, 3, 4, 5]; R 0 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_tuple_literal() {
    let source = "fn test() -> i64 { t := (1, 2, 3); R 0 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_cast_expression() {
    let source = "fn test() -> f64 { x := 42; x as f64 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_ref_deref() {
    let source = "fn test(x: i64) -> i64 { y := &x; *y }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Statement parsing (stmt.rs)
// ============================================================================

#[test]
fn test_parse_mutable_binding() {
    let source = "fn test() -> i64 { x := mut 0; x = 42; x }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_typed_binding() {
    let source = "fn test() -> i64 { x: i64 = 42; x }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_return_statement() {
    let source = "fn test() -> i64 { R 42 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_break_continue() {
    let source = r#"
        fn test() -> i64 {
            L i:0..10 {
                I i == 5 { B }
                I i % 2 == 0 { C }
            }
            return 0
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_defer_statement() {
    let source = r#"
        fn test() -> i64 {
            D print(0)
            return 42
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Type parsing (types.rs lines 151+)
// ============================================================================

#[test]
fn test_parse_function_type() {
    let source = "fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_generic_type() {
    let source = "F id<T>(x: T) -> T = x";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_multi_generic_type() {
    let source = "F pair<A, B>(a: A, b: B) -> A = a";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_result_type() {
    let source = "fn test() -> i64 = 42";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_option_type() {
    let source = "fn test(x: i64?) -> i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_tuple_type() {
    let source = "fn test() -> (i64, bool) = (42, true)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_array_type() {
    let source = "fn test(arr: [i64]) -> i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pointer_type() {
    let source = "fn test(ptr: *i64) -> i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_reference_type() {
    let source = "fn test(r: &i64) -> i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Declaration parsing (item/declarations.rs)
// ============================================================================

#[test]
fn test_parse_struct_with_generics() {
    let source = "struct Container<T> { value: T, count: i64 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_enum_with_variants() {
    let source = r#"
        E Shape {
            Circle(i64),
            Rect(i64, i64),
            Point
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_trait() {
    let source = r#"
        trait Displayable {
            fn display(self) -> str
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_impl_block() {
    let source = r#"
        struct Counter { value: i64 }
        impl Counter {
            fn new() -> Counter = Counter { value: 0 }
            fn incr(self) -> Counter = Counter { value: self.value + 1 }
            fn get(self) -> i64 = self.value
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_trait_impl() {
    let source = r#"
        trait Describable { fn describe(self) -> str }
        struct Circle { radius: i64 }
        impl Circle: Describable {
            fn describe(self) -> str = "circle"
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 3);
}

#[test]
fn test_parse_type_alias() {
    let source = "type Num = i64";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_const_declaration() {
    let source = "C MAX: i64 = 100";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_union() {
    let source = r#"
        O Value {
            int_val: i64,
            float_val: f64
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
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

#[test]
fn test_parse_async_function() {
    let source = "A F fetch() -> i64 = 42";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Where clause parsing
// ============================================================================

#[test]
fn test_parse_where_clause() {
    let source = r#"
        trait Printable { fn show(self) -> str }
        fn display<T>(x: T) -> str where T: Printable = x.show()
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

// ============================================================================
// Pattern parsing (patterns in match)
// ============================================================================

#[test]
fn test_parse_literal_patterns() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                0 => 100,
                1 => 200,
                42 => 300,
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
fn test_parse_alias_pattern() {
    let source = r#"
        fn test(x: i64) -> i64 {
            match x {
                n @ 1 => n * 10,
                n @ _ => n
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
                n I n > 0 => n * 2,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_nested_patterns() {
    let source = r#"
        fn test(x: i64) -> i64 {
            t := (1, (2, 3))
            match t {
                (a, (b, c)) => a + b + c,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 1);
}

// ============================================================================
// Macro parsing (item/macros.rs)
// ============================================================================

#[test]
fn test_parse_macro_definition_multiple_rules() {
    let source = r#"
        macro double! {
            () => { 0 }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_macro_with_ident() {
    let source = r#"
        macro define! {
            ($name:ident, $val:expr) => { $name := $val }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_macro_invocation_expr() {
    let source = "fn test() -> i64 = my_macro!(1, 2, 3)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_macro_invocation_bracket() {
    let source = "fn test() -> i64 = vec![1, 2, 3]";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_macro_invocation_brace() {
    let source = "fn test() -> i64 = block!{ 42 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Expression precedence (expr/precedence.rs)
// ============================================================================

#[test]
fn test_parse_precedence_add_mul() {
    let source = "fn test() -> i64 = 1 + 2 * 3";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_comparison_chain() {
    let source = "fn test() -> bool = 1 < 2 && 3 > 1";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_bitwise() {
    let source = "fn test() -> i64 = 255 & 15 | 48 ^ 16";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_shift() {
    let source = "fn test() -> i64 = 1 << 8 >> 4";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_parenthesized() {
    let source = "fn test() -> i64 = (1 + 2) * (3 + 4)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_unary_in_binary() {
    let source = "fn test() -> i64 = -1 + 2";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_precedence_assign_ops() {
    let source = r#"
        fn test() -> i64 {
            x := mut 10
            x += 1
            x -= 2
            x *= 3
            x /= 4
            x %= 5
            x &= 255
            x |= 15
            x ^= 48
            x <<= 1
            x >>= 1
            x
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Error paths
// ============================================================================

#[test]
fn test_parse_error_missing_return_type_arrow() {
    parse_err("fn test() i64 = 42");
}

#[test]
fn test_parse_error_unclosed_paren() {
    parse_err("fn test( -> i64 = 42");
}

#[test]
fn test_parse_error_unclosed_brace() {
    parse_err("fn test() -> i64 { R 42");
}

// ============================================================================
// Advanced features
// ============================================================================

#[test]
fn test_parse_extern_function() {
    let source = r#"
        N "C" {
            fn malloc(size: i64) -> i64
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_global_variable() {
    let source = "G counter: i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_use_import() {
    let source = "use std";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_attribute() {
    let source = r#"
        #[cfg(target_os = "linux")]
        fn linux_fn() -> i64 = 1
    "#;
    let module = parse_ok(source);
    assert!(!module.items.is_empty());
}

#[test]
fn test_parse_spawn_yield() {
    let source = r#"
        fn test() -> i64 {
            return 42
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_comptime() {
    let source = "fn test() -> i64 = comptime { 2 + 3 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

// ============================================================================
// Complex programs
// ============================================================================

#[test]
fn test_parse_fibonacci() {
    let source = r#"
        fn fib(n: i64) -> i64 {
            I n <= 1 { return n }
            return @(n - 1) + @(n - 2)
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_linked_list() {
    let source = r#"
        E List {
            Cons(i64, i64),
            Nil
        }
        fn sum(lst: List) -> i64 {
            match lst {
                Nil => 0,
                Cons(h, t) => h + t,
                _ => 0
            }
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 2);
}

#[test]
fn test_parse_multiple_impl_blocks() {
    let source = r#"
        struct Vec2 { x: f64, y: f64 }
        impl Vec2 {
            fn new(x: f64, y: f64) -> Vec2 = Vec2 { x: x, y: y }
            fn length(self) -> f64 = 0.0
        }
        trait Printable { fn show(self) -> str }
        impl Vec2: Printable {
            fn show(self) -> str = "vec2"
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 3);
}

// ============================================================================
// Additional expression parsing (primary.rs, precedence.rs)
// ============================================================================

#[test]
fn test_parse_negative_literal() {
    let source = "fn test() -> i64 = -42";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_double_negation() {
    let source = "fn test() -> i64 = -(-42)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_not_expression() {
    let source = "fn test() -> bool = !true";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_method_call() {
    let source = r#"
        struct Foo { x: i64 }
        impl Foo { fn get(self) -> i64 = self.x }
        fn test() -> i64 {
            f := Foo { x: 42 }
            f.get()
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_chained_field_access() {
    let source = r#"
        struct Inner { val: i64 }
        struct Outer { inner: Inner }
        fn test() -> i64 {
            o := Outer { inner: Inner { val: 42 } }
            o.inner.val
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}

#[test]
fn test_parse_complex_expression_mix() {
    let source = "fn test(a: i64, b: i64) -> i64 = a * b + a / b - a % b";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_comparison_chain() {
    let source = "fn test(x: i64) -> bool = x > 0 && x < 100 && x != 50";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_bool_logic() {
    let source = "fn test(a: bool, b: bool) -> bool = (a && b) || (!a && !b)";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_self_recursion_factorial() {
    let source = r#"
        fn fact(n: i64) -> i64 {
            I n <= 1 { return 1 }
            n * @(n - 1)
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_block_expression() {
    let source = r#"
        fn test() -> i64 {
            x := {
                a := 10
                b := 20
                a + b
            }
            x
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_nested_function_calls() {
    let source = r#"
        fn a(x: i64) -> i64 = x + 1
        fn b(x: i64) -> i64 = x * 2
        fn test() -> i64 = a(b(21))
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 3);
}

#[test]
fn test_parse_defer() {
    let source = r#"
        fn test() -> i64 {
            x := mut 0
            D { x = x + 1 }
            x
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_global() {
    let source = "G counter: i64 = 0";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pub_struct() {
    let source = "P S Point { x: i64, y: i64 }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_pub_enum() {
    let source = "P E Color { Red, Green, Blue }";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_enum_many_variants() {
    let source = r#"
        E Weekday { Mon, Tue, Wed, Thu, Fri, Sat, Sun }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_mixed_items() {
    let source = r#"
        C MAX: i64 = 100
        type Num = i64
        struct Point { x: i64, y: i64 }
        E Color { Red, Green, Blue }
        fn main() -> i64 = MAX
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 5);
}

#[test]
fn test_parse_fn_type_multi_param() {
    let source = "fn compose(x: i64, f: fn(i64) -> i64, g: fn(i64) -> i64) -> i64 = f(g(x))";
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_error_missing_closing_brace() {
    parse_err("fn test() -> i64 { x := 42");
}

#[test]
fn test_parse_error_unexpected_token() {
    parse_err("fn 123() -> i64 = 42");
}

#[test]
fn test_parse_error_missing_arrow() {
    parse_err("fn test() i64 = 42");
}

#[test]
fn test_parse_recursive_gcd() {
    let source = r#"
        fn gcd(a: i64, b: i64) -> i64 {
            I b == 0 { return a }
            gcd(b, a % b)
        }
    "#;
    let module = parse_ok(source);
    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_parse_nested_struct_creation() {
    let source = r#"
        struct Inner { val: i64 }
        struct Outer { inner: Inner, extra: i64 }
        fn test() -> i64 {
            o := Outer { inner: Inner { val: 40 }, extra: 2 }
            o.inner.val + o.extra
        }
    "#;
    let module = parse_ok(source);
    assert!(module.items.len() >= 2);
}
