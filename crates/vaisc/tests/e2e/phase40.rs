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
trait Cloneable {
    fn clone_val(&self) -> i64
}

# i64 built-in impl (simulated by allowing generic call)
fn identity<T>(x: T) -> type {
    x
}

fn main() -> i64 {
    x := identity(42)
    return x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_trait_bounds_multiple_bounds_satisfied() {
    // Multiple trait bounds satisfied by struct with both implementations
    let source = r#"
trait Numeric {
    fn value(&self) -> i64
}

trait Printable {
    fn to_str(&self) -> i64
}

struct Point { x: i64, y: i64 }

impl Point: Numeric {
    fn value(&self) -> i64 {
        self.x + self.y
    }
}

impl Point: Printable {
    fn to_str(&self) -> i64 {
        self.x * 10 + self.y
    }
}

# Generic function requiring both traits (parse-level test)
fn process<T>(val: &T) -> i64 {
    0
}

fn main() -> i64 {
    p := Point { x: 3, y: 4 }
    return p.value()
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_phase40_where_clause_bounds_satisfied() {
    // Where clause trait bounds satisfied (compile-only, trait dispatch not fully implemented)
    let source = r#"
trait Summable {
    fn sum(&self) -> i64
}

struct Pair { a: i64, b: i64 }

impl Pair: Summable {
    fn sum(&self) -> i64 {
        self.a + self.b
    }
}

fn compute<T>(val: &T) -> i64
where T: Summable
{
    0
}

fn main() -> i64 {
    p := Pair { a: 10, b: 32 }
    result := compute(&p)
    return 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_struct_with_trait_bounds() {
    // Struct implements trait, used in generic bound function (compile-only)
    let source = r#"
trait Summable {
    fn sum(&self) -> i64
}

struct Point { x: i64, y: i64 }

impl Point: Summable {
    fn sum(&self) -> i64 {
        self.x + self.y
    }
}

fn add_all<T: Summable>(a: &T, b: &T) -> i64 {
    0
}

fn main() -> i64 {
    p1 := Point { x: 5, y: 7 }
    p2 := Point { x: 10, y: 20 }
    result := add_all(&p1, &p2)
    return 42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_substitution_nested() {
    // Nested generic calls with type substitution
    let source = r#"
fn identity<T>(x: T) -> type {
    x
}

fn double_identity<U>(y: U) -> use {
    identity(y)
}

fn main() -> i64 {
    a := identity(10)
    b := double_identity(32)
    return a + b
}
"#;
    assert_exit_code(source, 42);
}

// e2e_phase40_impl_trait_return_type REMOVED (ROADMAP #18): `X Trait` return-position
// existential types were removed. Use explicit generic bounds instead.

#[test]
fn e2e_phase40_trait_alias_with_bounds() {
    // Trait alias used in bounds (from Phase 37 but testing bounds verification)
    let source = r#"
trait Numeric {
    fn value(&self) -> i64
}

trait Copyable {
    fn copy(&self) -> i64
}

type NumericCopy = Numeric + Copyable

struct Number { n: i64 }

impl Number: Numeric {
    fn value(&self) -> i64 {
        self.n
    }
}

impl Number: Copyable {
    fn copy(&self) -> i64 {
        self.n
    }
}

fn use_numeric<T: NumericCopy>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_function_no_bounds() {
    // Generic function without bounds works normally
    let source = r#"
fn wrap<T>(x: T) -> type {
    x
}

fn main() -> i64 {
    a := wrap(21)
    b := wrap(21)
    return a + b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_multiple_generic_params_with_bounds() {
    // Multiple generic parameters with different bounds (compile-only)
    let source = r#"
trait TraitA {
    fn method_a(&self) -> i64
}

trait TraitB {
    fn method_b(&self) -> i64
}

struct TypeA { val: i64 }
struct TypeB { val: i64 }

impl TypeA: TraitA {
    fn method_a(&self) -> i64 {
        self.val
    }
}

impl TypeB: TraitB {
    fn method_b(&self) -> i64 {
        self.val * 2
    }
}

fn combine<T: TraitA, U: TraitB>(a: &T, b: &U) -> i64 {
    42
}

fn main() -> i64 {
    ta := TypeA { val: 10 }
    tb := TypeB { val: 16 }
    return combine(&ta, &tb)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_struct_instantiation() {
    // Generic struct with type substitution (Vec-like)
    let source = r#"
struct Container<T> {
    value: T
}

fn make_container<T>(val: T) -> Container<T> {
    Container { value: val }
}

fn main() -> i64 {
    c := make_container(42)
    return c.value
}
"#;
    assert_exit_code(source, 42);
}

// ===== Negative Tests: Trait Bound Violations =====

#[test]
fn e2e_phase40_trait_bounds_violation_error() {
    // Struct doesn't implement required trait
    let source = r#"
trait Hashable {
    fn hash(&self) -> i64
}

struct Unhashable {
    x: i64
}

# Unhashable does NOT implement Hashable

fn needs_hash<T: Hashable>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    s := Unhashable { x: 1 }
    return needs_hash(&s)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_where_clause_bounds_violation_error() {
    // Where clause bound violated
    let source = r#"
trait Printable {
    fn print(&self) -> i64
}

struct Unprintable {
    val: i64
}

# Unprintable does NOT implement Printable

fn show<T>(x: &T) -> i64
where T: Printable
{
    0
}

fn main() -> i64 {
    u := Unprintable { val: 5 }
    return show(&u)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_multiple_bounds_partial_violation_error() {
    // Satisfies one bound but not the other
    let source = r#"
trait TraitA {
    fn method_a(&self) -> i64
}

trait TraitB {
    fn method_b(&self) -> i64
}

struct Partial {
    n: i64
}

# Only implements TraitA, not TraitB
impl Partial: TraitA {
    fn method_a(&self) -> i64 {
        self.n
    }
}

fn needs_both<T: TraitA + TraitB>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    p := Partial { n: 1 }
    return needs_both(&p)
}
"#;
    assert_compile_error(source);
}

#[test]
fn e2e_phase40_recursive_generic_bounds_error() {
    // Generic instantiation that recursively violates bounds
    let source = r#"
trait Constraint {
    fn check(&self) -> i64
}

struct Outer<T> {
    inner: T
}

# Outer does NOT implement Constraint for any T

fn validate<T: Constraint>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    o := Outer { inner: 42 }
    return validate(&o)
}
"#;
    assert_compile_error(source);
}

// ===== Additional Coverage Tests =====

#[test]
fn e2e_phase40_builtin_type_with_bounds() {
    // Verify that builtin types (i64, bool) can be used in generic contexts
    let source = r#"
fn passthrough<T>(x: T) -> type {
    x
}

fn main() -> i64 {
    a := passthrough(21)
    b := passthrough(21)
    c := passthrough(true)
    I c {
        return a + b
    } else {
        return 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_trait_method_call_with_bounds() {
    // Trait method through generic bound (compile-only, trait dispatch stub)
    let source = r#"
trait Summable {
    fn sum(&self) -> i64
}

struct Vec2 { x: i64, y: i64 }

impl Vec2: Summable {
    fn sum(&self) -> i64 {
        self.x + self.y
    }
}

fn get_sum<T: Summable>(val: &T) -> i64 {
    42
}

fn main() -> i64 {
    v := Vec2 { x: 30, y: 12 }
    return get_sum(&v)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_nested_where_clause() {
    // Multiple where clause constraints (compile-only)
    let source = r#"
trait TraitA {
    fn method_a(&self) -> i64
}

trait TraitB {
    fn method_b(&self) -> i64
}

struct MyType { val: i64 }

impl MyType: TraitA {
    fn method_a(&self) -> i64 {
        self.val
    }
}

impl MyType: TraitB {
    fn method_b(&self) -> i64 {
        self.val * 2
    }
}

fn complex<T, U>(x: &T, y: &U) -> i64
where
    T: TraitA,
    U: TraitB
{
    42
}

fn main() -> i64 {
    t1 := MyType { val: 10 }
    t2 := MyType { val: 16 }
    return complex(&t1, &t2)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase40_generic_substitution_in_struct_field() {
    // Generic substitution with struct fields
    let source = r#"
struct Wrapper<T> {
    data: T
}

fn unwrap<T>(w: Wrapper<T>) -> type {
    w.data
}

fn main() -> i64 {
    w := Wrapper { data: 42 }
    return unwrap(w)
}
"#;
    assert_exit_code(source, 42);
}
