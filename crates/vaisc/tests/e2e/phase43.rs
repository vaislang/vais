use super::helpers::*;

// ==================== Phase 43: Async Runtime E2E Tests ====================
// Phase 43 changes:
// 1. Yield TC: Always returns inner_type (not i64)
// 2. Await codegen: Text IR poll loop now includes sched_yield()

// ==================== Yield Tests ====================

#[test]
fn e2e_phase43_yield_returns_inner_type_i64() {
    let source = r#"
A F producer() -> i64 {
    yield 42
}

F main() -> i64 {
    result := producer().await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_yield_returns_inner_type_bool() {
    // Yield returns inner_type (bool) — async bool return with dynamic poll return type
    let source = r#"
A F check() -> bool {
    yield true
}

F main() -> i64 {
    result := check().await
    I result { R 0 } E { R 1 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_yield_expression() {
    let source = r#"
A F double(x: i64) -> i64 {
    yield (x * 2)
}

F main() -> i64 {
    result := double(21).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_yield_in_loop() {
    let source = r#"
A F sum(n: i64) -> i64 {
    total := mut 0
    L i:0..n {
        total = total + i
        yield total
    }
    total
}

F main() -> i64 {
    result := sum(10).await
    result - 45
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Async Basic Poll Tests ====================

#[test]
fn e2e_phase43_async_basic_poll() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x + 10
}

F main() -> i64 {
    result := compute(32).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_async_chained_poll() {
    let source = r#"
A F step1(x: i64) -> i64 {
    x + 5
}

A F step2(x: i64) -> i64 {
    step1(x).await
}

F main() -> i64 {
    result := step2(37).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Await Syntax Tests ====================

#[test]
fn e2e_phase43_async_y_syntax() {
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
fn e2e_phase43_async_nested_await() {
    let source = r#"
A F innermost(x: i64) -> i64 {
    x + 10
}

A F middle(x: i64) -> i64 {
    innermost(x).await
}

A F outer(x: i64) -> i64 {
    middle(x).await
}

F main() -> i64 {
    result := outer(32).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Async with Control Flow ====================

#[test]
fn e2e_phase43_async_with_if() {
    // Async function with early return via if/else — AsyncPollContext wraps return values
    let source = r#"
A F conditional(x: i64) -> i64 {
    I x > 0 {
        R x * 2
    } E {
        R 0
    }
}

F main() -> i64 {
    result := conditional(21).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_async_expression_body() {
    let source = r#"
A F compute(x: i64) -> i64 { x * 2 }

F main() -> i64 {
    result := compute(21).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Advanced Async Patterns ====================

#[test]
fn e2e_phase43_async_multiple_yields() {
    // Async generator with yield in loop + final return
    let source = r#"
A F generator(n: i64) -> i64 {
    L i:0..n {
        yield i
    }
    R n
}

F main() -> i64 {
    result := generator(42).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase43_async_return_early() {
    // Async function with early return + yield — AsyncPollContext handles both paths
    let source = r#"
A F check(x: i64) -> i64 {
    I x < 0 {
        R 0
    }
    yield x
}

F main() -> i64 {
    result := check(42).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Edge-case Negative Tests ====================
// These verify that the type checker correctly rejects invalid async usage.

#[test]
fn e2e_phase43_negative_await_on_non_future() {
    // await on a plain i64 should fail — not a Future<T>
    let source = r#"
F main() -> i64 {
    x := 42
    result := x.await
    result
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase43_negative_await_on_bool() {
    // await on bool should fail
    let source = r#"
F main() -> i64 {
    b := true
    result := b.await
    I result { R 0 } E { R 1 }
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase43_negative_await_on_string() {
    // await on str should fail
    let source = r#"
F main() -> i64 {
    s := "hello"
    result := s.await
    0
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase43_negative_double_await() {
    // Double await: Future<T>.await gives T, then T.await should fail
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    result := compute(21).await.await
    result
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase43_negative_yield_outside_async() {
    // yield in a non-async function currently compiles (no TC restriction)
    let source = r#"
F producer() -> i64 {
    yield 42
}

F main() -> i64 {
    result := producer()
    result - 42
}
"#;
    // yield in sync function compiles and executes — yield in sync context returns value directly
    assert_exit_code(source, 0);
}
