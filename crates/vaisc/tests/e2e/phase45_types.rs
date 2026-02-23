use super::helpers::*;

// ==================== Phase 45: Type System & Patterns E2E Tests ====================

// ==================== Tuple Destructuring ====================

#[test]
fn e2e_phase45t_tuple_destructuring() {
    let source = r#"
F main() -> i64 {
    (a, b) := (10, 20)
    R a + b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_phase45t_tuple_destructuring_fn() {
    let source = r#"
F pair() -> (i64, i64) { (3, 7) }
F main() -> i64 {
    (x, y) := pair()
    R x + y
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Default Parameters ====================

#[test]
fn e2e_phase45t_default_param_basic() {
    let source = r#"
F add(a: i64, b: i64 = 10) -> i64 { a + b }
F main() -> i64 { add(5) }
"#;
    // add(5) uses default b=10, so result = 5+10 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_phase45t_default_param_override() {
    let source = r#"
F add(a: i64, b: i64 = 10) -> i64 { a + b }
F main() -> i64 { add(5, 20) }
"#;
    assert_exit_code(source, 25);
}

// ==================== Contract Attributes ====================

#[test]
fn e2e_phase45t_requires_attr() {
    let source = r#"
#[requires(x > 0)]
F positive(x: i64) -> i64 { x }
F main() -> i64 { positive(5) }
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_phase45t_ensures_attr() {
    let source = r#"
#[ensures(return >= 0)]
F abs_val(x: i64) -> i64 { I x < 0 { -x } E { x } }
F main() -> i64 { abs_val(-3) }
"#;
    assert_exit_code(source, 3);
}

// ==================== Compound Assignment Operators ====================
// Note: compound_assign_add/sub/mul covered by phase41_string_numeric.rs
// (e2e_p41_compound_add_assign, e2e_p41_compound_sub_assign, e2e_p41_compound_mul_assign)

// ==================== Operator Precedence ====================

#[test]
fn e2e_phase45t_operator_precedence_mul_add() {
    let source = r#"
F main() -> i64 {
    R 2 + 3 * 4
}
"#;
    assert_exit_code(source, 14);
}

#[test]
fn e2e_phase45t_operator_precedence_parens() {
    let source = r#"
F main() -> i64 {
    R (2 + 3) * 4
}
"#;
    assert_exit_code(source, 20);
}

// ==================== Type Cast ====================

#[test]
fn e2e_phase45t_type_cast_parse() {
    let source = r#"
F main() -> i64 {
    x := 42
    R x as i64
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Where Clause ====================

#[test]
fn e2e_phase45t_where_clause() {
    let source = r#"
W Countable {
    F count(&self) -> i64
}
S Items { n: i64 }
X Items: Countable {
    F count(&self) -> i64 = self.n
}
F get_count<T>(x: T) -> i64 where T: Countable {
    x.count()
}
F main() -> i64 = get_count(Items { n: 7 })
"#;
    assert_exit_code(source, 7);
}

// ==================== Trait Alias ====================

#[test]
fn e2e_phase45t_trait_alias_parse() {
    let source = r#"
W Showable {
    F show(&self) -> i64
}
W Countable {
    F count(&self) -> i64
}
T DisplayCount = Showable + Countable
F main() -> i64 { 0 }
"#;
    assert_exit_code(source, 0);
}

// ==================== Struct Methods ====================

#[test]
fn e2e_phase45t_struct_method() {
    let source = r#"
S Point { x: i64, y: i64 }
X Point {
    F sum(&self) -> i64 { self.x + self.y }
}
F main() -> i64 {
    p := Point { x: 3, y: 7 }
    R p.sum()
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Enum Variant Match ====================

#[test]
fn e2e_phase45t_enum_variant_match() {
    let source = r#"
E Color {
    Red,
    Green,
    Blue
}
F to_num(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}
F main() -> i64 {
    R to_num(Green)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== Nested Struct ====================

#[test]
fn e2e_phase45t_nested_struct() {
    let source = r#"
S Inner { value: i64 }
S Outer { inner: Inner, scale: i64 }
F main() -> i64 {
    o := Outer { inner: Inner { value: 5 }, scale: 3 }
    R o.inner.value * o.scale
}
"#;
    assert_exit_code(source, 15);
}

// ==================== Type Alias ====================

#[test]
fn e2e_phase45t_type_alias() {
    let source = r#"
T Num = i64
F double(x: Num) -> Num { x * 2 }
F main() -> i64 { double(21) }
"#;
    assert_exit_code(source, 42);
}

// ==================== Phase 27: Type System Soundness ====================

// --- Positive tests ---

#[test]
fn phase27_generic_substitution_normal() {
    // Generic function with concrete call — substitution should work
    assert_exit_code(r#"
        F identity<T>(x: T) -> T { x }
        F main() -> i64 {
            identity(42)
        }
    "#, 42);
}

#[test]
fn phase27_associated_type_resolved() {
    // Associated type resolved through trait impl
    assert_exit_code(r#"
        W Container {
            F size(self) -> i64
        }
        S MyBox { val: i64 }
        X MyBox: Container {
            F size(self) -> i64 { self.val }
        }
        F main() -> i64 {
            box := MyBox { val: 10 }
            box.size()
        }
    "#, 10);
}

#[test]
fn phase27_simple_function_call_return() {
    // Simple function call returning a value
    assert_exit_code(r#"
        F make_val() -> i64 { 100 }
        F main() -> i64 { make_val() }
    "#, 100);
}

#[test]
fn phase27_two_arg_addition() {
    // Two-argument addition function
    assert_exit_code(r#"
        F add(a: i64, b: i64) -> i64 { a + b }
        F main() -> i64 { add(1, 2) }
    "#, 3);
}

#[test]
fn phase27_nested_generic_resolution() {
    // Nested generic types resolve correctly
    assert_exit_code(r#"
        F first<T>(a: T, b: T) -> T { a }
        F main() -> i64 {
            first(1, 2)
        }
    "#, 1);
}

#[test]
fn phase27_trait_method_dispatch() {
    // Trait method dispatch with concrete type
    assert_exit_code(r#"
        W Greet {
            F greet(self) -> i64
        }
        S Person { age: i64 }
        X Person: Greet {
            F greet(self) -> i64 { self.age }
        }
        F main() -> i64 {
            p := Person { age: 30 }
            p.greet()
        }
    "#, 30);
}

// --- Negative tests ---

#[test]
fn phase27_unresolved_param_type_error() {
    // Function parameter with no type info — should fail with InferFailed
    assert_compile_error(r#"
        F mystery(x) { x }
        F main() -> i64 { mystery(42) }
    "#);
}

#[test]
fn phase27_recursive_no_return_type_error() {
    // Recursive function without return type annotation — should fail
    assert_compile_error(r#"
        F factorial(n) {
            I n <= 1 { 1 } E { n * @(n - 1) }
        }
        F main() -> i64 { factorial(5) }
    "#);
}

#[test]
fn phase27_ambiguous_type_error() {
    // Ambiguous return type — no way to infer
    assert_compile_error(r#"
        F ambiguous(x) -> i64 {
            y := x
            42
        }
        F main() -> i64 { ambiguous(1) }
    "#);
}

#[test]
fn phase27_unconstrained_generic_error() {
    // Generic function called without enough type info
    assert_compile_error(r#"
        F pick<T>(a: T, b: T) -> T { a }
        F main() -> i64 {
            pick(1, "hello")
        }
    "#);
}

#[test]
fn phase27_missing_return_annotation_with_self_call() {
    // Self-call with no return type — should error
    assert_compile_error(r#"
        F loop_fn(n) {
            I n > 0 { @(n - 1) } E { 0 }
        }
        F main() -> i64 { loop_fn(3) }
    "#);
}

#[test]
fn phase27_type_mismatch_in_generic() {
    // Type mismatch: i64 vs str in generic
    assert_compile_error(r#"
        F same<T>(a: T, b: T) -> T { a }
        F main() -> i64 {
            same(42, "hello")
        }
    "#);
}
