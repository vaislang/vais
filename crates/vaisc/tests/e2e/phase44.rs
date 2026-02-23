use super::helpers::*;

// ==================== Phase 44: Selfhost Cross-Verification E2E Tests ====================
// Phase 44 verifies that Phase 40-43 features compile and execute correctly.
// These tests use the same examples as cross-verify but run via assert_exit_code.

// ==================== Trait Bounds (Phase 40) ====================

#[test]
fn e2e_phase44_trait_bounds_cross_verify() {
    let source = r#"
W Summable {
    F sum(&self) -> i64
}

S Pair { a: i64, b: i64 }

X Pair: Summable {
    F sum(&self) -> i64 {
        self.a + self.b
    }
}

F identity<T>(x: T) -> T {
    x
}

F main() -> i64 {
    p := Pair { a: 20, b: 22 }
    result := p.sum()
    x := identity(result)
    x - 42
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Range Loops (Phase 41) ====================

#[test]
fn e2e_phase44_range_loop_cross_verify() {
    let source = r#"
F main() -> i64 {
    sum := mut 0
    L i:0..5 {
        sum = sum + i
    }

    sum2 := mut 0
    L j:1..=4 {
        sum2 = sum2 + j
    }

    count := mut 0
    L x:0..3 {
        L y:0..3 {
            count = count + 1
        }
    }

    result := sum + sum2 + count
    result - 29
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Closures (Phase 42) ====================

#[test]
fn e2e_phase44_closure_cross_verify() {
    let source = r#"
F main() -> i64 {
    add_one := |x: i64| x + 1
    r1 := add_one(5)

    offset := 10
    add_offset := move |x: i64| x + offset
    r2 := add_offset(3)

    multiply := |a: i64, b: i64| a * b
    r3 := multiply(7, 2)

    result := r1 + r2 + r3
    result - 33
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Async/Await (Phase 43) ====================

#[test]
fn e2e_phase44_async_cross_verify() {
    let source = r#"
A F compute(x: i64) -> i64 {
    x * 2
}

A F step1(x: i64) -> i64 {
    x + 5
}

A F chained(x: i64) -> i64 {
    step1(x).await
}

F main() -> i64 {
    result := compute(21).await
    r2 := chained(37).await
    (result + r2) - 84
}
"#;
    assert_exit_code(source, 0);
}
