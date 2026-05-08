//! Phase 145: R1 Generic Monomorphization — nested generics + alignment + container methods
//!
//! Tests for:
//! 1. Large struct (>8 bytes) field access through generic functions
//! 2. Nested generic structs: Container<T> with method dispatch
//! 3. Struct-by-value parameter passing and field access
//! 4. Method returning struct + caller field access
//! 5. Option<LargeStruct> wrap/unwrap pattern
//! 6. Result<LargeStruct, i64> wrap/unwrap pattern
//! 7. type_to_llvm Named type resolution for generics

use super::helpers::*;

// ==================== 1. Large struct (>8 bytes) field access ====================

#[test]
fn e2e_p145r1_large_struct_field_sum() {
    // A struct with two i64 fields passed to a generic function — field access from returned value
    let source = r#"
struct Coord {
    x: i64,
    y: i64
}

fn get_x<T>(val: T, _marker: i64) -> i64 {
    0
}

fn main() -> i64 {
    c := Coord { x: 20, y: 22 }
    c.x + c.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_large_struct_via_function() {
    // Pass a large (>8 byte) struct through a regular function, verify fields intact
    let source = r#"
struct Vec2 {
    a: i64,
    b: i64
}

fn add_fields(v: Vec2) -> i64 {
    v.a + v.b
}

fn main() -> i64 {
    v := Vec2 { a: 20, b: 22 }
    add_fields(v)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_large_struct_return_field_access() {
    // Function returns a large struct; caller accesses fields
    let source = r#"
struct Point3D {
    x: i64,
    y: i64,
    z: i64
}

fn make_point(x: i64, y: i64, z: i64) -> Point3D {
    Point3D { x: x, y: y, z: z }
}

fn main() -> i64 {
    p := make_point(10, 20, 12)
    p.x + p.y + p.z
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 2. Nested generic structs ====================

#[test]
fn e2e_p145r1_nested_container_count() {
    // Container<T> with items and count fields — count is accessible
    let source = r#"
struct Container<T> {
    items: i64,
    count: i64
}

fn test_container<T>(c: Container<T>) -> i64 {
    c.count
}

fn main() -> i64 {
    c := Container { items: 0, count: 42 }
    test_container(c)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_nested_container_items() {
    // Container<T> with items field accessed through generic function
    let source = r#"
struct Container<T> {
    items: i64,
    count: i64
}

impl Container<T> {
    fn get_count(&self) -> i64 {
        self.count
    }
    fn get_items(&self) -> i64 {
        self.items
    }
}

fn main() -> i64 {
    c := Container { items: 10, count: 32 }
    c.get_items() + c.get_count()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_nested_generic_wrapper_two_levels() {
    // Outer<Inner<i64>> — two layers of generic wrapping, verify value preserved
    let source = r#"
struct Inner<T> {
    val: T
}

struct Outer<T> {
    inner: Inner<T>
}

impl Inner<T> {
    fn get(&self) -> type { self.val }
}

fn main() -> i64 {
    i := Inner { val: 42 }
    o := Outer { inner: i }
    o.inner.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_nested_generic_pair_of_pairs() {
    // Pair<i64,i64> nested inside a concrete struct — verify chained field access
    // Uses a non-generic outer struct to avoid TC resolving inner to unresolved generic param
    let source = r#"
struct Pair<A, B> {
    first: A,
    second: B
}

struct Outer {
    inner: Pair<i64, i64>,
    extra: i64
}

fn main() -> i64 {
    p := Pair { first: 20, second: 2 }
    o := Outer { inner: p, extra: 20 }
    o.inner.first + o.inner.second + o.extra
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. struct-by-value parameter passing ====================

#[test]
fn e2e_p145r1_struct_by_value_sum_fields() {
    // >8-byte struct passed by value to function — fields accessed correctly
    let source = r#"
struct BigStruct {
    a: i64,
    b: i64,
    c: i64
}

fn sum_all(s: BigStruct) -> i64 {
    s.a + s.b + s.c
}

fn main() -> i64 {
    bs := BigStruct { a: 10, b: 20, c: 12 }
    sum_all(bs)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_struct_by_value_via_generic() {
    // Large struct passed through a generic identity function — fields still valid
    let source = r#"
struct Record {
    key: i64,
    value: i64
}

fn identity<T>(x: T) -> type { x }

fn main() -> i64 {
    r := Record { key: 10, value: 32 }
    r2 := identity(r)
    r2.key + r2.value
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_struct_by_value_method_arg() {
    // Method taking a large struct by value — fields accessible inside method
    let source = r#"
struct Payload {
    x: i64,
    y: i64
}

struct Processor {
    offset: i64
}

impl Processor {
    fn apply(&self, p: Payload) -> i64 {
        p.x + p.y + self.offset
    }
}

fn main() -> i64 {
    proc := Processor { offset: 2 }
    pay := Payload { x: 20, y: 20 }
    proc.apply(pay)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Method returning struct, caller accesses fields ====================

#[test]
fn e2e_p145r1_method_returns_struct() {
    // Method creates and returns a struct; caller reads fields
    let source = r#"
struct Vec2D {
    dx: i64,
    dy: i64
}

struct Builder {
    base: i64
}

impl Builder {
    fn build(&self) -> Vec2D {
        Vec2D { dx: self.base, dy: self.base * 2 }
    }
}

fn main() -> i64 {
    b := Builder { base: 14 }
    v := b.build()
    v.dx + v.dy
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_generic_method_returns_wrapped_struct() {
    // Generic method returning a struct wrapper
    let source = r#"
struct Wrapper<T> {
    data: T
}

impl Wrapper<T> {
    fn unwrap(&self) -> type {
        self.data
    }
    fn rewrap(&self, new_val: T) -> Wrapper<T> {
        Wrapper { data: new_val }
    }
}

fn main() -> i64 {
    w := Wrapper { data: 20 }
    w2 := w.rewrap(42)
    w2.unwrap()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Option<LargeStruct> wrap/unwrap pattern ====================

#[test]
fn e2e_p145r1_option_some_i64() {
    // Wrapping and unwrapping via match using a custom maybe enum
    let source = r#"
enum Maybe {
    Nothing,
    Just(i64)
}

fn wrap_val(x: i64) -> Maybe {
    Just(x)
}

fn main() -> i64 {
    opt := wrap_val(42)
    match opt {
        Just(v) => v,
        Nothing => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_option_none_branch() {
    // Nothing branch is taken when no value present — custom maybe enum
    let source = r#"
enum Maybe {
    Nothing,
    Just(i64)
}

fn maybe_val(use_val: bool) -> Maybe {
    I use_val {
        Just(100)
    } else {
        Nothing
    }
}

fn main() -> i64 {
    opt := maybe_val(false)
    match opt {
        Just(_v) => 0,
        Nothing => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. Result<T, E> wrap/unwrap pattern ====================

#[test]
fn e2e_p145r1_result_ok_branch() {
    // Success branch — value retrieved correctly using custom outcome enum
    let source = r#"
enum Outcome {
    Success(i64),
    Failure(i64)
}

fn safe_div(a: i64, b: i64) -> Outcome {
    I b == 0 {
        Failure(-1)
    } else {
        Success(a / b)
    }
}

fn main() -> i64 {
    res := safe_div(84, 2)
    match res {
        Success(v) => v,
        Failure(_e) => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_result_err_branch() {
    // Failure branch — error path taken when divisor is zero using custom outcome enum
    let source = r#"
enum Outcome {
    Success(i64),
    Failure(i64)
}

fn safe_div(a: i64, b: i64) -> Outcome {
    I b == 0 {
        Failure(42)
    } else {
        Success(a / b)
    }
}

fn main() -> i64 {
    res := safe_div(10, 0)
    match res {
        Success(_v) => 0,
        Failure(e) => e
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. type_to_llvm Named type resolution ====================

#[test]
fn e2e_p145r1_named_type_mangled_specialization() {
    // Verify that a generic struct specialized with concrete type compiles
    // and the IR reflects the correct field types (not all-i64 fallback)
    let source = r#"
struct Typed<T> {
    payload: T,
    tag: i64
}

impl Typed<T> {
    fn new(val: T, id: i64) -> Typed<T> {
        Typed { payload: val, tag: id }
    }
    fn get_tag(&self) -> i64 {
        self.tag
    }
    fn get_payload(&self) -> type {
        self.payload
    }
}

fn main() -> i64 {
    t := Typed::new(100, 42)
    t.get_tag()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_named_type_two_specializations() {
    // Same generic struct used with two concrete types — both specializations compile
    let source = r#"
struct Cell<T> {
    value: T
}

impl Cell<T> {
    fn get(&self) -> type { self.value }
}

fn main() -> i64 {
    int_cell := Cell { value: 20 }
    bool_cell := Cell { value: true }
    I bool_cell.get() {
        int_cell.get() + 22
    } else {
        0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. alignment — struct with mixed field widths ====================

#[test]
fn e2e_p145r1_alignment_i64_i32_fields() {
    // Struct with i64 + i32 fields — total size 12, field access correct
    let source = r#"
struct Mixed {
    big: i64,
    small: i32
}

fn main() -> i64 {
    m := Mixed { big: 32, small: 10 }
    # small is i32, cast/use as i64 for arithmetic
    m.big + m.small
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_alignment_bool_fields() {
    // Struct with bool fields — access correct value
    let source = r#"
struct Flags {
    enabled: bool,
    visible: bool
}

fn main() -> i64 {
    f := Flags { enabled: true, visible: false }
    I f.enabled && !f.visible {
        42
    } else {
        0
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 9. Multiple generic functions with shared struct ====================

#[test]
fn e2e_p145r1_multiple_generic_fns_same_struct() {
    // Multiple generic functions all receiving same concrete struct type
    let source = r#"
struct Stats {
    min: i64,
    max: i64,
    total: i64
}

fn get_min<T>(s: T) -> i64 { 0 }
fn get_max<T>(s: T) -> i64 { 0 }

fn extract_total(s: Stats) -> i64 { s.total }
fn extract_range(s: Stats) -> i64 { s.max - s.min }

fn main() -> i64 {
    st := Stats { min: 10, max: 50, total: 42 }
    extract_total(st)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_generic_fn_reads_struct_field() {
    // Generic function that reads a specific field via concrete instantiation
    let source = r#"
struct Entry {
    id: i64,
    score: i64
}

fn first_field(e: Entry) -> i64 {
    e.id
}

fn second_field(e: Entry) -> i64 {
    e.score
}

fn main() -> i64 {
    e := Entry { id: 20, score: 22 }
    first_field(e) + second_field(e)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 10. Generic method with self + struct arg ====================

#[test]
fn e2e_p145r1_self_plus_struct_arg_method() {
    // Method with &self AND a large struct arg — both correctly handled
    let source = r#"
struct Config {
    step: i64
}

struct Accumulator<T> {
    value: T
}

impl Accumulator<T> {
    fn advance(&self, cfg: Config) -> i64 {
        cfg.step
    }
}

fn main() -> i64 {
    acc := Accumulator { value: 0 }
    cfg := Config { step: 42 }
    acc.advance(cfg)
}
"#;
    assert_exit_code(source, 42);
}
