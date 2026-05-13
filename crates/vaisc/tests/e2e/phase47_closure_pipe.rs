//! Phase 47 — Closure advanced, pipe chain, and string/expression body E2E tests
//!
//! Tests covering:
//! - Closure returning computed values from captured variables
//! - Closure passed as argument to higher-order functions
//! - Multiple closures in same scope
//! - Pipe operator with 4+ stages
//! - Pipe with various function types
//! - String puts with patterns
//! - Expression body functions with complex expressions
//! - Block expressions in various contexts

use super::helpers::*;

// ==================== 1. Closure: capture and multiply ====================

#[test]
fn e2e_p47_closure_capture_multiply() {
    // Closure captures a multiplier and applies it
    let source = r#"
F main() -> i64 {
    factor := 7
    mul := |x| x * factor
    mul(6)
}
"#;
    // 6 * 7 = 42
    assert_exit_code(source, 42);
}

// ==================== 2. Closure: capture two variables ====================

#[test]
fn e2e_p47_closure_capture_two_vars() {
    // Closure captures two variables and combines them with param
    let source = r#"
F main() -> i64 {
    base := 10
    offset := 5
    f := |x| base + offset + x
    f(3)
}
"#;
    // 10 + 5 + 3 = 18
    assert_exit_code(source, 18);
}

// ==================== 3. Closure passed to higher-order function ====================

#[test]
fn e2e_p47_closure_to_higher_order() {
    // Closure used as argument to apply()
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 { f(x) }
F main() -> i64 {
    result := apply(10, |x| x * 3)
    result
}
"#;
    // 10 * 3 = 30
    assert_exit_code(source, 30);
}

// ==================== 4. Two closures in same scope ====================

#[test]
fn e2e_p47_two_closures_same_scope() {
    // Two different closures defined and used in the same function
    let source = r#"
F main() -> i64 {
    add5 := |x| x + 5
    mul3 := |x| x * 3
    add5(10) + mul3(4)
}
"#;
    // 15 + 12 = 27
    assert_exit_code(source, 27);
}

// ==================== 5. Closure with conditional body ====================

#[test]
fn e2e_p47_closure_with_condition() {
    // Closure body contains if-else (requires block body for multi-expr)
    let source = r#"
F clamp_val(x: i64) -> i64 {
    I x > 100 { 100 } E { x }
}
F main() -> i64 {
    clamp_val(50) + clamp_val(200)
}
"#;
    // 50 + 100 = 150
    assert_exit_code(source, 150);
}

// ==================== 6. Closure accumulator in loop ====================

#[test]
fn e2e_p47_closure_accumulator_loop() {
    // Closure used as accumulator function in a loop
    let source = r#"
F main() -> i64 {
    acc := |a: i64, b: i64| a + b
    total := mut 0
    L i:1..6 {
        total = acc(total, i)
    }
    total
}
"#;
    // 1+2+3+4+5 = 15
    assert_exit_code(source, 15);
}

// ==================== 7. Higher-order: apply twice ====================

#[test]
fn e2e_p47_apply_twice() {
    // Apply a function twice to a value
    let source = r#"
F apply_twice(x: i64, f: fn(i64) -> i64) -> i64 { f(f(x)) }
F inc(x: i64) -> i64 { x + 1 }
F main() -> i64 {
    apply_twice(5, inc)
}
"#;
    // inc(inc(5)) = inc(6) = 7
    assert_exit_code(source, 7);
}

// ==================== 8. Closure returning zero or one ====================

#[test]
fn e2e_p47_closure_predicate() {
    // Closure acting as predicate (returns 0 or 1)
    let source = r#"
F main() -> i64 {
    is_positive := |x: i64| I x > 0 { 1 } E { 0 }
    is_positive(5) + is_positive(-3) + is_positive(0)
}
"#;
    // 1 + 0 + 0 = 1
    assert_exit_code(source, 1);
}

// ==================== 9. Pipe: four-stage chain ====================

#[test]
fn e2e_p47_pipe_four_stages() {
    // Four-stage pipeline
    let source = r#"
F add1(x: i64) -> i64 { x + 1 }
F double(x: i64) -> i64 { x * 2 }
F sub3(x: i64) -> i64 { x - 3 }
F square(x: i64) -> i64 { x * x }
F main() -> i64 {
    R 2 |> add1 |> double |> sub3 |> add1
}
"#;
    // 2 -> 3 -> 6 -> 3 -> 4
    assert_exit_code(source, 4);
}

// ==================== 10. Pipe: value through identity ====================

#[test]
fn e2e_p47_pipe_identity_chain() {
    // Multiple identity passes — value unchanged
    let source = r#"
F id(x: i64) -> i64 { x }
F main() -> i64 {
    R 99 |> id |> id |> id
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 11. Pipe: mixed operations ====================

#[test]
fn e2e_p47_pipe_mixed_ops() {
    // Pipeline with add, multiply, subtract
    let source = r#"
F add10(x: i64) -> i64 { x + 10 }
F triple(x: i64) -> i64 { x * 3 }
F sub5(x: i64) -> i64 { x - 5 }
F main() -> i64 {
    R 5 |> add10 |> triple |> sub5
}
"#;
    // 5 -> 15 -> 45 -> 40
    assert_exit_code(source, 40);
}

// ==================== 12. Pipe result stored in variable ====================

#[test]
fn e2e_p47_pipe_result_stored() {
    // Pipe result assigned to variable, then used
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F inc(x: i64) -> i64 { x + 1 }
F main() -> i64 {
    result := 10 |> double |> inc
    result + 8
}
"#;
    // 10 -> 20 -> 21, 21+8=29
    assert_exit_code(source, 29);
}

// ==================== 13. Expression body: complex arithmetic ====================

#[test]
fn e2e_p47_expr_body_complex() {
    // Expression body with multi-term arithmetic
    let source = r#"
F combo(a: i64, b: i64, c: i64) -> i64 = a * b + c
F main() -> i64 = combo(3, 4, 5)
"#;
    // 3*4 + 5 = 17
    assert_exit_code(source, 17);
}

// ==================== 14. Expression body: ternary ====================

#[test]
fn e2e_p47_expr_body_ternary() {
    // Expression body with ternary
    let source = r#"
F abs_val(x: i64) -> i64 = x < 0 ? -x : x
F main() -> i64 = abs_val(-15)
"#;
    assert_exit_code(source, 15);
}

// ==================== 15. Expression body: calling another expr body ====================

#[test]
fn e2e_p47_expr_body_chain_call() {
    // Chain of expression-body functions
    let source = r#"
F add1(x: i64) -> i64 = x + 1
F mul2(x: i64) -> i64 = x * 2
F main() -> i64 = mul2(add1(9))
"#;
    // add1(9)=10, mul2(10)=20
    assert_exit_code(source, 20);
}

// ==================== 16. Block expression: nested blocks ====================

#[test]
fn e2e_p47_block_nested_three_levels() {
    // Three-level nested block expressions
    let source = r#"
F main() -> i64 {
    result := {
        a := {
            x := 2
            y := 3
            x + y
        }
        b := {
            x := 10
            x * 2
        }
        a + b
    }
    result
}
"#;
    // a=5, b=20, 5+20=25
    assert_exit_code(source, 25);
}

// ==================== 17. Block expression in function argument ====================

#[test]
fn e2e_p47_block_in_fn_arg() {
    // Block expression used directly as a function argument
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F main() -> i64 {
    R double({
        a := 5
        b := 3
        a + b
    })
}
"#;
    // {5+3}=8, double(8)=16
    assert_exit_code(source, 16);
}

// ==================== 18. String puts then return ====================

#[test]
fn e2e_p47_puts_then_return() {
    // puts outputs string, then function returns a computed value
    let source = r#"
F main() -> i64 {
    puts("computing")
    x := 20
    y := 22
    x + y
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 42);
    assert!(result.stdout.contains("computing"));
}

// ==================== 19. Multiple puts with return code ====================

#[test]
fn e2e_p47_multiple_puts_exit_code() {
    // Multiple puts calls, then specific exit code
    let source = r#"
F main() -> i64 {
    puts("start")
    puts("middle")
    puts("end")
    R 33
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 33);
    assert!(result.stdout.contains("start"));
    assert!(result.stdout.contains("end"));
}

// ==================== 20. Closure: captures and returns constant ====================

#[test]
fn e2e_p47_closure_captures_returns_const() {
    // Closure captures a variable but returns a constant
    let source = r#"
F main() -> i64 {
    x := 999
    f := |_y: i64| 42
    f(x)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 21. Pipe into condition ====================

#[test]
fn e2e_p47_pipe_into_condition() {
    // Pipe result used in an if-else condition
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F main() -> i64 {
    val := 3 |> double
    I val > 5 { 1 } E { 0 }
}
"#;
    // double(3)=6, 6>5 = true, return 1
    assert_exit_code(source, 1);
}

// ==================== 22. Higher-order: function composition ====================

#[test]
fn e2e_p47_function_composition() {
    // Manual function composition via apply
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 { f(x) }
F add3(x: i64) -> i64 { x + 3 }
F mul2(x: i64) -> i64 { x * 2 }
F main() -> i64 {
    step1 := apply(4, add3)
    apply(step1, mul2)
}
"#;
    // add3(4)=7, mul2(7)=14
    assert_exit_code(source, 14);
}

// ==================== 23. Closure: used as map-like operation ====================

#[test]
fn e2e_p47_closure_map_like() {
    // Apply closure to multiple values, sum results
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 { f(x) }
F main() -> i64 {
    sq := |x: i64| x * x
    a := apply(2, sq)
    b := apply(3, sq)
    c := apply(4, sq)
    a + b + c
}
"#;
    // 4 + 9 + 16 = 29
    assert_exit_code(source, 29);
}

// ==================== 24. Expression body with self-recursion ====================

#[test]
fn e2e_p47_expr_body_recursion() {
    // Expression body function using @ self-recursion
    let source = r#"
F fac(n: i64) -> i64 = n <= 1 ? 1 : n * @(n - 1)
F main() -> i64 = fac(5)
"#;
    // 5! = 120
    assert_exit_code(source, 120);
}

// ==================== 25. Pipe: five-stage chain ====================

#[test]
fn e2e_p47_pipe_five_stages() {
    // Five-stage pipeline
    let source = r#"
F a(x: i64) -> i64 { x + 1 }
F b(x: i64) -> i64 { x * 2 }
F c(x: i64) -> i64 { x - 1 }
F d(x: i64) -> i64 { x + 5 }
F e(x: i64) -> i64 { x * 3 }
F main() -> i64 {
    R 1 |> a |> b |> c |> d |> e
}
"#;
    // 1 -> 2 -> 4 -> 3 -> 8 -> 24
    assert_exit_code(source, 24);
}

// ==================== 26. Block expression as return value ====================

#[test]
fn e2e_p47_block_as_return() {
    // Block expression as the function return value
    let source = r#"
F main() -> i64 {
    {
        x := 10
        y := 11
        x + y
    }
}
"#;
    assert_exit_code(source, 21);
}
