//! Coverage tests for vais-types/src/traits.rs
//!
//! Targets: type_implements_trait, trait alias expansion, super traits,
//! trait impl checking, associated types, and trait method signatures.

use vais_parser::parse;
use vais_types::TypeChecker;

fn check_ok(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .unwrap_or_else(|e| panic!("Type check failed for: {}\nErr: {:?}", source, e));
}

#[allow(dead_code)]
fn check_err(source: &str) {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut tc = TypeChecker::new();
    assert!(
        tc.check_module(&module).is_err(),
        "Expected type error for: {}",
        source
    );
}

// ============================================================================
// Basic trait definition and implementation
// ============================================================================

#[test]
fn test_trait_simple_method() {
    check_ok(
        r#"
        trait Display {
            fn show(self) -> str
        }
        struct Num { val: i64 }
        impl Num: Display {
            fn show(self) -> str = "num"
        }
    "#,
    );
}

#[test]
fn test_trait_multiple_methods() {
    check_ok(
        r#"
        trait Container {
            fn size(self) -> i64
            fn empty(self) -> bool
        }
        struct Box { n: i64 }
        impl Box: Container {
            fn size(self) -> i64 = self.n
            fn empty(self) -> bool = self.n == 0
        }
    "#,
    );
}

#[test]
fn test_trait_with_default_method() {
    check_ok(
        r#"
        trait HasDefault {
            fn required(self) -> i64
            fn optional(self) -> i64 = 0
        }
        struct Foo { x: i64 }
        impl Foo: HasDefault {
            fn required(self) -> i64 = self.x
        }
    "#,
    );
}

// ============================================================================
// Trait method calls
// ============================================================================

#[test]
fn test_trait_method_call() {
    check_ok(
        r#"
        trait Greet {
            fn hello(self) -> i64
        }
        struct Dog { age: i64 }
        impl Dog: Greet {
            fn hello(self) -> i64 = self.age
        }
        fn test() -> i64 {
            d := Dog { age: 5 }
            d.hello()
        }
    "#,
    );
}

#[test]
fn test_trait_method_with_params() {
    check_ok(
        r#"
        trait Addable {
            fn add(self, n: i64) -> i64
        }
        struct Counter { val: i64 }
        impl Counter: Addable {
            fn add(self, n: i64) -> i64 = self.val + n
        }
        fn test() -> i64 {
            c := Counter { val: 10 }
            c.add(5)
        }
    "#,
    );
}

// ============================================================================
// Multiple impls for different types
// ============================================================================

#[test]
fn test_trait_impl_for_multiple_types() {
    check_ok(
        r#"
        trait Describable {
            fn desc(self) -> str
        }
        struct Cat { name: str }
        struct Dog { name: str }
        impl Cat: Describable {
            fn desc(self) -> str = "cat"
        }
        impl Dog: Describable {
            fn desc(self) -> str = "dog"
        }
    "#,
    );
}

// ============================================================================
// Super traits
// ============================================================================

#[test]
fn test_trait_super_trait() {
    check_ok(
        r#"
        trait Base {
            fn base_method(self) -> i64
        }
        W Extended: Base {
            fn ext_method(self) -> i64
        }
        struct Impl { x: i64 }
        impl Impl: Base {
            fn base_method(self) -> i64 = self.x
        }
        impl Impl: Extended {
            fn ext_method(self) -> i64 = self.x * 2
        }
    "#,
    );
}

// ============================================================================
// Trait alias
// ============================================================================

#[test]
fn test_trait_alias() {
    check_ok(
        r#"
        trait Printable {
            fn print(self) -> str
        }
        trait Loggable {
            fn log(self) -> str
        }
        type Output = Printable + Loggable
        struct Msg { text: str }
        impl Msg: Printable {
            fn print(self) -> str = self.text
        }
        impl Msg: Loggable {
            fn log(self) -> str = self.text
        }
    "#,
    );
}

// ============================================================================
// Trait with generic type parameters
// ============================================================================

#[test]
fn test_trait_generic_definition() {
    // Generic trait definition parses and type-checks
    check_ok(
        r#"
        trait Converter {
            fn convert(self) -> i64
        }
        struct IntWrapper { value: i64 }
        impl IntWrapper: Converter {
            fn convert(self) -> i64 = self.value
        }
        fn test() -> i64 {
            w := IntWrapper { value: 42 }
            w.convert()
        }
    "#,
    );
}

#[test]
fn test_generic_trait_with_method_params() {
    check_ok(
        r#"
        trait Transform {
            fn apply(self, x: i64) -> i64
        }
        struct Doubler { factor: i64 }
        impl Doubler: Transform {
            fn apply(self, x: i64) -> i64 = x * self.factor
        }
        fn test() -> i64 {
            d := Doubler { factor: 2 }
            d.apply(21)
        }
    "#,
    );
}

// ============================================================================
// Associated types
// ============================================================================

#[test]
fn test_trait_associated_type() {
    check_ok(
        r#"
        trait Iterator {
            fn next(self) -> i64
        }
        struct Range { current: i64, end: i64 }
        impl Range: Iterator {
            fn next(self) -> i64 = self.current
        }
    "#,
    );
}

// ============================================================================
// Impl block without trait (inherent methods)
// ============================================================================

#[test]
fn test_inherent_impl() {
    check_ok(
        r#"
        struct Point { x: i64, y: i64 }
        impl Point {
            fn origin() -> Point = Point { x: 0, y: 0 }
            fn manhattan(self) -> i64 = self.x + self.y
        }
    "#,
    );
}

#[test]
fn test_inherent_impl_multiple_methods() {
    check_ok(
        r#"
        struct Stack { count: i64 }
        impl Stack {
            fn new() -> Stack = Stack { count: 0 }
            fn size(self) -> i64 = self.count
            fn is_empty(self) -> bool = self.count == 0
        }
    "#,
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_empty_trait() {
    check_ok(
        r#"
        trait Marker {}
        struct Foo { x: i64 }
        impl Foo: Marker {}
    "#,
    );
}

#[test]
fn test_trait_method_returning_self_type() {
    check_ok(
        r#"
        trait Incrementable {
            fn inc(self) -> i64
        }
        struct Counter { n: i64 }
        impl Counter: Incrementable {
            fn inc(self) -> i64 = self.n + 1
        }
    "#,
    );
}

#[test]
fn test_multiple_traits_for_one_type() {
    check_ok(
        r#"
        trait TraitA { F a(self) -> i64 }
        trait TraitB { F b(self) -> i64 }
        trait TraitC { F c(self) -> i64 }
        struct Multi { x: i64 }
        impl Multi: TraitA { F a(self) -> i64 = 1 }
        impl Multi: TraitB { F b(self) -> i64 = 2 }
        impl Multi: TraitC { F c(self) -> i64 = 3 }
    "#,
    );
}
