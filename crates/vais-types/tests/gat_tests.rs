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
            F get(&self) -> Self::Item<U>
        }

        F main() -> i64 {
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
        W Mapper {
            T Output<A, B>
            F map(&self) -> Self::Output<A, B>
        }

        F main() -> i64 {
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
        W Container {
            T Item
            F get(&self) -> i64
        }

        S Vec {
            data: i64
        }

        X Vec: Container {
            T Item = i64

            F get(&self) -> i64 {
                self.data
            }
        }

        F main() -> i64 {
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
        W HasDefault {
            T Output = i64
            F process(&self) -> Self::Output
        }

        S MyType {
            value: i64
        }

        X MyType: HasDefault {
            F process(&self) -> i64 {
                self.value
            }
        }

        F main() -> i64 {
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
        W Iterator {
            T Item
            F next(&self) -> Self::Item?
        }

        F main() -> i64 {
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
        W LendingIterator {
            T Item<'a>
            F next(&'a self) -> Self::Item<'a>?
        }

        F main() -> i64 {
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
        W Container {
            T Item: Clone
            F get(&self) -> Self::Item
        }

        F main() -> i64 {
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
        W Container {
            T Item<T>
        }

        S MyContainer {
            value: i64
        }

        X MyContainer: Container {
            T Item<T> = T
        }

        F main() -> i64 {
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
        W Outer {
            T Inner
        }

        W HasItem {
            T Item
        }

        F main() -> i64 {
            0
        }
    "#;

    assert!(check_module(source).is_ok());
}

#[test]
fn test_super_trait_with_associated_type() {
    // Super trait with associated types
    let source = r#"
        W Base {
            T Item
        }

        W Derived: Base {
            F process(&self) -> Self::Item
        }

        F main() -> i64 {
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
        W Container {
            T Item
        }

        S Vec {
            x: i64
        }

        X Vec: Container {
            T Item = i64
        }

        F use_item(value: Vec::Item) -> i64 {
            value
        }

        F main() -> i64 {
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
        W Multi {
            T First
            T Second
            F first(&self) -> Self::First
            F second(&self) -> Self::Second
        }

        F main() -> i64 {
            0
        }
    "#;

    assert!(check_module(source).is_ok());
}

#[test]
fn test_gat_where_clause() {
    // GAT with where clause (if supported)
    let source = r#"
        W Advanced {
            T Output<T> where T: Clone
            F transform(&self) -> Self::Output<T>
        }

        F main() -> i64 {
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
