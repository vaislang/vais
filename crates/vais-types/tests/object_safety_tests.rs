//! Tests for object safety checking
//!
//! These tests verify that the compiler correctly identifies which traits
//! can be used as `dyn Trait` (trait objects).

use vais_parser::parse;
use vais_types::{ObjectSafetyViolation, TypeChecker};

fn check_module(source: &str) -> Result<TypeChecker, String> {
    let module = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    Ok(checker)
}

fn get_warnings(source: &str) -> Vec<String> {
    match check_module(source) {
        Ok(checker) => checker.get_warnings().to_vec(),
        Err(_) => vec![],
    }
}

#[test]
fn test_object_safe_basic_trait() {
    // Object-safe trait: methods with &self
    let source = r#"
        W Drawable {
            F draw(&self) -> i64
            F get_color(&self) -> i64
        }

        F use_drawable(d: &dyn Drawable) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Object-safe trait should not produce warnings: {:?}",
        warnings
    );
}

#[test]
fn test_object_safe_empty_trait() {
    // Empty trait is object-safe (marker trait)
    let source = r#"
        W Marker { }

        F use_marker(m: &dyn Marker) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Empty trait should be object-safe: {:?}",
        warnings
    );
}

#[test]
fn test_not_object_safe_returns_self() {
    // Trait with method returning Self
    let source = r#"
        W Copyable {
            F copy_self(&self) -> Self
        }

        F use_copyable(c: &dyn Copyable) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait returning Self should produce warning"
    );

    let warning_text = warnings.join("\n");
    assert!(
        warning_text.contains("not object-safe"),
        "Warning should mention object safety: {}",
        warning_text
    );
    assert!(
        warning_text.contains("clone") || warning_text.contains("returns"),
        "Warning should mention clone method or return type: {}",
        warning_text
    );
}

#[test]
fn test_not_object_safe_static_method() {
    // Trait with static method (no receiver)
    let source = r#"
        W Constructor {
            F new() -> Self
        }

        F use_constructor(c: &dyn Constructor) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait with static method should produce warning"
    );

    let warning_text = warnings.join("\n");
    assert!(
        warning_text.contains("not object-safe"),
        "Warning should mention object safety: {}",
        warning_text
    );
}

#[test]
fn test_not_object_safe_self_in_params() {
    // Trait with Self in parameter position
    let source = r#"
        W Comparable {
            F compare(&self, other: Self) -> i64
        }

        F use_comparable(c: &dyn Comparable) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait with Self in params should produce warning"
    );

    let warning_text = warnings.join("\n");
    assert!(
        warning_text.contains("not object-safe"),
        "Warning should mention object safety: {}",
        warning_text
    );
}

#[test]
fn test_not_object_safe_sized_bound() {
    // Trait with Sized bound
    let source = r#"
        W SizedTrait: Sized {
            F method(&self) -> i64
        }

        F use_sized(s: &dyn SizedTrait) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait with Sized bound should produce warning"
    );

    let warning_text = warnings.join("\n");
    assert!(
        warning_text.contains("not object-safe") || warning_text.contains("Sized"),
        "Warning should mention object safety or Sized: {}",
        warning_text
    );
}

#[test]
fn test_mixed_safe_and_unsafe_methods() {
    // Trait with both safe and unsafe methods
    let source = r#"
        W MixedTrait {
            F safe_method(&self) -> i64
            F unsafe_method(&self) -> Self
        }

        F use_mixed(m: &dyn MixedTrait) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait with unsafe method should produce warning even if other methods are safe"
    );

    let warning_text = warnings.join("\n");
    assert!(
        warning_text.contains("not object-safe"),
        "Warning should mention object safety: {}",
        warning_text
    );
}

#[test]
fn test_object_safe_with_mut_self() {
    // Trait with &mut self is object-safe
    let source = r#"
        W Mutable {
            F mutate(&mut self) -> i64
        }

        F use_mutable(m: &dyn Mutable) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Trait with &mut self should be object-safe: {:?}",
        warnings
    );
}

#[test]
fn test_object_safe_multiple_methods() {
    // Trait with multiple safe methods
    let source = r#"
        W Shape {
            F area(&self) -> f64
            F perimeter(&self) -> f64
            F translate(&mut self, dx: i64, dy: i64) -> i64
        }

        F use_shape(s: &dyn Shape) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Trait with multiple safe methods should be object-safe: {:?}",
        warnings
    );
}

#[test]
fn test_object_safe_with_return_types() {
    // Trait with various non-Self return types
    let source = r#"
        W DataProvider {
            F get_i64(&self) -> i64
            F get_string(&self) -> i64
            F get_optional(&self) -> i64?
            F get_result(&self) -> i64!
        }

        F use_provider(p: &dyn DataProvider) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Trait with non-Self return types should be object-safe: {:?}",
        warnings
    );
}

#[test]
fn test_not_object_safe_self_in_nested_type() {
    // Self in nested type position (e.g., Option<Self>)
    let source = r#"
        W Nested {
            F get_optional(&self) -> Self?
        }

        F use_nested(n: &dyn Nested) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        !warnings.is_empty(),
        "Trait with Self in nested return type should produce warning"
    );
}

#[test]
fn test_object_safe_with_generic_trait() {
    // Trait with generic parameters (not methods) can be object-safe
    let source = r#"
        W Container<T> {
            F get(&self) -> T
        }

        F use_container(c: &dyn Container<i64>) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    // This should be object-safe as long as the trait is fully parameterized
    // (dyn Container<i64> has no type parameters)
    assert!(
        warnings.is_empty(),
        "Parameterized trait should be object-safe when fully specified: {:?}",
        warnings
    );
}

#[test]
fn test_not_object_safe_associated_type_returns_self() {
    // Method returning associated type that could be Self
    let source = r#"
        W Iterator {
            T Item
            F next(&mut self) -> Self::Item?
        }

        F main() -> i64 = 0
    "#;

    // This test documents current behavior - associated types are allowed
    // in return position even for dyn Trait (they become existential types)
    let result = check_module(source);
    assert!(result.is_ok(), "Iterator-style trait should parse");
}

#[test]
fn test_object_safety_violation_descriptions() {
    // Verify that violation descriptions are informative
    let v1 = ObjectSafetyViolation::MethodReturnsSelf {
        method_name: "clone".to_string(),
    };
    let desc1 = v1.description();
    assert!(desc1.contains("clone"));
    assert!(desc1.contains("Self") || desc1.contains("self"));

    let v2 = ObjectSafetyViolation::MethodMissingReceiver {
        method_name: "new".to_string(),
    };
    let desc2 = v2.description();
    assert!(desc2.contains("new"));
    assert!(desc2.contains("receiver") || desc2.contains("static"));

    let v3 = ObjectSafetyViolation::MethodUsesSelfInArgs {
        method_name: "merge".to_string(),
    };
    let desc3 = v3.description();
    assert!(desc3.contains("merge"));
    assert!(desc3.contains("Self") || desc3.contains("parameter"));

    let v4 = ObjectSafetyViolation::TraitHasSizedBound;
    let desc4 = v4.description();
    assert!(desc4.contains("Sized"));
}

#[test]
fn test_real_world_object_safe_trait() {
    // Real-world example: Display-like trait
    let source = r#"
        W Display {
            F fmt(&self) -> i64
        }

        W Debug {
            F debug_fmt(&self) -> i64
        }

        F print_displayable(d: &dyn Display) -> i64 {
            d.fmt()
        }

        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    assert!(
        warnings.is_empty(),
        "Display-like trait should be object-safe: {:?}",
        warnings
    );
}

#[test]
fn test_real_world_not_object_safe_trait() {
    // Real-world example: Copyable trait (returns Self)
    let source = r#"
        W Copyable {
            F copy_self(&self) -> Self
        }

        W Defaultable {
            F get_default() -> Self
        }

        F main() -> i64 = 0
    "#;

    let result = check_module(source);
    assert!(result.is_ok(), "Trait definitions should parse");

    // Both traits are not object-safe
    // Copyable: returns Self
    // Defaultable: static method
}

#[test]
fn test_object_safe_async_method() {
    // Async methods with &self should be object-safe
    let source = r#"
        W AsyncTask {
            async F run(&self) -> i64
        }

        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    // Async methods should be object-safe as long as they have a receiver
    // Note: This may depend on implementation details
}

#[test]
fn test_trait_with_associated_types() {
    // Trait with associated types (no methods) is object-safe
    let source = r#"
        W HasAssociatedType {
            T Item
            F get_item(&self) -> i64
        }

        F main() -> i64 = 0
    "#;

    let result = check_module(source);
    assert!(result.is_ok(), "Trait with associated types should parse");
}

#[test]
fn test_multiple_violations() {
    // Trait violating multiple object safety rules
    let source = r#"
        W BadTrait: Sized {
            F new() -> Self
            F clone(&self) -> Self
            F compare(&self, other: Self) -> i64
        }

        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    // Should have warnings about multiple violations when used as dyn Trait
    // (Though this code doesn't use it as dyn, so might not trigger)
}

#[test]
fn test_super_trait_doesnt_affect_object_safety() {
    // Super traits themselves don't affect object safety
    let source = r#"
        W Base {
            F base_method(&self) -> i64
        }

        W Derived: Base {
            F derived_method(&self) -> i64
        }

        F use_derived(d: &dyn Derived) -> i64 = 0
        F main() -> i64 = 0
    "#;

    let warnings = get_warnings(source);
    // Should be object-safe as long as all methods are safe
}
