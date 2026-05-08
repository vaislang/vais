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
trait Printable {
    fn to_num(&self) -> i64
}

type Display = Printable

fn main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse trait alias");
}

#[test]
fn e2e_trait_alias_multiple_bounds_parse() {
    // Verify multiple trait bounds in alias
    let source = r#"
trait Numeric {
    fn value(&self) -> i64
}

trait Addable {
    fn add(&self, other: i64) -> i64
}

type Combined = Numeric + Addable

fn main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse multi-bound trait alias");
}

#[test]
fn e2e_trait_alias_where_clause_parse() {
    // Verify trait alias in where clause
    let source = r#"
trait Printable {
    fn to_num(&self) -> i64
}

type Display = Printable

struct Point { x: i64, y: i64 }
impl Point: Printable {
    fn to_num(&self) -> i64 { self.x + self.y }
}

fn show<T>(val: &T) -> i64
where T: Display
{
    0
}

fn main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse trait alias in where clause");
}

#[test]
fn e2e_trait_alias_usage_in_bounds() {
    // Verify trait alias as generic bound
    let source = r#"
trait TraitA {
    fn method_a(&self) -> i64
}

trait TraitB {
    fn method_b(&self) -> i64
}

type AliasAB = TraitA + TraitB

struct MyStruct { val: i64 }
impl MyStruct: TraitA {
    fn method_a(&self) -> i64 { self.val }
}
impl MyStruct: TraitB {
    fn method_b(&self) -> i64 { self.val * 2 }
}

fn use_alias<T: AliasAB>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should use trait alias in bounds");
}

#[test]
fn e2e_trait_alias_nested_bounds() {
    // Verify nested trait alias (alias referencing alias)
    let source = r#"
trait Base {
    fn base_fn(&self) -> i64
}

type Alias1 = Base

type Alias2 = Alias1

struct Thing { n: i64 }
impl Thing: Base {
    fn base_fn(&self) -> i64 { self.n }
}

fn use_nested<T: Alias2>(x: &T) -> i64 {
    0
}

fn main() -> i64 {
    0
}
"#;
    compile_to_ir(source).expect("should parse nested trait aliases");
}

// ===== Existential Types (impl Trait) — REMOVED (ROADMAP #18) =====
// `X Trait` return-position existential types were removed from the language
// in favor of explicit generic parameters (`F foo<T: Trait>() -> T`).
// Prior tests (e2e_impl_trait_return_parse, e2e_impl_trait_multiple_bounds_parse,
// e2e_impl_trait_generic_function_parse, e2e_impl_trait_where_clause_parse) deleted.

// ===== Const Evaluation Tests =====
//
// NOTE: These tests verify that const evaluation expansion was added to the AST/parser.
// The actual evaluation may be handled at compile-time. These are parse-level tests.

#[test]
fn e2e_const_eval_basic_arithmetic() {
    // Verify basic arithmetic in const context compiles
    let source = r#"
fn main() -> i64 {
    x := 2 + 3
    y := 10 - 5
    z := 4 * 2
    w := 20 / 4
    return x + y + z + w
}
"#;
    assert_exit_code(source, 23); // 5 + 5 + 8 + 5 = 23
}

#[test]
fn e2e_const_eval_modulo_expr() {
    // Verify modulo expression parsing
    let source = r#"
fn main() -> i64 {
    x := 10 % 3
    return x
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_const_eval_bitwise_and_expr() {
    // Verify bitwise AND expression
    let source = r#"
fn main() -> i64 {
    x := 3 & 7
    return x
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_const_eval_bitwise_or_expr() {
    // Verify bitwise OR expression
    let source = r#"
fn main() -> i64 {
    x := 2 | 4
    return x
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_const_eval_shift_left_expr() {
    // Verify left shift expression
    let source = r#"
fn main() -> i64 {
    x := 1 << 3
    return x
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_const_eval_shift_right_expr() {
    // Verify right shift expression
    let source = r#"
fn main() -> i64 {
    x := 16 >> 2
    return x
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_const_eval_bitwise_xor_expr() {
    // Verify bitwise XOR expression
    let source = r#"
fn main() -> i64 {
    x := 5 ^ 3
    return x
}
"#;
    assert_exit_code(source, 6);
}

#[test]
fn e2e_const_eval_complex_expression() {
    // Verify complex nested expressions
    let source = r#"
fn main() -> i64 {
    x := (2 + 3) * 2
    y := ((1 << 4) - 2) / 2
    return x + y
}
"#;
    assert_exit_code(source, 17);
}
