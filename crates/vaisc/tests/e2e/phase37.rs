//! Phase 37: Advanced type system features
//!
//! Tests for:
//! - Trait aliases (T Name = TraitA + TraitB)
//! - Existential types (impl Trait / X Trait in return position)
//! - Const evaluation expansion (bitwise, modulo, shift, negation)
//!
//! NOTE: Most tests are compile-only as Phase 37 features may still be in progress.

use super::helpers::*;

// ===== Trait Alias Tests =====

#[test]
fn e2e_trait_alias_basic_parse() {
    // Verify trait alias parsing works
    let source = r#"
W Printable {
    F to_num(&self) -> i64
}

T Display = Printable

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse trait alias");
}

#[test]
fn e2e_trait_alias_multiple_bounds_parse() {
    // Verify multiple trait bounds in alias
    let source = r#"
W Numeric {
    F value(&self) -> i64
}

W Addable {
    F add(&self, other: i64) -> i64
}

T Combined = Numeric + Addable

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse multi-bound trait alias");
}

#[test]
fn e2e_trait_alias_where_clause_parse() {
    // Verify trait alias in where clause
    let source = r#"
W Printable {
    F to_num(&self) -> i64
}

T Display = Printable

S Point { x: i64, y: i64 }
X Point: Printable {
    F to_num(&self) -> i64 { self.x + self.y }
}

F show<T>(val: &T) -> i64
where T: Display
{
    0
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse trait alias in where clause");
}

#[test]
fn e2e_trait_alias_usage_in_bounds() {
    // Verify trait alias as generic bound
    let source = r#"
W TraitA {
    F method_a(&self) -> i64
}

W TraitB {
    F method_b(&self) -> i64
}

T AliasAB = TraitA + TraitB

S MyStruct { val: i64 }
X MyStruct: TraitA {
    F method_a(&self) -> i64 { self.val }
}
X MyStruct: TraitB {
    F method_b(&self) -> i64 { self.val * 2 }
}

F use_alias<T: AliasAB>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should use trait alias in bounds");
}

#[test]
fn e2e_trait_alias_nested_bounds() {
    // Verify nested trait alias (alias referencing alias)
    let source = r#"
W Base {
    F base_fn(&self) -> i64
}

T Alias1 = Base

T Alias2 = Alias1

S Thing { n: i64 }
X Thing: Base {
    F base_fn(&self) -> i64 { self.n }
}

F use_nested<T: Alias2>(x: &T) -> i64 {
    0
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse nested trait aliases");
}

// ===== Existential Types (impl Trait) Tests =====

#[test]
fn e2e_impl_trait_return_parse() {
    // Verify impl Trait return type parsing
    let source = r#"
W Numeric {
    F value(&self) -> i64
}

S MyNum { n: i64 }
X MyNum: Numeric {
    F value(&self) -> i64 { self.n }
}

F make_num() -> X Numeric {
    MyNum { n: 42 }
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse impl trait return");
}

#[test]
fn e2e_impl_trait_multiple_bounds_parse() {
    // Verify impl Trait with multiple bounds
    let source = r#"
W TraitA {
    F method_a(&self) -> i64
}

W TraitB {
    F method_b(&self) -> i64
}

S Impl { val: i64 }
X Impl: TraitA {
    F method_a(&self) -> i64 { self.val }
}
X Impl: TraitB {
    F method_b(&self) -> i64 { self.val * 2 }
}

F make_thing() -> X TraitA + TraitB {
    Impl { val: 10 }
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse multi-bound impl trait");
}

#[test]
fn e2e_impl_trait_generic_function_parse() {
    // Verify impl Trait in generic function
    let source = r#"
W Display {
    F show(&self) -> i64
}

S MyType { n: i64 }
X MyType: Display {
    F show(&self) -> i64 { self.n }
}

F create<T: Display>(val: i64) -> X Display {
    MyType { n: val }
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse impl trait in generic");
}

#[test]
fn e2e_impl_trait_where_clause_parse() {
    // Verify impl Trait with where clause
    let source = r#"
W Trait {
    F method(&self) -> i64
}

S Thing { x: i64 }
X Thing: Trait {
    F method(&self) -> i64 { self.x }
}

F produce<T>() -> X Trait
where T: Trait
{
    Thing { x: 99 }
}

F main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse impl trait with where");
}

// ===== Const Evaluation Tests =====
//
// NOTE: These tests verify that const evaluation expansion was added to the AST/parser.
// The actual evaluation may be handled at compile-time. These are parse-level tests.

#[test]
fn e2e_const_eval_basic_arithmetic() {
    // Verify basic arithmetic in const context compiles
    let source = r#"
F main() -> i64 {
    x := 2 + 3
    y := 10 - 5
    z := 4 * 2
    w := 20 / 4
    R x + y + z + w
}
"#;
    assert_exit_code(source, 23); // 5 + 5 + 8 + 5 = 23
}

#[test]
fn e2e_const_eval_modulo_expr() {
    // Verify modulo expression parsing
    let source = r#"
F main() -> i64 {
    x := 10 % 3
    R x
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_const_eval_bitwise_and_expr() {
    // Verify bitwise AND expression
    let source = r#"
F main() -> i64 {
    x := 3 & 7
    R x
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_const_eval_bitwise_or_expr() {
    // Verify bitwise OR expression
    let source = r#"
F main() -> i64 {
    x := 2 | 4
    R x
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_const_eval_shift_left_expr() {
    // Verify left shift expression
    let source = r#"
F main() -> i64 {
    x := 1 << 3
    R x
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_const_eval_shift_right_expr() {
    // Verify right shift expression
    let source = r#"
F main() -> i64 {
    x := 16 >> 2
    R x
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_const_eval_bitwise_xor_expr() {
    // Verify bitwise XOR expression
    let source = r#"
F main() -> i64 {
    x := 5 ^ 3
    R x
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_const_eval_complex_expression() {
    // Verify complex nested expressions
    let source = r#"
F main() -> i64 {
    x := (2 + 3) * 2
    y := ((1 << 4) - 2) / 2
    R x + y
}
"#;
    assert_exit_code(source, 17);
}
