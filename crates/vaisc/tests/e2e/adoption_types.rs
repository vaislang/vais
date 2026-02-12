use super::helpers::*;

// ==================== Phase 22: 대형 프로젝트 도입 전략 - Stage 2 (Medium Scale) ====================

#[test]
fn test_adoption_generic_trait_integration() {
    let source = r#"
S Container<T> {
    value: T,
    count: i64
}
X Container {
    F get_count(&self) -> i64 = self.count
}
E Status {
    Active,
    Inactive
}
F check(s: i64) -> i64 {
    M s {
        0 => 0,
        _ => 1
    }
}
F main() -> i64 {
    c := Container { value: 42, count: 3 }
    I c.get_count() == 3 {
        println("Container: OK")
    }
    check(0)
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        result.exit_code, result.stdout, result.stderr
    );
    assert!(
        result.stdout.contains("Container: OK"),
        "Expected stdout to contain 'Container: OK', got: {}",
        result.stdout
    );
}

#[test]
fn test_adoption_closure_recursion() {
    let source = r#"
F fib(n: i64) -> i64 {
    I n < 2 { R n }
    @(n - 1) + @(n - 2)
}
F main() -> i64 {
    scale := |x: i64| x * 2
    result := scale(fib(10))
    I result == 110 {
        println("Closure+Recursion: OK")
        R 0
    }
    1
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        result.exit_code, result.stdout, result.stderr
    );
    assert!(
        result.stdout.contains("Closure+Recursion: OK"),
        "Expected stdout to contain 'Closure+Recursion: OK', got: {}",
        result.stdout
    );
}

#[test]
fn test_adoption_mutable_loop() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    i := mut 0
    L {
        I i >= 10 { B }
        sum = sum + i
        i = i + 1
    }
    I sum == 45 {
        println("MutableLoop: OK")
        R 0
    }
    1
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        result.exit_code, result.stdout, result.stderr
    );
    assert!(
        result.stdout.contains("MutableLoop: OK"),
        "Expected stdout to contain 'MutableLoop: OK', got: {}",
        result.stdout
    );
}

#[test]
fn test_adoption_f64_arithmetic() {
    let source = r#"
F main() -> i64 {
    x := 3.14
    y := 2.0
    z := x * y
    I z > 6.0 {
        println("F64 Arithmetic: OK")
        R 0
    }
    1
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        result.exit_code, result.stdout, result.stderr
    );
    assert!(
        result.stdout.contains("F64 Arithmetic: OK"),
        "Expected stdout to contain 'F64 Arithmetic: OK', got: {}",
        result.stdout
    );
}

#[test]
fn test_adoption_complex_struct() {
    let source = r#"
S Point {
    x: i64,
    y: i64
}
X Point {
    F distance_sq(&self) -> i64 = self.x * self.x + self.y * self.y
}
S Line {
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64
}
F main() -> i64 {
    p := Point { x: 3, y: 4 }
    d := p.distance_sq()
    I d == 25 {
        println("Complex Struct: OK")
        R 0
    }
    1
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "Expected exit code 0, got {}.\nstdout: {}\nstderr: {}",
        result.exit_code, result.stdout, result.stderr
    );
    assert!(
        result.stdout.contains("Complex Struct: OK"),
        "Expected stdout to contain 'Complex Struct: OK', got: {}",
        result.stdout
    );
}

// ==================== Float Printf Tests ====================

#[test]
fn test_float_printf_simple() {
    let source = r#"
F main() -> i64 {
    x := 3.14
    printf("%f\n", x)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("3.14"),
        "Expected stdout to contain '3.14', got: {}",
        result.stdout
    );
}

#[test]
fn test_float_printf_binop() {
    let source = r#"
F main() -> i64 {
    x := 3.14
    result := x + 1.0
    printf("result = %f\n", result)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("4.14"),
        "Expected stdout to contain '4.14', got: {}",
        result.stdout
    );
}

#[test]
fn test_float_printf_multiple_args() {
    let source = r#"
F main() -> i64 {
    a := 2.71828
    b := 3.14159
    printf("%f %f\n", a, b)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("2.71"),
        "Expected stdout to contain '2.71', got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("3.14"),
        "Expected stdout to contain '3.14', got: {}",
        result.stdout
    );
}

// ==================== f64 Array / Pointer Arithmetic ====================

#[test]
fn e2e_f64_array_access() {
    let source = r#"
F main() -> i64 {
    arr: *f64 = [1.5, 2.5, 3.5, 42.0]
    x := arr[3]
    printf("%f\n", x)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("42.0"),
        "Expected stdout to contain '42.0', got: {}",
        result.stdout
    );
}

#[test]
fn e2e_f64_array_mutation() {
    let source = r#"
F main() -> i64 {
    arr: *f64 = [0.0, 0.0, 0.0]
    arr[1] = 3.14
    printf("%f\n", arr[1])
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("3.14"),
        "Expected stdout to contain '3.14', got: {}",
        result.stdout
    );
}

#[test]
fn e2e_f64_array_sum() {
    let source = r#"
F main() -> i64 {
    arr: *f64 = [1.0, 2.0, 3.0]
    sum := arr[0] + arr[1] + arr[2]
    printf("%f\n", sum)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("6.0"),
        "Expected stdout to contain '6.0', got: {}",
        result.stdout
    );
}

#[test]
fn e2e_f64_array_with_variable_index() {
    let source = r#"
F main() -> i64 {
    arr: *f64 = [10.0, 20.0, 30.0, 40.0, 50.0]
    i := 3
    printf("%f\n", arr[i])
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("40.0"),
        "Expected stdout to contain '40.0', got: {}",
        result.stdout
    );
}

#[test]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "libm linking differs on Linux (-lm required)"
)]
fn test_float_printf_math_functions() {
    let source = r#"
F main() -> i64 {
    x := sqrt(4.0)
    printf("%f\n", x)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("2.0"),
        "Expected stdout to contain '2.0', got: {}",
        result.stdout
    );
}

// ===== String Interpolation Tests =====

#[test]
fn test_string_interp_basic() {
    let source = r#"
F main() -> i64 {
    name := "world"
    println("hello {name}")
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("hello world"),
        "Expected 'hello world', got: {}",
        result.stdout
    );
}

#[test]
fn test_string_interp_arithmetic() {
    let source = r#"
F main() -> i64 {
    x := 5
    println("x+1={x + 1}")
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("x+1=6"),
        "Expected 'x+1=6', got: {}",
        result.stdout
    );
}

#[test]
fn test_string_interp_escaped_braces() {
    let source = r#"
F main() -> i64 {
    println("literal {{braces}}")
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("literal {braces}"),
        "Expected 'literal {{braces}}', got: {}",
        result.stdout
    );
}

#[test]
fn test_string_interp_backward_compat() {
    let source = r#"
F main() -> i64 {
    println("x = {}", 42)
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("x = 42"),
        "Expected 'x = 42', got: {}",
        result.stdout
    );
}

#[test]
fn test_string_interp_multiple_exprs() {
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    println("{a} + {b} = {a + b}")
    0
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("10 + 20 = 30"),
        "Expected '10 + 20 = 30', got: {}",
        result.stdout
    );
}

// ===== Parameter Type Inference Tests =====

#[test]
fn test_param_type_infer_simple() {
    let source = r#"
F add(a, b) -> i64 = a + b
F main() -> i64 {
    R add(3, 4)
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn test_param_type_infer_block_body() {
    let source = r#"
F multiply(x, y) -> i64 {
    R x * y
}
F main() -> i64 {
    R multiply(5, 6)
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn test_param_type_infer_mixed() {
    let source = r#"
F mixed(a: i64, b) -> i64 = a + b
F main() -> i64 {
    R mixed(10, 20)
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn test_param_type_infer_recursive() {
    let source = r#"
F factorial(n) -> i64 {
    I n <= 1 { R 1 }
    R n * @(n - 1)
}
F main() -> i64 {
    I factorial(5) == 120 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_param_type_infer_multi_param() {
    let source = r#"
F clamp(val, lo, hi) -> i64 {
    I val < lo { R lo }
    I val > hi { R hi }
    R val
}
F main() -> i64 {
    I clamp(15, 0, 10) == 10 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

// ===== Return Type Inference Tests =====

#[test]
fn test_ret_type_infer_simple() {
    let source = r#"
F double(x: i64) { x * 2 }
F main() -> i64 {
    R double(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_ret_type_infer_block_body() {
    let source = r#"
F max(a: i64, b: i64) {
    I a > b { a } E { b }
}
F main() -> i64 {
    R max(10, 20)
}
"#;
    assert_exit_code(source, 20);
}

#[test]
fn test_ret_type_infer_with_return() {
    let source = r#"
F abs(n: i64) {
    I n < 0 { R 0 - n }
    R n
}
F main() -> i64 {
    I abs(0 - 7) == 7 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_ret_type_infer_both_param_and_ret() {
    // When all params and return type are inferred, explicit return type is needed
    // since params are unconstrained within the function body alone
    let source = r#"
F add(a, b) -> i64 { a + b }
F main() -> i64 {
    R add(17, 25)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_ret_type_infer_recursive() {
    // Recursive functions (using @) require explicit return type annotation
    let source = r#"
F fib(n: i64) {
    I n <= 1 { R n }
    R @(n - 1) + @(n - 2)
}
F main() -> i64 { R 0 }
"#;
    assert_compile_error(source);
}

#[test]
fn test_ret_type_infer_recursive_with_annotation() {
    // Recursive function with explicit return type works fine
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    R @(n - 1) + @(n - 2)
}
F main() -> i64 {
    I fib(10) == 55 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_ret_type_infer_nested_calls() {
    let source = r#"
F inc(x: i64) { x + 1 }
F triple(x: i64) { x * 3 }
F main() -> i64 {
    R triple(inc(13))
}
"#;
    assert_exit_code(source, 42);
}

// ===== Type Inference Safety Tests =====

// --- Positive tests: inference succeeds ---

#[test]
fn test_infer_from_binary_ops() {
    let source = r#"
F compute(a, b, c) -> i64 { a + b * c }
F main() -> i64 {
    R compute(1, 2, 3)
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn test_infer_from_comparison() {
    // Param 'x' inferred as i64 from comparison with literal 0
    let source = r#"
F check(x) -> i64 {
    I x > 0 { R 1 }
    R 0
}
F main() -> i64 {
    R check(5)
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn test_infer_mixed_explicit_inferred() {
    let source = r#"
F clamp_max(x: i64, limit) -> i64 {
    I x > limit { R limit }
    R x
}
F main() -> i64 {
    R clamp_max(50, 42)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_infer_with_if_else() {
    let source = r#"
F safe_div(a, b) -> i64 {
    I b == 0 { R 0 }
    R a / b
}
F main() -> i64 {
    R safe_div(84, 2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_infer_param_from_call_chain() {
    // Parameter type inferred from being passed to a typed function
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F apply(n) -> i64 { double(n) }
F main() -> i64 {
    R apply(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_infer_ret_from_body() {
    let source = r#"
F square(x: i64) { x * x }
F main() -> i64 {
    R square(6)
}
"#;
    assert_exit_code(source, 36);
}

#[test]
fn test_infer_param_from_string_concat() {
    let source = r#"
F greet(name) -> str { "hello " + name }
F main() -> i64 {
    s := greet("world")
    R 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_infer_f64_param() {
    // Param inferred as f64 from float operation
    let source = r#"
F half(x) -> f64 { x / 2.0 }
F main() -> i64 {
    R 0
}
"#;
    assert_exit_code(source, 0);
}

// --- Negative tests: inference correctly fails ---

#[test]
fn test_infer_fail_unconstrained_param() {
    // Parameter 'x' is never used, so its type cannot be inferred
    let source = r#"
F unused(x) -> i64 { 42 }
F main() -> i64 { R unused(1) }
"#;
    assert_compile_error(source);
}

#[test]
fn test_infer_fail_recursive_no_ret() {
    // Recursive function with @ but no return type annotation
    let source = r#"
F count(n: i64) {
    I n <= 0 { R 0 }
    R 1 + @(n - 1)
}
F main() -> i64 { R 0 }
"#;
    assert_compile_error(source);
}

#[test]
fn test_infer_fail_unconstrained_return() {
    // Both params and return unconstrained — can't determine types
    let source = r#"
F identity(x) { x }
F main() -> i64 { R 0 }
"#;
    assert_compile_error(source);
}

#[test]
fn test_infer_fail_all_unconstrained() {
    // All unconstrained: a, b, and return type
    let source = r#"
F swap_first(a, b) { a }
F main() -> i64 { R 0 }
"#;
    assert_compile_error(source);
}

// ===== Tilde Mut Abbreviation Tests =====

#[test]
fn test_tilde_mut_basic() {
    let source = r#"
F main() -> i64 {
    x := ~ 0;
    x += 10;
    x
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn test_tilde_mut_with_compound_assign() {
    let source = r#"
F main() -> i64 {
    counter := ~ 1;
    counter *= 3;
    counter += 2;
    counter
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn test_tilde_mut_backward_compat() {
    let source = r#"
F main() -> i64 {
    a := mut 5;
    a += 5;
    b := ~ 10;
    b += 10;
    I a == 10 && b == 20 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_tilde_bitwise_not_in_parens() {
    let source = r#"
F main() -> i64 {
    x := (~0);
    I x == -1 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

// ===== Pipe Operator Tests =====

#[test]
fn test_pipe_simple() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 {
    5 |> double
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn test_pipe_chained() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1
F main() -> i64 {
    3 |> double |> add_one
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn test_pipe_triple_chain() {
    let source = r#"
F inc(x: i64) -> i64 = x + 1
F double(x: i64) -> i64 = x * 2
F square(x: i64) -> i64 = x * x
F main() -> i64 {
    2 |> inc |> double |> square
}
"#;
    assert_exit_code(source, 36);
}

#[test]
fn test_pipe_in_binding() {
    let source = r#"
F negate(x: i64) -> i64 = -x
F main() -> i64 {
    result := 42 |> negate;
    I result == -42 { R 0 }
    R 1
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_pipe_with_tilde_mut() {
    let source = r#"
F double(x: i64) -> i64 = x * 2
F main() -> i64 {
    x := ~ (5 |> double);
    x += 1;
    x
}
"#;
    assert_exit_code(source, 11);
}
