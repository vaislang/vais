use super::helpers::*;

// ==================== Phase 45: Type System & Patterns E2E Tests ====================

// ==================== Tuple Destructuring ====================

#[test]
fn e2e_phase45t_tuple_destructuring() {
    let source = r#"
F main() -> i64 {
    (a, b) := (10, 20)
    R a + b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_phase45t_tuple_destructuring_fn() {
    let source = r#"
F pair() -> (i64, i64) { (3, 7) }
F main() -> i64 {
    (x, y) := pair()
    R x + y
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Default Parameters ====================

#[test]
fn e2e_phase45t_default_param_basic() {
    let source = r#"
F add(a: i64, b: i64 = 10) -> i64 { a + b }
F main() -> i64 { add(5) }
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase45t_default_param_override() {
    let source = r#"
F add(a: i64, b: i64 = 10) -> i64 { a + b }
F main() -> i64 { add(5, 20) }
"#;
    assert_compiles(source);
}

// ==================== Contract Attributes ====================

#[test]
fn e2e_phase45t_requires_attr() {
    let source = r#"
#[requires(x > 0)]
F positive(x: i64) -> i64 { x }
F main() -> i64 { positive(5) }
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase45t_ensures_attr() {
    let source = r#"
#[ensures(return >= 0)]
F abs_val(x: i64) -> i64 { I x < 0 { -x } E { x } }
F main() -> i64 { abs_val(-3) }
"#;
    assert_compiles(source);
}

// ==================== Compound Assignment Operators ====================

#[test]
fn e2e_phase45t_compound_assign_add() {
    let source = r#"
F main() -> i64 {
    x := mut 10
    x += 5
    R x
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_phase45t_compound_assign_sub() {
    let source = r#"
F main() -> i64 {
    x := mut 20
    x -= 7
    R x
}
"#;
    assert_exit_code(source, 13);
}

#[test]
fn e2e_phase45t_compound_assign_mul() {
    let source = r#"
F main() -> i64 {
    x := mut 3
    x *= 4
    R x
}
"#;
    assert_exit_code(source, 12);
}

// ==================== Operator Precedence ====================

#[test]
fn e2e_phase45t_operator_precedence_mul_add() {
    let source = r#"
F main() -> i64 {
    R 2 + 3 * 4
}
"#;
    assert_exit_code(source, 14);
}

#[test]
fn e2e_phase45t_operator_precedence_parens() {
    let source = r#"
F main() -> i64 {
    R (2 + 3) * 4
}
"#;
    assert_exit_code(source, 20);
}

// ==================== Type Cast ====================

#[test]
fn e2e_phase45t_type_cast_parse() {
    let source = r#"
F main() -> i64 {
    x := 42
    R x as i64
}
"#;
    assert_compiles(source);
}

// ==================== Where Clause ====================

#[test]
fn e2e_phase45t_where_clause() {
    let source = r#"
W Countable {
    F count(&self) -> i64
}
F get_count<T>(x: T) -> i64 where T: Countable {
    x.count()
}
F main() -> i64 { 0 }
"#;
    assert_compiles(source);
}

// ==================== Trait Alias ====================

#[test]
fn e2e_phase45t_trait_alias_parse() {
    let source = r#"
W Showable {
    F show(&self) -> i64
}
W Countable {
    F count(&self) -> i64
}
T DisplayCount = Showable + Countable
F main() -> i64 { 0 }
"#;
    assert_compiles(source);
}

// ==================== Struct Methods ====================

#[test]
fn e2e_phase45t_struct_method() {
    let source = r#"
S Point { x: i64, y: i64 }
X Point {
    F sum(&self) -> i64 { self.x + self.y }
}
F main() -> i64 {
    p := Point { x: 3, y: 7 }
    R p.sum()
}
"#;
    assert_exit_code(source, 10);
}

// ==================== Enum Variant Match ====================

#[test]
fn e2e_phase45t_enum_variant_match() {
    let source = r#"
E Color {
    Red,
    Green,
    Blue
}
F to_num(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3
    }
}
F main() -> i64 {
    R to_num(Green)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== Nested Struct ====================

#[test]
fn e2e_phase45t_nested_struct() {
    let source = r#"
S Inner { value: i64 }
S Outer { inner: Inner, scale: i64 }
F main() -> i64 {
    o := Outer { inner: Inner { value: 5 }, scale: 3 }
    R o.inner.value * o.scale
}
"#;
    assert_exit_code(source, 15);
}

// ==================== Type Alias ====================

#[test]
fn e2e_phase45t_type_alias() {
    let source = r#"
T Num = i64
F double(x: Num) -> Num { x * 2 }
F main() -> i64 { double(21) }
"#;
    assert_compiles(source);
}
