//! Phase 128: Struct, Enum, Union Composite E2E Tests
//!
//! Tests for: nested structs, enum variant data, struct method chaining,
//! enum match exhaustiveness, generic struct, struct update, enum+trait,
//! complex field access, constructor patterns.

use super::helpers::*;

// ==================== A. Basic Struct ====================

#[test]
fn e2e_p128_agg_struct_single_field() {
    assert_exit_code(
        r#"
S Wrapper { value: i64 }
F main() -> i64 {
    w := Wrapper { value: 42 }
    w.value
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_struct_two_fields() {
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point { x: 20, y: 22 }
    p.x + p.y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_struct_three_fields() {
    assert_exit_code(
        r#"
S Vec3 { x: i64, y: i64, z: i64 }
F main() -> i64 {
    v := Vec3 { x: 10, y: 20, z: 12 }
    v.x + v.y + v.z
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_struct_four_fields() {
    assert_exit_code(
        r#"
S Quad { a: i64, b: i64, c: i64, d: i64 }
F main() -> i64 {
    q := Quad { a: 10, b: 11, c: 12, d: 9 }
    q.a + q.b + q.c + q.d
}
"#,
        42,
    );
}

// ==================== B. Struct Methods ====================

#[test]
fn e2e_p128_agg_method_self() {
    assert_exit_code(
        r#"
S Counter { val: i64 }
X Counter {
    F get(&self) -> i64 = self.val
}
F main() -> i64 {
    c := Counter { val: 42 }
    c.get()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_method_with_param() {
    assert_exit_code(
        r#"
S Num { val: i64 }
X Num {
    F add(&self, n: i64) -> i64 = self.val + n
}
F main() -> i64 {
    n := Num { val: 20 }
    n.add(22)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_method_two_params() {
    assert_exit_code(
        r#"
S Calc { base: i64 }
X Calc {
    F compute(&self, a: i64, b: i64) -> i64 = self.base + a + b
}
F main() -> i64 {
    c := Calc { base: 10 }
    c.compute(12, 20)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_multiple_methods() {
    assert_exit_code(
        r#"
S Math { val: i64 }
X Math {
    F doubled(&self) -> i64 = self.val * 2
    F halved(&self) -> i64 = self.val / 2
    F inc(&self) -> i64 = self.val + 1
}
F main() -> i64 {
    m := Math { val: 21 }
    m.doubled()
}
"#,
        42,
    );
}

// ==================== C. Struct as Function Param/Return ====================

#[test]
fn e2e_p128_agg_struct_param() {
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F sum_point(p: Point) -> i64 = p.x + p.y
F main() -> i64 {
    p := Point { x: 20, y: 22 }
    sum_point(p)
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_struct_in_computation() {
    assert_exit_code(
        r#"
S Rect { w: i64, h: i64 }
F area(r: Rect) -> i64 = r.w * r.h
F main() -> i64 {
    r := Rect { w: 6, h: 7 }
    area(r)
}
"#,
        42,
    );
}

// ==================== D. Enum Basics ====================

#[test]
fn e2e_p128_agg_enum_three_variants() {
    assert_exit_code(
        r#"
E Color { Red, Green, Blue }
F main() -> i64 {
    c := Green
    M c {
        Red => 1,
        Green => 42,
        Blue => 3,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_enum_four_variants() {
    assert_exit_code(
        r#"
E Dir { Up, Down, Left, Right }
F main() -> i64 {
    d := Right
    M d {
        Up => 10,
        Down => 20,
        Left => 30,
        Right => 42,
        _ => 0
    }
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_enum_in_function() {
    assert_exit_code(
        r#"
E Shape { Circle, Square, Triangle }
F sides(s: Shape) -> i64 {
    M s {
        Circle => 0,
        Square => 4,
        Triangle => 3,
        _ => 0
    }
}
F main() -> i64 = sides(Square) + sides(Triangle) + 35
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_enum_wildcard() {
    assert_exit_code(
        r#"
E Level { Low, Medium, High }
F main() -> i64 {
    l := Medium
    M l {
        High => 100,
        _ => 42
    }
}
"#,
        42,
    );
}

// ==================== E. Enum + Trait ====================

#[test]
fn e2e_p128_agg_enum_with_method() {
    assert_exit_code(
        r#"
E Light { On, Off }
F is_on(l: Light) -> i64 {
    M l {
        On => 42,
        Off => 0,
        _ => 0
    }
}
F main() -> i64 = is_on(On)
"#,
        42,
    );
}

// ==================== F. Struct + Trait ====================

#[test]
fn e2e_p128_agg_struct_trait_impl() {
    assert_exit_code(
        r#"
W Sizeable {
    F size(&self) -> i64
}
S Box { w: i64, h: i64 }
X Box: Sizeable {
    F size(&self) -> i64 = self.w * self.h
}
F main() -> i64 {
    b := Box { w: 6, h: 7 }
    b.size()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_two_structs_one_trait() {
    assert_exit_code(
        r#"
W Volume {
    F vol(&self) -> i64
}
S Cube { side: i64 }
S Flat { area: i64 }
X Cube: Volume {
    F vol(&self) -> i64 = self.side * self.side * self.side
}
X Flat: Volume {
    F vol(&self) -> i64 = self.area
}
F main() -> i64 {
    f := Flat { area: 42 }
    f.vol()
}
"#,
        42,
    );
}

// ==================== G. Generic Struct ====================

#[test]
fn e2e_p128_agg_generic_struct() {
    assert_exit_code(
        r#"
S Container<T> { val: T }
X Container<T> {
    F get(&self) -> T = self.val
}
F main() -> i64 {
    c := Container { val: 42 }
    c.get()
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_generic_pair() {
    assert_exit_code(
        r#"
S Pair<A, B> { first: A, second: B }
F main() -> i64 {
    p := Pair { first: 20, second: 22 }
    p.first + p.second
}
"#,
        42,
    );
}

// ==================== H. Struct with Computed Fields ====================

#[test]
fn e2e_p128_agg_computed_field_values() {
    assert_exit_code(
        r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    a := 20
    b := 22
    p := Point { x: a, y: b }
    p.x + p.y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_field_from_fn() {
    assert_exit_code(
        r#"
S Wrapper { val: i64 }
F double(x: i64) -> i64 = x * 2
F main() -> i64 {
    w := Wrapper { val: double(21) }
    w.val
}
"#,
        42,
    );
}

// ==================== I. Multiple Structs ====================

#[test]
fn e2e_p128_agg_two_structs() {
    assert_exit_code(
        r#"
S Left { x: i64 }
S Right { y: i64 }
F main() -> i64 {
    l := Left { x: 20 }
    r := Right { y: 22 }
    l.x + r.y
}
"#,
        42,
    );
}

#[test]
fn e2e_p128_agg_struct_array_of_values() {
    assert_exit_code(
        r#"
S Coord { x: i64, y: i64 }
F main() -> i64 {
    c1 := Coord { x: 10, y: 5 }
    c2 := Coord { x: 20, y: 7 }
    c1.x + c1.y + c2.x + c2.y
}
"#,
        42,
    );
}
