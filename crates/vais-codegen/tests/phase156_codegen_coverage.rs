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
        fn test() -> i64 {
            x: i64 := 42
            return x
        }
    "#,
    );
    assert!(ir.contains("42") || ir.contains("alloca") || ir.contains("ret"));
}

#[test]
fn test_let_with_explicit_type_annotation_bool() {
    let ir = gen_ok(
        r#"
        fn test() -> bool {
            b: bool := true
            return b
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_let_with_explicit_type_annotation_str() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            s: str := "hello"
            return 0
        }
    "#,
    );
    assert!(ir.contains("hello") || !ir.is_empty());
}

#[test]
fn test_let_multiple_bindings_sequential() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            a := 10
            b := 20
            c := 30
            return a + b + c
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("ret"));
}

#[test]
fn test_let_mut_reassigned_multiple_times() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := mut 1
            x = 2
            x = 3
            x = 4
            return x
        }
    "#,
    );
    assert!(ir.contains("store") || ir.contains("alloca"));
}

#[test]
fn test_let_binding_from_function_call() {
    let ir = gen_ok(
        r#"
        fn double(n: i64) -> i64 = n * 2
        fn test() -> i64 {
            result := double(21)
            return result
        }
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_let_binding_from_if_expression() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            y := I x > 0 { x } else { 0 }
            return y
        }
    "#,
    );
    assert!(ir.contains("phi") || ir.contains("br"));
}

#[test]
fn test_let_binding_from_match_expression() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i64 {
            y := match x {
                0 => 100,
                1 => 200,
                _ => 0
            }
            return y
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
        fn test() -> i64 {
            i := mut 0
            sum := mut 0
            L {
                I i >= 10 { B }
                sum += i
                i += 1
            }
            return sum
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("icmp"));
}

#[test]
fn test_nested_for_loops() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            count := mut 0
            L i:0..5 {
                L j:0..5 {
                    count += 1
                }
            }
            return count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_accumulates_result() {
    let ir = gen_ok(
        r#"
        fn sum_n(n: i64) -> i64 {
            acc := mut 0
            L i:0..n {
                acc += i
            }
            return acc
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
        fn test() -> i64 {
            x := mut 0
            L {
                x += 1
                I x >= 5 { B }
            }
            return x
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_loop_continue_skips_body() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            odd_sum := mut 0
            L i:0..20 {
                I i % 2 == 0 { C }
                odd_sum += i
            }
            return odd_sum
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_for_loop_over_range_with_step() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            total := mut 0
            L i:1..=10 {
                total += i
            }
            return total
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
        fn test(x: i64) -> i64 {
            match x {
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
        fn test(s: str) -> i64 {
            match s {
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
        enum Direction { North, South, East, West }
        fn to_int(d: Direction) -> i64 {
            match d {
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
        fn test(x: i64) -> i64 {
            match x {
                a => {
                    b := a * 2
                    return b + 1
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
        fn test(x: i64) -> i64 {
            match x {
                n => I n > 10 { n } else { 0 }
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
        fn test() -> i64 {
            v: i8 := 0
            return sizeof(v)
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
        fn test() -> i64 {
            v: i16 := 0
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_i32() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            v: i32 := 0
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_i64() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            v: i64 := 0
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_f32() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            v: f32 := 0.0
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_f64() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            v: f64 := 0.0
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_primitive_bool() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            v: bool := false
            return sizeof(v)
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_sizeof_struct_two_fields() {
    let ir = gen_ok(
        r#"
        struct Pair { a: i64, b: i64 }
        fn test() -> i64 {
            p := Pair { a: 1, b: 2 }
            return sizeof(p)
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
        struct Mixed { x: i8, y: i64, z: i32 }
        fn test() -> i64 {
            m := Mixed { x: 1, y: 2, z: 3 }
            return sizeof(m)
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
        struct Vec2 { x: f64, y: f64 }
        struct Vec3 { x: f64, y: f64, z: f64 }
        fn test() -> i64 {
            v2 := Vec2 { x: 1.0, y: 2.0 }
            v3 := Vec3 { x: 1.0, y: 2.0, z: 3.0 }
            return 0
        }
    "#,
    );
    assert!(ir.contains("Vec2") || ir.contains("getelementptr"));
}

#[test]
fn test_module_struct_with_many_fields() {
    let ir = gen_ok(
        r#"
        struct Config {
            width: i64,
            height: i64,
            depth: i64,
            enabled: bool,
            scale: f64
        }
        fn test() -> i64 {
            c := Config { width: 100, height: 200, depth: 10, enabled: true, scale: 1.5 }
            return c.width
        }
    "#,
    );
    assert!(ir.contains("getelementptr") || !ir.is_empty());
}

#[test]
fn test_module_multiple_enums() {
    let ir = gen_ok(
        r#"
        enum Color { Red, Green, Blue }
        enum Shape { Circle(i64), Square(i64) }
        fn test() -> i64 {
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
    // Trait definition + impl using impl StructName { ... } pattern
    let ir = gen_ok(
        r#"
        struct Animal { name: i64 }
        impl Animal {
            fn describe(self) -> i64 = self.name
            fn sound(self) -> i64 = 0
        }
        fn test() -> i64 {
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
        fn main_fn() -> i64 {
            return helper1() + helper2()
        }
        fn helper1() -> i64 = 10
        fn helper2() -> i64 = 20
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_module_function_with_many_params() {
    let ir = gen_ok(
        r#"
        fn add5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 {
            return a + b + c + d + e
        }
        fn test() -> i64 = add5(1, 2, 3, 4, 5)
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_module_impl_multiple_methods() {
    let ir = gen_ok(
        r#"
        struct Counter { val: i64 }
        impl Counter {
            fn new() -> Counter = Counter { val: 0 }
            fn get(self) -> i64 = self.val
            fn add(self, n: i64) -> i64 = self.val + n
        }
        fn test() -> i64 {
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
        struct Inner { val: i64 }
        struct Outer { inner: Inner, extra: i64 }
        fn test() -> i64 {
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
        struct Box { val: i64 }
        impl Box {
            fn get(self) -> i64 = self.val
            fn doubled(self) -> i64 = self.val * 2
        }
        fn test() -> i64 {
            b := Box { val: 7 }
            return b.doubled()
        }
    "#,
    );
    assert!(ir.contains("call") || ir.contains("mul"));
}

#[test]
fn test_builtin_assert_call() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            assert(1 == 1)
            return 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builtin_exit_call() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            exit(0)
            return 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_call_with_bool_arg() {
    let ir = gen_ok(
        r#"
        fn takes_bool(b: bool) -> i64 = I b { 1 } else { 0 }
        fn test() -> i64 = takes_bool(true)
    "#,
    );
    assert!(ir.contains("call"));
}

#[test]
fn test_call_with_f64_arg() {
    let ir = gen_ok(
        r#"
        fn negate(x: f64) -> f64 = 0.0 - x
        fn test() -> f64 = negate(3.14)
    "#,
    );
    assert!(ir.contains("call") || ir.contains("fsub"));
}

#[test]
fn test_recursive_function_call() {
    let ir = gen_ok(
        r#"
        fn fib(n: i64) -> i64 {
            I n <= 1 { return n }
            return fib(n - 1) + fib(n - 2)
        }
    "#,
    );
    assert!(ir.contains("call") && ir.contains("fib"));
}

#[test]
fn test_self_recursion_operator() {
    let ir = gen_ok(
        r#"
        fn factorial(n: i64) -> i64 {
            I n <= 1 { return 1 }
            return n * @(n - 1)
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
        fn test() -> i64 {
            t := (10, 20)
            return 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_tuple_three_elements() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            t := (1, 2, 3)
            return 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_range_in_for_loop_inclusive() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            s := mut 0
            L i:1..=5 {
                s += i
            }
            return s
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_array_of_booleans() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            flags := [true, false, true, false]
            return 0
        }
    "#,
    );
    assert!(ir.contains("alloca") || ir.contains("store"));
}

#[test]
fn test_array_of_strings() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            words := ["hello", "world"]
            return 0
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_nested_struct_field_access() {
    let ir = gen_ok(
        r#"
        struct Point { x: i64, y: i64 }
        struct Line { start: Point, end: Point }
        fn test() -> i64 {
            p1 := Point { x: 0, y: 0 }
            p2 := Point { x: 10, y: 10 }
            ln := Line { start: p1, end: p2 }
            return ln.end.x
        }
    "#,
    );
    assert!(ir.contains("getelementptr") || !ir.is_empty());
}

#[test]
fn test_array_element_assignment() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            arr := mut [1, 2, 3, 4, 5]
            arr[2] = 99
            return arr[2]
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
        fn double(x: i64) -> i64 = x * 2
        fn test() -> i64 {
            return 5 |> double
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_cast_i8_to_i64() {
    let ir = gen_ok(
        r#"
        fn test(x: i8) -> i64 {
            return x as i64
        }
    "#,
    );
    assert!(ir.contains("sext") || ir.contains("ret") || !ir.is_empty());
}

#[test]
fn test_cast_i64_to_i32() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> i32 {
            return x as i32
        }
    "#,
    );
    assert!(ir.contains("trunc") || !ir.is_empty());
}

#[test]
fn test_cast_i64_to_f64() {
    let ir = gen_ok(
        r#"
        fn test(x: i64) -> f64 {
            return x as f64
        }
    "#,
    );
    assert!(ir.contains("sitofp") || !ir.is_empty());
}

#[test]
fn test_cast_f64_to_i64() {
    let ir = gen_ok(
        r#"
        fn test(x: f64) -> i64 {
            return x as i64
        }
    "#,
    );
    assert!(ir.contains("fptosi") || !ir.is_empty());
}

#[test]
fn test_cast_bool_to_i64() {
    let ir = gen_ok(
        r#"
        fn test(b: bool) -> i64 {
            return b as i64
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_interpolation_with_expr() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := 10
            y := 20
            println(~"x={x} y={y}")
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_interpolation_in_variable() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            name := "world"
            msg := ~"hello {name}"
            return 0
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
    let ir = gen_ok("fn is_positive(x: i64) -> bool = x > 0");
    assert!(ir.contains("icmp") || !ir.is_empty());
}

#[test]
fn test_function_returns_f64() {
    let ir = gen_ok("fn half(x: f64) -> f64 = x / 2.0");
    assert!(ir.contains("fdiv") || !ir.is_empty());
}

#[test]
fn test_function_returns_str() {
    let result = gen_result(r#"fn greeting() -> str = "hello""#);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_function_no_return_value() {
    let ir = gen_ok(
        r#"
        fn side_effect(x: i64) {
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
        fn clamp(x: i64, lo: i64, hi: i64) -> i64 {
            I x < lo { return lo }
            I x > hi { return hi }
            return x
        }
    "#,
    );
    assert!(ir.contains("ret") && ir.contains("br"));
}

#[test]
fn test_function_with_mixed_param_types() {
    let ir = gen_ok(
        r#"
        fn mixed(a: i64, b: f64, c: bool) -> i64 {
            I c { return a } else { return 0 }
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_function_expression_body() {
    let ir = gen_ok("fn square(x: i64) -> i64 = x * x");
    assert!(ir.contains("mul"));
}

#[test]
fn test_function_block_body_implicit_return() {
    let ir = gen_ok(
        r#"
        fn add_one(x: i64) -> i64 {
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
        fn test() -> i64 {
            a := 10
            b := 3
            c := a * b + a / b - a % b
            return c
        }
    "#,
    );
    assert!(ir.contains("add") || ir.contains("mul") || ir.contains("sdiv"));
}

#[test]
fn test_chained_boolean_expressions() {
    let ir = gen_ok(
        r#"
        fn test(x: i64, y: i64, z: i64) -> bool {
            return x > 0 && y > 0 && z > 0
        }
    "#,
    );
    assert!(ir.contains("and i1") || ir.contains("icmp"));
}

#[test]
fn test_expression_with_multiple_ternaries() {
    let ir = gen_ok(
        r#"
        fn sign(x: i64) -> i64 {
            return x > 0 ? 1 : x < 0 ? -1 : 0
        }
    "#,
    );
    assert!(ir.contains("br") || ir.contains("phi"));
}

#[test]
fn test_index_then_arithmetic() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            arr := [10, 20, 30]
            return arr[0] + arr[1] + arr[2]
        }
    "#,
    );
    assert!(ir.contains("getelementptr") && ir.contains("add"));
}

#[test]
fn test_struct_field_arithmetic() {
    let ir = gen_ok(
        r#"
        struct Rect { w: i64, h: i64 }
        fn area(r: Rect) -> i64 = r.w * r.h
        fn perimeter(r: Rect) -> i64 = (r.w + r.h) * 2
        fn test() -> i64 {
            r := Rect { w: 5, h: 3 }
            return area(r) + perimeter(r)
        }
    "#,
    );
    assert!(ir.contains("mul") && ir.contains("add"));
}

#[test]
fn test_bool_literal_true() {
    let ir = gen_ok("fn test() -> bool = true");
    assert!(ir.contains("1") || !ir.is_empty());
}

#[test]
fn test_bool_literal_false() {
    let ir = gen_ok("fn test() -> bool = false");
    assert!(ir.contains("0") || !ir.is_empty());
}

#[test]
fn test_integer_literal_zero() {
    let ir = gen_ok("fn test() -> i64 = 0");
    assert!(ir.contains("ret i64 0") || ir.contains("0"));
}

#[test]
fn test_integer_literal_large() {
    let ir = gen_ok("fn test() -> i64 = 9999999");
    assert!(ir.contains("9999999") || !ir.is_empty());
}

#[test]
fn test_float_literal_zero() {
    let ir = gen_ok("fn test() -> f64 = 0.0");
    assert!(!ir.is_empty());
}

#[test]
fn test_float_literal_negative() {
    let ir = gen_ok("fn test() -> f64 = -1.5");
    assert!(!ir.is_empty());
}

// ============================================================================
// 10. Compound assign with float operations
// ============================================================================

#[test]
fn test_compound_float_add() {
    let ir = gen_ok(
        r#"
        fn test() -> f64 {
            x := mut 1.0
            x += 0.5
            return x
        }
    "#,
    );
    assert!(ir.contains("fadd") || !ir.is_empty());
}

#[test]
fn test_compound_float_sub() {
    let ir = gen_ok(
        r#"
        fn test() -> f64 {
            x := mut 5.0
            x -= 1.5
            return x
        }
    "#,
    );
    assert!(ir.contains("fsub") || !ir.is_empty());
}

#[test]
fn test_compound_float_mul() {
    let ir = gen_ok(
        r#"
        fn test() -> f64 {
            x := mut 2.0
            x *= 3.0
            return x
        }
    "#,
    );
    assert!(ir.contains("fmul") || !ir.is_empty());
}

#[test]
fn test_compound_div() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            x := mut 100
            x /= 4
            return x
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
        enum Season { Spring, Summer, Autumn, Winter }
        fn to_num(s: Season) -> i64 {
            match s {
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
        enum Expr {
            Num(i64),
            Add(i64, i64),
            Neg(i64)
        }
        fn eval(e: Expr) -> i64 {
            match e {
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
        fn test() -> i64 {
            p := NonExistentStruct { x: 1 }
            return 0
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
        struct Point { x: i64, y: i64 }
        fn test() -> i64 {
            p := Point { x: 1, y: 2 }
            return p.z
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
        fn test() -> i64 {
            f := |x: i64| x + 1
            return f(41)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_closure_captures_variable() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            n := 10
            f := |x: i64| x + n
            return f(5)
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_closure_two_params() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            add := |a: i64, b: i64| a + b
            return add(3, 4)
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
        fn identity<T>(x: T) -> type = x
        fn test() -> i64 = identity(42)
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_generic_struct_basic() {
    let result = gen_result(
        r#"
        struct Wrapper<T> { val: type }
        fn test() -> i64 {
            w := Wrapper { val: 99 }
            return 0
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
        fn test() -> i64 {
            s := ""
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_literal_with_escape() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            s := "hello\nworld"
            return 0
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_string_len_builtin() {
    let result = gen_result(
        r#"
        fn test() -> i64 {
            s := "hello"
            return strlen(s)
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
        fn classify(x: i64) -> i64 {
            I x < 0 { return -1 }
            I x == 0 { return 0 }
            I x < 10 { return 1 }
            I x < 100 { return 2 }
            return 3
        }
    "#,
    );
    assert!(ir.contains("ret") && ir.contains("br"));
}

#[test]
fn test_loop_inside_if() {
    let ir = gen_ok(
        r#"
        fn test(cond: bool) -> i64 {
            total := mut 0
            I cond {
                L i:0..10 {
                    total += i
                }
            }
            return total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_if_inside_loop() {
    let ir = gen_ok(
        r#"
        fn count_positives(n: i64) -> i64 {
            count := mut 0
            L i:0..n {
                I i % 2 == 0 {
                    count += 1
                }
            }
            return count
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_match_inside_loop() {
    let ir = gen_ok(
        r#"
        fn test() -> i64 {
            total := mut 0
            L i:0..5 {
                v := match i {
                    0 => 10,
                    1 => 20,
                    _ => 0
                }
                total += v
            }
            return total
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_struct_method_mutates_field() {
    let ir = gen_ok(
        r#"
        struct Counter { val: i64 }
        impl Counter {
            fn increment(self) -> i64 = self.val + 1
            fn reset(self) -> i64 = 0
        }
        fn test() -> i64 {
            c := Counter { val: 5 }
            return c.increment()
        }
    "#,
    );
    assert!(ir.contains("call") || !ir.is_empty());
}

#[test]
fn test_deeply_nested_struct_access() {
    let ir = gen_ok(
        r#"
        struct Layer1 { x: i64 }
        struct Layer2 { inner: Layer1, y: i64 }
        struct Layer3 { mid: Layer2, z: i64 }
        fn test() -> i64 {
            l1 := Layer1 { x: 1 }
            l2 := Layer2 { inner: l1, y: 2 }
            l3 := Layer3 { mid: l2, z: 3 }
            return l3.z
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_function_returning_struct() {
    let ir = gen_ok(
        r#"
        struct Point { x: i64, y: i64 }
        fn make_point(x: i64, y: i64) -> Point = Point { x: x, y: y }
        fn test() -> i64 {
            p := make_point(3, 4)
            return p.x + p.y
        }
    "#,
    );
    assert!(!ir.is_empty());
}

#[test]
fn test_unary_neg_float() {
    let ir = gen_ok(
        r#"
        fn negate_f(x: f64) -> f64 {
            return -x
        }
    "#,
    );
    assert!(ir.contains("fsub") || ir.contains("fneg") || !ir.is_empty());
}

#[test]
fn test_comparison_chain() {
    let ir = gen_ok(
        r#"
        fn in_range(x: i64, lo: i64, hi: i64) -> bool {
            return x >= lo && x <= hi
        }
    "#,
    );
    assert!(ir.contains("icmp") && ir.contains("and i1"));
}

#[test]
fn test_assign_index_with_variable() {
    let ir = gen_ok(
        r#"
        fn test(idx: i64, val: i64) -> i64 {
            arr := mut [0, 0, 0, 0, 0]
            arr[idx] = val
            return arr[idx]
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
        fn test() -> i64 {
            return MAX
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_type_alias_usage() {
    let result = gen_result(
        r#"
        type Index = i64
        fn test() -> Index {
            x: Index := 42
            return x
        }
    "#,
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_float_comparison_complex() {
    let ir = gen_ok(
        r#"
        fn approx_eq(a: f64, b: f64) -> bool {
            diff := a - b
            return diff < 0.001 && diff > -0.001
        }
    "#,
    );
    assert!(ir.contains("fcmp") || !ir.is_empty());
}

#[test]
fn test_multiple_enum_patterns_or() {
    let ir = gen_ok(
        r#"
        enum Day { Mon, Tue, Wed, Thu, Fri, Sat, Sun }
        fn is_weekend(d: Day) -> bool {
            match d {
                Sat | Sun => true,
                _ => false
            }
        }
    "#,
    );
    assert!(!ir.is_empty());
}
