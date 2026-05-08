//! Phase 90 -- Struct Operations
//!
//! Tests for struct construction, field access, methods,
//! nested structs, and struct patterns.

use super::helpers::*;

// ==================== Basic Struct ====================

#[test]
fn e2e_struct_simple_create() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn main() -> i64 {
    p := Point { x: 42, y: 10 }
    p.x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_access_second_field() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn main() -> i64 {
    p := Point { x: 10, y: 42 }
    p.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_three_fields() {
    let source = r#"
struct Vec3 { x: i64, y: i64, z: i64 }
fn main() -> i64 {
    v := Vec3 { x: 10, y: 20, z: 12 }
    v.x + v.y + v.z
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_single_field() {
    let source = r#"
struct Wrapper { value: i64 }
fn main() -> i64 {
    w := Wrapper { value: 42 }
    w.value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_ref_clone_passed_to_value_constructor() {
    let source = r#"
struct Params { value: i64 }
struct Executor { params: Params }

impl Executor {
    fn new(params: Params) -> Executor {
        Executor { params }
    }
}

fn wrap(params: &Params) -> Executor {
    Executor.new(params.clone())
}

fn main() -> i64 {
    params := Params { value: 42 }
    executor := wrap(&params)
    executor.params.value
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Struct Methods ====================

#[test]
fn e2e_struct_method_self() {
    let source = r#"
struct Counter { value: i64 }
impl Counter {
    fn get(self) -> i64 = self.value
}
fn main() -> i64 {
    c := Counter { value: 42 }
    c.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_method_with_param() {
    let source = r#"
struct Counter { value: i64 }
impl Counter {
    fn add(self, n: i64) -> i64 = self.value + n
}
fn main() -> i64 {
    c := Counter { value: 32 }
    c.add(10)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Struct as Parameter ====================

#[test]
fn e2e_struct_passed_to_function() {
    let source = r#"
struct Pair { a: i64, b: i64 }
fn sum_pair(p: Pair) -> i64 = p.a + p.b
fn main() -> i64 {
    p := Pair { a: 20, b: 22 }
    sum_pair(p)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_returned_from_function() {
    let source = r#"
struct Pair { a: i64, b: i64 }
fn make_pair(x: i64, y: i64) -> Pair = Pair { a: x, b: y }
fn main() -> i64 {
    p := make_pair(20, 22)
    p.a + p.b
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Nested Structs ====================

#[test]
fn e2e_struct_nested() {
    let source = r#"
struct Inner { val: i64 }
struct Outer { inner: Inner, extra: i64 }
fn main() -> i64 {
    o := Outer { inner: Inner { val: 32 }, extra: 10 }
    o.inner.val + o.extra
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Struct with Boolean Fields ====================

#[test]
fn e2e_struct_bool_field() {
    let source = r#"
struct Config { debug: bool, value: i64 }
fn main() -> i64 {
    c := Config { debug: true, value: 42 }
    I c.debug { c.value } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_bool_field_false() {
    let source = r#"
struct Config { debug: bool, value: i64 }
fn main() -> i64 {
    c := Config { debug: false, value: 0 }
    I c.debug { 0 } else { 42 }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Multiple Struct Instances ====================

#[test]
fn e2e_struct_multiple_instances() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn main() -> i64 {
    p1 := Point { x: 10, y: 20 }
    p2 := Point { x: 5, y: 7 }
    p1.x + p1.y + p2.x + p2.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_field_in_expression() {
    let source = r#"
struct Rect { w: i64, h: i64 }
fn area(r: Rect) -> i64 = r.w * r.h
fn main() -> i64 {
    r := Rect { w: 6, h: 7 }
    area(r)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Struct Computation Patterns ====================

#[test]
fn e2e_struct_distance_squared() {
    let source = r#"
struct Point { x: i64, y: i64 }
fn dist_sq(a: Point, b: Point) -> i64 {
    dx := a.x - b.x
    dy := a.y - b.y
    dx * dx + dy * dy
}
fn main() -> i64 {
    a := Point { x: 0, y: 0 }
    b := Point { x: 3, y: 4 }
    dist_sq(a, b)
}
"#;
    assert_exit_code(source, 25);
}

#[test]
fn e2e_struct_builder_pattern() {
    let source = r#"
struct Config { a: i64, b: i64, c: i64 }
fn default_config() -> Config = Config { a: 10, b: 20, c: 12 }
fn main() -> i64 {
    c := default_config()
    c.a + c.b + c.c
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_struct_with_zero_fields_value() {
    let source = r#"
struct Data { a: i64, b: i64 }
fn main() -> i64 {
    d := Data { a: 0, b: 42 }
    d.a + d.b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_ref_box_deref_passed_to_ref_param() {
    let source = r#"
struct Box<T> {
    ptr: i64
}

impl Box<T> {
    fn new(value: T) -> Box<T> {
        ptr := malloc(8)
        store_typed(ptr, value)
        Box { ptr }
    }
}

fn read_boxed_value(x: &i64) -> i64 {
    *x
}

fn main() -> i64 {
    b := Box.new(42)
    read_boxed_value(&*b)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_double_deref_box_struct_local_keeps_struct_type() {
    let source = r#"
struct Pair {
    a: i64,
    b: i64,
}

struct Box<T> {
    ptr: i64
}

impl Box<T> {
    fn new(value: T) -> Box<T> {
        ptr := malloc(16)
        store_typed(ptr, value)
        Box { ptr }
    }
}

fn main() -> i64 {
    b := Box.new(Pair { a: 20, b: 22 })
    pair := mut **b
    pair.a + pair.b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_vec_struct_index_assignment_accepts_cloned_value() {
    let source = r#"
struct Item {
    value: i64,
}

fn main() -> i64 {
    items: *Item = [Item { value: 0 }]
    replacement := mut Item { value: 42 }
    items[0] = replacement.clone()
    items[0].value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_generic_function_forwards_struct_param_by_value() {
    let source = r#"
struct Item {
    value: i64,
}

fn set<T>(value: T) -> type {
    value
}

fn insert<T>(value: T) -> type {
    set(value)
}

fn main() -> i64 {
    item := Item { value: 42 }
    out := insert(item)
    out.value
}
"#;
    assert_exit_code(source, 42);
}
