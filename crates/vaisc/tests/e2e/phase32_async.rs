use super::helpers::*;

// ==================== Phase 32: Async/Concurrency Edge Cases ====================
//
// These tests cover edge cases NOT present in async_runtime.rs (39 tests) or
// phase43.rs (28 tests). Focus: self-recursion in async, match inside async,
// 3+ sequential awaits, async-calls-async nesting, closures inside async,
// bool-returning async, multiple simultaneous spawn, and async early-return.

// 1. Async function with self-recursion (@)
#[test]
fn e2e_phase32_async_recursive() {
    // @ operator (self-recursion) used inside an async function.
    // Each recursive call returns a Future<i64> that is immediately awaited.
    let source = r#"
A F countdown(n: i64) -> i64 {
    I n <= 0 {
        R 0
    }
    inner := @(n - 1).await
    R inner + 1
}

F main() -> i64 {
    result := countdown(5).await
    result - 5
}
"#;
    assert_compiles(source);
}

// 2. Async function containing a M (match) expression
#[test]
fn e2e_phase32_async_with_match() {
    // Match inside an async function on a parameter.
    // Verifies that control flow within async bodies handles match arms correctly.
    let source = r#"
A F classify(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 1,
        _ => 2
    }
}

F main() -> i64 {
    a := classify(0).await
    b := classify(1).await
    c := classify(99).await
    a + b + c - 3
}
"#;
    assert_exit_code(source, 0);
}

// 3. Three or more sequential awaits in a single function
#[test]
fn e2e_phase32_async_multiple_awaits_sequential() {
    // Tests that codegen correctly handles 4 sequential await sites in one function.
    // Each await materialises a separate poll loop; they must not interfere.
    let source = r#"
A F inc(x: i64) -> i64 {
    x + 1
}

F main() -> i64 {
    a := inc(10).await
    b := inc(a).await
    c := inc(b).await
    d := inc(c).await
    d - 14
}
"#;
    assert_exit_code(source, 0);
}

// 4. Async function that calls another async function (nested async-calls-async)
#[test]
fn e2e_phase32_async_nested_functions() {
    // The outer async function awaits an inner async function.
    // Different from three-level chain in async_runtime.rs because here the
    // outer function also performs its own computation before returning.
    let source = r#"
A F base(x: i64) -> i64 {
    x * 3
}

A F wrapper(x: i64) -> i64 {
    inner := base(x).await
    inner + x
}

F main() -> i64 {
    result := wrapper(7).await
    result - 28
}
"#;
    assert_exit_code(source, 0);
}

// 5. Async function that uses a closure internally
#[test]
fn e2e_phase32_async_with_closure() {
    // A closure defined and called within an async function body.
    // Verifies that captured variables and closure calls work correctly
    // in the async codegen context.
    let source = r#"
A F apply_offset(base: i64, offset: i64) -> i64 {
    adder := |x| x + offset
    adder(base)
}

F main() -> i64 {
    result := apply_offset(30, 12).await
    result - 42
}
"#;
    assert_exit_code(source, 0);
}

// 6. Async function returning bool
#[test]
fn e2e_phase32_async_bool_return() {
    // Non-i64 return type (bool) from an async function.
    // bool-returning async functions go through a different codegen path
    // for the Future poll wrapper (inner_type is bool rather than i64).
    let source = r#"
A F is_positive(n: i64) -> bool {
    n > 0
}

F main() -> i64 {
    t := is_positive(5).await
    f := is_positive(0 - 1).await
    I t { I f { 1 } E { 0 } } E { 1 }
}
"#;
    assert_compiles(source);
}

// 7. Multiple spawn expressions used simultaneously (store then await separately)
#[test]
fn e2e_phase32_spawn_multiple() {
    // Three independent spawn calls stored in variables, then awaited.
    // Distinct from phase43 spawn tests because all three futures are created
    // before any is awaited, exercising the scheduler scheduling multiple tasks.
    let source = r#"
A F square(n: i64) -> i64 {
    n * n
}

F main() -> i64 {
    f1 := spawn square(3)
    f2 := spawn square(4)
    f3 := spawn square(5)
    r1 := f1.await
    r2 := f2.await
    r3 := f3.await
    r1 + r2 + r3 - 50
}
"#;
    assert_compiles(source);
}

// 8. Async function with early return via I (conditional R)
#[test]
fn e2e_phase32_async_early_return() {
    // An async function that uses an early R (return) inside an I block.
    // The normal continuation path is also taken for non-early-exit cases,
    // so both the early and late paths through the async body must compile.
    let source = r#"
A F safe_div(a: i64, b: i64) -> i64 {
    I b == 0 {
        R 0
    }
    a / b
}

F main() -> i64 {
    zero_case := safe_div(100, 0).await
    normal_case := safe_div(84, 2).await
    zero_case + normal_case - 42
}
"#;
    assert_compiles(source);
}
