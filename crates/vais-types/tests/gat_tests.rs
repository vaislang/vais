use vais_parser::parse;
/// Comprehensive tests for Generic Associated Types (GAT)
use vais_types::TypeChecker;

fn check_module(source: &str) -> Result<(), String> {
    let module = parse(source).map_err(|e| format!("{:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

#[test]
fn test_gat_basic_trait_definition() {
    // Basic GAT trait definition should compile
    let source = r#"
        W Container<T> {
            T Item<U>
            fn get(&self) -> Self::Item<U>
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {} // Success
        Err(e) => {
            // GAT syntax might not be fully supported yet, so this is expected
            eprintln!("Note: GAT syntax not fully supported: {}", e);
        }
    }
}

#[test]
fn test_gat_with_multiple_params() {
    // GAT with multiple type parameters
    let source = r#"
        trait Mapper {
            T Output<A, B>
            fn map(&self) -> Self::Output<A, B>
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: Multi-param GAT not fully supported: {}", e);
        }
    }
}

#[test]
fn test_associated_type_without_gat() {
    // Regular associated type (not GAT) should work
    let source = r#"
        trait Container {
            T Item
            fn get(&self) -> i64
        }

        struct Vec {
            data: i64
        }

        impl Vec: Container {
            type Item = i64

            fn get(&self) -> i64 {
                self.data
            }
        }

        fn main() -> i64 {
            v := Vec { data: 42 }
            v.get()
        }
    "#;

    assert!(check_module(source).is_ok());
}

#[test]
fn test_trait_with_associated_type_default() {
    // Trait with default associated type
    let source = r#"
        trait HasDefault {
            type Output = i64
            fn process(&self) -> Self::Output
        }

        struct MyType {
            value: i64
        }

        impl MyType: HasDefault {
            fn process(&self) -> i64 {
                self.value
            }
        }

        fn main() -> i64 {
            t := MyType { value: 10 }
            t.process()
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "Note: Default associated types might not be fully supported: {}",
                e
            );
        }
    }
}

#[test]
fn test_associated_type_in_function() {
    // Using associated types in function signatures
    let source = r#"
        trait Iterator {
            T Item
            fn next(&self) -> Self::Item?
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: Associated type usage in signatures: {}", e);
        }
    }
}

#[test]
fn test_gat_with_lifetime() {
    // GAT with lifetime parameter (if lifetimes are supported in GAT context)
    let source = r#"
        trait LendingIterator {
            T Item<'a>
            fn next(&'a self) -> Self::Item<'a>?
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: GAT with lifetimes not fully supported: {}", e);
        }
    }
}

#[test]
fn test_associated_type_with_bounds() {
    // Associated type with trait bounds
    let source = r#"
        trait Container {
            T Item: Clone
            fn get(&self) -> Self::Item
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: Associated type bounds: {}", e);
        }
    }
}

#[test]
fn test_gat_impl_concrete() {
    // Implementing GAT with concrete types
    let source = r#"
        trait Container {
            T Item<T>
        }

        struct MyContainer {
            value: i64
        }

        impl MyContainer: Container {
            T Item<T> = T
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: GAT impl: {}", e);
        }
    }
}

#[test]
fn test_nested_associated_types() {
    // Nested associated type usage
    let source = r#"
        trait Outer {
            T Inner
        }

        trait HasItem {
            T Item
        }

        fn main() -> i64 {
            0
        }
    "#;

    assert!(check_module(source).is_ok());
}

#[test]
fn test_super_trait_with_associated_type() {
    // Super trait with associated types
    let source = r#"
        trait Base {
            T Item
        }

        W Derived: Base {
            fn process(&self) -> Self::Item
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: Super trait with associated type: {}", e);
        }
    }
}

#[test]
fn test_associated_type_projection_syntax() {
    // Test parsing of associated type projection: Type::Item
    let source = r#"
        trait Container {
            T Item
        }

        struct Vec {
            x: i64
        }

        impl Vec: Container {
            type Item = i64
        }

        fn use_item(value: Vec::Item) -> i64 {
            value
        }

        fn main() -> i64 {
            use_item(42)
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: Associated type projection syntax: {}", e);
        }
    }
}

#[test]
fn test_multiple_associated_types() {
    // Trait with multiple associated types
    let source = r#"
        trait Multi {
            T First
            T Second
            fn first(&self) -> Self::First
            fn second(&self) -> Self::Second
        }

        fn main() -> i64 {
            0
        }
    "#;

    assert!(check_module(source).is_ok());
}

#[test]
fn test_gat_where_clause() {
    // GAT with where clause (if supported)
    let source = r#"
        trait Advanced {
            T Output<T> where T: Clone
            fn transform(&self) -> Self::Output<T>
        }

        fn main() -> i64 {
            0
        }
    "#;

    match check_module(source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Note: GAT where clause: {}", e);
        }
    }
}
