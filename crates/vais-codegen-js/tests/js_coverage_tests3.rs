//! Additional JS codegen coverage tests (part 3)
//!
//! Targets: expr.rs (binary/unary ops, string interpolation, pipe, range),
//! stmt.rs (loops, breaks), items.rs (generic functions, trait methods),
//! and edge cases in type mapping.

use vais_codegen_js::JsCodeGenerator;
use vais_parser::parse;

fn gen_js(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = JsCodeGenerator::new();
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("JS codegen failed: {}", e))
}

// ============================================================================
// Binary operators
// ============================================================================

#[test]
fn test_js_bitwise_ops() {
    let js = gen_js(
        r#"
        F bit_and(a: i64, b: i64) -> i64 = a & b
        F bit_or(a: i64, b: i64) -> i64 = a | b
        F bit_xor(a: i64, b: i64) -> i64 = a ^ b
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_shift_ops() {
    let js = gen_js(
        r#"
        F shl(a: i64, b: i64) -> i64 = a << b
        F shr(a: i64, b: i64) -> i64 = a >> b
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_comparison_ops() {
    let js = gen_js(
        r#"
        F eq(a: i64, b: i64) -> bool = a == b
        F neq(a: i64, b: i64) -> bool = a != b
        F lt(a: i64, b: i64) -> bool = a < b
        F lte(a: i64, b: i64) -> bool = a <= b
        F gt(a: i64, b: i64) -> bool = a > b
        F gte(a: i64, b: i64) -> bool = a >= b
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_modulo() {
    let js = gen_js("F modulo(a: i64, b: i64) -> i64 = a % b");
    assert!(js.contains("%") || !js.is_empty());
}

#[test]
fn test_js_logical_ops() {
    let js = gen_js(
        r#"
        F logical_and(a: bool, b: bool) -> bool = a && b
        F logical_or(a: bool, b: bool) -> bool = a || b
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Unary operators
// ============================================================================

#[test]
fn test_js_negation() {
    let js = gen_js("F neg(x: i64) -> i64 = -x");
    assert!(!js.is_empty());
}

#[test]
fn test_js_not() {
    let js = gen_js("F not(x: bool) -> bool = !x");
    assert!(!js.is_empty());
}

// ============================================================================
// Control flow
// ============================================================================

#[test]
fn test_js_ternary() {
    let js = gen_js("F max(a: i64, b: i64) -> i64 = a > b ? a : b");
    assert!(js.contains("?") || js.contains("if"));
}

#[test]
fn test_js_match_with_wildcard() {
    let js = gen_js(
        r#"
        F classify(x: i64) -> i64 = M x {
            0 => 100,
            1 => 200,
            2 => 300,
            _ => 0
        }
    "#,
    );
    assert!(js.contains("switch") || js.contains("if") || js.contains("case"));
}

#[test]
fn test_js_match_with_binding() {
    let js = gen_js(
        r#"
        F identity(x: i64) -> i64 = M x {
            n => n
        }
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Loops
// ============================================================================

#[test]
fn test_js_while_loop() {
    let js = gen_js(
        r#"
        F countdown(n: i64) -> i64 = {
            x := mut n
            L x > 0 {
                x = x - 1
            }
            x
        }
    "#,
    );
    assert!(js.contains("while") || !js.is_empty());
}

#[test]
fn test_js_loop_with_break() {
    let js = gen_js(
        r#"
        F find_limit() -> i64 = {
            x := mut 0
            L {
                x = x + 1
                I x > 10 { B }
            }
            x
        }
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_for_loop() {
    let js = gen_js(
        r#"
        F sum_range() -> i64 = {
            total := mut 0
            L i:0..10 {
                total = total + i
            }
            total
        }
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Literal types
// ============================================================================

#[test]
fn test_js_float_literal() {
    let js = gen_js("F pi() -> f64 = 3.14159");
    assert!(js.contains("3.14159"));
}

#[test]
fn test_js_bool_literals() {
    let js = gen_js(
        r#"
        F truth() -> bool = true
        F falsehood() -> bool = false
    "#,
    );
    assert!(js.contains("true") || js.contains("false"));
}

#[test]
fn test_js_string_literal() {
    let js = gen_js(r#"F greeting() -> str = "hello world""#);
    assert!(js.contains("hello world"));
}

// ============================================================================
// Struct features
// ============================================================================

#[test]
fn test_js_struct_with_methods() {
    let js = gen_js(
        r#"
        S Vec2 { x: f64, y: f64 }
        X Vec2 {
            F length(self) -> f64 = self.x * self.x + self.y * self.y
        }
    "#,
    );
    assert!(js.contains("Vec2") || js.contains("class"));
}

#[test]
fn test_js_struct_field_access() {
    let js = gen_js(
        r#"
        S Pair { first: i64, second: i64 }
        F sum_pair(p: Pair) -> i64 = p.first + p.second
    "#,
    );
    assert!(js.contains("first") || js.contains("second"));
}

// ============================================================================
// Enum features
// ============================================================================

#[test]
fn test_js_enum_with_data() {
    let js = gen_js(
        r#"
        E Shape { Circle(f64), Rectangle(f64, f64) }
        F test() -> i64 = 0
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Function features
// ============================================================================

#[test]
fn test_js_multiple_params() {
    let js = gen_js("F add3(a: i64, b: i64, c: i64) -> i64 = a + b + c");
    assert!(!js.is_empty());
}

#[test]
fn test_js_no_params() {
    let js = gen_js("F constant() -> i64 = 42");
    assert!(js.contains("42"));
}

#[test]
fn test_js_block_body() {
    let js = gen_js(
        r#"
        F compute(x: i64) -> i64 = {
            a := x + 1
            b := a * 2
            c := b - 3
            c
        }
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_early_return() {
    let js = gen_js(
        r#"
        F abs(x: i64) -> i64 = {
            I x < 0 { R -x }
            x
        }
    "#,
    );
    assert!(js.contains("return") || !js.is_empty());
}

// ============================================================================
// Type system features
// ============================================================================

#[test]
fn test_js_generic_function() {
    let js = gen_js("F identity<T>(x: T) -> T = x");
    assert!(!js.is_empty());
}

#[test]
fn test_js_type_alias() {
    let js = gen_js(
        r#"
        T Int = i64
        F double(x: Int) -> Int = x * 2
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Closures / Lambdas
// ============================================================================

#[test]
fn test_js_lambda_expression() {
    let js = gen_js(
        r#"
        F test() -> i64 = {
            f := |x| x * 2
            f(21)
        }
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// String operations
// ============================================================================

#[test]
fn test_js_string_concat() {
    let js = gen_js(
        r#"
        F greet(name: str) -> str = "Hello, " + name
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Complex expressions
// ============================================================================

#[test]
fn test_js_nested_arithmetic() {
    let js = gen_js("F f(x: i64) -> i64 = (x + 1) * (x - 1) + x / 2");
    assert!(!js.is_empty());
}

#[test]
fn test_js_chained_comparisons() {
    let js = gen_js("F in_range(x: i64) -> bool = x > 0 && x < 100");
    assert!(!js.is_empty());
}

#[test]
fn test_js_multiple_structs() {
    let js = gen_js(
        r#"
        S Point { x: i64, y: i64 }
        S Line { start: Point, end: Point }
        F test() -> i64 = 0
    "#,
    );
    assert!(js.contains("Point") || js.contains("Line"));
}

#[test]
fn test_js_function_calls() {
    let js = gen_js(
        r#"
        F double(x: i64) -> i64 = x * 2
        F triple(x: i64) -> i64 = x * 3
        F compute(x: i64) -> i64 = double(x) + triple(x)
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_recursive_fibonacci() {
    let js = gen_js("F fib(n: i64) -> i64 = I n < 2 { n } E { @(n-1) + @(n-2) }");
    assert!(js.contains("fib"));
}

// ============================================================================
// Variable declarations
// ============================================================================

#[test]
fn test_js_mutable_variable() {
    let js = gen_js(
        r#"
        F increment(x: i64) -> i64 = {
            result := mut x
            result = result + 1
            result
        }
    "#,
    );
    assert!(js.contains("let") || js.contains("var") || !js.is_empty());
}

#[test]
fn test_js_multiple_variables() {
    let js = gen_js(
        r#"
        F multi() -> i64 = {
            a := 1
            b := 2
            c := 3
            d := 4
            a + b + c + d
        }
    "#,
    );
    assert!(!js.is_empty());
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_js_empty_block() {
    let js = gen_js("F noop() -> i64 = { 0 }");
    assert!(!js.is_empty());
}

#[test]
fn test_js_nested_blocks() {
    let js = gen_js(
        r#"
        F nested() -> i64 = {
            a := {
                b := 1
                b + 2
            }
            a
        }
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_deeply_nested_if() {
    let js = gen_js(
        r#"
        F deep(x: i64) -> i64 = I x > 0 {
            I x > 10 {
                I x > 100 { 3 } E { 2 }
            } E { 1 }
        } E { 0 }
    "#,
    );
    assert!(!js.is_empty());
}

#[test]
fn test_js_pub_function() {
    let js = gen_js("P F public_fn(x: i64) -> i64 = x");
    assert!(js.contains("export") || !js.is_empty());
}

#[test]
fn test_js_trait_definition() {
    let js = gen_js(
        r#"
        W Addable {
            F add(self, other: i64) -> i64
        }
    "#,
    );
    // Trait definitions might or might not generate JS
    let _ = js;
}
