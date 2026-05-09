//! Tests for dynamic trait object types (dyn Trait)

use vais_parser::parse;
use vais_types::{ResolvedType, TypeChecker};

fn check_module(source: &str) -> Result<(), String> {
    let module = parse(source).map_err(|e| format!("{:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

#[test]
fn test_dyn_trait_type_parsing() {
    // Test that dyn Trait syntax parses correctly
    let source = r#"
        trait Drawable {
            fn draw(&self) -> i64
        }

        fn use_drawable(d: &dyn Drawable) -> i64 = 0
        fn main() -> i64 = 0
    "#;
    assert!(check_module(source).is_ok(), "dyn Trait type should parse");
}

#[test]
fn test_dyn_trait_with_generics() {
    // Test dyn Trait with generic parameters
    let source = r#"
        W Iterator<T> {
            fn next(&self) -> T?
        }

        fn use_iter(it: &dyn Iterator<i64>) -> i64 = 0
        fn main() -> i64 = 0
    "#;
    assert!(check_module(source).is_ok(), "dyn Trait<T> should parse");
}

#[test]
fn test_dyn_trait_display() {
    // Test Display implementation for DynTrait
    let ty = ResolvedType::DynTrait {
        trait_name: "Drawable".to_string(),
        generics: vec![],
    };
    assert_eq!(format!("{}", ty), "dyn Drawable");

    let ty_gen = ResolvedType::DynTrait {
        trait_name: "Iterator".to_string(),
        generics: vec![ResolvedType::I64],
    };
    assert_eq!(format!("{}", ty_gen), "dyn Iterator<i64>");
}

#[test]
fn test_static_dispatch_still_works() {
    // Existing static dispatch should still work
    let source = r#"
        trait Printable {
            fn print(&self) -> i64
        }

        struct Counter { value: i64 }

        impl Counter: Printable {
            fn print(&self) -> i64 = self.value
        }

        fn main() -> i64 {
            c := Counter { value: 42 }
            c.print()
        }
    "#;
    assert!(check_module(source).is_ok(), "Static dispatch should work");
}

#[test]
fn test_trait_definition() {
    // Test basic trait definition
    let source = r#"
        trait Shape {
            fn area(&self) -> f64
            fn perimeter(&self) -> f64
        }
        fn main() -> i64 = 0
    "#;
    assert!(check_module(source).is_ok(), "Trait definition should work");
}

#[test]
fn test_trait_impl() {
    // Test trait implementation
    let source = r#"
        trait Greet {
            fn greet(&self) -> i64
        }

        struct Person { age: i64 }

        impl Person: Greet {
            fn greet(&self) -> i64 = self.age
        }

        fn main() -> i64 {
            p := Person { age: 25 }
            p.greet()
        }
    "#;
    assert!(check_module(source).is_ok(), "Trait impl should work");
}

#[test]
fn test_box_dyn_method_dispatch() {
    // Phase 6.27c.5: method call on Box<dyn Trait> must resolve to trait method.
    // Previously `find_trait_method` only peeled Ref/RefMut; Box<dyn T> was opaque.
    let source = r#"
        trait Executor {
            fn next(&mut self) -> i64
        }

        fn drive(e: &mut Box<dyn Executor>) -> i64 {
            e.next()
        }
        fn main() -> i64 = 0
    "#;
    assert!(
        check_module(source).is_ok(),
        "Box<dyn Trait> method dispatch should work"
    );
}

#[test]
fn test_ref_dyn_method_dispatch() {
    // Regression: &dyn T / &mut dyn T method dispatch.
    let source = r#"
        trait Printable {
            fn render(&self) -> i64
        }

        fn show(p: &dyn Printable) -> i64 {
            p.render()
        }
        fn main() -> i64 = 0
    "#;
    assert!(
        check_module(source).is_ok(),
        "&dyn Trait method dispatch should work"
    );
}
