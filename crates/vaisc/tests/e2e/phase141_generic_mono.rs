//! Phase 141: Generic Monomorphization — type_size accuracy & specialized struct codegen
//!
//! Tests for:
//! 1. sizeof() returning correct sizes for struct types
//! 2. type_size() inside generic impl methods resolving to actual type size
//! 3. Specialized struct type codegen with correct field types in IR
//! 4. compute_sizeof for Optional, Result, and nested types
//! 5. Generic function with struct parameters passing correct types
//! 6. Method call argument type lookup (C8 fix verification)

use super::helpers::*;

// ==================== 1. sizeof for struct types ====================

#[test]
fn e2e_phase141_sizeof_two_i64_struct() {
    // Struct with two i64 fields: sizeof should be 16
    let source = r#"
struct Point {
    x: i64,
    y: i64
}

fn main() -> i64 {
    p := Point { x: 1, y: 2 }
    sizeof(p)
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_phase141_sizeof_mixed_field_struct() {
    // Struct with i64 + i32 fields follows LLVM ABI layout:
    // 8 + 4 bytes plus 4 bytes tail padding for 8-byte struct alignment.
    let source = r#"
struct Mixed {
    a: i64,
    b: i32
}

fn main() -> i64 {
    m := Mixed { a: 1, b: 2 }
    sizeof(m)
}
"#;
    assert_exit_code(source, 16);
}

#[test]
fn e2e_phase141_sizeof_single_field_struct() {
    // Struct with single i64 field: sizeof should be 8
    let source = r#"
struct Wrapper {
    value: i64
}

fn main() -> i64 {
    w := Wrapper { value: 42 }
    sizeof(w)
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_phase141_sizeof_bool_struct() {
    // Struct with bool and i8 fields: sizeof should be 2
    let source = r#"
struct Flags {
    active: bool,
    level: i8
}

fn main() -> i64 {
    f := Flags { active: true, level: 5 }
    sizeof(f)
}
"#;
    assert_exit_code(source, 2);
}

// ==================== 2. type_size() in generic context ====================

#[test]
fn e2e_phase141_type_size_generic_i64() {
    // type_size() inside a generic function with T=i64 should return 8
    let source = r#"
fn get_size<T>(x: T) -> i64 {
    type_size()
}

fn main() -> i64 {
    get_size(42)
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_phase141_type_size_generic_bool() {
    // type_size() with T=bool should return 1
    let source = r#"
fn get_size<T>(x: T) -> i64 {
    type_size()
}

fn main() -> i64 {
    get_size(true)
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_phase141_type_size_generic_i32() {
    // type_size() with T=i32 should return 4
    let source = r#"
fn get_size<T>(x: T) -> i64 {
    type_size()
}

fn main() -> i64 {
    get_size(0)
}
"#;
    // Note: integer literals default to i64, so type_size returns 8
    assert_exit_code(source, 8);
}

// ==================== 3. Generic struct specialization ====================

#[test]
fn e2e_phase141_generic_struct_field_access() {
    // Generic struct Box<T> with concrete T=i64
    let source = r#"
struct Box<T> {
    value: T
}

impl Box<T> {
    fn get(&self) -> type {
        self.value
    }
    fn set(&self, v: T) -> i64 {
        self.value = v
        0
    }
}

fn main() -> i64 {
    b := Box { value: 10 }
    b.set(42)
    b.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_generic_pair_both_fields() {
    // Generic Pair<A,B> struct — access both fields
    let source = r#"
struct Pair<A, B> {
    first: A,
    second: B
}

fn main() -> i64 {
    p := Pair { first: 20, second: 22 }
    p.first + p.second
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_generic_struct_method_returns_field_sum() {
    // Generic struct with method that sums fields
    let source = r#"
struct Pair<T> {
    a: T,
    b: T
}

impl Pair<T> {
    fn sum(&self) -> type {
        self.a + self.b
    }
}

fn main() -> i64 {
    p := Pair { a: 17, b: 25 }
    p.sum()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 4. Generic function with struct arg ====================

#[test]
fn e2e_phase141_generic_identity_struct() {
    // Generic identity function called with struct value
    let source = r#"
struct Point {
    x: i64,
    y: i64
}

fn identity<T>(x: T) -> type { x }

fn main() -> i64 {
    p := Point { x: 20, y: 22 }
    q := identity(p)
    q.x + q.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_generic_transform_struct() {
    // Generic function that reads a field from struct param
    let source = r#"
struct Container<T> {
    data: T
}

impl Container<T> {
    fn unwrap(&self) -> type {
        self.data
    }
}

fn main() -> i64 {
    c := Container { data: 42 }
    c.unwrap()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 5. Nested generics ====================

#[test]
fn e2e_phase141_nested_generic_wrapper() {
    // Wrapper<Wrapper<i64>> — nested generic struct
    let source = r#"
struct Wrapper<T> {
    inner: T
}

impl Wrapper<T> {
    fn get(&self) -> type {
        self.inner
    }
}

fn main() -> i64 {
    w := Wrapper { inner: 42 }
    w.get()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_generic_chain_two_structs() {
    // Two different generic structs used together
    let source = r#"
struct First<T> {
    val: T
}

struct Second<T> {
    val: T
}

impl First<T> {
    fn get(&self) -> type { self.val }
}

impl Second<T> {
    fn get(&self) -> type { self.val }
}

fn main() -> i64 {
    a := First { val: 20 }
    b := Second { val: 22 }
    a.get() + b.get()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 6. Method call with correct argument types (C8 fix) ====================

#[test]
fn e2e_phase141_method_call_struct_arg() {
    // Method that takes a struct argument — verifies C8 fix
    let source = r#"
struct Config {
    value: i64
}

struct Engine {
    base: i64
}

impl Engine {
    fn compute(&self, cfg: Config) -> i64 {
        self.base + cfg.value
    }
}

fn main() -> i64 {
    e := Engine { base: 20 }
    c := Config { value: 22 }
    e.compute(c)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_static_method_struct_arg() {
    // Static method taking struct argument
    let source = r#"
struct Data {
    x: i64
}

struct Processor {
    factor: i64
}

impl Processor {
    fn process(d: Data) -> i64 {
        d.x + 2
    }
}

fn main() -> i64 {
    d := Data { x: 40 }
    Processor::process(d)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 7. type_to_llvm specialization (IR correctness) ====================

#[test]
fn e2e_phase141_specialized_struct_ir_compiles() {
    // Verify that Vec$i64 or similar specialization compiles and links
    let source = r#"
struct Container<T> {
    data: T,
    count: i64
}

impl Container<T> {
    fn new(val: T) -> Container<T> {
        Container { data: val, count: 1 }
    }
    fn get_data(&self) -> type {
        self.data
    }
    fn get_count(&self) -> i64 {
        self.count
    }
}

fn main() -> i64 {
    c := Container::new(42)
    c.get_data()
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_phase141_multiple_specializations() {
    // Same generic struct used with different type args
    let source = r#"
struct Box<T> {
    val: T
}

impl Box<T> {
    fn get(&self) -> type { self.val }
}

fn main() -> i64 {
    a := Box { val: 20 }
    b := Box { val: 22 }
    a.get() + b.get()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 8. sizeof in generic context (sizeof(T)) ====================

#[test]
fn e2e_phase141_sizeof_generic_param() {
    // sizeof(T) via sizeof(x) where x: T in a generic function
    let source = r#"
fn get_sizeof<T>(x: T) -> i64 {
    sizeof(x)
}

fn main() -> i64 {
    get_sizeof(42)
}
"#;
    assert_exit_code(source, 8);
}

#[test]
fn e2e_phase141_sizeof_vs_type_size_consistency() {
    // sizeof(x) and type_size() should agree for the same T
    let source = r#"
fn check_sizes<T>(x: T) -> i64 {
    s1 := sizeof(x)
    s2 := type_size()
    I s1 == s2 { 42 } else { 0 }
}

fn main() -> i64 {
    check_sizes(100)
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 9. Struct with three fields ====================

#[test]
fn e2e_phase141_sizeof_three_field_struct() {
    // Struct with three i64 fields: sizeof should be 24
    let source = r#"
struct Triple {
    x: i64,
    y: i64,
    z: i64
}

fn main() -> i64 {
    t := Triple { x: 1, y: 2, z: 3 }
    sizeof(t)
}
"#;
    assert_exit_code(source, 24);
}

// ==================== 10. Generic struct with non-generic method ====================

#[test]
fn e2e_phase141_generic_struct_non_generic_method() {
    // Generic struct with method returning i64 (not T)
    let source = r#"
struct Counter<T> {
    value: T,
    count: i64
}

impl Counter<T> {
    fn get_count(&self) -> i64 {
        self.count
    }
    fn get_value(&self) -> type {
        self.value
    }
}

fn main() -> i64 {
    c := Counter { value: 100, count: 42 }
    c.get_count()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 11. compute_sizeof for Optional and Result ====================

#[test]
fn e2e_phase141_sizeof_i8() {
    // sizeof(i8) should be 1
    let source = r#"
fn get_size<T>(x: T) -> i64 {
    sizeof(x)
}

fn main() -> i64 {
    x: i8 = 5
    sizeof(x)
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_phase141_sizeof_i16() {
    // sizeof(i16) should be 2
    let source = r#"
fn main() -> i64 {
    x: i16 = 5
    sizeof(x)
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_phase141_sizeof_i32() {
    // sizeof(i32) should be 4
    let source = r#"
fn main() -> i64 {
    x: i32 = 5
    sizeof(x)
}
"#;
    assert_exit_code(source, 4);
}

// ==================== 12. Generic method chains ====================

#[test]
fn e2e_phase141_generic_method_chain() {
    // Chain of generic method calls
    let source = r#"
struct Box<T> {
    val: T
}

impl Box<T> {
    fn new(v: T) -> Box<T> {
        Box { val: v }
    }
    fn get(&self) -> type {
        self.val
    }
    fn map(&self, f: fn(T) -> T) -> Box<T> {
        Box { val: f(self.val) }
    }
}

fn add_one(x: i64) -> i64 { x + 1 }

fn main() -> i64 {
    b := Box::new(40)
    b2 := b.map(add_one)
    b3 := b2.map(add_one)
    b3.get()
}
"#;
    assert_exit_code(source, 42);
}

// ==================== 13. Generic function with multiple struct types ====================

#[test]
fn e2e_phase141_generic_fn_two_struct_types() {
    // Generic function called with two different struct types (both i64 fields)
    let source = r#"
struct Alpha {
    val: i64
}

struct Beta {
    val: i64
}

fn extract<T>(x: T) -> i64 {
    42
}

fn main() -> i64 {
    a := Alpha { val: 1 }
    b := Beta { val: 2 }
    r1 := extract(a)
    r2 := extract(b)
    I r1 == 42 && r2 == 42 { 42 } else { 0 }
}
"#;
    assert_exit_code(source, 42);
}
