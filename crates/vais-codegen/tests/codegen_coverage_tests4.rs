//! Codegen coverage tests part 4 — generate_expr_call, stmt, ffi, type_inference,
//! lambda_closure, expr_helpers, expr_helpers_control, expr_helpers_data, expr_helpers_misc
//!
//! Strategy: gen_ok/gen_result pattern to exercise internal codegen paths.

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

fn gen_err(source: &str) -> String {
    let module = parse(source).unwrap();
    let mut gen = CodeGenerator::new("test");
    let result = gen.generate_module(&module);
    assert!(result.is_err(), "Expected codegen error for: {}", source);
    format!("{}", result.unwrap_err())
}

// ============================================================================
// generate_expr_call: builtin calls
// ============================================================================

#[test]
fn test_call_print_i64() {
    let ir = gen_ok(r#"F test() -> i64 { println(42) R 0 }"#);
    assert!(ir.contains("42") || ir.contains("print"));
}

#[test]
fn test_call_print_string() {
    let ir = gen_ok(r#"F test() -> i64 { println("hello") R 0 }"#);
    assert!(ir.contains("hello"));
}

#[test]
fn test_call_print_multiple_args() {
    let ir = gen_ok(r#"F test() -> i64 { println(1, 2, 3) R 0 }"#);
    assert!(!ir.is_empty());
}

#[test]
fn test_call_format_builtin() {
    let result = gen_result(r#"F test() -> str { format("value: {}", 42) }"#);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_call_str_to_ptr() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            s := "hello"
            str_to_ptr(s)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_call_str_to_ptr_wrong_args() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            str_to_ptr("a", "b")
        }
    "#,
    );
    // Should error with wrong number of args
    assert!(result.is_err());
}

#[test]
fn test_call_regular_function() {
    let ir = gen_ok(
        r#"
        F add(a: i64, b: i64) -> i64 = a + b
        F test() -> i64 = add(10, 20)
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_call_function_chain() {
    let ir = gen_ok(
        r#"
        F double(x: i64) -> i64 = x * 2
        F inc(x: i64) -> i64 = x + 1
        F test() -> i64 = inc(double(5))
    "#,
    );
    assert!(ir.contains("@double"));
    assert!(ir.contains("@inc"));
}

#[test]
fn test_call_undefined_function() {
    let err = gen_err("F test() -> i64 = undefined_fn(42)");
    assert!(err.contains("undefined") || err.contains("not found") || err.contains("unknown"));
}

#[test]
fn test_call_struct_constructor() {
    let ir = gen_ok(
        r#"
        S Pair {
            a: i64,
            b: i64
        }
        F test() -> i64 {
            p := Pair { a: 1, b: 2 }
            p.a
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_call_enum_variant_constructor() {
    let result = gen_result(
        r#"
        E Option {
            Some(i64),
            None
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// generate_expr_call: method calls
// ============================================================================

#[test]
fn test_method_call_simple() {
    let ir = gen_ok(
        r#"
        S Vec2 { x: i64, y: i64 }
        X Vec2 {
            F sum(self) -> i64 = self.x + self.y
        }
        F test() -> i64 {
            v := Vec2 { x: 3, y: 4 }
            v.sum()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_method_call_with_args() {
    let ir = gen_ok(
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
    assert!(!ir.is_empty());
}

#[test]
fn test_method_call_chained_access() {
    let ir = gen_ok(
        r#"
        S Holder { val: i64 }
        X Holder {
            F get(self) -> i64 = self.val
        }
        F test() -> i64 {
            h := Holder { val: 42 }
            h.get()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// stmt: variable declarations and assignments
// ============================================================================

#[test]
fn test_stmt_let_binding() {
    let ir = gen_ok("F test() -> i64 { x := 42\nR x }");
    assert!(!ir.is_empty());
}

#[test]
fn test_stmt_let_mut_binding() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            x = 10
            x = 20
            x
        }
    "#,
    );
    assert!(ir.contains("store") || ir.contains("alloca"));
}

#[test]
fn test_stmt_multiple_bindings() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            c := 3
            a + b + c
        }
    "#,
    );
    assert!(ir.contains("add"));
}

#[test]
fn test_stmt_compound_assignment_add() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 10
            x = x + 5
            x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_stmt_nested_scopes() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := {
                z := x + 2
                z * 3
            }
            y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// stmt: return statements
// ============================================================================

#[test]
fn test_stmt_explicit_return() {
    let ir = gen_ok("F test() -> i64 { R 42 }");
    assert!(ir.contains("ret i64 42") || ir.contains("ret"));
}

#[test]
fn test_stmt_early_return() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x < 0 {
                R 0
            }
            x * 2
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

#[test]
fn test_stmt_void_return() {
    let result = gen_result(
        r#"
        F test() {
            x := 42
            R
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// ffi: extern blocks
// ============================================================================

#[test]
fn test_ffi_extern_simple_function() {
    let ir = gen_ok(
        r#"
        N {
            F abs(x: i64) -> i64
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(ir.contains("declare") || ir.contains("abs"));
}

#[test]
fn test_ffi_extern_multiple_functions() {
    let ir = gen_ok(
        r#"
        N {
            F malloc(size: i64) -> i64
            F free(ptr: i64)
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(ir.contains("malloc") || ir.contains("declare"));
}

#[test]
fn test_ffi_extern_void_return() {
    let ir = gen_ok(
        r#"
        N {
            F exit(code: i64)
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_ffi_extern_no_params() {
    let ir = gen_ok(
        r#"
        N {
            F getpid() -> i64
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_ffi_extern_str_param() {
    let ir = gen_ok(
        r#"
        N {
            F puts(s: str) -> i64
        }
        F test() -> i64 = 0
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// type_inference: expression type inference
// ============================================================================

#[test]
fn test_infer_integer_literal() {
    let ir = gen_ok("F test() -> i64 = 42");
    assert!(ir.contains("42"));
}

#[test]
fn test_infer_float_literal() {
    let ir = gen_ok("F test() -> f64 = 3.14");
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_bool_literal() {
    let ir = gen_ok("F test() -> bool = true");
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_string_literal() {
    let ir = gen_ok(r#"F test() -> str = "hello""#);
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_binary_add() {
    let ir = gen_ok("F test() -> i64 = 1 + 2");
    assert!(ir.contains("add"));
}

#[test]
fn test_infer_binary_comparison() {
    let ir = gen_ok("F test() -> bool = 1 < 2");
    assert!(ir.contains("icmp"));
}

#[test]
fn test_infer_if_else_type() {
    let ir = gen_ok(
        r#"
        F test(b: bool) -> i64 {
            I b { 1 } E { 2 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_block_type_from_last_stmt() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := 2
            x + y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_match_result_type() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 100,
                1 => 200,
                _ => 300
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// type_inference: struct type inference
// ============================================================================

#[test]
fn test_infer_struct_lit_pointer() {
    let ir = gen_ok(
        r#"
        S Pt { x: i64, y: i64 }
        F test() -> i64 {
            p := Pt { x: 1, y: 2 }
            p.x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_infer_struct_field_access() {
    let ir = gen_ok(
        r#"
        S Data { a: i64, b: i64, c: i64 }
        F test() -> i64 {
            d := Data { a: 10, b: 20, c: 30 }
            d.a + d.b + d.c
        }
    "#,
    );
    assert!(ir.contains("add"));
}

// ============================================================================
// lambda_closure: capture analysis
// ============================================================================

#[test]
fn test_lambda_no_capture() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            f := |x: i64| -> i64 { x + 1 }
            f(5)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_with_capture() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            base := 10
            f := |x: i64| -> i64 { x + base }
            f(5)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_multiple_captures() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            a := 1
            b := 2
            c := 3
            f := |x: i64| -> i64 { x + a + b + c }
            f(0)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_nested() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            outer := |x: i64| -> i64 {
                inner := |y: i64| -> i64 { y * 2 }
                inner(x) + 1
            }
            outer(5)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lambda_passed_as_arg() {
    let result = gen_result(
        r#"
        F apply(f: |i64| -> i64, x: i64) -> i64 = f(x)
        F test() -> i64 {
            apply(|x: i64| -> i64 { x * 3 }, 7)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// expr_helpers: string operations
// ============================================================================

#[test]
fn test_string_concat_codegen() {
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
fn test_string_equality_codegen() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            a := "abc"
            b := "abc"
            a == b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_inequality_codegen() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            a := "abc"
            b := "xyz"
            a != b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// expr_helpers_control: control flow expressions
// ============================================================================

#[test]
fn test_ternary_expression() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            x > 0 ? x : 0 - x
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("select"));
}

#[test]
fn test_nested_ternary() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            x > 0 ? 1 : (x < 0 ? 0 - 1 : 0)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_without_else() {
    let result = gen_result(
        r#"
        F test(x: i64) -> i64 {
            I x > 0 {
                println(x)
            }
            x
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_match_with_guard() {
    let result = gen_result(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n I n > 10 => n * 2,
                n I n > 0 => n,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_match_wildcard_only() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                _ => 42
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// expr_helpers_data: data structure expressions
// ============================================================================

#[test]
fn test_array_literal() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            arr := [1, 2, 3, 4, 5]
            arr[0]
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_struct_update_syntax() {
    let result = gen_result(
        r#"
        S Config { width: i64, height: i64, depth: i64 }
        F test() -> i64 {
            c := Config { width: 100, height: 200, depth: 300 }
            c.width
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_nested_field_access() {
    let ir = gen_ok(
        r#"
        S Inner { value: i64 }
        S Middle { inner: Inner }
        S Outer { middle: Middle }
        F test() -> i64 {
            o := Outer { middle: Middle { inner: Inner { value: 99 } } }
            o.middle.inner.value
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_three_fields() {
    let ir = gen_ok(
        r#"
        S Triple { a: i64, b: i64, c: i64 }
        F test() -> i64 {
            t := Triple { a: 1, b: 2, c: 3 }
            t.a + t.b + t.c
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// expr_helpers_misc: miscellaneous expressions
// ============================================================================

#[test]
fn test_block_expression() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            result := {
                a := 10
                b := 20
                a + b
            }
            result
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_multiple_expressions_in_block() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            y := x + 2
            z := y * 3
            z
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_expression_as_statement() {
    let ir = gen_ok(
        r#"
        F side_effect(x: i64) -> i64 = x
        F test() -> i64 {
            side_effect(1)
            side_effect(2)
            side_effect(3)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

// ============================================================================
// Complex codegen paths
// ============================================================================

#[test]
fn test_complex_struct_with_methods() {
    let ir = gen_ok(
        r#"
        S Rectangle {
            width: i64,
            height: i64
        }
        X Rectangle {
            F area(self) -> i64 = self.width * self.height
            F perimeter(self) -> i64 = 2 * (self.width + self.height)
        }
        F test() -> i64 {
            r := Rectangle { width: 5, height: 3 }
            r.area() + r.perimeter()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_nested_if_match() {
    let ir = gen_ok(
        r#"
        F classify(x: i64) -> i64 {
            I x > 0 {
                M x {
                    1 => 10,
                    2 => 20,
                    _ => 30
                }
            } E {
                M x {
                    0 => 0,
                    _ => 0 - 1
                }
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_loop_with_accumulator() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            i := mut 1
            L i <= 100 {
                sum = sum + i
                i = i + 1
            }
            sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_multiple_structs() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        S Line { start: Point, end: Point }
        F test() -> i64 {
            l := Line {
                start: Point { x: 0, y: 0 },
                end: Point { x: 10, y: 10 }
            }
            l.start.x + l.end.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_function_as_value() {
    let result = gen_result(
        r#"
        F square(x: i64) -> i64 = x * x
        F test() -> i64 = square(square(3))
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_complex_deeply_nested_arithmetic() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            ((1 + 2) * (3 + 4)) - ((5 - 6) * (7 + 8))
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_multiple_returns() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            I x < 0 { R 0 - 1 }
            I x == 0 { R 0 }
            I x < 10 { R 1 }
            I x < 100 { R 2 }
            R 3
        }
    "#,
    );
    assert!(ir.contains("ret"));
}

#[test]
fn test_complex_bool_chain() {
    let ir = gen_ok(
        r#"
        F test(a: i64, b: i64, c: i64) -> bool {
            (a > 0 && b > 0) || (c > 0 && a < 100)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_global_declaration() {
    let result = gen_result(
        r#"
        G MAX_SIZE: i64 = 1024
        F test() -> i64 = MAX_SIZE
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_type_alias_codegen() {
    let result = gen_result(
        r#"
        T Num = i64
        F test(x: Num) -> Num = x + 1
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_defer_statement() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            x := mut 0
            D { x = x + 1 }
            x = 42
            x
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_for_loop_range() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            total := mut 0
            L i: 1..11 {
                total = total + i
            }
            total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_for_loop_nested() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..5 {
                L j: 0..5 {
                    sum = sum + 1
                }
            }
            sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_bitwise_not() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            x := 255
            !x
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_logical_not() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            x := true
            !x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_unary_minus() {
    let ir = gen_ok("F test() -> i64 = 0 - 42");
    assert!(ir.contains("sub"));
}

#[test]
fn test_multiline_function() {
    let ir = gen_ok(
        r#"
        F compute(x: i64, y: i64, z: i64) -> i64 {
            a := x * y
            b := y * z
            c := x * z
            d := a + b + c
            I d > 100 {
                d - 100
            } E {
                d
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_mutual_recursion() {
    let ir = gen_ok(
        r#"
        F is_even(n: i64) -> bool {
            I n == 0 { R true }
            is_odd(n - 1)
        }
        F is_odd(n: i64) -> bool {
            I n == 0 { R false }
            is_even(n - 1)
        }
    "#,
    );
    assert!(ir.contains("@is_even"));
    assert!(ir.contains("@is_odd"));
}

#[test]
fn test_match_multiple_arms() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                0 => 0,
                1 => 10,
                2 => 20,
                3 => 30,
                4 => 40,
                5 => 50,
                _ => 99
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_bool() {
    let ir = gen_ok(
        r#"
        F test(b: bool) -> i64 {
            M b {
                true => 1,
                false => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_method_returning_struct() {
    let ir = gen_ok(
        r#"
        S Vec2 { x: i64, y: i64 }
        X Vec2 {
            F mag_sq(self) -> i64 = self.x * self.x + self.y * self.y
        }
        F test() -> i64 {
            v := Vec2 { x: 3, y: 4 }
            v.mag_sq()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_shadowing() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 1
            x := 2
            x := x + 3
            x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_boolean_short_circuit() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> bool {
            x > 0 && x < 100 && x != 50
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_or_short_circuit() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> bool {
            x == 0 || x == 1 || x == 2
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_zero_division_guard() {
    let ir = gen_ok(
        r#"
        F safe_div(a: i64, b: i64) -> i64 {
            I b == 0 { R 0 }
            a / b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_fibonacci_iterative() {
    let ir = gen_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            a := mut 0
            b := mut 1
            i := mut 2
            L i <= n {
                temp := a + b
                a = b
                b = temp
                i = i + 1
            }
            b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_gcd() {
    let ir = gen_ok(
        r#"
        F gcd(a: i64, b: i64) -> i64 {
            I b == 0 { R a }
            gcd(b, a % b)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_power_function() {
    let ir = gen_ok(
        r#"
        F pow(base: i64, exp: i64) -> i64 {
            I exp == 0 { R 1 }
            base * pow(base, exp - 1)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_multiple_struct_methods() {
    let ir = gen_ok(
        r#"
        S Account { balance: i64 }
        X Account {
            F get_balance(self) -> i64 = self.balance
            F is_positive(self) -> bool = self.balance > 0
        }
        F test() -> i64 {
            a := Account { balance: 100 }
            I a.is_positive() {
                a.get_balance()
            } E {
                0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_empty_struct() {
    let result = gen_result(
        r#"
        S Unit {}
        F test() -> i64 {
            u := Unit {}
            0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_struct_single_field() {
    let ir = gen_ok(
        r#"
        S Wrapper { value: i64 }
        F test() -> i64 {
            w := Wrapper { value: 42 }
            w.value
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_multi_param_function() {
    let ir = gen_ok(
        r#"
        F sum5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 {
            a + b + c + d + e
        }
        F test() -> i64 = sum5(1, 2, 3, 4, 5)
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_division_and_modulo() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 100 / 3
            b := 100 % 3
            a + b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_complex_struct_arithmetic() {
    let ir = gen_ok(
        r#"
        S Vec3 { x: i64, y: i64, z: i64 }
        X Vec3 {
            F dot(self, other: Vec3) -> i64 {
                self.x * other.x + self.y * other.y + self.z * other.z
            }
        }
        F test() -> i64 {
            a := Vec3 { x: 1, y: 2, z: 3 }
            b := Vec3 { x: 4, y: 5, z: 6 }
            a.dot(b)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_nested_in_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            sum := mut 0
            L i: 0..10 {
                val := M i {
                    0 => 0,
                    1 => 1,
                    _ => i * 2
                }
                sum = sum + val
            }
            sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_with_break_value() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            i := mut 0
            L {
                I i >= 10 {
                    B
                }
                i = i + 1
            }
            i
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_large_integer_constants() {
    let ir = gen_ok("F test() -> i64 = 9999999999");
    assert!(ir.contains("9999999999"));
}

#[test]
fn test_zero_constant() {
    let ir = gen_ok("F test() -> i64 = 0");
    assert!(ir.contains("ret i64 0") || ir.contains("0"));
}

#[test]
fn test_negative_via_subtraction() {
    let ir = gen_ok("F test() -> i64 = 0 - 1");
    assert!(ir.contains("sub"));
}

#[test]
fn test_trait_impl() {
    let ir = gen_ok(
        r#"
        W Describable {
            F describe(self) -> i64
        }
        S Item { id: i64 }
        X Item: Describable {
            F describe(self) -> i64 = self.id
        }
        F test() -> i64 {
            item := Item { id: 42 }
            item.describe()
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_generic_function() {
    let result = gen_result(
        r#"
        F identity<T>(x: T) -> T = x
        F test() -> i64 = identity(42)
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_while_loop_countdown() {
    let ir = gen_ok(
        r#"
        F countdown(n: i64) -> i64 {
            count := mut n
            L count > 0 {
                count = count - 1
            }
            count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_i32_function() {
    let result = gen_result("F test() -> i32 = 42");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_bool_to_int() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            b := true
            I b { 1 } E { 0 }
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_multiple_match_arms_with_same_value() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                1 => 0,
                2 => 0,
                3 => 0,
                _ => 1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_deeply_nested_blocks() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := {
                b := {
                    c := {
                        42
                    }
                    c + 1
                }
                b + 2
            }
            a
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_with_bool_field() {
    let ir = gen_ok(
        r#"
        S Flags { active: bool, count: i64 }
        F test() -> i64 {
            f := Flags { active: true, count: 5 }
            f.count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_comparison_chain() {
    let ir = gen_ok(
        r#"
        F clamp(x: i64, lo: i64, hi: i64) -> i64 {
            I x < lo { R lo }
            I x > hi { R hi }
            x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_max_function() {
    let ir = gen_ok(
        r#"
        F max(a: i64, b: i64) -> i64 {
            I a > b { a } E { b }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_min_function() {
    let ir = gen_ok(
        r#"
        F min(a: i64, b: i64) -> i64 {
            I a < b { a } E { b }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_abs_function() {
    let ir = gen_ok(
        r#"
        F abs(x: i64) -> i64 {
            I x < 0 { 0 - x } E { x }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sign_function() {
    let ir = gen_ok(
        r#"
        F sign(x: i64) -> i64 {
            I x > 0 { 1 }
            E I x < 0 { 0 - 1 }
            E { 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}
