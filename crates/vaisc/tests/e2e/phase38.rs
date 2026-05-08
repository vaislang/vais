//! Phase 38: Advanced type system features
//!
//! Tests for:
//! - Procedural macros — #[derive(Clone, PartialEq, Default)] auto-generation

use super::helpers::*;

// ===== Procedural Macro Tests =====

#[test]
fn e2e_derive_clone_attribute() {
    // Verify #[derive(Clone)] parsing
    let source = r#"
#[derive(Clone)]
struct Point { x: i64, y: i64 }

fn main() -> i64 {
    p := Point { x: 1, y: 2 }
    0
}
"#;
    compile_to_ir(source).expect("should parse derive Clone attribute");
}

#[test]
fn e2e_derive_multiple_traits() {
    // Verify #[derive(Clone, PartialEq)] parsing
    let source = r#"
#[derive(Clone, PartialEq)]
struct Point { x: i64, y: i64 }

#[derive(Clone, PartialEq, Default)]
struct Vec3 { x: i64, y: i64, z: i64 }

fn main() -> i64 {
    p := Point { x: 1, y: 2 }
    v := Vec3 { x: 0, y: 0, z: 0 }
    0
}
"#;
    compile_to_ir(source).expect("should parse multiple derive traits");
}
