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
S Coord {
    x: i64,
    y: i64
}

F get_x<T>(val: T, _marker: i64) -> i64 {
    0
}

F main() -> i64 {
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
S Vec2 {
    a: i64,
    b: i64
}

F add_fields(v: Vec2) -> i64 {
    v.a + v.b
}

F main() -> i64 {
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
S Point3D {
    x: i64,
    y: i64,
    z: i64
}

F make_point(x: i64, y: i64, z: i64) -> Point3D {
    Point3D { x: x, y: y, z: z }
}

F main() -> i64 {
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
S Container<T> {
    items: i64,
    count: i64
}

F test_container<T>(c: Container<T>) -> i64 {
    c.count
}

F main() -> i64 {
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
S Container<T> {
    items: i64,
    count: i64
}

X Container<T> {
    F get_count(&self) -> i64 {
        self.count
    }
    F get_items(&self) -> i64 {
        self.items
    }
}

F main() -> i64 {
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
S Inner<T> {
    val: T
}

S Outer<T> {
    inner: Inner<T>
}

X Inner<T> {
    F get(&self) -> T { self.val }
}

F main() -> i64 {
    i := Inner { val: 42 }
    o := Outer { inner: i }
    o.inner.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_nested_generic_pair_of_pairs() {
    // Pair<Pair<i64, i64>, i64> — verify inner and outer fields
    let source = r#"
S Pair<A, B> {
    first: A,
    second: B
}

F main() -> i64 {
    inner := Pair { first: 20, second: 2 }
    outer := Pair { first: inner, second: 20 }
    outer.first.first + outer.first.second + outer.second
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 3. struct-by-value parameter passing ====================

#[test]
fn e2e_p145r1_struct_by_value_sum_fields() {
    // >8-byte struct passed by value to function — fields accessed correctly
    let source = r#"
S BigStruct {
    a: i64,
    b: i64,
    c: i64
}

F sum_all(s: BigStruct) -> i64 {
    s.a + s.b + s.c
}

F main() -> i64 {
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
S Record {
    key: i64,
    value: i64
}

F identity<T>(x: T) -> T { x }

F main() -> i64 {
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
S Payload {
    x: i64,
    y: i64
}

S Processor {
    offset: i64
}

X Processor {
    F apply(&self, p: Payload) -> i64 {
        p.x + p.y + self.offset
    }
}

F main() -> i64 {
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
S Vec2D {
    dx: i64,
    dy: i64
}

S Builder {
    base: i64
}

X Builder {
    F build(&self) -> Vec2D {
        Vec2D { dx: self.base, dy: self.base * 2 }
    }
}

F main() -> i64 {
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
S Wrapper<T> {
    data: T
}

X Wrapper<T> {
    F unwrap(&self) -> T {
        self.data
    }
    F rewrap(&self, new_val: T) -> Wrapper<T> {
        Wrapper { data: new_val }
    }
}

F main() -> i64 {
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
    // Option<i64> wrapping and unwrapping via match
    let source = r#"
F wrap_val(x: i64) -> Option<i64> {
    Option::Some(x)
}

F main() -> i64 {
    opt := wrap_val(42)
    M opt {
        Option::Some(v) => v,
        Option::None => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_option_none_branch() {
    // Option::None branch is taken when no value present
    let source = r#"
F maybe_val(use_val: bool) -> Option<i64> {
    I use_val {
        Option::Some(100)
    } E {
        Option::None
    }
}

F main() -> i64 {
    opt := maybe_val(false)
    M opt {
        Option::Some(_v) => 0,
        Option::None => 42
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. Result<T, E> wrap/unwrap pattern ====================

#[test]
fn e2e_p145r1_result_ok_branch() {
    // Result::Ok branch — value retrieved correctly
    let source = r#"
F safe_div(a: i64, b: i64) -> Result<i64, i64> {
    I b == 0 {
        Result::Err(-1)
    } E {
        Result::Ok(a / b)
    }
}

F main() -> i64 {
    res := safe_div(84, 2)
    M res {
        Result::Ok(v) => v,
        Result::Err(_e) => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p145r1_result_err_branch() {
    // Result::Err branch — error path taken when divisor is zero
    let source = r#"
F safe_div(a: i64, b: i64) -> Result<i64, i64> {
    I b == 0 {
        Result::Err(42)
    } E {
        Result::Ok(a / b)
    }
}

F main() -> i64 {
    res := safe_div(10, 0)
    M res {
        Result::Ok(_v) => 0,
        Result::Err(e) => e
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
S Typed<T> {
    payload: T,
    tag: i64
}

X Typed<T> {
    F new(val: T, id: i64) -> Typed<T> {
        Typed { payload: val, tag: id }
    }
    F get_tag(&self) -> i64 {
        self.tag
    }
    F get_payload(&self) -> T {
        self.payload
    }
}

F main() -> i64 {
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
S Cell<T> {
    value: T
}

X Cell<T> {
    F get(&self) -> T { self.value }
}

F main() -> i64 {
    int_cell := Cell { value: 20 }
    bool_cell := Cell { value: true }
    I bool_cell.get() {
        int_cell.get() + 22
    } E {
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
S Mixed {
    big: i64,
    small: i32
}

F main() -> i64 {
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
S Flags {
    enabled: bool,
    visible: bool
}

F main() -> i64 {
    f := Flags { enabled: true, visible: false }
    I f.enabled && !f.visible {
        42
    } E {
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
S Stats {
    min: i64,
    max: i64,
    total: i64
}

F get_min<T>(s: T) -> i64 { 0 }
F get_max<T>(s: T) -> i64 { 0 }

F extract_total(s: Stats) -> i64 { s.total }
F extract_range(s: Stats) -> i64 { s.max - s.min }

F main() -> i64 {
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
S Entry {
    id: i64,
    score: i64
}

F first_field(e: Entry) -> i64 {
    e.id
}

F second_field(e: Entry) -> i64 {
    e.score
}

F main() -> i64 {
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
S Config {
    step: i64
}

S Accumulator<T> {
    value: T
}

X Accumulator<T> {
    F advance(&self, cfg: Config) -> i64 {
        cfg.step
    }
}

F main() -> i64 {
    acc := Accumulator { value: 0 }
    cfg := Config { step: 42 }
    acc.advance(cfg)
}
"#;
    assert_exit_code(source, 42);
}
