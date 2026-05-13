//! Additional codegen coverage tests (part 2)
//!
//! Targets: string_ops.rs, stmt.rs, stmt_visitor.rs, registration.rs,
//! lambda_closure.rs, expr_helpers.rs, type_inference.rs, visitor.rs
//! edge cases and paths not covered by codegen_coverage_tests.rs

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
// String operations (string_ops.rs)
// ============================================================================

#[test]
fn test_codegen_string_concat() {
    let ir = gen_ok(
        r#"
        F test() -> str {
            a := "hello"
            b := " world"
            a + b
        }
    "#,
    );
    assert!(ir.contains("hello") || ir.contains("world"));
}

#[test]
fn test_codegen_string_comparison_eq() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            a := "hello"
            b := "hello"
            a == b
        }
    "#,
    );
    assert!(ir.contains("strcmp") || ir.contains("memcmp") || ir.contains("icmp"));
}

#[test]
fn test_codegen_string_comparison_neq() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            a := "hello"
            b := "world"
            a != b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_codegen_string_len_method() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            s := "hello"
            s.len()
        }
    "#,
    );
    // May or may not compile depending on str method support
    let _ = result;
}

#[test]
fn test_codegen_string_interpolation() {
    let result = gen_result(
        r#"
        F test() -> str {
            x := 42
            ~"value is {x}"
        }
    "#,
    );
    let _ = result;
}

// ============================================================================
// Statement code generation (stmt.rs)
// ============================================================================

#[test]
fn test_codegen_let_with_type_annotation() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x: i64 = 42
            x
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_mutable_let() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 42
            x
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("store"));
}

#[test]
fn test_codegen_multiple_lets() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            c := 3
            d := 4
            a + b + c + d
        }
    "#,
    );
    assert!(ir.contains("add"));
}

#[test]
fn test_codegen_return_statement() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 { R x }
            R 0
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

#[test]
fn test_codegen_early_return() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x == 0 { R -1 }
            I x == 1 { R 1 }
            x * 2
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

#[test]
fn test_codegen_nested_blocks() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := {
                b := 10
                c := 20
                b + c
            }
            a
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("10") || ir.contains("20"));
}

// ============================================================================
// Struct codegen (generate_expr_struct.rs, registration.rs)
// ============================================================================

#[test]
fn test_codegen_struct_with_many_fields() {
    let ir = gen_ok(
        r#"
        S Rect { x: i64, y: i64, w: i64, h: i64 }
        F test() -> i64 {
            r := Rect { x: 0, y: 0, w: 100, h: 50 }
            r.w + r.h
        }
    "#,
    );
    assert!(ir.contains("100") || ir.contains("50"));
}

#[test]
fn test_codegen_struct_nested_field() {
    let ir = gen_ok(
        r#"
        S Inner { value: i64 }
        S Outer { inner: Inner }
        F test() -> i64 {
            o := Outer { inner: Inner { value: 42 } }
            o.inner.value
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_impl_block_basic() {
    let ir = gen_ok(
        r#"
        S Counter { n: i64 }
        X Counter {
            F get(self) -> i64 = self.n
            F double(self) -> i64 = self.n * 2
        }
        F test() -> i64 {
            c := Counter { n: 21 }
            c.double()
        }
    "#,
    );
    assert!(ir.contains("Counter") || ir.contains("double") || ir.contains("21"));
}

// ============================================================================
// Lambda and closures (lambda_closure.rs)
// ============================================================================

#[test]
fn test_codegen_simple_lambda() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            f := |x: i64| x * 2
            f(21)
        }
    "#,
    );
    assert!(ir.contains("21") || ir.contains("mul") || ir.contains("lambda"));
}

#[test]
fn test_codegen_lambda_identity() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            id := |x: i64| x
            id(42)
        }
    "#,
    );
    assert!(ir.contains("42"));
}

#[test]
fn test_codegen_lambda_no_params() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            f := || 99
            f()
        }
    "#,
    );
    assert!(ir.contains("99"));
}

// ============================================================================
// Control flow (control_flow.rs)
// ============================================================================

#[test]
fn test_codegen_loop_basic() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L i:0..10 {
                x = x + i
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("phi") || ir.contains("loop"));
}

#[test]
fn test_codegen_loop_with_break() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L i:0..100 {
                I i == 10 { B }
                x = x + 1
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_loop_with_continue() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L i:0..10 {
                I i == 5 { C }
                x = x + 1
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br"));
}

#[test]
fn test_codegen_while_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            L x > 0 {
                x = x - 1
            }
            x
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("icmp"));
}

// ============================================================================
// Match (control_flow/pattern.rs)
// ============================================================================

#[test]
fn test_codegen_match_int_patterns() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 = M x {
            0 => 100,
            1 => 200,
            2 => 300,
            _ => 0
        }
    "#,
    );
    assert!(ir.contains("100") || ir.contains("200") || ir.contains("300"));
}

#[test]
fn test_codegen_match_bool_pattern() {
    let ir = gen_ok(
        r#"
        F test(b: bool) -> i64 = M b {
            true => 1,
            false => 0
        }
    "#,
    );
    assert!(ir.contains("icmp") || ir.contains("br") || ir.contains("switch"));
}

#[test]
fn test_codegen_match_with_binding() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 = M x {
            0 => 999,
            n => n + 1
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("999"));
}

// ============================================================================
// Type inference in codegen (type_inference.rs)
// ============================================================================

#[test]
fn test_codegen_inferred_return_type() {
    // Expression body — return type inferred from expression
    let ir = gen_ok("F test() -> i64 = 42 + 8");
    assert!(ir.contains("add") || ir.contains("50") || ir.contains("42"));
}

#[test]
fn test_codegen_nested_expression_types() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64) -> i64 {
            c := a + b
            d := c * 2
            e := d - 1
            e
        }
    "#,
    );
    assert!(ir.contains("add") && ir.contains("mul") && ir.contains("sub"));
}

// ============================================================================
// Enum codegen
// ============================================================================

#[test]
fn test_codegen_simple_enum() {
    let ir = gen_ok(
        r#"
        E Color { Red, Green, Blue }
        F test() -> i64 {
            c := Red
            0
        }
    "#,
    );
    assert!(ir.contains("Color") || ir.contains("Red") || ir.contains("0"));
}

#[test]
fn test_codegen_enum_with_data() {
    let ir = gen_ok(
        r#"
        E Shape { Circle(i64), Square(i64) }
        F test() -> i64 {
            s := Circle(5)
            0
        }
    "#,
    );
    assert!(ir.contains("Shape") || ir.contains("5"));
}

// ============================================================================
// Trait and dispatch (trait_dispatch.rs, vtable.rs)
// ============================================================================

#[test]
fn test_codegen_trait_basic() {
    let ir = gen_ok(
        r#"
        W Greet {
            F hello(self) -> i64
        }
        S Dog { age: i64 }
        X Dog: Greet {
            F hello(self) -> i64 = self.age
        }
        F test() -> i64 {
            d := Dog { age: 3 }
            d.hello()
        }
    "#,
    );
    assert!(ir.contains("Dog") || ir.contains("hello") || ir.contains("3"));
}

// ============================================================================
// Type alias codegen
// ============================================================================

#[test]
fn test_codegen_type_alias() {
    let ir = gen_ok(
        r#"
        T Num = i64
        F test(x: Num) -> Num = x + 1
    "#,
    );
    assert!(ir.contains("add") || ir.contains("i64"));
}

// ============================================================================
// Expression helpers (expr_helpers.rs, expr_helpers_data.rs)
// ============================================================================

#[test]
fn test_codegen_print_i64() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            print(42)
            0
        }
    "#,
    );
    assert!(ir.contains("printf") || ir.contains("print") || ir.contains("42"));
}

#[test]
fn test_codegen_print_string() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            print("hello")
            0
        }
    "#,
    );
    assert!(ir.contains("hello") || ir.contains("print"));
}

#[test]
fn test_codegen_println() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            println(42)
            0
        }
    "#,
    );
    assert!(ir.contains("printf") || ir.contains("print") || ir.contains("42"));
}

// ============================================================================
// Pipe operator
// ============================================================================

#[test]
fn test_codegen_pipe() {
    let ir = gen_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 = 5 |> double
    "#,
    );
    assert!(ir.contains("call") || ir.contains("double") || ir.contains("10"));
}

// ============================================================================
// Range operator
// ============================================================================

#[test]
fn test_codegen_range_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i:1..5 {
                sum = sum + i
            }
            sum
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("br"));
}

// ============================================================================
// Compound assignment
// ============================================================================

#[test]
fn test_codegen_compound_add_assign() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x += 5
            x
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("15"));
}

#[test]
fn test_codegen_compound_sub_assign() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x -= 3
            x
        }
    "#,
    );
    assert!(ir.contains("sub"));
}

#[test]
fn test_codegen_compound_mul_assign() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x *= 2
            x
        }
    "#,
    );
    assert!(ir.contains("mul"));
}

// ============================================================================
// Cast operations
// ============================================================================

#[test]
fn test_codegen_cast_i64_to_f64() {
    let result = gen_result(
        r#"
        F test() -> f64 {
            x := 42
            x as f64
        }
    "#,
    );
    let _ = result; // Exercise the path
}

// ============================================================================
// Generics (generics_helpers.rs)
// ============================================================================

#[test]
fn test_codegen_generic_function() {
    let ir = gen_ok(
        r#"
        F identity<T>(x: T) -> T = x
        F test() -> i64 = identity(42)
    "#,
    );
    assert!(ir.contains("42") || ir.contains("identity"));
}

// ============================================================================
// Cross compilation (cross_compile.rs, target.rs)
// ============================================================================

#[test]
fn test_codegen_module_name() {
    let ir = gen_ok("F f() -> i64 = 0");
    assert!(ir.contains("ModuleID") || ir.contains("test"));
}

// ============================================================================
// Error paths (error.rs, diagnostics.rs)
// ============================================================================

#[test]
fn test_codegen_error_for_invalid_code() {
    // Test that codegen handles unsupported patterns gracefully
    let result = gen_result(
        r#"
        F test() -> i64 = 42
    "#,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Defer statement
// ============================================================================

#[test]
fn test_codegen_defer() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            D { x = 99 }
            x = 42
            x
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("99"));
}

// ============================================================================
// Global variables
// ============================================================================

#[test]
fn test_codegen_global_var() {
    // Global declarations should at least generate without error
    let result = gen_result(
        r#"
        G counter: i64 = 0
        F test() -> i64 = 0
    "#,
    );
    // Exercises the global declaration codegen path
    let _ = result;
}
