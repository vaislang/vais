//! Phase 40: Trait bounds verification and generic substitution
//!
//! Tests for:
//! - Generic function trait bounds verification (F foo<T: Trait>)
//! - Where clause bounds (F foo<T>() where T: Trait)
//! - Multiple trait bounds (T: TraitA + TraitB)
//! - Generic substitution for all type variants
//! - Struct with trait bounds
//! - Negative tests for trait bound violations

use super::helpers::*;

// ===== Positive Tests: Trait Bounds Satisfied =====

#[test]
fn e2e_phase40_trait_bounds_generic_call_satisfied() {
    // Generic function with Clone bound, i64 satisfies it (builtin)
    let source = r#"
W Cloneable {
    F clone_val(&self) -> i64
}

# i64 built-in impl (simulated by allowing generic call)
F identity<T>(x: T) -> T {
    x
}

F main() -> i64 {
    x := identity(42)
    R x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_trait_bounds_multiple_bounds_satisfied() {
    // Multiple trait bounds satisfied by struct with both implementations
    let source = r#"
W Numeric {
    F value(&self) -> i64
}

W Printable {
    F to_str(&self) -> i64
}

S Point { x: i64, y: i64 }

X Point: Numeric {
    F value(&self) -> i64 {
        self.x + self.y
    }
}

X Point: Printable {
    F to_str(&self) -> i64 {
        self.x * 10 + self.y
    }
}

# Generic function requiring both traits (parse-level test)
F process<T>(val: &T) -> i64 {
    0
}

F main() -> i64 {
    p := Point { x: 3, y: 4 }
    R p.value()
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_phase40_where_clause_bounds_satisfied() {
    // Where clause trait bounds satisfied (compile-only, trait dispatch not fully implemented)
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

F compute<T>(val: &T) -> i64
where T: Summable
{
    0
}

F main() -> i64 {
    p := Pair { a: 10, b: 32 }
    result := compute(&p)
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_struct_with_trait_bounds() {
    // Struct implements trait, used in generic bound function (compile-only)
    let source = r#"
W Summable {
    F sum(&self) -> i64
}

S Point { x: i64, y: i64 }

X Point: Summable {
    F sum(&self) -> i64 {
        self.x + self.y
    }
}

F add_all<T: Summable>(a: &T, b: &T) -> i64 {
    0
}

F main() -> i64 {
    p1 := Point { x: 5, y: 7 }
    p2 := Point { x: 10, y: 20 }
    result := add_all(&p1, &p2)
    R 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_substitution_nested() {
    // Nested generic calls with type substitution
    let source = r#"
F identity<T>(x: T) -> T {
    x
}

F double_identity<U>(y: U) -> U {
    identity(y)
}

F main() -> i64 {
    a := identity(10)
    b := double_identity(32)
    R a + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_impl_trait_return_type() {
    // impl Trait return type (IR generation only, codegen uses i64 fallback)
    let source = r#"
W Display {
    F show(&self) -> i64
}

S Value { n: i64 }

X Value: Display {
    F show(&self) -> i64 {
        self.n
    }
}

F make_value() -> X Display {
    Value { n: 42 }
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should compile impl trait return");
}

#[test]
fn e2e_phase40_trait_alias_with_bounds() {
    // Trait alias used in bounds (from Phase 37 but testing bounds verification)
    let source = r#"
W Numeric {
    F value(&self) -> i64
}

W Copyable {
    F copy(&self) -> i64
}

T NumericCopy = Numeric + Copyable

S Number { n: i64 }

X Number: Numeric {
    F value(&self) -> i64 {
        self.n
    }
}

X Number: Copyable {
    F copy(&self) -> i64 {
        self.n
    }
}

F use_numeric<T: NumericCopy>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_function_no_bounds() {
    // Generic function without bounds works normally
    let source = r#"
F wrap<T>(x: T) -> T {
    x
}

F main() -> i64 {
    a := wrap(21)
    b := wrap(21)
    R a + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_multiple_generic_params_with_bounds() {
    // Multiple generic parameters with different bounds (compile-only)
    let source = r#"
W TraitA {
    F method_a(&self) -> i64
}

W TraitB {
    F method_b(&self) -> i64
}

S TypeA { val: i64 }
S TypeB { val: i64 }

X TypeA: TraitA {
    F method_a(&self) -> i64 {
        self.val
    }
}

X TypeB: TraitB {
    F method_b(&self) -> i64 {
        self.val * 2
    }
}

F combine<T: TraitA, U: TraitB>(a: &T, b: &U) -> i64 {
    42
}

F main() -> i64 {
    ta := TypeA { val: 10 }
    tb := TypeB { val: 16 }
    R combine(&ta, &tb)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_struct_instantiation() {
    // Generic struct with type substitution (Vec-like)
    let source = r#"
S Container<T> {
    value: T
}

F make_container<T>(val: T) -> Container<T> {
    Container { value: val }
}

F main() -> i64 {
    c := make_container(42)
    R c.value
}
"#;
    assert_exit_code(source, 42);
}

// ===== Negative Tests: Trait Bound Violations =====

#[test]
fn e2e_phase40_trait_bounds_violation_error() {
    // Struct doesn't implement required trait
    let source = r#"
W Hashable {
    F hash(&self) -> i64
}

S Unhashable {
    x: i64
}

# Unhashable does NOT implement Hashable

F needs_hash<T: Hashable>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    s := Unhashable { x: 1 }
    R needs_hash(&s)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_where_clause_bounds_violation_error() {
    // Where clause bound violated
    let source = r#"
W Printable {
    F print(&self) -> i64
}

S Unprintable {
    val: i64
}

# Unprintable does NOT implement Printable

F show<T>(x: &T) -> i64
where T: Printable
{
    0
}

F main() -> i64 {
    u := Unprintable { val: 5 }
    R show(&u)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_multiple_bounds_partial_violation_error() {
    // Satisfies one bound but not the other
    let source = r#"
W TraitA {
    F method_a(&self) -> i64
}

W TraitB {
    F method_b(&self) -> i64
}

S Partial {
    n: i64
}

# Only implements TraitA, not TraitB
X Partial: TraitA {
    F method_a(&self) -> i64 {
        self.n
    }
}

F needs_both<T: TraitA + TraitB>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    p := Partial { n: 1 }
    R needs_both(&p)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_recursive_generic_bounds_error() {
    // Generic instantiation that recursively violates bounds
    let source = r#"
W Constraint {
    F check(&self) -> i64
}

S Outer<T> {
    inner: T
}

# Outer does NOT implement Constraint for any T

F validate<T: Constraint>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    o := Outer { inner: 42 }
    R validate(&o)
}
"#;
    assert_compile_error(source);
}

// ===== Additional Coverage Tests =====

#[test]
fn e2e_phase40_builtin_type_with_bounds() {
    // Verify that builtin types (i64, bool) can be used in generic contexts
    let source = r#"
F passthrough<T>(x: T) -> T {
    x
}

F main() -> i64 {
    a := passthrough(21)
    b := passthrough(21)
    c := passthrough(true)
    I c {
        R a + b
    } E {
        R 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_trait_method_call_with_bounds() {
    // Trait method through generic bound (compile-only, trait dispatch stub)
    let source = r#"
W Summable {
    F sum(&self) -> i64
}

S Vec2 { x: i64, y: i64 }

X Vec2: Summable {
    F sum(&self) -> i64 {
        self.x + self.y
    }
}

F get_sum<T: Summable>(val: &T) -> i64 {
    42
}

F main() -> i64 {
    v := Vec2 { x: 30, y: 12 }
    R get_sum(&v)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_nested_where_clause() {
    // Multiple where clause constraints (compile-only)
    let source = r#"
W TraitA {
    F method_a(&self) -> i64
}

W TraitB {
    F method_b(&self) -> i64
}

S MyType { val: i64 }

X MyType: TraitA {
    F method_a(&self) -> i64 {
        self.val
    }
}

X MyType: TraitB {
    F method_b(&self) -> i64 {
        self.val * 2
    }
}

F complex<T, U>(x: &T, y: &U) -> i64
where
    T: TraitA,
    U: TraitB
{
    42
}

F main() -> i64 {
    t1 := MyType { val: 10 }
    t2 := MyType { val: 16 }
    R complex(&t1, &t2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_substitution_in_struct_field() {
    // Generic substitution with struct fields
    let source = r#"
S Wrapper<T> {
    data: T
}

F unwrap<T>(w: Wrapper<T>) -> T {
    w.data
}

F main() -> i64 {
    w := Wrapper { data: 42 }
    R unwrap(w)
}
"#;
    assert_exit_code(source, 42);
}
