//! Phase 47 — Trait impl, associated types, method dispatch E2E tests
//!
//! Tests covering:
//! - Trait impl with single/multiple methods
//! - Trait dispatch across multiple struct types
//! - Struct impl (non-trait) methods
//! - Enum impl methods
//! - Method return value used in expressions
//! - Trait + struct composition patterns

use super::helpers::*;

// ==================== 1. Trait: Single method impl ====================

#[test]
fn e2e_p47_trait_single_method() {
    // Trait with one method, implemented on a struct
    let source = r#"
W Sizeable {
    F size(self) -> i64
}
S Bag { items: i64 }
X Bag: Sizeable {
    F size(self) -> i64 { self.items }
}
F main() -> i64 {
    b := Bag { items: 7 }
    b.size()
}
"#;
    assert_exit_code(source, 7);
}

// ==================== 2. Trait: Two methods both called ====================

#[test]
fn e2e_p47_trait_two_methods_called() {
    // Trait with two methods, both exercised in main
    let source = r#"
W Stats {
    F min_val(self) -> i64
    F max_val(self) -> i64
}
S Range { lo: i64, hi: i64 }
X Range: Stats {
    F min_val(self) -> i64 { self.lo }
    F max_val(self) -> i64 { self.hi }
}
F main() -> i64 {
    r := Range { lo: 3, hi: 50 }
    r.max_val() - r.min_val()
}
"#;
    // 50 - 3 = 47
    assert_exit_code(source, 47);
}

// ==================== 3. Same trait on two structs ====================

#[test]
fn e2e_p47_trait_two_impls() {
    // Same trait implemented for two different structs
    let source = r#"
W Weight {
    F weight(self) -> i64
}
S Apple { grams: i64 }
S Stone { kg: i64 }
X Apple: Weight {
    F weight(self) -> i64 { self.grams }
}
X Stone: Weight {
    F weight(self) -> i64 { self.kg * 1000 }
}
F main() -> i64 {
    a := Apple { grams: 150 }
    s := Stone { kg: 0 }
    a.weight() - s.weight()
}
"#;
    // 150 - 0 = 150
    assert_exit_code(source, 150);
}

// ==================== 4. Trait method uses arithmetic ====================

#[test]
fn e2e_p47_trait_method_arithmetic() {
    // Trait method performs arithmetic on struct fields
    let source = r#"
W Area {
    F area(self) -> i64
}
S Rect { w: i64, h: i64 }
X Rect: Area {
    F area(self) -> i64 { self.w * self.h }
}
F main() -> i64 {
    r := Rect { w: 7, h: 8 }
    r.area()
}
"#;
    // 7 * 8 = 56
    assert_exit_code(source, 56);
}

// ==================== 5. Struct impl (no trait) single method ====================

#[test]
fn e2e_p47_struct_impl_single_method() {
    // Plain struct impl block with one method
    let source = r#"
S Counter { val: i64 }
X Counter {
    F current(self) -> i64 { self.val }
}
F main() -> i64 {
    c := Counter { val: 33 }
    c.current()
}
"#;
    assert_exit_code(source, 33);
}

// ==================== 6. Struct impl with two methods ====================

#[test]
fn e2e_p47_struct_impl_two_methods() {
    // Two methods in one impl block
    let source = r#"
S Pair { a: i64, b: i64 }
X Pair {
    F sum(self) -> i64 { self.a + self.b }
    F diff(self) -> i64 { self.a - self.b }
}
F main() -> i64 {
    p := Pair { a: 30, b: 10 }
    p.sum() - p.diff()
}
"#;
    // sum=40, diff=20, 40-20=20
    assert_exit_code(source, 20);
}

// ==================== 7. Method result used in if-else ====================

#[test]
fn e2e_p47_method_result_in_if() {
    // Method return value used as condition
    let source = r#"
S Val { n: i64 }
X Val {
    F get(self) -> i64 { self.n }
}
F main() -> i64 {
    v := Val { n: 10 }
    I v.get() > 5 { 1 } E { 0 }
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 8. Method result in arithmetic expression ====================

#[test]
fn e2e_p47_method_result_arithmetic() {
    // Method results combined in arithmetic
    let source = r#"
S Box { w: i64, h: i64 }
X Box {
    F width(self) -> i64 { self.w }
    F height(self) -> i64 { self.h }
}
F main() -> i64 {
    b := Box { w: 5, h: 9 }
    b.width() + b.height()
}
"#;
    // 5 + 9 = 14
    assert_exit_code(source, 14);
}

// ==================== 9. Trait impl with &self ====================

#[test]
fn e2e_p47_trait_ref_self() {
    // Trait method with &self (borrow)
    let source = r#"
W Readable {
    F read(&self) -> i64
}
S Sensor { reading: i64 }
X Sensor: Readable {
    F read(&self) -> i64 { self.reading }
}
F main() -> i64 {
    s := Sensor { reading: 42 }
    s.read()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 10. Multiple trait impls on same struct ====================

#[test]
fn e2e_p47_two_traits_one_struct() {
    // One struct implements two different traits
    let source = r#"
W HasWidth {
    F width(self) -> i64
}
W HasHeight {
    F height(self) -> i64
}
S Panel { w: i64, h: i64 }
X Panel: HasWidth {
    F width(self) -> i64 { self.w }
}
X Panel: HasHeight {
    F height(self) -> i64 { self.h }
}
F main() -> i64 {
    p := Panel { w: 8, h: 12 }
    p.width() + p.height()
}
"#;
    // 8 + 12 = 20
    assert_exit_code(source, 20);
}

// ==================== 11. Method called from function ====================

#[test]
fn e2e_p47_method_called_from_fn() {
    // A free function calls a method on a struct parameter
    let source = r#"
S Score { points: i64 }
X Score {
    F get(self) -> i64 { self.points }
}
F extract(s: Score) -> i64 {
    s.get()
}
F main() -> i64 {
    sc := Score { points: 88 }
    extract(sc)
}
"#;
    assert_exit_code(source, 88);
}

// ==================== 12. Trait method used in loop ====================

#[test]
fn e2e_p47_trait_method_in_loop() {
    // Trait method value accumulated in a loop
    let source = r#"
W GetVal {
    F val(self) -> i64
}
S Num { v: i64 }
X Num: GetVal {
    F val(self) -> i64 { self.v }
}
F main() -> i64 {
    n := Num { v: 5 }
    total := mut 0
    L i:0..4 {
        total = total + n.val()
    }
    total
}
"#;
    // 5 * 4 = 20
    assert_exit_code(source, 20);
}

// ==================== 13. Struct with boolean-returning method ====================

#[test]
fn e2e_p47_struct_method_bool_return() {
    // Method returns a boolean-like value (0 or 1)
    let source = r#"
S Threshold { limit: i64 }
X Threshold {
    F exceeds(self, val: i64) -> i64 {
        I val > self.limit { 1 } E { 0 }
    }
}
F main() -> i64 {
    t := Threshold { limit: 10 }
    t.exceeds(15) + t.exceeds(5)
}
"#;
    // exceeds(15)=1, exceeds(5)=0, 1+0=1
    assert_exit_code(source, 1);
}

// ==================== 14. Struct method with two params ====================

#[test]
fn e2e_p47_struct_method_two_params() {
    // Method that takes two extra parameters
    let source = r#"
S Calc { base: i64 }
X Calc {
    F compute(self, a: i64, b: i64) -> i64 {
        self.base + a * b
    }
}
F main() -> i64 {
    c := Calc { base: 10 }
    c.compute(3, 4)
}
"#;
    // 10 + 3*4 = 22
    assert_exit_code(source, 22);
}

// ==================== 15. Enum impl method ====================

#[test]
fn e2e_p47_enum_impl_method() {
    // Enum with impl block — method matches on self
    let source = r#"
E Dir {
    Up,
    Down,
    Left,
    Right
}
X Dir {
    F to_num(self) -> i64 {
        M self {
            Up => 1,
            Down => 2,
            Left => 3,
            Right => 4
        }
    }
}
F main() -> i64 {
    d := Right
    d.to_num()
}
"#;
    assert_exit_code(source, 4);
}

// ==================== 16. Enum impl with data variant ====================

#[test]
fn e2e_p47_enum_impl_data_variant() {
    // Enum method that extracts data from variant
    let source = r#"
E Value {
    Num(i64),
    None
}
X Value {
    F unwrap_or(self, default: i64) -> i64 {
        M self {
            Num(n) => n,
            None => default
        }
    }
}
F main() -> i64 {
    v := Num(42)
    v.unwrap_or(0)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 17. Enum method on None variant ====================

#[test]
fn e2e_p47_enum_method_none_variant() {
    // Same enum method but called on None variant
    let source = r#"
E MaybeInt {
    Some(i64),
    Nothing
}
X MaybeInt {
    F get_or(self, fallback: i64) -> i64 {
        M self {
            Some(n) => n,
            Nothing => fallback
        }
    }
}
F main() -> i64 {
    v := Nothing
    v.get_or(99)
}
"#;
    assert_exit_code(source, 99);
}

// ==================== 18. Trait + struct impl both on same type ====================

#[test]
fn e2e_p47_trait_and_inherent_methods() {
    // Both trait impl and inherent (non-trait) impl on same struct
    let source = r#"
W Printable {
    F code(self) -> i64
}
S Item { id: i64, qty: i64 }
X Item {
    F total(self) -> i64 { self.qty }
}
X Item: Printable {
    F code(self) -> i64 { self.id }
}
F main() -> i64 {
    it := Item { id: 10, qty: 5 }
    it.code() + it.total()
}
"#;
    // 10 + 5 = 15
    assert_exit_code(source, 15);
}

// ==================== 19. Method returning zero ====================

#[test]
fn e2e_p47_method_returns_zero() {
    // Method that always returns zero (edge case)
    let source = r#"
S Empty { x: i64 }
X Empty {
    F zero(self) -> i64 { 0 }
}
F main() -> i64 {
    e := Empty { x: 999 }
    e.zero()
}
"#;
    assert_exit_code(source, 0);
}

// ==================== 20. Struct method called multiple times ====================

#[test]
fn e2e_p47_method_called_repeatedly() {
    // Same method called multiple times on same instance
    let source = r#"
S Fixed { val: i64 }
X Fixed {
    F get(self) -> i64 { self.val }
}
F main() -> i64 {
    f := Fixed { val: 7 }
    f.get() + f.get() + f.get()
}
"#;
    // 7 + 7 + 7 = 21
    assert_exit_code(source, 21);
}

// ==================== 21. Trait impl on struct with 3 fields ====================

#[test]
fn e2e_p47_trait_impl_three_fields() {
    // Struct with 3 fields, trait method uses all
    let source = r#"
W Volume {
    F volume(self) -> i64
}
S Cuboid { l: i64, w: i64, h: i64 }
X Cuboid: Volume {
    F volume(self) -> i64 { self.l * self.w * self.h }
}
F main() -> i64 {
    c := Cuboid { l: 2, w: 3, h: 4 }
    c.volume()
}
"#;
    // 2 * 3 * 4 = 24
    assert_exit_code(source, 24);
}

// ==================== 22. Method result stored in variable ====================

#[test]
fn e2e_p47_method_result_stored() {
    // Method return value assigned to variable, then used
    let source = r#"
S Data { n: i64 }
X Data {
    F doubled(self) -> i64 { self.n * 2 }
}
F main() -> i64 {
    d := Data { n: 15 }
    result := d.doubled()
    result + 1
}
"#;
    // 15*2 + 1 = 31
    assert_exit_code(source, 31);
}

// ==================== 23. Trait dispatch: different return values ====================

#[test]
fn e2e_p47_trait_dispatch_different_returns() {
    // Two structs implementing same trait return different computed values
    let source = r#"
W Priority {
    F level(self) -> i64
}
S Urgent { factor: i64 }
S Normal { factor: i64 }
X Urgent: Priority {
    F level(self) -> i64 { self.factor * 10 }
}
X Normal: Priority {
    F level(self) -> i64 { self.factor }
}
F main() -> i64 {
    u := Urgent { factor: 3 }
    n := Normal { factor: 5 }
    u.level() + n.level()
}
"#;
    // 3*10 + 5 = 35
    assert_exit_code(source, 35);
}

// ==================== 24. Struct method with conditional logic ====================

#[test]
fn e2e_p47_struct_method_conditional() {
    // Method contains if-else logic
    let source = r#"
S Clamped { val: i64, max: i64 }
X Clamped {
    F get(self) -> i64 {
        I self.val > self.max { self.max } E { self.val }
    }
}
F main() -> i64 {
    c := Clamped { val: 200, max: 100 }
    c.get()
}
"#;
    assert_exit_code(source, 100);
}

// ==================== 25. Struct method returns field directly ====================

#[test]
fn e2e_p47_struct_method_field_direct() {
    // Simplest possible method — returns one field
    let source = r#"
S Wrapper { inner: i64 }
X Wrapper {
    F unwrap(self) -> i64 { self.inner }
}
F main() -> i64 {
    Wrapper { inner: 77 }.unwrap()
}
"#;
    assert_exit_code(source, 77);
}

// ==================== 26. Trait method result passed to function ====================

#[test]
fn e2e_p47_trait_result_passed_to_fn() {
    // Trait method result passed as argument to another function
    let source = r#"
W Source {
    F emit(self) -> i64
}
S Generator { seed: i64 }
X Generator: Source {
    F emit(self) -> i64 { self.seed + 1 }
}
F double(x: i64) -> i64 { x * 2 }
F main() -> i64 {
    g := Generator { seed: 10 }
    double(g.emit())
}
"#;
    // emit()=11, double(11)=22
    assert_exit_code(source, 22);
}
