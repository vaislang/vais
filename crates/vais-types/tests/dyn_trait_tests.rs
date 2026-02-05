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
        W Drawable {
            F draw(&self) -> i64
        }

        F use_drawable(d: &dyn Drawable) -> i64 = 0
        F main() -> i64 = 0
    "#;
    assert!(check_module(source).is_ok(), "dyn Trait type should parse");
}

#[test]
fn test_dyn_trait_with_generics() {
    // Test dyn Trait with generic parameters
    let source = r#"
        W Iterator<T> {
            F next(&self) -> T?
        }

        F use_iter(it: &dyn Iterator<i64>) -> i64 = 0
        F main() -> i64 = 0
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
        W Printable {
            F print(&self) -> i64
        }

        S Counter { value: i64 }

        X Counter: Printable {
            F print(&self) -> i64 = self.value
        }

        F main() -> i64 {
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
        W Shape {
            F area(&self) -> f64
            F perimeter(&self) -> f64
        }
        F main() -> i64 = 0
    "#;
    assert!(check_module(source).is_ok(), "Trait definition should work");
}

#[test]
fn test_trait_impl() {
    // Test trait implementation
    let source = r#"
        W Greet {
            F greet(&self) -> i64
        }

        S Person { age: i64 }

        X Person: Greet {
            F greet(&self) -> i64 = self.age
        }

        F main() -> i64 {
            p := Person { age: 25 }
            p.greet()
        }
    "#;
    assert!(check_module(source).is_ok(), "Trait impl should work");
}
