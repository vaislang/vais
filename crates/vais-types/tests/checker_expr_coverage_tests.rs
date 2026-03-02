//! Coverage tests for vais-types/src/checker_expr/
//!
//! Targets: calls.rs, collections.rs, control_flow.rs, special.rs, stmts.rs
//! Expression type checking edge cases and error paths.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

fn check_err(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for: {}",
        source
    );
}

// ============================================================================
// Call expression type checking (calls.rs)
// ============================================================================

#[test]
fn test_check_call_simple() {
    check_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F test() -> i64 = add(1, 2)
    "#,
    );
}

#[test]
fn test_check_call_no_args() {
    check_ok(
        r#"
        F answer() -> i64 = 42
        F test() -> i64 = answer()
    "#,
    );
}

#[test]
fn test_check_call_single_arg() {
    check_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = double(21)
    "#,
    );
}

#[test]
fn test_check_method_call() {
    check_ok(
        r#"
        S Point { x: i64, y: i64 }
        X Point {
            F sum(self) -> i64 = self.x + self.y
        }
        F test() -> i64 {
            p := Point { x: 3, y: 4 }
            p.sum()
        }
    "#,
    );
}

#[test]
fn test_check_method_call_with_args() {
    check_ok(
        r#"
        S Calc { base: i64 }
        X Calc {
            F add(self, n: i64) -> i64 = self.base + n
        }
        F test() -> i64 {
            c := Calc { base: 10 }
            c.add(5)
        }
    "#,
    );
}

#[test]
fn test_check_chained_calls() {
    check_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = double(inc(20))
    "#,
    );
}

#[test]
fn test_check_builtin_print() {
    check_ok(
        r#"
        F test() -> i64 {
            print("hello")
            0
        }
    "#,
    );
}

#[test]
fn test_check_builtin_println() {
    check_ok(
        r#"
        F test() -> i64 {
            println("hello")
            0
        }
    "#,
    );
}

// ============================================================================
// Collection expressions (collections.rs)
// ============================================================================

#[test]
fn test_check_array_literal_i64() {
    check_ok(
        r#"
        F test() -> i64 {
            arr := [1, 2, 3]
            arr[0]
        }
    "#,
    );
}

#[test]
fn test_check_array_literal_bool() {
    check_ok(
        r#"
        F test() -> bool {
            arr := [true, false, true]
            arr[0]
        }
    "#,
    );
}

#[test]
fn test_check_array_index() {
    check_ok(
        r#"
        F test() -> i64 {
            arr := [10, 20, 30]
            arr[1]
        }
    "#,
    );
}

#[test]
fn test_check_tuple_literal() {
    check_ok(
        r#"
        F test() -> i64 {
            t := (1, true, 3)
            0
        }
    "#,
    );
}

#[test]
fn test_check_struct_literal() {
    check_ok(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            p.x + p.y
        }
    "#,
    );
}

#[test]
fn test_check_struct_field_access() {
    check_ok(
        r#"
        S Pair { first: i64, second: str }
        F test() -> i64 {
            p := Pair { first: 42, second: "hello" }
            p.first
        }
    "#,
    );
}

// ============================================================================
// Control flow expressions (control_flow.rs)
// ============================================================================

#[test]
fn test_check_if_expression() {
    check_ok("F f(x: i64) -> i64 = I x > 0 { 1 } E { 0 }");
}

#[test]
fn test_check_if_without_else() {
    check_ok(
        r#"
        F f(x: i64) -> i64 {
            I x > 0 { R 1 }
            0
        }
    "#,
    );
}

#[test]
fn test_check_if_else_if() {
    check_ok(
        r#"
        F classify(x: i64) -> str {
            I x > 0 { "positive" }
            E I x < 0 { "negative" }
            E { "zero" }
        }
    "#,
    );
}

#[test]
fn test_check_ternary() {
    check_ok("F max(a: i64, b: i64) -> i64 = a > b ? a : b");
}

#[test]
fn test_check_match_int() {
    check_ok(
        r#"
        F f(x: i64) -> str = M x {
            0 => "zero",
            1 => "one",
            _ => "other"
        }
    "#,
    );
}

#[test]
fn test_check_match_bool() {
    check_ok(
        r#"
        F f(b: bool) -> i64 = M b {
            true => 1,
            false => 0
        }
    "#,
    );
}

#[test]
fn test_check_for_loop() {
    check_ok(
        r#"
        F sum() -> i64 {
            total := mut 0
            L i:0..10 {
                total = total + i
            }
            total
        }
    "#,
    );
}

#[test]
fn test_check_while_loop() {
    check_ok(
        r#"
        F countdown(n: i64) -> i64 {
            x := mut n
            L x > 0 {
                x = x - 1
            }
            x
        }
    "#,
    );
}

#[test]
fn test_check_loop_break() {
    check_ok(
        r#"
        F f() -> i64 {
            L i:0..100 {
                I i == 10 { B }
            }
            0
        }
    "#,
    );
}

#[test]
fn test_check_loop_continue() {
    check_ok(
        r#"
        F f() -> i64 {
            total := mut 0
            L i:0..10 {
                I i == 5 { C }
                total = total + i
            }
            total
        }
    "#,
    );
}

// ============================================================================
// Special expressions (special.rs)
// ============================================================================

#[test]
fn test_check_block_expression() {
    check_ok(
        r#"
        F f() -> i64 {
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

#[test]
fn test_check_nested_block() {
    check_ok(
        r#"
        F f() -> i64 {
            x := {
                a := {
                    b := 42
                    b
                }
                a + 1
            }
            x
        }
    "#,
    );
}

#[test]
fn test_check_pipe_operator() {
    check_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = 5 |> double
    "#,
    );
}

#[test]
fn test_check_self_recursion() {
    check_ok(
        r#"
        F fib(n: i64) -> i64 = I n < 2 { n } E { @(n-1) + @(n-2) }
    "#,
    );
}

// ============================================================================
// Statement type checking (stmts.rs)
// ============================================================================

#[test]
fn test_check_let_inferred() {
    check_ok(
        r#"
        F f() -> i64 {
            x := 42
            x
        }
    "#,
    );
}

#[test]
fn test_check_let_annotated() {
    check_ok(
        r#"
        F f() -> i64 {
            x: i64 = 42
            x
        }
    "#,
    );
}

#[test]
fn test_check_assignment() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
}

#[test]
fn test_check_compound_assignment() {
    check_ok(
        r#"
        F f() -> i64 {
            x := mut 10
            x += 5
            x -= 3
            x *= 2
            x
        }
    "#,
    );
}

#[test]
fn test_check_return_expr() {
    check_ok(
        r#"
        F f() -> i64 {
            R 42
        }
    "#,
    );
}

#[test]
fn test_check_return_void() {
    check_ok("F f() { R }");
}

#[test]
fn test_check_defer() {
    check_ok(
        r#"
        F f() -> i64 {
            D { 0 }
            42
        }
    "#,
    );
}

// ============================================================================
// Binary expression variety
// ============================================================================

#[test]
fn test_check_arithmetic_ops() {
    check_ok("F f(a: i64, b: i64) -> i64 = a + b - a * b / a");
}

#[test]
fn test_check_modulo() {
    check_ok("F f(a: i64, b: i64) -> i64 = a % b");
}

#[test]
fn test_check_bitwise_ops() {
    check_ok(
        r#"
        F f(a: i64, b: i64) -> i64 {
            c := a & b
            d := c | b
            e := d ^ a
            f := e << 2
            g := f >> 1
            g
        }
    "#,
    );
}

#[test]
fn test_check_comparison_ops() {
    check_ok(
        r#"
        F f(a: i64, b: i64) -> bool {
            c := a == b
            d := a != b
            e := a < b
            f := a <= b
            g := a > b
            h := a >= b
            c && d && e && f && g && h
        }
    "#,
    );
}

#[test]
fn test_check_logical_and_or() {
    check_ok(
        r#"
        F f(a: bool, b: bool) -> bool = (a && b) || (!a && !b)
    "#,
    );
}

// ============================================================================
// Unary expressions
// ============================================================================

#[test]
fn test_check_unary_neg() {
    check_ok("F f(x: i64) -> i64 = -x");
}

#[test]
fn test_check_unary_not() {
    check_ok("F f(x: bool) -> bool = !x");
}

// ============================================================================
// String operations
// ============================================================================

#[test]
fn test_check_string_eq() {
    check_ok(
        r#"
        F f() -> bool {
            a := "hello"
            b := "hello"
            a == b
        }
    "#,
    );
}

#[test]
fn test_check_string_concat() {
    check_ok(
        r#"
        F f() -> str {
            a := "hello"
            b := " world"
            a + b
        }
    "#,
    );
}

// ============================================================================
// Complex expression scenarios
// ============================================================================

#[test]
fn test_check_nested_function_calls() {
    check_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F double(x: i64) -> i64 = x * 2
        F triple(x: i64) -> i64 = x * 3
        F test() -> i64 = triple(double(inc(0)))
    "#,
    );
}

#[test]
fn test_check_complex_match_with_binding() {
    check_ok(
        r#"
        F f(x: i64) -> i64 = M x {
            0 => 100,
            n => n * 2
        }
    "#,
    );
}

#[test]
fn test_check_deeply_nested_expression() {
    check_ok("F f(a: i64, b: i64) -> i64 = ((a + b) * (a - b)) + ((a * b) / (a + 1))");
}
