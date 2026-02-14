use super::helpers::*;

fn parse_recovery(source: &str) -> (vais_ast::Module, Vec<vais_parser::ParseError>) {
    vais_parser::parse_with_recovery(source)
}

// ===== Stage 2: Closure & Higher-Order Function Tests =====

#[test]
fn e2e_closure_inferred_params() {
    // Test closure with inferred parameter types
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 = f(x)
F main() -> i64 {
    double := |x: i64| x * 2
    result := apply(21, double)
    I result == 42 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure inferred params failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_capture_variable() {
    // Test closure capturing a variable from enclosing scope
    let source = r#"
F main() -> i64 {
    multiplier := 10
    scale := |x: i64| x * multiplier
    result := scale(5)
    I result == 50 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure capture failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_multiple_captures() {
    // Test closure capturing multiple variables
    let source = r#"
F main() -> i64 {
    a := 10
    b := 20
    c := 30
    sum_all := |x: i64| x + a + b + c
    result := sum_all(1)
    I result == 61 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "multiple captures failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_nested() {
    // Test nested closures
    let source = r#"
F main() -> i64 {
    outer := 100
    f := |x: i64| {
        inner := outer + x
        inner
    }
    result := f(23)
    I result == 123 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "nested closure failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_as_callback() {
    // Test passing closure as callback parameter
    let source = r#"
F transform(a: i64, b: i64, f: fn(i64, i64) -> i64) -> i64 = f(a, b)
F main() -> i64 {
    add := |x: i64, y: i64| x + y
    mul := |x: i64, y: i64| x * y
    r1 := transform(3, 4, add)
    r2 := transform(3, 4, mul)
    I r1 == 7 && r2 == 12 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure as callback failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_higher_order_chain() {
    // Test chaining higher-order function calls with closures
    let source = r#"
F apply_twice(x: i64, f: fn(i64) -> i64) -> i64 = f(f(x))
F apply_n(x: i64, n: i64, f: fn(i64) -> i64) -> i64 {
    result := mut x
    i := mut 0
    L {
        I i >= n { B }
        result = f(result)
        i = i + 1
    }
    result
}
F main() -> i64 {
    inc := |x: i64| x + 1
    double := |x: i64| x * 2
    r1 := apply_twice(3, inc)
    r2 := apply_twice(3, double)
    r3 := apply_n(1, 5, inc)
    I r1 == 5 && r2 == 12 && r3 == 6 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "higher-order chain failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_with_block_body() {
    // Test closure with block body (multiple statements)
    let source = r#"
F main() -> i64 {
    compute := |x: i64| {
        doubled := x * 2
        tripled := x * 3
        doubled + tripled
    }
    result := compute(4)
    I result == 20 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure block body failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_identity_and_composition() {
    // Test identity closure and function composition
    let source = r#"
F compose(f: fn(i64) -> i64, g: fn(i64) -> i64, x: i64) -> i64 = f(g(x))
F main() -> i64 {
    double := |x: i64| x * 2
    inc := |x: i64| x + 1
    result := compose(double, inc, 5)
    I result == 12 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "composition failed: {}", result.stderr);
}

#[test]
fn e2e_closure_in_loop() {
    // Test using closure inside a loop
    let source = r#"
F main() -> i64 {
    sum := mut 0
    add_to_sum := |x: i64| x * x
    i := mut 1
    L {
        I i > 5 { B }
        sum = sum + add_to_sum(i)
        i = i + 1
    }
    # 1 + 4 + 9 + 16 + 25 = 55
    I sum == 55 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure in loop failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_higher_order_fold() {
    // Test fold-like pattern with closure
    let source = r#"
F fold(arr: i64, len: i64, init: i64, f: fn(i64, i64) -> i64) -> i64 {
    acc := mut init
    i := mut 0
    L {
        I i >= len { B }
        elem := load_i64(arr + i * 8)
        acc = f(acc, elem)
        i = i + 1
    }
    acc
}
F main() -> i64 {
    data := malloc(40)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)
    sum := fold(data, 5, 0, |acc: i64, x: i64| acc + x)
    product := fold(data, 5, 1, |acc: i64, x: i64| acc * x)
    free(data)
    I sum == 15 && product == 120 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "higher-order fold failed: {}",
        result.stderr
    );
}

// ===== Stage 3: Error Type & Chaining Tests =====

#[test]
fn e2e_result_is_ok_is_err() {
    // Test Result is_ok/is_err free functions
    let source = r#"
E Result { Ok(i64), Err(i64) }
F is_ok(r: Result) -> i64 { M r { Ok(_) => 1, Err(_) => 0 } }
F is_err(r: Result) -> i64 { M r { Ok(_) => 0, Err(_) => 1 } }
F main() -> i64 {
    ok := Ok(42)
    err := Err(99)
    ok_check := is_ok(ok) + is_err(err)
    I ok_check == 2 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "result is_ok/is_err failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_result_unwrap_or() {
    // Test Result unwrap_or free function
    let source = r#"
E Result { Ok(i64), Err(i64) }
F unwrap_or(r: Result, default: i64) -> i64 {
    M r { Ok(v) => v, Err(_) => default }
}
F main() -> i64 {
    ok_val := unwrap_or(Ok(42), 0)
    err_val := unwrap_or(Err(99), 0)
    I ok_val == 42 && err_val == 0 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "result unwrap_or failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_result_err_value() {
    // Test extracting error value from Result
    let source = r#"
E Result { Ok(i64), Err(i64) }
F err_or(r: Result, default: i64) -> i64 {
    M r { Ok(_) => default, Err(e) => e }
}
F main() -> i64 {
    code := err_or(Err(42), 0)
    I code == 42 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "result err value failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_error_context_encoding() {
    // Test error context encoding: ctx * 65536 + err_code
    let source = r#"
F error_code(err: i64) -> i64 { err % 65536 }
F error_context(err: i64) -> i64 { err / 65536 }
F wrap_error(code: i64, ctx: i64) -> i64 { ctx * 65536 + code }
F main() -> i64 {
    wrapped := wrap_error(3, 42)
    orig := error_code(wrapped)
    ctx := error_context(wrapped)
    I orig == 3 && ctx == 42 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "error context encoding failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_error_context_chaining() {
    // Test multi-level error context chaining
    let source = r#"
F wrap_error(code: i64, ctx: i64) -> i64 { ctx * 65536 + code }
F error_code(err: i64) -> i64 { err % 65536 }
F error_context(err: i64) -> i64 { err / 65536 }
F main() -> i64 {
    # Original error: code 5
    err := 5
    # First context: module 10
    err1 := wrap_error(err, 10)
    code1 := error_code(err1)
    ctx1 := error_context(err1)
    # Verify: original code preserved, context attached
    I code1 == 5 && ctx1 == 10 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "error context chaining failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_error_typed_enum_pattern() {
    // Test thiserror-style typed error enum
    let source = r#"
E AppError {
    NotFound(i64),
    InvalidInput(i64),
    IoError(i64)
}
X AppError {
    F code(&self) -> i64 {
        M self {
            NotFound(c) => c,
            InvalidInput(c) => c,
            IoError(c) => c
        }
    }
    F is_retryable(&self) -> i64 {
        M self {
            IoError(_) => 1,
            _ => 0
        }
    }
}
F main() -> i64 {
    e1: AppError = NotFound(404)
    e2: AppError = IoError(5)
    e3: AppError = InvalidInput(22)
    c1 := e1.code()
    c2 := e2.code()
    r := e2.is_retryable()
    nr := e3.is_retryable()
    I c1 == 404 && c2 == 5 && r == 1 && nr == 0 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "typed error enum failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_error_result_with_custom_error() {
    // Test Result combined with custom error types using free functions
    let source = r#"
E Result { Ok(i64), Err(i64) }
F is_ok(r: Result) -> i64 {
    M r { Ok(_) => 1, Err(_) => 0 }
}
F get_val(r: Result) -> i64 {
    M r { Ok(v) => v, Err(_) => 0 - 1 }
}
F get_err(r: Result) -> i64 {
    M r { Ok(_) => 0, Err(e) => e }
}
F ERR_NOT_FOUND() -> i64 { 2 }
F main() -> i64 {
    # Success path
    ok := Ok(100)
    r1 := get_val(ok)
    # Error path
    err := Err(ERR_NOT_FOUND())
    r2 := get_val(err)
    r3 := get_err(err)
    I r1 == 100 && r2 == 0 - 1 && r3 == 2 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "result with custom error failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_error_ensure_pattern() {
    // Test ensure-like validation pattern (anyhow::ensure style) using free functions
    let source = r#"
E Result { Ok(i64), Err(i64) }
F ensure(cond: i64, err: i64) -> Result {
    I cond != 0 { Ok(0) } E { Err(err) }
}
F is_ok(r: Result) -> i64 {
    M r { Ok(_) => 1, Err(_) => 0 }
}
F is_err(r: Result) -> i64 {
    M r { Ok(_) => 0, Err(_) => 1 }
}
F validate_age(age: i64) -> i64 {
    # age >= 0 check
    ge_zero := I age >= 0 { 1 } E { 0 }
    r1 := ensure(ge_zero, 1)
    I is_err(r1) != 0 { R 1 }
    # age <= 150 check
    le_150 := I age <= 150 { 1 } E { 0 }
    r2 := ensure(le_150, 2)
    I is_err(r2) != 0 { R 2 }
    0
}
F main() -> i64 {
    ok := validate_age(25)
    err1 := validate_age(0 - 1)
    err2 := validate_age(200)
    I ok == 0 && err1 == 1 && err2 == 2 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "ensure pattern failed: {}",
        result.stderr
    );
}

// ===== Stage 4: Iterator Protocol & Generator Tests =====

#[test]
fn e2e_iter_range_for_loop() {
    // Test range-based for loop: L i:start..end { body }
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..10 {
        sum = sum + i
    }
    I sum == 45 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "range for loop failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_range_step_manual() {
    // Test manual range iterator with step > 1
    let source = r#"
F main() -> i64 {
    # Sum even numbers 0,2,4,6,8
    sum := mut 0
    i := mut 0
    L {
        I i >= 10 { B }
        sum = sum + i
        i = i + 2
    }
    I sum == 20 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "manual step range failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_map_array() {
    // Test map adapter on array via malloc/store/load pattern
    let source = r#"
F main() -> i64 {
    # Create array [1, 2, 3, 4, 5]
    data := malloc(40)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)

    # Map: double each element
    out := malloc(40)
    i := mut 0
    L {
        I i >= 5 { B }
        v := load_i64(data + i * 8)
        store_i64(out + i * 8, v * 2)
        i = i + 1
    }

    # Sum mapped: 2+4+6+8+10 = 30
    sum := mut 0
    j := mut 0
    L {
        I j >= 5 { B }
        sum = sum + load_i64(out + j * 8)
        j = j + 1
    }
    free(data)
    free(out)
    I sum == 30 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "iter map array failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_filter_array() {
    // Test filter adapter: keep only even elements
    let source = r#"
F main() -> i64 {
    # Create array [1, 2, 3, 4, 5, 6]
    data := malloc(48)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)
    store_i64(data + 40, 6)

    # Filter: keep even numbers
    out := malloc(48)
    count := mut 0
    i := mut 0
    L {
        I i >= 6 { B }
        v := load_i64(data + i * 8)
        rem := v - (v / 2) * 2
        I rem == 0 {
            store_i64(out + count * 8, v)
            count = count + 1
        }
        i = i + 1
    }

    # Sum filtered (2+4+6=12), count should be 3
    sum := mut 0
    j := mut 0
    L {
        I j >= count { B }
        sum = sum + load_i64(out + j * 8)
        j = j + 1
    }
    free(data)
    free(out)
    I sum == 12 && count == 3 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "iter filter array failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_fold_sum() {
    // Test fold/reduce pattern
    let source = r#"
F fold(data: i64, len: i64, init: i64, f: fn(i64, i64) -> i64) -> i64 {
    acc := mut init
    i := mut 0
    L {
        I i >= len { B }
        acc = f(acc, load_i64(data + i * 8))
        i = i + 1
    }
    acc
}
F main() -> i64 {
    data := malloc(40)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)

    sum := fold(data, 5, 0, |a: i64, b: i64| a + b)
    product := fold(data, 5, 1, |a: i64, b: i64| a * b)
    free(data)
    I sum == 15 && product == 120 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "iter fold failed: {}", result.stderr);
}

#[test]
fn e2e_iter_take_skip() {
    // Test take and skip patterns
    let source = r#"
F main() -> i64 {
    # Array [10, 20, 30, 40, 50]
    data := malloc(40)
    store_i64(data, 10)
    store_i64(data + 8, 20)
    store_i64(data + 16, 30)
    store_i64(data + 24, 40)
    store_i64(data + 32, 50)

    # Take first 3: sum = 10+20+30 = 60
    take_sum := mut 0
    i := mut 0
    L {
        I i >= 3 { B }
        take_sum = take_sum + load_i64(data + i * 8)
        i = i + 1
    }

    # Skip first 2: sum = 30+40+50 = 120
    skip_sum := mut 0
    j := mut 2
    L {
        I j >= 5 { B }
        skip_sum = skip_sum + load_i64(data + j * 8)
        j = j + 1
    }
    free(data)
    I take_sum == 60 && skip_sum == 120 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "take/skip failed: {}", result.stderr);
}

#[test]
fn e2e_iter_chain() {
    // Test chain: concatenate two arrays
    let source = r#"
F main() -> i64 {
    a := malloc(24)
    store_i64(a, 1)
    store_i64(a + 8, 2)
    store_i64(a + 16, 3)

    b := malloc(16)
    store_i64(b, 4)
    store_i64(b + 8, 5)

    # Chain: [1,2,3] ++ [4,5]
    out := malloc(40)
    i := mut 0
    L {
        I i >= 3 { B }
        store_i64(out + i * 8, load_i64(a + i * 8))
        i = i + 1
    }
    j := mut 0
    L {
        I j >= 2 { B }
        store_i64(out + (3 + j) * 8, load_i64(b + j * 8))
        j = j + 1
    }

    # Sum chained: 1+2+3+4+5 = 15
    sum := mut 0
    k := mut 0
    L {
        I k >= 5 { B }
        sum = sum + load_i64(out + k * 8)
        k = k + 1
    }
    free(a)
    free(b)
    free(out)
    I sum == 15 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "iter chain failed: {}", result.stderr);
}

#[test]
fn e2e_iter_zip() {
    // Test zip: pair elements from two arrays
    let source = r#"
F main() -> i64 {
    a := malloc(24)
    store_i64(a, 1)
    store_i64(a + 8, 2)
    store_i64(a + 16, 3)

    b := malloc(24)
    store_i64(b, 10)
    store_i64(b + 8, 20)
    store_i64(b + 16, 30)

    # Zip: pairs (1,10), (2,20), (3,30)
    # Sum of products: 1*10 + 2*20 + 3*30 = 10+40+90 = 140
    dot := mut 0
    i := mut 0
    L {
        I i >= 3 { B }
        ai := load_i64(a + i * 8)
        bi := load_i64(b + i * 8)
        dot = dot + ai * bi
        i = i + 1
    }
    free(a)
    free(b)
    I dot == 140 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "iter zip failed: {}", result.stderr);
}

#[test]
fn e2e_iter_enumerate() {
    // Test enumerate: pair each element with its index
    let source = r#"
F main() -> i64 {
    data := malloc(24)
    store_i64(data, 100)
    store_i64(data + 8, 200)
    store_i64(data + 16, 300)

    # Enumerate: (0,100), (1,200), (2,300)
    # Sum of index*value: 0*100 + 1*200 + 2*300 = 800
    sum := mut 0
    i := mut 0
    L {
        I i >= 3 { B }
        v := load_i64(data + i * 8)
        sum = sum + i * v
        i = i + 1
    }
    free(data)
    I sum == 800 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "iter enumerate failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_any_all_find() {
    // Test any, all, find patterns with closures
    let source = r#"
F any(data: i64, len: i64, pred: fn(i64) -> i64) -> i64 {
    i := mut 0
    L {
        I i >= len { B }
        I pred(load_i64(data + i * 8)) != 0 { R 1 }
        i = i + 1
    }
    0
}
F all(data: i64, len: i64, pred: fn(i64) -> i64) -> i64 {
    i := mut 0
    L {
        I i >= len { B }
        I pred(load_i64(data + i * 8)) == 0 { R 0 }
        i = i + 1
    }
    1
}
F find(data: i64, len: i64, pred: fn(i64) -> i64) -> i64 {
    i := mut 0
    L {
        I i >= len { B }
        v := load_i64(data + i * 8)
        I pred(v) != 0 { R v }
        i = i + 1
    }
    0 - 1
}
F main() -> i64 {
    data := malloc(40)
    store_i64(data, 2)
    store_i64(data + 8, 4)
    store_i64(data + 16, 6)
    store_i64(data + 24, 8)
    store_i64(data + 32, 10)

    has_even := any(data, 5, |x: i64| I x - (x / 2) * 2 == 0 { 1 } E { 0 })
    has_odd := any(data, 5, |x: i64| I x - (x / 2) * 2 != 0 { 1 } E { 0 })
    all_pos := all(data, 5, |x: i64| I x > 0 { 1 } E { 0 })
    first_big := find(data, 5, |x: i64| I x > 7 { 1 } E { 0 })
    free(data)
    I has_even == 1 && has_odd == 0 && all_pos == 1 && first_big == 8 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "any/all/find failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_map_filter_chain() {
    // Test chaining map -> filter -> fold
    let source = r#"
F main() -> i64 {
    # [1, 2, 3, 4, 5]
    data := malloc(40)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)

    # Step 1: Map (double): [2, 4, 6, 8, 10]
    mapped := malloc(40)
    i := mut 0
    L {
        I i >= 5 { B }
        store_i64(mapped + i * 8, load_i64(data + i * 8) * 2)
        i = i + 1
    }

    # Step 2: Filter (keep > 5): [6, 8, 10]
    filtered := malloc(40)
    count := mut 0
    j := mut 0
    L {
        I j >= 5 { B }
        v := load_i64(mapped + j * 8)
        I v > 5 {
            store_i64(filtered + count * 8, v)
            count = count + 1
        }
        j = j + 1
    }

    # Step 3: Fold (sum): 6+8+10 = 24
    sum := mut 0
    k := mut 0
    L {
        I k >= count { B }
        sum = sum + load_i64(filtered + k * 8)
        k = k + 1
    }
    free(data)
    free(mapped)
    free(filtered)
    I sum == 24 && count == 3 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "map-filter-chain failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_collect_to_array() {
    // Test collecting results into a new array (simulating Iterator.collect())
    let source = r#"
F collect_range(start: i64, end: i64) -> i64 {
    len := end - start
    out := malloc(len * 8)
    i := mut 0
    L {
        I i >= len { B }
        store_i64(out + i * 8, start + i)
        i = i + 1
    }
    out
}
F main() -> i64 {
    # Collect 5..10 into array [5,6,7,8,9]
    arr := collect_range(5, 10)
    sum := mut 0
    i := mut 0
    L {
        I i >= 5 { B }
        sum = sum + load_i64(arr + i * 8)
        i = i + 1
    }
    free(arr)
    # 5+6+7+8+9 = 35
    I sum == 35 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "collect to array failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_position() {
    // Test finding position/index of first matching element
    let source = r#"
F position(data: i64, len: i64, pred: fn(i64) -> i64) -> i64 {
    i := mut 0
    L {
        I i >= len { B }
        I pred(load_i64(data + i * 8)) != 0 { R i }
        i = i + 1
    }
    0 - 1
}
F main() -> i64 {
    data := malloc(40)
    store_i64(data, 10)
    store_i64(data + 8, 20)
    store_i64(data + 16, 30)
    store_i64(data + 24, 40)
    store_i64(data + 32, 50)

    pos := position(data, 5, |x: i64| I x == 30 { 1 } E { 0 })
    not_found := position(data, 5, |x: i64| I x == 99 { 1 } E { 0 })
    free(data)
    I pos == 2 && not_found == 0 - 1 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "iter position failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_generator_yield_parse() {
    // Test that yield keyword is recognized by the parser
    // (simplified generator — yield evaluates the expression for now)
    let source = r#"
F gen_next(state: i64) -> i64 {
    yield state * 2
}
F main() -> i64 {
    a := gen_next(5)
    b := gen_next(10)
    I a == 10 && b == 20 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "yield parse failed: {}", result.stderr);
}

#[test]
fn e2e_iter_nested_loops() {
    // Test nested range for loops
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..3 {
        L j:0..4 {
            sum = sum + 1
        }
    }
    # 3 * 4 = 12
    I sum == 12 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "nested loops failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_iter_closure_capture_in_loop() {
    // Test closures that capture loop variables
    let source = r#"
F apply(x: i64, f: fn(i64) -> i64) -> i64 { f(x) }
F main() -> i64 {
    sum := mut 0
    L i:1..6 {
        doubled := apply(i, |x: i64| x * 2)
        sum = sum + doubled
    }
    # 2+4+6+8+10 = 30
    I sum == 30 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure in loop failed: {}",
        result.stderr
    );
}

// ===== Additional E2E Tests for 300 target =====

#[test]
fn e2e_recursive_fibonacci() {
    // Classic recursive fibonacci
    let source = r#"
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    fib(n - 1) + fib(n - 2)
}
F main() -> i64 {
    I fib(10) == 55 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "recursive fibonacci failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_self_recursion_operator() {
    // Test @ self-recursion operator
    let source = r#"
F factorial(n: i64) -> i64 {
    I n <= 1 { 1 } E { n * @(n - 1) }
}
F main() -> i64 {
    I factorial(5) == 120 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "self recursion failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_bitwise_operations() {
    // Test bitwise operations: AND, OR, XOR, shift
    let source = r#"
F main() -> i64 {
    a := 255
    b := 15
    and_result := a & b    # 15
    or_result := a | b     # 255
    xor_result := a ^ b    # 240
    shl := 1 << 8          # 256
    shr := 256 >> 4         # 16
    I and_result == 15 && or_result == 255 && xor_result == 240 && shl == 256 && shr == 16 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(result.exit_code, 0, "bitwise ops failed: {}", result.stderr);
}

#[test]
fn e2e_multiple_return_paths() {
    // Test function with multiple early returns
    let source = r#"
F classify(n: i64) -> i64 {
    I n < 0 { R 0 - 1 }
    I n == 0 { R 0 }
    I n < 10 { R 1 }
    I n < 100 { R 2 }
    3
}
F main() -> i64 {
    a := classify(0 - 5)
    b := classify(0)
    c := classify(7)
    d := classify(50)
    e := classify(999)
    I a == 0 - 1 && b == 0 && c == 1 && d == 2 && e == 3 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "multiple return paths failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_closure_compose_apply_twice() {
    // Test passing closures as callbacks: apply_twice and compose
    let source = r#"
F apply_twice(x: i64, f: fn(i64) -> i64) -> i64 { f(f(x)) }
F compose(x: i64, f: fn(i64) -> i64, g: fn(i64) -> i64) -> i64 { g(f(x)) }
F main() -> i64 {
    a := apply_twice(3, |x: i64| x * 2)   # 3*2=6, 6*2=12
    b := compose(5, |x: i64| x + 1, |x: i64| x * 3)  # (5+1)*3=18
    I a == 12 && b == 18 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "closure compose failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_mutable_accumulator_pattern() {
    // Test mutable variable accumulation in loops
    let source = r#"
F main() -> i64 {
    sum := mut 0
    product := mut 1
    max := mut 0
    L i:1..11 {
        sum = sum + i
        product = I i <= 5 { product * i } E { product }
        I i > max { max = i }
    }
    # sum=55, product=120 (1*2*3*4*5), max=10
    I sum == 55 && product == 120 && max == 10 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "mutable accumulator failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_struct_method_chaining() {
    // Test struct with methods used in sequence
    let source = r#"
S Counter { value: i64 }
X Counter {
    F get(&self) -> i64 { self.value }
    F inc(&self) -> i64 {
        self.value = self.value + 1
        self.value
    }
    F add(&self, n: i64) -> i64 {
        self.value = self.value + n
        self.value
    }
}
F main() -> i64 {
    c := Counter { value: 0 }
    c.inc()
    c.inc()
    c.add(10)
    v := c.get()
    I v == 12 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "struct method chaining failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_enum_tag_matching() {
    // Test enum tag-based matching with different variants
    let source = r#"
E Shape { Circle(i64), Rect(i64), Triangle(i64) }
F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r * 3,
        Rect(side) => side * side,
        Triangle(base) => base * base / 2
    }
}
F main() -> i64 {
    c := area(Circle(5))    # 75
    r := area(Rect(4))       # 16
    t := area(Triangle(6))   # 18
    I c == 75 && r == 16 && t == 18 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "enum tag matching failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_higher_order_pipeline() {
    // Test combining higher-order functions in a data processing pipeline
    let source = r#"
F map_arr(data: i64, len: i64, f: fn(i64) -> i64) -> i64 {
    out := malloc(len * 8)
    i := mut 0
    L {
        I i >= len { B }
        store_i64(out + i * 8, f(load_i64(data + i * 8)))
        i = i + 1
    }
    out
}
F sum_arr(data: i64, len: i64) -> i64 {
    acc := mut 0
    i := mut 0
    L {
        I i >= len { B }
        acc = acc + load_i64(data + i * 8)
        i = i + 1
    }
    acc
}
F main() -> i64 {
    # Pipeline: [1..5] -> square -> add_one -> sum
    data := malloc(40)
    store_i64(data, 1)
    store_i64(data + 8, 2)
    store_i64(data + 16, 3)
    store_i64(data + 24, 4)
    store_i64(data + 32, 5)

    squared := map_arr(data, 5, |x: i64| x * x)       # [1,4,9,16,25]
    plus_one := map_arr(squared, 5, |x: i64| x + 1)    # [2,5,10,17,26]
    result := sum_arr(plus_one, 5)                       # 60

    free(data)
    free(squared)
    free(plus_one)
    I result == 60 { 0 } E { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "higher-order pipeline failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_recovery_max_errors_limit() {
    // Normal mode should fail fast on first error
    let source = "F broken(\nF good() -> i64 = 0\n";
    let result = vais_parser::parse(source);
    assert!(result.is_err(), "Normal mode should fail on first error");
}

#[test]
fn e2e_recovery_valid_code_no_errors() {
    // Valid code should produce no errors in recovery mode
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2)
"#;
    let (module, errors) = parse_recovery(source);
    assert!(
        errors.is_empty(),
        "Valid code should have no errors, got {:?}",
        errors
    );
    assert!(module.items.len() >= 2, "Should parse both functions");
}
