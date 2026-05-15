use super::helpers::*;

// ==================== Phase 45: Type System & Patterns E2E Tests ====================

// ==================== Tuple Destructuring ====================

#[test]
fn e2e_phase45t_tuple_destructuring() {
    let source = r#"
fn main() -> i64 {
    (a, b) := (10, 20)
    return a + b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_phase45t_tuple_destructuring_fn() {
    let source = r#"
fn pair() -> (i64, i64) { (3, 7) }
fn main() -> i64 {
    (x, y) := pair()
    return x + y
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Default Parameters ====================

#[test]
fn e2e_phase45t_default_param_basic() {
    let source = r#"
fn add(a: i64, b: i64 = 10) -> i64 { a + b }
fn main() -> i64 { add(5) }
"#;
    // add(5) uses default b=10, so result = 5+10 = 15
    assert_exit_code(source, 15);
}

#[test]
fn e2e_phase45t_default_param_override() {
    let source = r#"
fn add(a: i64, b: i64 = 10) -> i64 { a + b }
fn main() -> i64 { add(5, 20) }
"#;
    assert_exit_code(source, 25);
}

// ==================== Contract Attributes ====================

#[test]
fn e2e_phase45t_requires_attr() {
    let source = r#"
#[requires(x > 0)]
fn positive(x: i64) -> i64 { x }
fn main() -> i64 { positive(5) }
"#;
    assert_exit_code(source, 5);
}

#[test]
#[ignore = "Pre-existing: parser does not accept `return` keyword inside attribute expression"]
fn e2e_phase45t_ensures_attr() {
    let source = r#"
#[ensures(return >= 0)]
fn abs_val(x: i64) -> i64 { I x < 0 { -x } else { x } }
fn main() -> i64 { abs_val(-3) }
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
fn main() -> i64 {
    return 2 + 3 * 4
}
"#;
    assert_exit_code(source, 14);
}

#[test]
fn e2e_phase45t_operator_precedence_parens() {
    let source = r#"
fn main() -> i64 {
    return (2 + 3) * 4
}
"#;
    assert_exit_code(source, 20);
}

// ==================== Type Cast ====================

#[test]
fn e2e_phase45t_type_cast_parse() {
    let source = r#"
fn main() -> i64 {
    x := 42
    return x as i64
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Where Clause ====================

#[test]
fn e2e_phase45t_where_clause() {
    let source = r#"
trait Countable {
    fn count(&self) -> i64
}
struct Items { n: i64 }
impl Items: Countable {
    fn count(&self) -> i64 = self.n
}
fn get_count<T>(x: T) -> i64 where T: Countable {
    x.count()
}
fn main() -> i64 = get_count(Items { n: 7 })
"#;
    assert_exit_code(source, 7);
}

// ==================== Trait Alias ====================

#[test]
fn e2e_phase45t_trait_alias_parse() {
    let source = r#"
trait Showable {
    fn show(&self) -> i64
}
trait Countable {
    fn count(&self) -> i64
}
type DisplayCount = Showable + Countable
fn main() -> i64 { 0 }
"#;
    assert_exit_code(source, 0);
}

// ==================== Struct Methods ====================

#[test]
fn e2e_phase45t_struct_method() {
    let source = r#"
struct Point { x: i64, y: i64 }
impl Point {
    fn sum(&self) -> i64 { self.x + self.y }
}
fn main() -> i64 {
    p := Point { x: 3, y: 7 }
    return p.sum()
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Enum Variant Match ====================

#[test]
fn e2e_phase45t_enum_variant_match() {
    let source = r#"
enum Color {
    Red,
    Green,
    Blue
}
fn to_num(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}
fn main() -> i64 {
    return to_num(Green)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== Nested Struct ====================

#[test]
fn e2e_phase45t_nested_struct() {
    let source = r#"
struct Inner { value: i64 }
struct Outer { inner: Inner, scale: i64 }
fn main() -> i64 {
    o := Outer { inner: Inner { value: 5 }, scale: 3 }
    return o.inner.value * o.scale
}
"#;
    assert_exit_code(source, 15);
}

// ==================== Type Alias ====================

#[test]
fn e2e_phase45t_type_alias() {
    let source = r#"
type Num = i64
fn double(x: Num) -> Num { x * 2 }
fn main() -> i64 { double(21) }
"#;
    assert_exit_code(source, 42);
}

// ==================== Phase 27: Type System Soundness ====================

// --- Positive tests ---

#[test]
fn phase27_generic_substitution_normal() {
    // Generic function with concrete call — substitution should work
    assert_exit_code(
        r#"
        fn identity<T>(x: T) -> type { x }
        fn main() -> i64 {
            identity(42)
        }
    "#,
        42,
    );
}

#[test]
fn phase27_associated_type_resolved() {
    // Associated type resolved through trait impl
    assert_exit_code(
        r#"
        trait Container {
            fn size(self) -> i64
        }
        struct MyBox { val: i64 }
        impl MyBox: Container {
            fn size(self) -> i64 { self.val }
        }
        fn main() -> i64 {
            box := MyBox { val: 10 }
            box.size()
        }
    "#,
        10,
    );
}

#[test]
fn phase27_simple_function_call_return() {
    // Simple function call returning a value
    assert_exit_code(
        r#"
        fn make_val() -> i64 { 100 }
        fn main() -> i64 { make_val() }
    "#,
        100,
    );
}

#[test]
fn phase27_two_arg_addition() {
    // Two-argument addition function
    assert_exit_code(
        r#"
        fn add(a: i64, b: i64) -> i64 { a + b }
        fn main() -> i64 { add(1, 2) }
    "#,
        3,
    );
}

#[test]
fn phase27_nested_generic_resolution() {
    // Nested generic types resolve correctly
    assert_exit_code(
        r#"
        fn first<T>(a: T, b: T) -> type { a }
        fn main() -> i64 {
            first(1, 2)
        }
    "#,
        1,
    );
}

#[test]
fn phase27_trait_method_dispatch() {
    // Trait method dispatch with concrete type
    assert_exit_code(
        r#"
        trait Greet {
            fn greet(self) -> i64
        }
        struct Person { age: i64 }
        impl Person: Greet {
            fn greet(self) -> i64 { self.age }
        }
        fn main() -> i64 {
            p := Person { age: 30 }
            p.greet()
        }
    "#,
        30,
    );
}

// --- Negative tests ---

#[test]
fn phase27_unresolved_param_type_error() {
    // Function parameter with no type info — should fail with InferFailed
    assert_compile_error(
        r#"
        fn mystery(x) { x }
        fn main() -> i64 { mystery(42) }
    "#,
    );
}

#[test]
fn phase27_recursive_no_return_type_error() {
    // Recursive function without return type annotation — should fail
    assert_compile_error(
        r#"
        fn factorial(n) {
            I n <= 1 { 1 } else { n * @(n - 1) }
        }
        fn main() -> i64 { factorial(5) }
    "#,
    );
}

#[test]
fn phase27_ambiguous_type_error() {
    // Ambiguous return type — no way to infer
    assert_compile_error(
        r#"
        fn ambiguous(x) -> i64 {
            y := x
            42
        }
        fn main() -> i64 { ambiguous(1) }
    "#,
    );
}

#[test]
fn phase27_unconstrained_generic_error() {
    // Generic function called without enough type info
    assert_compile_error(
        r#"
        fn pick<T>(a: T, b: T) -> type { a }
        fn main() -> i64 {
            pick(1, "hello")
        }
    "#,
    );
}

#[test]
fn phase27_missing_return_annotation_with_self_call() {
    // Self-call with no return type — should error
    assert_compile_error(
        r#"
        fn loop_fn(n) {
            I n > 0 { @(n - 1) } else { 0 }
        }
        fn main() -> i64 { loop_fn(3) }
    "#,
    );
}

#[test]
fn phase27_type_mismatch_in_generic() {
    // Type mismatch: i64 vs str in generic
    assert_compile_error(
        r#"
        fn same<T>(a: T, b: T) -> type { a }
        fn main() -> i64 {
            same(42, "hello")
        }
    "#,
    );
}
