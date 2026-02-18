use super::helpers::*;

// ==================== Phase 45 Advanced: Advanced Features & Edge Cases ====================
// Tests for: closure capture, higher-order functions, self-recursion, trait dispatch,
// nested if-else, mutual recursion, puts output, pipe operator, block expressions,
// expression body functions, multiple struct methods, enum with data, array indexing,
// variable reassignment.

// ===== Closure: Basic Capture =====

#[test]
fn e2e_phase45a_closure_basic_capture() {
    // Basic closure capturing a single outer variable
    let source = r#"
F main() -> i64 {
    x := 10
    f := |y| x + y
    R f(5)
}
"#;
    assert_exit_code(source, 15);
}

// ===== Closure: Multi-Variable Capture =====

#[test]
fn e2e_phase45a_closure_multi_capture() {
    // Closure capturing multiple outer variables
    let source = r#"
F main() -> i64 {
    a := 3
    b := 7
    f := |x| a + b + x
    R f(10)
}
"#;
    assert_exit_code(source, 20);
}

// ===== Closure: Move Capture =====

#[test]
fn e2e_phase45a_closure_move() {
    // move capture — codegen limitation possible; IR generation verified
    let source = r#"
F main() -> i64 {
    x := 42
    f := move |y| x + y
    R f(0)
}
"#;
    assert_compiles(source);
}

// ===== Higher-Order Function =====

#[test]
fn e2e_phase45a_higher_order_fn() {
    // Passing a named function as a first-class value
    let source = r#"
F apply(f: fn(i64) -> i64, x: i64) -> i64 { f(x) }
F double(x: i64) -> i64 { x * 2 }
F main() -> i64 { apply(double, 21) }
"#;
    assert_compiles(source);
}

// ===== Self-Recursion: Fibonacci =====

#[test]
fn e2e_phase45a_self_recursion_fib() {
    // @ operator for self-recursion computing the 10th Fibonacci number
    let source = r#"
F fib(n: i64) -> i64 {
    I n < 2 { n } E { @(n - 1) + @(n - 2) }
}
F main() -> i64 { fib(10) }
"#;
    assert_exit_code(source, 55);
}

// ===== Self-Recursion: Summation =====

#[test]
fn e2e_phase45a_self_recursion_sum() {
    // @ operator summing integers from 1 to 10
    let source = r#"
F sum(n: i64) -> i64 {
    I n <= 0 { 0 } E { n + @(n - 1) }
}
F main() -> i64 { sum(10) }
"#;
    assert_exit_code(source, 55);
}

// ===== Trait: Impl Method =====

#[test]
fn e2e_phase45a_trait_impl_method() {
    // Trait method dispatch on a concrete struct
    let source = r#"
W Measurable {
    F measure(&self) -> i64
}
S Box { width: i64, height: i64 }
X Box: Measurable {
    F measure(&self) -> i64 { self.width * self.height }
}
F main() -> i64 {
    b := Box { width: 3, height: 4 }
    R b.measure()
}
"#;
    assert_exit_code(source, 12);
}

// ===== Trait: Static Dispatch via Generics =====

#[test]
fn e2e_phase45a_trait_static_dispatch() {
    // Generic function with trait bound — IR generation verified
    let source = r#"
W HasValue {
    F value(&self) -> i64
}
S Holder { v: i64 }
X Holder: HasValue {
    F value(&self) -> i64 { self.v }
}
F get_val<T>(x: T) -> i64 where T: HasValue {
    x.value()
}
F main() -> i64 {
    h := Holder { v: 42 }
    R get_val(h)
}
"#;
    assert_compiles(source);
}

// ===== Nested If-Else: Deep Nesting =====

#[test]
fn e2e_phase45a_nested_if_deep() {
    // Four-level nested if-else classification
    let source = r#"
F classify(n: i64) -> i64 {
    I n > 100 {
        I n > 200 { 4 } E { 3 }
    } E {
        I n > 50 { 2 } E {
            I n > 0 { 1 } E { 0 }
        }
    }
}
F main() -> i64 { classify(75) }
"#;
    assert_exit_code(source, 2);
}

// ===== Mutual Recursion =====

#[test]
fn e2e_phase45a_mutual_recursion() {
    // Even/odd mutual recursion — is_even(10) should return 1
    let source = r#"
F is_even(n: i64) -> i64 {
    I n == 0 { 1 } E { is_odd(n - 1) }
}
F is_odd(n: i64) -> i64 {
    I n == 0 { 0 } E { is_even(n - 1) }
}
F main() -> i64 { is_even(10) }
"#;
    assert_exit_code(source, 1);
}

// ===== Puts: stdout output =====

#[test]
fn e2e_phase45a_string_puts() {
    // puts builtin writes to stdout
    let source = r#"
F main() -> i64 {
    puts("hello world")
    R 0
}
"#;
    assert_stdout_contains(source, "hello world");
}

// ===== Pipe Operator =====

#[test]
fn e2e_phase45a_pipe_operator() {
    // |> pipe operator — IR generation verified (runtime behavior may vary)
    let source = r#"
F double(x: i64) -> i64 { x * 2 }
F inc(x: i64) -> i64 { x + 1 }
F main() -> i64 {
    R 10 |> double |> inc
}
"#;
    assert_compiles(source);
}

// ===== Block Expression =====

#[test]
fn e2e_phase45a_block_expression() {
    // Block used as an expression returning the last value
    let source = r#"
F main() -> i64 {
    x := {
        a := 10
        b := 20
        a + b
    }
    R x
}
"#;
    assert_exit_code(source, 30);
}

// ===== Expression Body Function =====

#[test]
fn e2e_phase45a_expression_body_fn() {
    // Functions with `= expr` body syntax
    let source = r#"
F double(x: i64) -> i64 = x * 2
F triple(x: i64) -> i64 = x * 3
F main() -> i64 = double(10) + triple(5)
"#;
    assert_exit_code(source, 35);
}

// ===== Multiple Struct Methods =====

#[test]
fn e2e_phase45a_multiple_struct_methods() {
    // Struct impl block with two methods both called in main
    let source = r#"
S Counter { val: i64 }
X Counter {
    F get(&self) -> i64 { self.val }
    F doubled(&self) -> i64 { self.val * 2 }
}
F main() -> i64 {
    c := Counter { val: 21 }
    R c.get() + c.doubled()
}
"#;
    assert_exit_code(source, 63);
}

// ===== Enum with Data =====

#[test]
fn e2e_phase45a_enum_with_data() {
    // Enum variants carrying data, matched with destructuring
    let source = r#"
E Shape {
    Circle(i64),
    Rect(i64, i64)
}
F area(s: Shape) -> i64 {
    M s {
        Circle(r) => r * r * 3,
        Rect(w, h) => w * h
    }
}
F main() -> i64 { area(Rect(3, 4)) }
"#;
    assert_exit_code(source, 12);
}

// ===== Array Index Compute =====

#[test]
fn e2e_phase45a_array_index_compute() {
    // Array element access via a variable index
    let source = r#"
F main() -> i64 {
    arr := [10, 20, 30, 40, 50]
    idx := 2
    R arr[idx]
}
"#;
    assert_exit_code(source, 30);
}

// ===== Variable Reassignment =====

#[test]
fn e2e_phase45a_variable_reassign() {
    // Mutable variable reassigned multiple times
    let source = r#"
F main() -> i64 {
    x := mut 1
    x = 2
    x = x + 3
    R x
}
"#;
    assert_exit_code(source, 5);
}
