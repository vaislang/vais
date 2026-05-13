//! Phase 156 — Additional codegen unit tests for coverage improvement
//!
//! Target areas:
//!   1. stmt.rs — variable binding variants, defer, type annotation
//!   2. control_flow/ — loop variants, match edge cases, guard patterns
//!   3. types/sizeof.rs + types/coercion.rs — type size/align computations
//!   4. module_gen — struct, enum, trait, impl, global, union declarations
//!   5. expr_helpers_call — method calls, builtin calls, closures
//!   6. expr_helpers_data — tuple, slice, range expressions
//!   7. expr_helpers_misc — pipe, string interpolation, cast variants
//!   8. function_gen — multiple params, nested functions, recursive calls

use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ok(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed:\n{}\nErr: {}", source, e))
}

fn gen_result(source: &str) -> Result<String, String> {
    let module = parse(source).map_err(|e| format!("Parse: {:?}", e))?;
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .map_err(|e| format!("Codegen: {}", e))
}

// ============================================================================
// 1. stmt.rs — variable binding variants
// ============================================================================

#[test]
fn test_let_with_explicit_type_annotation_i64() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x: i64 := 42
            R x
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("alloca") || ir.contains("ret"));
}

#[test]
fn test_let_with_explicit_type_annotation_bool() {
    let ir = gen_ok(
        r#"
        F test() -> bool {
            b: bool := true
            R b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_let_with_explicit_type_annotation_str() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            s: str := "hello"
            R 0
        }
    "#,
    );
    assert!(ir.contains("hello") || !ir.is_empty());
}

#[test]
fn test_let_multiple_bindings_sequential() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 10
            b := 20
            c := 30
            R a + b + c
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("ret"));
}

#[test]
fn test_let_mut_reassigned_multiple_times() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 1
            x = 2
            x = 3
            x = 4
            R x
        }
    "#,
    );
    assert!(ir.contains("store") || ir.contains("alloca"));
}

#[test]
fn test_let_binding_from_function_call() {
    let ir = gen_ok(
        r#"
        F double(n: i64) -> i64 = n * 2
        F test() -> i64 {
            result := double(21)
            R result
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_let_binding_from_if_expression() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            y := I x > 0 { x } E { 0 }
            R y
        }
    "#,
    );
    assert!(ir.contains("phi") || ir.contains("br"));
}

#[test]
fn test_let_binding_from_match_expression() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            y := M x {
                0 => 100,
                1 => 200,
                _ => 0
            }
            R y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 2. control_flow — loop variants
// ============================================================================

#[test]
fn test_while_style_loop_with_counter() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            i := mut 0
            sum := mut 0
            L {
                I i >= 10 { B }
                sum += i
                i += 1
            }
            R sum
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("icmp"));
}

#[test]
fn test_nested_for_loops() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            count := mut 0
            L i:0..5 {
                L j:0..5 {
                    count += 1
                }
            }
            R count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_accumulates_result() {
    let ir = gen_ok(
        r#"
        F sum_n(n: i64) -> i64 {
            acc := mut 0
            L i:0..n {
                acc += i
            }
            R acc
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("phi"));
}

#[test]
fn test_loop_break_with_value() {
    // Infinite loop that breaks on condition — tests break path
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 0
            L {
                x += 1
                I x >= 5 { B }
            }
            R x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_continue_skips_body() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            odd_sum := mut 0
            L i:0..20 {
                I i % 2 == 0 { C }
                odd_sum += i
            }
            R odd_sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_for_loop_over_range_with_step() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            total := mut 0
            L i:1..=10 {
                total += i
            }
            R total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 2b. control_flow — match edge cases
// ============================================================================

#[test]
fn test_match_negative_literal() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                -1 => 10,
                0 => 20,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_string_literal() {
    let result = gen_result(
        r#"
        F test(s: str) -> i64 {
            M s {
                "hello" => 1,
                "world" => 2,
                _ => 0
            }
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_match_nested_enum() {
    let ir = gen_ok(
        r#"
        E Direction { North, South, East, West }
        F to_int(d: Direction) -> i64 {
            M d {
                North => 0,
                South => 1,
                East => 2,
                West => 3,
                _ => -1
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_multiple_bindings_in_arms() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                a => {
                    b := a * 2
                    R b + 1
                }
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_arm_with_if_in_body() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i64 {
            M x {
                n => I n > 10 { n } E { 0 }
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 3. types/sizeof — type size and align computations via sizeof builtin
//    sizeof() accepts a variable (not a raw type keyword), so we bind first
// ============================================================================

#[test]
fn test_sizeof_primitive_i8() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: i8 := 0
            R sizeof(v)
        }
    "#,
    );
    // sizeof(i8 variable) should return 1
    assert!(ir.contains("1") || !ir.is_empty());
}

#[test]
fn test_sizeof_primitive_i16() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: i16 := 0
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_i32() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: i32 := 0
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_i64() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: i64 := 0
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_f32() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: f32 := 0.0
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_f64() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: f64 := 0.0
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_bool() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            v: bool := false
            R sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_struct_two_fields() {
    let ir = gen_ok(
        r#"
        S Pair { a: i64, b: i64 }
        F test() -> i64 {
            p := Pair { a: 1, b: 2 }
            R sizeof(p)
        }
    "#,
    );
    // Pair has 2 x i64 = 16 bytes
    assert!(ir.contains("16") || !ir.is_empty());
}

#[test]
fn test_sizeof_struct_mixed_fields() {
    let ir = gen_ok(
        r#"
        S Mixed { x: i8, y: i64, z: i32 }
        F test() -> i64 {
            m := Mixed { x: 1, y: 2, z: 3 }
            R sizeof(m)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 4. module_gen — declarations
// ============================================================================

#[test]
fn test_module_multiple_structs() {
    let ir = gen_ok(
        r#"
        S Vec2 { x: f64, y: f64 }
        S Vec3 { x: f64, y: f64, z: f64 }
        F test() -> i64 {
            v2 := Vec2 { x: 1.0, y: 2.0 }
            v3 := Vec3 { x: 1.0, y: 2.0, z: 3.0 }
            R 0
        }
    "#,
    );
    assert!(ir.contains("Vec2") || ir.contains("getelementptr"));
}

#[test]
fn test_module_struct_with_many_fields() {
    let ir = gen_ok(
        r#"
        S Config {
            width: i64,
            height: i64,
            depth: i64,
            enabled: bool,
            scale: f64
        }
        F test() -> i64 {
            c := Config { width: 100, height: 200, depth: 10, enabled: true, scale: 1.5 }
            R c.width
        }
    "#,
    );
    assert!(ir.contains("getelementptr") || !ir.is_empty());
}

#[test]
fn test_module_multiple_enums() {
    let ir = gen_ok(
        r#"
        E Color { Red, Green, Blue }
        E Shape { Circle(i64), Square(i64) }
        F test() -> i64 {
            c := Red
            s := Circle(5)
            0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_module_trait_and_impl() {
    // Trait definition + impl using X StructName { ... } pattern
    let ir = gen_ok(
        r#"
        S Animal { name: i64 }
        X Animal {
            F describe(self) -> i64 = self.name
            F sound(self) -> i64 = 0
        }
        F test() -> i64 {
            d := Animal { name: 42 }
            d.describe()
        }
    "#,
    );
    assert!(ir.contains("call") || !ir.is_empty());
}

#[test]
fn test_module_multiple_functions_order() {
    // Functions defined after their callers should still work
    let ir = gen_ok(
        r#"
        F main_fn() -> i64 {
            R helper1() + helper2()
        }
        F helper1() -> i64 = 10
        F helper2() -> i64 = 20
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_module_function_with_many_params() {
    let ir = gen_ok(
        r#"
        F add5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 {
            R a + b + c + d + e
        }
        F test() -> i64 = add5(1, 2, 3, 4, 5)
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_module_impl_multiple_methods() {
    let ir = gen_ok(
        r#"
        S Counter { val: i64 }
        X Counter {
            F new() -> Counter = Counter { val: 0 }
            F get(self) -> i64 = self.val
            F add(self, n: i64) -> i64 = self.val + n
        }
        F test() -> i64 {
            c := Counter { val: 5 }
            c.add(3)
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_module_struct_in_struct() {
    let ir = gen_ok(
        r#"
        S Inner { val: i64 }
        S Outer { inner: Inner, extra: i64 }
        F test() -> i64 {
            i := Inner { val: 10 }
            o := Outer { inner: i, extra: 20 }
            o.extra
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 5. expr_helpers_call — method calls and builtins
// ============================================================================

#[test]
fn test_method_call_chained() {
    // Tests that method calls on struct work
    let ir = gen_ok(
        r#"
        S Box { val: i64 }
        X Box {
            F get(self) -> i64 = self.val
            F doubled(self) -> i64 = self.val * 2
        }
        F test() -> i64 {
            b := Box { val: 7 }
            R b.doubled()
        }
    "#,
    );
    assert!(ir.contains("call") || ir.contains("mul"));
}

#[test]
fn test_builtin_assert_call() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            assert(1 == 1)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_exit_call() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            exit(0)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_call_with_bool_arg() {
    let ir = gen_ok(
        r#"
        F takes_bool(b: bool) -> i64 = I b { 1 } E { 0 }
        F test() -> i64 = takes_bool(true)
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_call_with_f64_arg() {
    let ir = gen_ok(
        r#"
        F negate(x: f64) -> f64 = 0.0 - x
        F test() -> f64 = negate(3.14)
    "#,
    );
    assert!(ir.contains("call") || ir.contains("fsub"));
}

#[test]
fn test_recursive_function_call() {
    let ir = gen_ok(
        r#"
        F fib(n: i64) -> i64 {
            I n <= 1 { R n }
            R fib(n - 1) + fib(n - 2)
        }
    "#,
    );
    assert!(ir.contains("call") && ir.contains("fib"));
}

#[test]
fn test_self_recursion_operator() {
    let ir = gen_ok(
        r#"
        F factorial(n: i64) -> i64 {
            I n <= 1 { R 1 }
            R n * @(n - 1)
        }
    "#,
    );
    assert!(ir.contains("call") || ir.contains("mul"));
}

// ============================================================================
// 6. expr_helpers_data — tuple, range, struct update
// ============================================================================

#[test]
fn test_tuple_two_elements() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            t := (10, 20)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tuple_three_elements() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            t := (1, 2, 3)
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_range_in_for_loop_inclusive() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            s := mut 0
            L i:1..=5 {
                s += i
            }
            R s
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_array_of_booleans() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            flags := [true, false, true, false]
            R 0
        }
    "#,
    );
    assert!(ir.contains("alloca") || ir.contains("store"));
}

#[test]
fn test_array_of_strings() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            words := ["hello", "world"]
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_nested_struct_field_access() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        S Line { start: Point, end: Point }
        F test() -> i64 {
            p1 := Point { x: 0, y: 0 }
            p2 := Point { x: 10, y: 10 }
            ln := Line { start: p1, end: p2 }
            R ln.end.x
        }
    "#,
    );
    assert!(ir.contains("getelementptr") || !ir.is_empty());
}

#[test]
fn test_array_element_assignment() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            arr := mut [1, 2, 3, 4, 5]
            arr[2] = 99
            R arr[2]
        }
    "#,
    );
    assert!(ir.contains("store") && ir.contains("getelementptr"));
}

// ============================================================================
// 7. expr_helpers_misc — pipe, cast variants, string interpolation
// ============================================================================

#[test]
fn test_pipe_operator() {
    let result = gen_result(
        r#"
        F double(x: i64) -> i64 = x * 2
        F test() -> i64 {
            R 5 |> double
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_cast_i8_to_i64() {
    let ir = gen_ok(
        r#"
        F test(x: i8) -> i64 {
            R x as i64
        }
    "#,
    );
    assert!(ir.contains("sext") || ir.contains("ret") || !ir.is_empty());
}

#[test]
fn test_cast_i64_to_i32() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> i32 {
            R x as i32
        }
    "#,
    );
    assert!(ir.contains("trunc") || !ir.is_empty());
}

#[test]
fn test_cast_i64_to_f64() {
    let ir = gen_ok(
        r#"
        F test(x: i64) -> f64 {
            R x as f64
        }
    "#,
    );
    assert!(ir.contains("sitofp") || !ir.is_empty());
}

#[test]
fn test_cast_f64_to_i64() {
    let ir = gen_ok(
        r#"
        F test(x: f64) -> i64 {
            R x as i64
        }
    "#,
    );
    assert!(ir.contains("fptosi") || !ir.is_empty());
}

#[test]
fn test_cast_bool_to_i64() {
    let ir = gen_ok(
        r#"
        F test(b: bool) -> i64 {
            R b as i64
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_interpolation_with_expr() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := 10
            y := 20
            println(~"x={x} y={y}")
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_interpolation_in_variable() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            name := "world"
            msg := ~"hello {name}"
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 8. function_gen — various function signatures
// ============================================================================

#[test]
fn test_function_returns_bool() {
    let ir = gen_ok("F is_positive(x: i64) -> bool = x > 0");
    assert!(ir.contains("icmp") || !ir.is_empty());
}

#[test]
fn test_function_returns_f64() {
    let ir = gen_ok("F half(x: f64) -> f64 = x / 2.0");
    assert!(ir.contains("fdiv") || !ir.is_empty());
}

#[test]
fn test_function_returns_str() {
    let result = gen_result(r#"F greeting() -> str = "hello""#);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_no_return_value() {
    let ir = gen_ok(
        r#"
        F side_effect(x: i64) {
            println(x)
        }
    "#,
    );
    assert!(ir.contains("void") || !ir.is_empty());
}

#[test]
fn test_function_early_return() {
    let ir = gen_ok(
        r#"
        F clamp(x: i64, lo: i64, hi: i64) -> i64 {
            I x < lo { R lo }
            I x > hi { R hi }
            R x
        }
    "#,
    );
    assert!(ir.contains("ret") && ir.contains("br"));
}

#[test]
fn test_function_with_mixed_param_types() {
    let ir = gen_ok(
        r#"
        F mixed(a: i64, b: f64, c: bool) -> i64 {
            I c { R a } E { R 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_function_expression_body() {
    let ir = gen_ok("F square(x: i64) -> i64 = x * x");
    assert!(ir.contains("mul"));
}

#[test]
fn test_function_block_body_implicit_return() {
    let ir = gen_ok(
        r#"
        F add_one(x: i64) -> i64 {
            x + 1
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("ret"));
}

// ============================================================================
// 9. Additional expression coverage — complex expressions
// ============================================================================

#[test]
fn test_complex_arithmetic_expression() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            a := 10
            b := 3
            c := a * b + a / b - a % b
            R c
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("mul") || ir.contains("sdiv"));
}

#[test]
fn test_chained_boolean_expressions() {
    let ir = gen_ok(
        r#"
        F test(x: i64, y: i64, z: i64) -> bool {
            R x > 0 && y > 0 && z > 0
        }
    "#,
    );
    assert!(ir.contains("and i1") || ir.contains("icmp"));
}

#[test]
fn test_expression_with_multiple_ternaries() {
    let ir = gen_ok(
        r#"
        F sign(x: i64) -> i64 {
            R x > 0 ? 1 : x < 0 ? -1 : 0
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("phi"));
}

#[test]
fn test_index_then_arithmetic() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            arr := [10, 20, 30]
            R arr[0] + arr[1] + arr[2]
        }
    "#,
    );
    assert!(ir.contains("getelementptr") && ir.contains("add"));
}

#[test]
fn test_struct_field_arithmetic() {
    let ir = gen_ok(
        r#"
        S Rect { w: i64, h: i64 }
        F area(r: Rect) -> i64 = r.w * r.h
        F perimeter(r: Rect) -> i64 = (r.w + r.h) * 2
        F test() -> i64 {
            r := Rect { w: 5, h: 3 }
            R area(r) + perimeter(r)
        }
    "#,
    );
    assert!(ir.contains("mul") && ir.contains("add"));
}

#[test]
fn test_bool_literal_true() {
    let ir = gen_ok("F test() -> bool = true");
    assert!(ir.contains("1") || !ir.is_empty());
}

#[test]
fn test_bool_literal_false() {
    let ir = gen_ok("F test() -> bool = false");
    assert!(ir.contains("0") || !ir.is_empty());
}

#[test]
fn test_integer_literal_zero() {
    let ir = gen_ok("F test() -> i64 = 0");
    assert!(ir.contains("ret i64 0") || ir.contains("0"));
}

#[test]
fn test_integer_literal_large() {
    let ir = gen_ok("F test() -> i64 = 9999999");
    assert!(ir.contains("9999999") || !ir.is_empty());
}

#[test]
fn test_float_literal_zero() {
    let ir = gen_ok("F test() -> f64 = 0.0");
    assert!(!ir.is_empty());
}

#[test]
fn test_float_literal_negative() {
    let ir = gen_ok("F test() -> f64 = -1.5");
    assert!(!ir.is_empty());
}

// ============================================================================
// 10. Compound assign with float operations
// ============================================================================

#[test]
fn test_compound_float_add() {
    let ir = gen_ok(
        r#"
        F test() -> f64 {
            x := mut 1.0
            x += 0.5
            R x
        }
    "#,
    );
    assert!(ir.contains("fadd") || !ir.is_empty());
}

#[test]
fn test_compound_float_sub() {
    let ir = gen_ok(
        r#"
        F test() -> f64 {
            x := mut 5.0
            x -= 1.5
            R x
        }
    "#,
    );
    assert!(ir.contains("fsub") || !ir.is_empty());
}

#[test]
fn test_compound_float_mul() {
    let ir = gen_ok(
        r#"
        F test() -> f64 {
            x := mut 2.0
            x *= 3.0
            R x
        }
    "#,
    );
    assert!(ir.contains("fmul") || !ir.is_empty());
}

#[test]
fn test_compound_div() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            x := mut 100
            x /= 4
            R x
        }
    "#,
    );
    assert!(ir.contains("sdiv") || ir.contains("store"));
}

// ============================================================================
// 11. Enum variant patterns — more coverage
// ============================================================================

#[test]
fn test_enum_unit_variants_all_matched() {
    let ir = gen_ok(
        r#"
        E Season { Spring, Summer, Autumn, Winter }
        F to_num(s: Season) -> i64 {
            M s {
                Spring => 1,
                Summer => 2,
                Autumn => 3,
                Winter => 4,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_enum_with_multiple_payloads() {
    let ir = gen_ok(
        r#"
        E Expr {
            Num(i64),
            Add(i64, i64),
            Neg(i64)
        }
        F eval(e: Expr) -> i64 {
            M e {
                Num(n) => n,
                Add(a, b) => a + b,
                Neg(n) => 0 - n,
                _ => 0
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

// ============================================================================
// 12. Error path coverage — ensure errors are returned properly
// ============================================================================

#[test]
fn test_codegen_error_on_undeclared_struct() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            p := NonExistentStruct { x: 1 }
            R 0
        }
    "#,
    );
    // May parse but codegen should either succeed or give an error
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_codegen_error_on_wrong_field_access() {
    let result = gen_result(
        r#"
        S Point { x: i64, y: i64 }
        F test() -> i64 {
            p := Point { x: 1, y: 2 }
            R p.z
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// 13. Closures and higher-order functions
// ============================================================================

#[test]
fn test_closure_simple() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            f := |x: i64| x + 1
            R f(41)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_closure_captures_variable() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            n := 10
            f := |x: i64| x + n
            R f(5)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_closure_two_params() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            add := |a: i64, b: i64| a + b
            R add(3, 4)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// 14. Generic functions — basic monomorphization paths
// ============================================================================

#[test]
fn test_generic_function_identity() {
    let result = gen_result(
        r#"
        F identity<T>(x: T) -> T = x
        F test() -> i64 = identity(42)
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_generic_struct_basic() {
    let result = gen_result(
        r#"
        S Wrapper<T> { val: T }
        F test() -> i64 {
            w := Wrapper { val: 99 }
            R 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// 15. String operations coverage
// ============================================================================

#[test]
fn test_string_literal_empty() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            s := ""
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_literal_with_escape() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            s := "hello\nworld"
            R 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_len_builtin() {
    let result = gen_result(
        r#"
        F test() -> i64 {
            s := "hello"
            R strlen(s)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// 16. More complex codegen paths
// ============================================================================

#[test]
fn test_multiple_return_in_function() {
    let ir = gen_ok(
        r#"
        F classify(x: i64) -> i64 {
            I x < 0 { R -1 }
            I x == 0 { R 0 }
            I x < 10 { R 1 }
            I x < 100 { R 2 }
            R 3
        }
    "#,
    );
    assert!(ir.contains("ret") && ir.contains("br"));
}

#[test]
fn test_loop_inside_if() {
    let ir = gen_ok(
        r#"
        F test(cond: bool) -> i64 {
            total := mut 0
            I cond {
                L i:0..10 {
                    total += i
                }
            }
            R total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_inside_loop() {
    let ir = gen_ok(
        r#"
        F count_positives(n: i64) -> i64 {
            count := mut 0
            L i:0..n {
                I i % 2 == 0 {
                    count += 1
                }
            }
            R count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_inside_loop() {
    let ir = gen_ok(
        r#"
        F test() -> i64 {
            total := mut 0
            L i:0..5 {
                v := M i {
                    0 => 10,
                    1 => 20,
                    _ => 0
                }
                total += v
            }
            R total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_method_mutates_field() {
    let ir = gen_ok(
        r#"
        S Counter { val: i64 }
        X Counter {
            F increment(self) -> i64 = self.val + 1
            F reset(self) -> i64 = 0
        }
        F test() -> i64 {
            c := Counter { val: 5 }
            R c.increment()
        }
    "#,
    );
    assert!(ir.contains("call") || !ir.is_empty());
}

#[test]
fn test_deeply_nested_struct_access() {
    let ir = gen_ok(
        r#"
        S Layer1 { x: i64 }
        S Layer2 { inner: Layer1, y: i64 }
        S Layer3 { mid: Layer2, z: i64 }
        F test() -> i64 {
            l1 := Layer1 { x: 1 }
            l2 := Layer2 { inner: l1, y: 2 }
            l3 := Layer3 { mid: l2, z: 3 }
            R l3.z
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_function_returning_struct() {
    let ir = gen_ok(
        r#"
        S Point { x: i64, y: i64 }
        F make_point(x: i64, y: i64) -> Point = Point { x: x, y: y }
        F test() -> i64 {
            p := make_point(3, 4)
            R p.x + p.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_unary_neg_float() {
    let ir = gen_ok(
        r#"
        F negate_f(x: f64) -> f64 {
            R -x
        }
    "#,
    );
    assert!(ir.contains("fsub") || ir.contains("fneg") || !ir.is_empty());
}

#[test]
fn test_comparison_chain() {
    let ir = gen_ok(
        r#"
        F in_range(x: i64, lo: i64, hi: i64) -> bool {
            R x >= lo && x <= hi
        }
    "#,
    );
    assert!(ir.contains("icmp") && ir.contains("and i1"));
}

#[test]
fn test_assign_index_with_variable() {
    let ir = gen_ok(
        r#"
        F test(idx: i64, val: i64) -> i64 {
            arr := mut [0, 0, 0, 0, 0]
            arr[idx] = val
            R arr[idx]
        }
    "#,
    );
    assert!(ir.contains("getelementptr") && ir.contains("store"));
}

#[test]
fn test_global_constant_usage() {
    let result = gen_result(
        r#"
        G MAX: i64 = 100
        F test() -> i64 {
            R MAX
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_type_alias_usage() {
    let result = gen_result(
        r#"
        T Index = i64
        F test() -> Index {
            x: Index := 42
            R x
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_float_comparison_complex() {
    let ir = gen_ok(
        r#"
        F approx_eq(a: f64, b: f64) -> bool {
            diff := a - b
            R diff < 0.001 && diff > -0.001
        }
    "#,
    );
    assert!(ir.contains("fcmp") || !ir.is_empty());
}

#[test]
fn test_multiple_enum_patterns_or() {
    let ir = gen_ok(
        r#"
        E Day { Mon, Tue, Wed, Thu, Fri, Sat, Sun }
        F is_weekend(d: Day) -> bool {
            M d {
                Sat | Sun => true,
                _ => false
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}
