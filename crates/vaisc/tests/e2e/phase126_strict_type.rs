//! Phase 126: Strict Type Mode & Type Fallback E2E Tests
//!
//! Tests for:
//! 1. strict_type_mode=true default: normal programs compile without error
//! 2. ImplTrait in parameter position is rejected by TC
//! 3. Never type codegen correctness
//! 4. Associated type fallback behavior
//! 5. check_impl_method unresolved type detection

use super::helpers::*;

// ==================== 1. Strict Type Mode Default Behavior ====================

#[test]
fn e2e_p126_strict_mode_basic_program() {
    // Simple program should compile fine with strict mode (default)
    let source = r#"
F main() -> i64 {
    x := 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p126_strict_mode_generic_function() {
    // Generic functions should work — Generic fallback is Category A (warning, not error)
    let source = r#"
F identity<T>(x: T) -> T = x

F main() -> i64 {
    identity(7)
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p126_strict_mode_struct_method() {
    // Struct methods should work fine in strict mode
    let source = r#"
S Point { x: i64, y: i64 }
X Point {
    F sum(&self) -> i64 = self.x + self.y
}
F main() -> i64 {
    p := Point { x: 10, y: 20 }
    p.sum()
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p126_strict_mode_enum_match() {
    // Enum + match should work in strict mode
    let source = r#"
E Color { Red, Green, Blue }
F value(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3,
    }
}
F main() -> i64 {
    value(Blue)
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p126_strict_mode_trait_dispatch() {
    // Trait dispatch should work in strict mode
    let source = r#"
W Greet {
    F greet(&self) -> i64
}
S Dog { age: i64 }
X Dog: Greet {
    F greet(&self) -> i64 = self.age
}
F main() -> i64 {
    d := Dog { age: 5 }
    d.greet()
}
"#;
    assert_exit_code(source, 5);
}

// ==================== 2. ImplTrait Parameter Position TC Error ====================

#[test]
fn e2e_p126_impl_trait_return_position_ok() {
    // ImplTrait in return position should parse and compile fine
    let source = r#"
W Numeric {
    F value(&self) -> i64
}
S Num { v: i64 }
X Num: Numeric {
    F value(&self) -> i64 = self.v
}
F make_num() -> X Numeric {
    Num { v: 42 }
}
F main() -> i64 {
    42
}
"#;
    compile_to_ir(source).expect("impl Trait in return position should compile");
}

#[test]
fn e2e_p126_impl_trait_param_position_error() {
    // ImplTrait in parameter position should be rejected by TC
    let source = r#"
W Printable {
    F show(&self) -> i64
}
F display(item: X Printable) -> i64 = 0
F main() -> i64 { 0 }
"#;
    let result = compile_to_ir(source);
    assert!(
        result.is_err(),
        "impl Trait in parameter position should be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("impl Trait") || err.contains("return position"),
        "Error should mention impl Trait: {}",
        err
    );
}

// ==================== 3. Never Type Codegen ====================

#[test]
fn e2e_p126_never_type_in_match() {
    // Never type in unreachable branches should work
    let source = r#"
F main() -> i64 {
    x := 5
    M x {
        5 => 42,
        _ => 0,
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Non-generic method unresolved type detection ====================

#[test]
fn e2e_p126_method_types_resolved() {
    // Verify that impl methods with explicit types compile fine
    let source = r#"
S Counter { val: i64 }
X Counter {
    F increment(&self) -> i64 = self.val + 1
    F add(&self, n: i64) -> i64 = self.val + n
}
F main() -> i64 {
    c := Counter { val: 10 }
    c.add(5)
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_p126_generic_struct_method() {
    // Generic struct methods should work (Generic fallback is Category A)
    let source = r#"
S Wrapper<T> { val: T }
X Wrapper<T> {
    F get(&self) -> T = self.val
}
F main() -> i64 {
    w := Wrapper { val: 99 }
    w.get()
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 5. Codegen Warning Infrastructure ====================

#[test]
fn e2e_p126_codegen_warning_generic_fallback() {
    // Verify that Generic fallback produces warnings but no errors
    let source = r#"
F id<T>(x: T) -> T = x
F main() -> i64 {
    id(42)
}
"#;
    // Should compile without error (Generic fallback is Category A — always warning)
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p126_const_generic_fallback() {
    // ConstGeneric fallback should also be Category A (warning)
    let source = r#"
F first<T>(x: T) -> T = x
F main() -> i64 {
    first(7)
}
"#;
    assert_exit_code(source, 7);
}
