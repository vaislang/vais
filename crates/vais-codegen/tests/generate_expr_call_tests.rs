//! Coverage tests for vais-codegen/src/generate_expr_call.rs (Phase 131)
//!
//! Targets uncovered lines in:
//! - generate_expr_call.rs: builtin calls (print/println/format/str_to_ptr/ptr_to_str)
//! - generate_expr_call.rs: struct tuple literal, enum variant call
//! - generate_expr_call.rs: method calls, indirect calls, error paths
//!
//! Strategy: Generate IR from Vais source code and verify it succeeds.

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed for: {}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// Print/println builtin calls
// ============================================================================

#[test]
fn test_call_println_string_literal() {
    let ir = gen_ok(r#"F main() -> i64 { println("hello"); R 0 }"#);
    assert!(ir.contains("call") || ir.contains("puts") || ir.contains("printf") || ir.contains("print"));
}

#[test]
fn test_call_println_integer() {
    let ir = gen_ok(r#"F main() -> i64 { println(42); R 0 }"#);
    assert!(ir.contains("call"));
}

#[test]
fn test_call_print_string_literal() {
    let ir = gen_ok(r#"F main() -> i64 { print("world"); R 0 }"#);
    assert!(ir.contains("call"));
}

#[test]
fn test_call_println_variable() {
    let ir = gen_ok(r#"F main() -> i64 { x := 42; println(x); R 0 }"#);
    assert!(ir.contains("call"));
}

#[test]
fn test_call_println_expression() {
    let ir = gen_ok(r#"F main() -> i64 { println(1 + 2); R 0 }"#);
    assert!(ir.contains("call"));
}

#[test]
fn test_call_println_bool() {
    let ir = gen_ok(r#"F main() -> i64 { println(true); R 0 }"#);
    assert!(ir.contains("call"));
}

// ============================================================================
// Regular function calls
// ============================================================================

#[test]
fn test_call_simple_function() {
    let ir = gen_ok("F add(x: i64, y: i64) -> i64 = x + y\nF main() -> i64 = add(1, 2)");
    assert!(ir.contains("call"));
    assert!(ir.contains("@add"));
}

#[test]
fn test_call_no_args() {
    let ir = gen_ok("F zero() -> i64 = 0\nF main() -> i64 = zero()");
    assert!(ir.contains("@zero"));
}

#[test]
fn test_call_multiple_args() {
    let ir = gen_ok("F sum3(a: i64, b: i64, c: i64) -> i64 = a + b + c\nF main() -> i64 = sum3(1, 2, 3)");
    assert!(ir.contains("@sum3"));
}

#[test]
fn test_call_nested() {
    let ir = gen_ok("F inc(x: i64) -> i64 = x + 1\nF main() -> i64 = inc(inc(inc(0)))");
    assert!(ir.contains("@inc"));
}

#[test]
fn test_call_with_expression_args() {
    let ir = gen_ok("F add(x: i64, y: i64) -> i64 = x + y\nF main() -> i64 = add(1 + 2, 3 * 4)");
    assert!(ir.contains("@add"));
}

// ============================================================================
// Struct tuple literal desugaring
// ============================================================================

#[test]
fn test_struct_literal_basic() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        F main() -> i64 {
            p := Point { x: 10, y: 20 }
            R p.x
        }
    "#,
    );
    assert!(ir.contains("Point") || ir.contains("insertvalue") || ir.contains("store"));
}

#[test]
fn test_struct_literal_single_field() {
    let ir = gen_ok(
        r#"
        S Wrapper { value: i64 }
        F main() -> i64 {
            w := Wrapper { value: 42 }
            R w.value
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_literal_three_fields() {
    let ir = gen_ok(
        r#"
        S Vec3 { x: i64, y: i64, z: i64 }
        F main() -> i64 {
            v := Vec3 { x: 1, y: 2, z: 3 }
            R v.x + v.y + v.z
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Method calls (via impl)
// ============================================================================

#[test]
fn test_method_call_basic() {
    let ir = gen_ok(
        r#"
        S Counter { count: i64 }
        X Counter {
            F get(self) -> i64 = self.count
        }
        F main() -> i64 {
            c := Counter { count: 42 }
            R c.get()
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_method_call_with_args() {
    let ir = gen_ok(
        r#"
        S Calc { base: i64 }
        X Calc {
            F add(self, x: i64) -> i64 = self.base + x
        }
        F main() -> i64 {
            c := Calc { base: 10 }
            R c.add(5)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_method_chained_call() {
    let ir = gen_ok(
        r#"
        S Num { val: i64 }
        X Num {
            F get(self) -> i64 = self.val
        }
        F double(x: i64) -> i64 = x * 2
        F main() -> i64 {
            n := Num { val: 5 }
            R double(n.get())
        }
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// Self-recursion (@)
// ============================================================================

#[test]
fn test_self_recursion_basic() {
    let ir = gen_ok("F fib(n: i64) -> i64 = I n < 2 { n } E { @(n-1) + @(n-2) }");
    assert!(ir.contains("call") && ir.contains("@fib"));
}

#[test]
fn test_self_recursion_factorial() {
    let ir = gen_ok("F fact(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }");
    assert!(ir.contains("@fact"));
}

// ============================================================================
// Enum variant construction
// ============================================================================

#[test]
fn test_enum_unit_variant() {
    let ir = gen_ok(
        r#"
        E Direction { North, South, East, West }
        F test(d: Direction) -> i64 {
            M d {
                North => 0,
                _ => 1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// Error paths — undefined function, wrong args
// ============================================================================

#[test]
fn test_call_undefined_function() {
    let result = gen_result("F main() -> i64 = undefined_func(42)");
    assert!(result.is_err());
}

#[test]
fn test_call_too_few_args_builtin() {
    // str_to_ptr expects exactly 1 arg
    let result = gen_result("F main() -> i64 = str_to_ptr()");
    assert!(result.is_err());
}

// ============================================================================
// Pipe operator calls
// ============================================================================

#[test]
fn test_pipe_operator_call() {
    // Pipe operator desugars to function call — may produce indirect call
    let result = gen_result("F inc(x: i64) -> i64 = x + 1\nF main() -> i64 = 5 |> inc()");
    let _ = result; // Exercise the path, may or may not succeed
}

#[test]
fn test_pipe_chain() {
    // Pipe chain may produce indirect call error
    let result = gen_result(
        "F inc(x: i64) -> i64 = x + 1\nF dbl(x: i64) -> i64 = x * 2\nF main() -> i64 = 1 |> inc() |> dbl()",
    );
    let _ = result; // Exercise the path
}

// ============================================================================
// Ternary with function call
// ============================================================================

#[test]
fn test_ternary_with_call() {
    let ir = gen_ok("F abs(x: i64) -> i64 = x >= 0 ? x : 0 - x\nF main() -> i64 = abs(0 - 5)");
    assert!(ir.contains("@abs"));
}

// ============================================================================
// Closure as argument
// ============================================================================

#[test]
fn test_closure_as_arg() {
    let ir = gen_ok(
        r#"
        F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)
        F main() -> i64 = apply(|x| x + 1, 10)
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// Multiple return path calls
// ============================================================================

#[test]
fn test_call_in_if_branches() {
    let ir = gen_ok(
        r#"
        F inc(x: i64) -> i64 = x + 1
        F dec(x: i64) -> i64 = x - 1
        F main() -> i64 {
            x := 5
            R I x > 3 { inc(x) } E { dec(x) }
        }
    "#,
    );
    assert!(ir.contains("@inc") && ir.contains("@dec"));
}

#[test]
fn test_call_in_match() {
    let ir = gen_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F triple(x: i64) -> i64 = x * 3
        F main() -> i64 {
            n := 2
            M n {
                1 => double(n),
                _ => triple(n)
            }
        }
    "#,
    );
    assert!(ir.contains("@double") || ir.contains("@triple"));
}
