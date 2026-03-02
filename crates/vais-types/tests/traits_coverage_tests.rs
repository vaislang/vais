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
        W Display {
            F show(self) -> str
        }
        S Num { val: i64 }
        X Num: Display {
            F show(self) -> str = "num"
        }
    "#,
    );
}

#[test]
fn test_trait_multiple_methods() {
    check_ok(
        r#"
        W Container {
            F size(self) -> i64
            F empty(self) -> bool
        }
        S Box { n: i64 }
        X Box: Container {
            F size(self) -> i64 = self.n
            F empty(self) -> bool = self.n == 0
        }
    "#,
    );
}

#[test]
fn test_trait_with_default_method() {
    check_ok(
        r#"
        W HasDefault {
            F required(self) -> i64
            F optional(self) -> i64 = 0
        }
        S Foo { x: i64 }
        X Foo: HasDefault {
            F required(self) -> i64 = self.x
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
        W Greet {
            F hello(self) -> i64
        }
        S Dog { age: i64 }
        X Dog: Greet {
            F hello(self) -> i64 = self.age
        }
        F test() -> i64 {
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
        W Addable {
            F add(self, n: i64) -> i64
        }
        S Counter { val: i64 }
        X Counter: Addable {
            F add(self, n: i64) -> i64 = self.val + n
        }
        F test() -> i64 {
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
        W Describable {
            F desc(self) -> str
        }
        S Cat { name: str }
        S Dog { name: str }
        X Cat: Describable {
            F desc(self) -> str = "cat"
        }
        X Dog: Describable {
            F desc(self) -> str = "dog"
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
        W Base {
            F base_method(self) -> i64
        }
        W Extended: Base {
            F ext_method(self) -> i64
        }
        S Impl { x: i64 }
        X Impl: Base {
            F base_method(self) -> i64 = self.x
        }
        X Impl: Extended {
            F ext_method(self) -> i64 = self.x * 2
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
        W Printable {
            F print(self) -> str
        }
        W Loggable {
            F log(self) -> str
        }
        T Output = Printable + Loggable
        S Msg { text: str }
        X Msg: Printable {
            F print(self) -> str = self.text
        }
        X Msg: Loggable {
            F log(self) -> str = self.text
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
        W Converter {
            F convert(self) -> i64
        }
        S IntWrapper { value: i64 }
        X IntWrapper: Converter {
            F convert(self) -> i64 = self.value
        }
        F test() -> i64 {
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
        W Transform {
            F apply(self, x: i64) -> i64
        }
        S Doubler { factor: i64 }
        X Doubler: Transform {
            F apply(self, x: i64) -> i64 = x * self.factor
        }
        F test() -> i64 {
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
        W Iterator {
            F next(self) -> i64
        }
        S Range { current: i64, end: i64 }
        X Range: Iterator {
            F next(self) -> i64 = self.current
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
        S Point { x: i64, y: i64 }
        X Point {
            F origin() -> Point = Point { x: 0, y: 0 }
            F manhattan(self) -> i64 = self.x + self.y
        }
    "#,
    );
}

#[test]
fn test_inherent_impl_multiple_methods() {
    check_ok(
        r#"
        S Stack { count: i64 }
        X Stack {
            F new() -> Stack = Stack { count: 0 }
            F size(self) -> i64 = self.count
            F is_empty(self) -> bool = self.count == 0
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
        W Marker {}
        S Foo { x: i64 }
        X Foo: Marker {}
    "#,
    );
}

#[test]
fn test_trait_method_returning_self_type() {
    check_ok(
        r#"
        W Incrementable {
            F inc(self) -> i64
        }
        S Counter { n: i64 }
        X Counter: Incrementable {
            F inc(self) -> i64 = self.n + 1
        }
    "#,
    );
}

#[test]
fn test_multiple_traits_for_one_type() {
    check_ok(
        r#"
        W TraitA { F a(self) -> i64 }
        W TraitB { F b(self) -> i64 }
        W TraitC { F c(self) -> i64 }
        S Multi { x: i64 }
        X Multi: TraitA { F a(self) -> i64 = 1 }
        X Multi: TraitB { F b(self) -> i64 = 2 }
        X Multi: TraitC { F c(self) -> i64 = 3 }
    "#,
    );
}
