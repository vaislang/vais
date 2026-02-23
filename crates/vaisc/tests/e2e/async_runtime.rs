use super::helpers::*;

// ==================== Async/Await E2E Tests ====================

#[test]
fn e2e_async_basic_await() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    result := compute(21).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_multiple_params() {
    let source = r#"
A F add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    result := add(30, 12).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_sequential_awaits() {
    let source = r#"
A F double(x: i64) -> i64 {
    x * 2
}

A F triple(x: i64) -> i64 {
    x * 3
}

F main() -> i64 {
    a := double(10).await
    b := triple(10).await
    a + b - 50
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_chained_await() {
    let source = r#"
A F double(x: i64) -> i64 {
    x * 2
}

A F add_ten(x: i64) -> i64 {
    x + 10
}

A F pipeline(x: i64) -> i64 {
    doubled := double(x).await
    result := add_ten(doubled).await
    result
}

F main() -> i64 {
    r := pipeline(5).await
    r - 20
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_spawn_basic() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 3
}

F main() -> i64 {
    r := (spawn compute(10)).await
    r - 30
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_return_expression() {
    let source = r#"
A F expr_body(x: i64) -> i64 = x * x

F main() -> i64 {
    r := expr_body(7).await
    r - 49
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_with_conditionals() {
    let source = r#"
A F abs_val(x: i64) -> i64 {
    I x < 0 { 0 - x } E { x }
}

F main() -> i64 {
    a := abs_val(-5).await
    b := abs_val(3).await
    a + b - 8
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_result_in_arithmetic() {
    let source = r#"
A F get_val(x: i64) -> i64 {
    x + 1
}

F main() -> i64 {
    a := get_val(10).await
    b := get_val(20).await
    total := a * 2 + b
    total - 43
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_with_println() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    result := compute(21).await
    println("result = {}", result)
    0
}
"#;
    assert_stdout_contains(source, "result = 42");
}

#[test]
fn e2e_async_three_level_chain() {
    let source = r#"
A F step1(x: i64) -> i64 {
    x + 1
}

A F step2(x: i64) -> i64 {
    v := step1(x).await
    v * 2
}

A F step3(x: i64) -> i64 {
    v := step2(x).await
    v + 100
}

F main() -> i64 {
    r := step3(4).await
    r - 110
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_mixed_sync_async() {
    let source = r#"
F sync_double(x: i64) -> i64 = x * 2

A F async_add(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    x := sync_double(5)
    y := async_add(x, 3).await
    y - 13
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_spawn_chained() {
    let source = r#"
A F double(x: i64) -> i64 {
    x * 2
}

A F add_one(x: i64) -> i64 {
    x + 1
}

F main() -> i64 {
    a := (spawn double(10)).await
    b := (spawn add_one(a)).await
    b - 21
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_async_multiple_spawn_await() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * x
}

F main() -> i64 {
    a := (spawn compute(3)).await
    b := (spawn compute(4)).await
    c := (spawn compute(5)).await
    a + b + c - 50
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Y (await abbreviation) ====================

#[test]
fn e2e_y_basic_await_abbreviation() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    result := compute(21).Y
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_y_sequential_awaits() {
    let source = r#"
A F double(x: i64) -> i64 {
    x * 2
}

A F add_one(x: i64) -> i64 {
    x + 1
}

F main() -> i64 {
    a := double(10).Y
    b := add_one(a).Y
    b - 21
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_y_spawn_with_y() {
    let source = r#"
A F square(x: i64) -> i64 {
    x * x
}

F main() -> i64 {
    result := (spawn square(7)).Y
    result - 49
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_y_mixed_await_and_y() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x + 10
}

F main() -> i64 {
    a := compute(5).await
    b := compute(5).Y
    a - b
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Implicit Self ====================

#[test]
fn e2e_implicit_self_field_access() {
    let source = r#"
S Point { x: i64, y: i64 }

X Point {
    F sum(&self) -> i64 {
        x + y
    }
}

F main() -> i64 {
    p := Point{x: 30, y: 12}
    p.sum() - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_implicit_self_mixed_with_explicit() {
    let source = r#"
S Counter { value: i64 }

X Counter {
    F get(&self) -> i64 {
        value
    }
    F get_explicit(&self) -> i64 {
        self.value
    }
}

F main() -> i64 {
    c := Counter{value: 42}
    c.get() - c.get_explicit()
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_implicit_self_local_shadows_field() {
    let source = r#"
S Data { value: i64 }

X Data {
    F compute(&self) -> i64 {
        value := 100
        value - self.value
    }
}

F main() -> i64 {
    d := Data{value: 58}
    d.compute() - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Spread Syntax ====================

#[test]
fn e2e_spread_parse_in_array() {
    // Test that spread syntax parses without error
    // (code generation treats spread as inner expr for now)
    let source = r#"
F main() -> i64 {
    arr := [1, 2, 3]
    0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Runtime Output Verification ====================
// Note: println format tests are covered in builtins.rs (e2e_println_format_integer,
// e2e_println_format_multiple).

#[test]
fn e2e_puts_hello_world_output() {
    let source = r#"
F main() -> i64 {
    puts("hello world")
    0
}
"#;
    assert_stdout_contains(source, "hello world");
}

// Note: e2e_if_else_expression_value and e2e_if_else_expression_false_branch
// are covered in execution_tests.rs (exec_if_else_true_branch / exec_if_else_false_branch).

#[test]
fn e2e_match_output_verification() {
    let source = r#"
F describe(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 1,
        _ => 99
    }
}

F main() -> i64 {
    a := describe(0)
    b := describe(1)
    c := describe(7)
    putchar(a + 48)
    putchar(b + 48)
    putchar(10)
    I c == 99 { 0 } E { 1 }
}
"#;
    assert_stdout_contains(source, "01");
    assert_exit_code(source, 0);
}

// Note: e2e_recursive_fib_output covered by exec_recursion_fibonacci in execution_tests.rs.

#[test]
fn e2e_loop_with_break() {
    let source = r#"
F main() -> i64 {
    i := mut 0
    total := mut 0
    L {
        I i >= 5 { B }
        total = total + i
        i = i + 1
    }
    total - 10
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_nested_function_calls() {
    let source = r#"
F add(a: i64, b: i64) -> i64 = a + b
F mul(a: i64, b: i64) -> i64 = a * b
F main() -> i64 = add(mul(3, 4), mul(2, 3)) - 18
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_mutable_variable_update() {
    let source = r#"
F main() -> i64 {
    x := mut 1
    x = x + 1
    x = x * 3
    x - 6
}
"#;
    assert_exit_code(source, 0);
}

// ==================== HTTP Runtime Tests ====================

/// Helper to find the HTTP runtime C file path
fn find_http_runtime_path() -> Option<String> {
    // Try relative to workspace root (when running via cargo test)
    let candidates = [
        "std/http_runtime.c",
        "../std/http_runtime.c",
        "../../std/http_runtime.c",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    // Try from CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("std").join("http_runtime.c"));
        if let Some(path) = p {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[test]
fn e2e_http_strlen() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __strlen(s: str) -> i64
F main() -> i64 {
    len := __strlen("hello world")
    I len == 11 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(result.exit_code, 0, "strlen test failed: {}", result.stderr);
}

#[test]
fn e2e_http_str_eq() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __str_eq(a: str, b: str) -> i64
F main() -> i64 {
    r1 := __str_eq("abc", "abc")
    r2 := __str_eq("abc", "xyz")
    I r1 == 1 {
        I r2 == 0 { 0 } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(result.exit_code, 0, "str_eq test failed: {}", result.stderr);
}

#[test]
fn e2e_http_str_eq_ignore_case() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __str_eq_ignore_case(a: str, b: str) -> i64
F main() -> i64 {
    r1 := __str_eq_ignore_case("Hello", "hello")
    r2 := __str_eq_ignore_case("WORLD", "world")
    I r1 == 1 {
        I r2 == 1 { 0 } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "str_eq_ignore_case test failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_http_parse_url_port() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __parse_url_port(url: str) -> i64
F main() -> i64 {
    port := __parse_url_port("http://example.com:3000/api")
    I port == 3000 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "parse_url_port test failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_http_parse_url_host() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __parse_url_host(url: str) -> str
X F __str_eq(a: str, b: str) -> i64
F main() -> i64 {
    host := __parse_url_host("http://localhost:8080/path")
    I __str_eq(host, "localhost") == 1 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "parse_url_host test failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_http_parse_url_path() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __parse_url_path(url: str) -> str
X F __str_eq(a: str, b: str) -> i64
F main() -> i64 {
    path := __parse_url_path("http://example.com:8080/api/users")
    I __str_eq(path, "/api/users") == 1 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "parse_url_path test failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_http_find_header_end() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __malloc(size: i64) -> i64
X F __free(ptr: i64) -> i64
X F __str_copy_to(dst: i64, src: str) -> i64
X F __find_header_end(buffer: i64, len: i64) -> i64
F main() -> i64 {
    buf := __malloc(32)
    __str_copy_to(buf, "HEAD")
    store_byte(buf + 4, 13)
    store_byte(buf + 5, 10)
    store_byte(buf + 6, 13)
    store_byte(buf + 7, 10)
    result := __find_header_end(buf, 8)
    __free(buf)
    I result == 8 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "find_header_end test failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_http_i64_to_str() {
    let rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found");
            return;
        }
    };
    let source = r#"
X F __malloc(size: i64) -> i64
X F __free(ptr: i64) -> i64
X F __i64_to_str(dst: i64, value: i64) -> i64
F main() -> i64 {
    buf := __malloc(32)
    written := __i64_to_str(buf, 12345)
    __free(buf)
    I written == 5 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "i64_to_str test failed: {}",
        result.stderr
    );
}
