//! Phase 136 — Standard Library Compilation Verification E2E Tests
//!
//! Tests that core stdlib module patterns compile and execute correctly.
//! Each test inlines the relevant stdlib logic to verify codegen correctness.
//! Covers: vec, string, hashmap, option, result, math, json, io, file,
//!         memory, hash, set, deque, arena, base64

use super::helpers::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// ==================== 1. Vec ====================

#[test]
fn e2e_p136_stdlib_vec_push_pop() {
    // Vec<i64>: push 3 elements, pop them, verify LIFO order
    let source = r#"
fn main() -> i64 {
    data := malloc(64)
    len := mut 0
    cap := 8

    # push 10, 20, 30
    store_i64(data + len * 8, 10)
    len = len + 1
    store_i64(data + len * 8, 20)
    len = len + 1
    store_i64(data + len * 8, 30)
    len = len + 1

    result := mut 0
    # pop -> 30
    len = len - 1
    I load_i64(data + len * 8) == 30 { result = result + 1 }
    # pop -> 20
    len = len - 1
    I load_i64(data + len * 8) == 20 { result = result + 1 }
    # pop -> 10
    len = len - 1
    I load_i64(data + len * 8) == 10 { result = result + 1 }
    I len == 0 { result = result + 1 }

    free(data)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_stdlib_vec_grow() {
    // Vec grow pattern: realloc when capacity exceeded
    let source = r#"
fn main() -> i64 {
    cap := mut 2
    len := mut 0
    data := mut malloc(cap * 8)

    # Push 4 elements (needs grow at element 3)
    i := mut 0
    L {
        I i >= 4 { B }
        I len >= cap {
            new_cap := cap * 2
            new_data := malloc(new_cap * 8)
            memcpy(new_data, data, len * 8)
            free(data)
            data = new_data
            cap = new_cap
        }
        store_i64(data + len * 8, (i + 1) * 10)
        len = len + 1
        i = i + 1
    }

    result := mut 0
    I len == 4 { result = result + 1 }
    I cap == 4 { result = result + 1 }
    I load_i64(data) == 10 { result = result + 1 }
    I load_i64(data + 24) == 40 { result = result + 1 }

    free(data)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_stdlib_vec_map_filter() {
    // Higher-order: map (double) and filter (>10)
    let source = r#"
fn main() -> i64 {
    data := malloc(40)
    store_i64(data, 3)
    store_i64(data + 8, 7)
    store_i64(data + 16, 2)
    store_i64(data + 24, 8)
    store_i64(data + 32, 1)
    len := 5

    # Map: double each
    mapped := malloc(40)
    i := mut 0
    L {
        I i >= len { B }
        store_i64(mapped + i * 8, load_i64(data + i * 8) * 2)
        i = i + 1
    }

    # Filter: keep > 10
    filtered := malloc(40)
    flen := mut 0
    i = 0
    L {
        I i >= len { B }
        val := load_i64(mapped + i * 8)
        I val > 10 {
            store_i64(filtered + flen * 8, val)
            flen = flen + 1
        }
        i = i + 1
    }

    result := mut 0
    I flen == 2 { result = result + 1 }
    I load_i64(filtered) == 14 { result = result + 1 }     # 7*2=14
    I load_i64(filtered + 8) == 16 { result = result + 1 }  # 8*2=16

    free(data)
    free(mapped)
    free(filtered)
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== 2. String ====================

#[test]
fn e2e_p136_stdlib_string_push_char() {
    // String: push_char builds a string byte by byte
    let source = r#"
fn main() -> i64 {
    cap := 16
    data := malloc(cap)
    store_byte(data, 0)
    len := mut 0

    # Push 'H' 'i'
    store_byte(data + len, 72)
    len = len + 1
    store_byte(data + len, 0)

    store_byte(data + len, 105)
    len = len + 1
    store_byte(data + len, 0)

    result := mut 0
    I len == 2 { result = result + 1 }
    I load_byte(data) == 72 { result = result + 1 }
    I load_byte(data + 1) == 105 { result = result + 1 }
    I load_byte(data + 2) == 0 { result = result + 1 }

    free(data)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_stdlib_string_concat() {
    // String concat: "ab" + "cd" = "abcd"
    let source = r#"
fn main() -> i64 {
    a := str_to_ptr("ab")
    b := str_to_ptr("cd")
    a_len := 2
    b_len := 2
    new_len := a_len + b_len
    buf := malloc(new_len + 1)
    memcpy(buf, a, a_len)
    memcpy(buf + a_len, b, b_len + 1)

    result := mut 0
    I load_byte(buf) == 97 { result = result + 1 }       # 'a'
    I load_byte(buf + 1) == 98 { result = result + 1 }    # 'b'
    I load_byte(buf + 2) == 99 { result = result + 1 }    # 'c'
    I load_byte(buf + 3) == 100 { result = result + 1 }   # 'd'
    I load_byte(buf + 4) == 0 { result = result + 1 }     # null

    free(buf)
    result
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p136_stdlib_string_to_upper() {
    // ASCII uppercase conversion
    let source = r#"
fn to_upper_byte(c: i64) -> i64 {
    I c >= 97 && c <= 122 { c - 32 } else { c }
}

fn main() -> i64 {
    s := str_to_ptr("hello")
    buf := malloc(6)
    i := mut 0
    L {
        I i >= 5 { B }
        store_byte(buf + i, to_upper_byte(load_byte(s + i)))
        i = i + 1
    }
    store_byte(buf + 5, 0)

    result := mut 0
    I load_byte(buf) == 72 { result = result + 1 }     # 'H'
    I load_byte(buf + 1) == 69 { result = result + 1 }  # 'E'
    I load_byte(buf + 2) == 76 { result = result + 1 }  # 'L'
    I load_byte(buf + 3) == 76 { result = result + 1 }  # 'L'
    I load_byte(buf + 4) == 79 { result = result + 1 }  # 'O'

    free(buf)
    result
}
"#;
    assert_exit_code(source, 5);
}

// ==================== 3. HashMap ====================

#[test]
fn e2e_p136_stdlib_hashmap_set_get() {
    // Simple hash map: set and get with chaining
    let source = r#"
fn mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } else { h }
}

fn main() -> i64 {
    cap := 8
    buckets := malloc(cap * 8)
    # Init buckets to 0
    i := mut 0
    L {
        I i >= cap { B }
        store_i64(buckets + i * 8, 0)
        i = i + 1
    }

    # Insert key=42 value=100
    h := mult_hash(42) % cap
    entry := malloc(24)
    store_i64(entry, 42)       # key
    store_i64(entry + 8, 100)  # value
    store_i64(entry + 16, 0)   # next
    store_i64(buckets + h * 8, entry)

    # Insert key=7 value=200
    h2 := mult_hash(7) % cap
    entry2 := malloc(24)
    store_i64(entry2, 7)
    store_i64(entry2 + 8, 200)
    store_i64(entry2 + 16, load_i64(buckets + h2 * 8))
    store_i64(buckets + h2 * 8, entry2)

    # Get key=42
    result := mut 0
    gh := mult_hash(42) % cap
    ptr := mut load_i64(buckets + gh * 8)
    L {
        I ptr == 0 { B }
        I load_i64(ptr) == 42 {
            I load_i64(ptr + 8) == 100 { result = result + 1 }
            B
        }
        ptr = load_i64(ptr + 16)
    }

    # Get key=7
    gh2 := mult_hash(7) % cap
    ptr2 := mut load_i64(buckets + gh2 * 8)
    L {
        I ptr2 == 0 { B }
        I load_i64(ptr2) == 7 {
            I load_i64(ptr2 + 8) == 200 { result = result + 1 }
            B
        }
        ptr2 = load_i64(ptr2 + 16)
    }

    # Cleanup
    free(entry)
    free(entry2)
    free(buckets)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_stdlib_hashmap_contains() {
    // HashMap contains check
    let source = r#"
fn mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } else { h }
}

fn contains(buckets: i64, cap: i64, key: i64) -> i64 {
    h := mult_hash(key) % cap
    ptr := mut load_i64(buckets + h * 8)
    L {
        I ptr == 0 { return 0 }
        I load_i64(ptr) == key { return 1 }
        ptr = load_i64(ptr + 16)
    }
    0
}

fn main() -> i64 {
    cap := 8
    buckets := malloc(cap * 8)
    i := mut 0
    L {
        I i >= cap { B }
        store_i64(buckets + i * 8, 0)
        i = i + 1
    }

    # Insert key=5
    h := mult_hash(5) % cap
    e := malloc(24)
    store_i64(e, 5)
    store_i64(e + 8, 99)
    store_i64(e + 16, 0)
    store_i64(buckets + h * 8, e)

    result := mut 0
    I contains(buckets, cap, 5) == 1 { result = result + 1 }
    I contains(buckets, cap, 99) == 0 { result = result + 1 }

    free(e)
    free(buckets)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_stdlib_hashmap_keys_values_typed_vec() {
    let source = r#"
use std/hashmap
use std/vec

struct Pair {
    left: i64,
    right: i64,
}

fn main() -> i64 {
    m: HashMap<i64, Pair> := mut HashMap.with_capacity(8)
    m.insert(10, Pair { left: 11, right: 1 })
    m.insert(20, Pair { left: 22, right: 2 })

    keys: Vec<i64> := mut m.keys()
    vals: Vec<Pair> := mut m.values()

    I keys.len() != 2 { return 1 }
    I vals.len() != 2 { return 2 }

    key_sum := mut 0
    i := mut 0
    L {
        I i >= keys.len() { B }
        key_sum = key_sum + keys[i]
        i = i + 1
    }

    value_sum := mut 0
    j := mut 0
    L {
        I j >= vals.len() { B }
        p := vals[j]
        value_sum = value_sum + p.left + p.right
        j = j + 1
    }

    I key_sum != 30 { return 3 }
    I value_sum != 36 { return 4 }
    return 0
}
"#;

    let temp = TempDir::new().expect("failed to create std HashMap smoke temp dir");
    let source_path = temp.path().join("hashmap_keys_values_typed_vec.vais");
    let exe_path = temp.path().join("hashmap_keys_values_typed_vec");
    std::fs::write(&source_path, source).expect("failed to write std HashMap smoke source");

    let compiler_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root");
    let build = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&source_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("--force-rebuild")
        .env("VAIS_STD_PATH", compiler_root.join("std"))
        .output()
        .expect("failed to spawn vaisc std HashMap smoke build");

    assert!(
        build.status.success(),
        "std HashMap keys()/values() typed Vec smoke failed to build.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe_path)
        .output()
        .expect("failed to run std HashMap smoke executable");

    assert_eq!(
        run.status.code().unwrap_or(-1),
        0,
        "std HashMap keys()/values() typed Vec smoke exited unexpectedly.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

// ==================== 4. Option ====================

#[test]
fn e2e_p136_stdlib_option_some_none() {
    // Option: tag-based Some/None
    let source = r#"
enum Option {
    None,
    Some(i64)
}

fn main() -> i64 {
    a := Some(42)
    b := None

    result := mut 0
    match a {
        Some(v) => { I v == 42 { result = result + 1 } },
        None => { }
    }
    match b {
        Some(_) => { },
        None => { result = result + 1 }
    }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_stdlib_option_unwrap_or() {
    // Option unwrap_or pattern
    let source = r#"
enum Option {
    None,
    Some(i64)
}

fn unwrap_or(opt: Option, default: i64) -> i64 {
    match opt {
        Some(v) => v,
        None => default
    }
}

fn main() -> i64 {
    a := Some(10)
    b := None

    result := mut 0
    I unwrap_or(a, 0) == 10 { result = result + 1 }
    I unwrap_or(b, 99) == 99 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_qualified_option_constructor_implicit_block_return() {
    // Regression: `Option.Some(...)` in block tail position must be loaded
    // from the constructor alloca before returning the enum aggregate.
    let source = r#"
enum Option {
    None,
    Some(i64)
}

fn pick(flag: i64) -> Option {
    I flag == 0 {
        return Option.None
    }
    Option.Some(42)
}

fn main() -> i64 {
    opt := pick(1)
    match opt {
        Some(v) => v,
        None => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p136_tail_if_option_none_uses_return_context() {
    // Regression: an unqualified `None` in an implicit-return if branch must
    // use the function's expected enum type, not an arbitrary same-named enum
    // variant from the global registry.
    let source = r#"
enum QuantizationStrategy {
    None
}

enum Option {
    None,
    Some(i64)
}

fn pick(flag: i64) -> Option {
    I flag == 0 {
        None
    } else {
        Some(42)
    }
}

fn main() -> i64 {
    opt := pick(1)
    match opt {
        Some(v) => v,
        None => 0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p136_option_none_local_promotes_to_str_across_match_arm() {
    // Regression: assigning Option.Some(str) to a mutable local initialized
    // with Option.None inside a match arm must update the outer local's type.
    // Otherwise the later Some(v) binding is lowered as i64 instead of str.
    let source = r#"
enum Option {
    None,
    Some(str)
}

fn score(s: &str) -> i64 {
    s.len()
}

fn main() -> i64 {
    col := mut Option.None

    match 1 {
        1 => {
            col = Option.Some("abc")
        },
        _ => {}
    }

    match col {
        Option.Some(v) => score(v),
        Option.None => 0
    }
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p136_struct_field_none_uses_field_context() {
    // Regression: a bare None in a struct literal field must use that field's
    // enum type, not another enum that happens to have a None variant.
    let source = r#"
enum QuantizationStrategy {
    None
}

enum Option {
    None,
    Some(i64)
}

struct Holder {
    value: Option
}

fn main() -> i64 {
    holder := Holder { value: None }
    match holder.value {
        Some(v) => v,
        None => 7
    }
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p136_nested_match_str_does_not_use_function_return_context() {
    // Regression: a nested match used for a local str must not inherit the
    // enclosing function's Result return type as its phi type.
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

enum SqlValue {
    StringVal { v: str },
    Null
}

fn text_len(value: &SqlValue) -> Result {
    fallback := mut "fallback"
    text := mut match value {
        SqlValue.StringVal { v } => v.clone(),
        _ => fallback
    }
    Ok(text.len())
}

fn main() -> i64 {
    val := SqlValue.StringVal { v: "abc" }
    result := text_len(&val)
    match result {
        Ok(v) => v,
        Err(e) => e
    }
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p136_field_assignment_none_uses_field_context() {
    // Regression: a bare None on the RHS of a struct-field assignment must use
    // the field's enum type, not another same-named unit variant.
    let source = r#"
enum QuantizationStrategy {
    None
}

enum MyOption {
    None,
    Some(i64)
}

struct Holder {
    value: MyOption
}

fn clear(mut holder: Holder) -> Holder {
    holder.value = None
    holder
}

fn main() -> i64 {
    holder := Holder { value: MyOption.Some(3) }
    cleared := clear(holder)
    match cleared.value {
        Some(v) => v,
        None => 9
    }
}
"#;
    assert_exit_code(source, 9);
}

#[test]
fn e2e_p136_impl_method_self_return_resolves_to_impl_type() {
    // Regression: impl methods returning `Self` must lower to the concrete impl
    // struct type, not an undefined `%Self` LLVM type.
    let source = r#"
struct Builder {
    value: i64
}

impl Builder {
    fn set(mut self, value: i64) -> Self {
        self.value = value
        self
    }
}

fn main() -> i64 {
    builder := Builder { value: 0 }
    _updated := builder.set(42)
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_p136_mut_unit_variant_let_uses_inferred_enum_context() {
    // Regression: `x := mut None` in a non-tail statement must use the enum type
    // inferred from later assignments/return, not an arbitrary `None` variant.
    let source = r#"
enum Option {
    None
}

enum Strategy {
    None,
    Scalar,
    PQ
}

struct Manager {
    strategy: Strategy
}

impl Manager {
    fn select(mut self, use_scalar: bool) -> Strategy {
        selected := mut None
        I use_scalar {
            selected = Scalar
        } else {
            selected = PQ
        }
        self.strategy = selected
        selected
    }
}

fn main() -> i64 {
    manager := Manager { strategy: Strategy.None }
    selected := manager.select(true)
    match selected {
        Scalar => 11,
        PQ => 22,
        None => 33
    }
}
"#;
    assert_exit_code(source, 11);
}

#[test]
fn e2e_p136_match_unit_try_phi_uses_actual_success_block() {
    // Regression: a match arm containing `Result<(), E>?` must record the
    // `try_ok` continuation as the phi predecessor, not the original arm label.
    let source = r#"
enum Result<T, E> {
    Ok(T),
    Err(E)
}

enum Strategy {
    None,
    Scalar,
    PQ
}

fn ok_unit() -> Result<(), i64> {
    Ok(())
}

fn train_enum(strategy: Strategy) -> Result<(), i64> {
    match strategy {
        None => {
            Ok(())
        },
        Scalar => {
            ok_unit()?;
            Ok(())
        },
        PQ => {
            ok_unit()?;
            Ok(())
        },
    }
}

fn train_tag(tag: i64) -> Result<(), i64> {
    match tag {
        0 => {
            ok_unit()?;
            Ok(())
        },
        1 => {
            Ok(())
        },
        _ => {
            return Err(9)
        }
    }
}

fn main() -> i64 {
    enum_result := train_enum(Scalar)
    tag_result := train_tag(0)
    total := mut 0
    match enum_result {
        Ok(_) => { total = total + 10 },
        Err(e) => { total = total + e }
    }
    match tag_result {
        Ok(_) => { total = total + 20 },
        Err(e) => { total = total + e }
    }
    total
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_p136_option_ok_or_try_preserves_result_abi() {
    // Regression: Option<T>.ok_or(E)? must lower to the same Named Result ABI
    // used by Ok/Err constructors, not to an erased i64 or an undeclared method.
    let source = r#"
enum Option<T> {
    None,
    Some(T)
}

enum Result<T, E> {
    Ok(T),
    Err(E)
}

struct BuildError {
    code: i64
}

struct Builder {
    name: Option<str>
}

impl Builder {
    fn build(self) -> Result<str, BuildError> {
        Ok(self.name.ok_or(BuildError { code: 7 })?)
    }
}

fn main() -> i64 {
    ok_builder := Builder { name: Some("abc") }
    err_builder := Builder { name: None }
    ok_result := ok_builder.build()
    err_result := err_builder.build()

    total := mut 0
    match ok_result {
        Ok(v) => { total = total + v.len() },
        Err(e) => { total = total + e.code }
    }
    match err_result {
        Ok(v) => { total = total + v.len() },
        Err(e) => { total = total + e.code }
    }
    total
}
"#;
    assert_exit_code(source, 10);
}

// ==================== 5. Result ====================

#[test]
fn e2e_p136_stdlib_result_ok_err() {
    // Result: Ok/Err pattern matching
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn divide(a: i64, b: i64) -> Result {
    I b == 0 { Err(1) }
    else { Ok(a / b) }
}

fn main() -> i64 {
    r1 := divide(10, 2)
    r2 := divide(10, 0)

    result := mut 0
    match r1 {
        Ok(v) => { I v == 5 { result = result + 1 } },
        Err(_) => { }
    }
    match r2 {
        Ok(_) => { },
        Err(e) => { I e == 1 { result = result + 1 } }
    }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_stdlib_result_is_ok_is_err() {
    // Result: is_ok / is_err checks
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn is_ok(r: Result) -> i64 {
    match r { Ok(_) => 1, Err(_) => 0 }
}
fn is_err(r: Result) -> i64 {
    match r { Ok(_) => 0, Err(_) => 1 }
}

fn main() -> i64 {
    result := mut 0
    I is_ok(Ok(1)) == 1 { result = result + 1 }
    I is_err(Err(2)) == 1 { result = result + 1 }
    I is_ok(Err(3)) == 0 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== 6. Math ====================

#[test]
fn e2e_p136_stdlib_math_abs_minmax() {
    // Math: abs, min, max, clamp for i64
    let source = r#"
fn abs_i64(x: i64) -> i64 {
    I x < 0 { 0 - x } else { x }
}
fn min_i64(a: i64, b: i64) -> i64 {
    I a < b { a } else { b }
}
fn max_i64(a: i64, b: i64) -> i64 {
    I a > b { a } else { b }
}
fn clamp_i64(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { lo } else I x > hi { hi } else { x }
}

fn main() -> i64 {
    result := mut 0
    I abs_i64(0 - 5) == 5 { result = result + 1 }
    I abs_i64(3) == 3 { result = result + 1 }
    I min_i64(3, 7) == 3 { result = result + 1 }
    I max_i64(3, 7) == 7 { result = result + 1 }
    I clamp_i64(10, 0, 5) == 5 { result = result + 1 }
    I clamp_i64(0 - 3, 0, 5) == 0 { result = result + 1 }
    I clamp_i64(3, 0, 5) == 3 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p136_stdlib_math_power() {
    // Integer power (recursive)
    let source = r#"
fn ipow(base: i64, exp: i64) -> i64 {
    I exp == 0 { 1 }
    else { base * ipow(base, exp - 1) }
}

fn main() -> i64 {
    result := mut 0
    I ipow(2, 0) == 1 { result = result + 1 }
    I ipow(2, 10) == 1024 { result = result + 1 }
    I ipow(3, 4) == 81 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== 7. JSON ====================

#[test]
fn e2e_p136_stdlib_json_value_types() {
    // JSON value discriminant: null=0, bool=1, number=2
    let source = r#"
fn json_new(tag: i64, data: i64) -> i64 {
    v := malloc(24)
    store_i64(v, tag)
    store_i64(v + 8, data)
    store_i64(v + 16, 0)
    v
}

fn json_tag(v: i64) -> i64 { load_i64(v) }
fn json_data(v: i64) -> i64 { load_i64(v + 8) }

fn main() -> i64 {
    null_v := json_new(0, 0)
    bool_v := json_new(1, 1)
    num_v := json_new(2, 42)

    result := mut 0
    I json_tag(null_v) == 0 { result = result + 1 }
    I json_tag(bool_v) == 1 { result = result + 1 }
    I json_data(bool_v) == 1 { result = result + 1 }
    I json_tag(num_v) == 2 { result = result + 1 }
    I json_data(num_v) == 42 { result = result + 1 }

    free(null_v)
    free(bool_v)
    free(num_v)
    result
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p136_stdlib_json_object_set_get() {
    // JSON object: simple hash-based key-value store
    let source = r#"
fn hash_str(s: i64, len: i64) -> i64 {
    hash_str_rec(s, len, 0, 5381)
}
fn hash_str_rec(s: i64, len: i64, idx: i64, h: i64) -> i64 {
    I idx >= len { I h < 0 { 0 - h } else { h } }
    else {
        c := load_byte(s + idx)
        hash_str_rec(s, len, idx + 1, h * 33 + c)
    }
}

fn main() -> i64 {
    cap := 16
    buckets := malloc(cap * 8)
    i := mut 0
    L { I i >= cap { B } store_i64(buckets + i * 8, 0) i = i + 1 }

    # Store "age" -> 25
    key := str_to_ptr("age")
    h := hash_str(key, 3) % cap
    entry := malloc(32)
    store_i64(entry, key)
    store_i64(entry + 8, 3)     # key_len
    store_i64(entry + 16, 25)   # value
    store_i64(entry + 24, 0)    # next
    store_i64(buckets + h * 8, entry)

    # Retrieve "age"
    h2 := hash_str(key, 3) % cap
    ptr := load_i64(buckets + h2 * 8)
    result := mut 0
    I ptr != 0 {
        I load_i64(ptr + 16) == 25 { result = result + 1 }
    }

    free(entry)
    free(buckets)
    result
}
"#;
    assert_exit_code(source, 1);
}

// ==================== 8. IO ====================

#[test]
fn e2e_p136_stdlib_io_puts() {
    // IO: basic output with puts
    let source = r#"
fn main() -> i64 {
    puts("hello")
    42
}
"#;
    assert_stdout_contains(source, "hello");
}

#[test]
fn e2e_p136_stdlib_io_buffer_ops() {
    // IO: buffer read/write pattern
    let source = r#"
fn main() -> i64 {
    buf := malloc(32)
    # Write message into buffer
    store_byte(buf, 65)     # 'A'
    store_byte(buf + 1, 66) # 'B'
    store_byte(buf + 2, 67) # 'C'
    store_byte(buf + 3, 0)

    result := mut 0
    I load_byte(buf) == 65 { result = result + 1 }
    I load_byte(buf + 1) == 66 { result = result + 1 }
    I load_byte(buf + 2) == 67 { result = result + 1 }
    I load_byte(buf + 3) == 0 { result = result + 1 }

    free(buf)
    result
}
"#;
    assert_exit_code(source, 4);
}

// ==================== 9. File ====================

#[test]
fn e2e_p136_stdlib_file_struct() {
    // File: struct pattern with handle/mode
    let source = r#"
struct File {
    handle: i64,
    mode: i64
}

fn file_is_open(f_handle: i64) -> i64 {
    I f_handle != 0 { 1 } else { 0 }
}

fn main() -> i64 {
    result := mut 0
    I file_is_open(0) == 0 { result = result + 1 }
    I file_is_open(12345) == 1 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p136_stdlib_file_seek_constants() {
    // File: SEEK_SET/CUR/END constants
    let source = r#"
fn main() -> i64 {
    seek_set := 0
    seek_cur := 1
    seek_end := 2
    result := mut 0
    I seek_set == 0 { result = result + 1 }
    I seek_cur == 1 { result = result + 1 }
    I seek_end == 2 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== 10. Memory ====================

#[test]
fn e2e_p136_stdlib_memory_copy_fill() {
    // Memory: copy and fill patterns
    let source = r#"
fn mem_fill_i64(dest: i64, value: i64, count: i64) -> i64 {
    i := mut 0
    L {
        I i >= count { B }
        store_i64(dest + i * 8, value)
        i = i + 1
    }
    dest
}

fn main() -> i64 {
    src := malloc(32)
    mem_fill_i64(src, 77, 4)

    dst := malloc(32)
    memcpy(dst, src, 32)

    result := mut 0
    I load_i64(dst) == 77 { result = result + 1 }
    I load_i64(dst + 8) == 77 { result = result + 1 }
    I load_i64(dst + 16) == 77 { result = result + 1 }
    I load_i64(dst + 24) == 77 { result = result + 1 }

    free(src)
    free(dst)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_stdlib_memory_zero() {
    // Memory: zero fill
    let source = r#"
fn mem_zero(dest: i64, n: i64) -> i64 {
    i := mut 0
    L {
        I i >= n { B }
        store_byte(dest + i, 0)
        i = i + 1
    }
    dest
}

fn main() -> i64 {
    buf := malloc(16)
    store_i64(buf, 12345)
    store_i64(buf + 8, 67890)
    mem_zero(buf, 16)

    result := mut 0
    I load_i64(buf) == 0 { result = result + 1 }
    I load_i64(buf + 8) == 0 { result = result + 1 }

    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}

// ==================== 11. Hash ====================

#[test]
fn e2e_p136_stdlib_hash_mult() {
    // Hash: multiplicative hash function
    let source = r#"
fn mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } else { h }
}

fn main() -> i64 {
    result := mut 0
    h1 := mult_hash(42)
    h2 := mult_hash(42)
    h3 := mult_hash(0)
    I h1 == h2 { result = result + 1 }     # deterministic
    I h1 > 0 { result = result + 1 }       # positive
    I h3 == 0 { result = result + 1 }       # hash(0) = 0
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p136_stdlib_hash_string_djb2() {
    // Hash: DJB2 string hash
    let source = r#"
fn hash_string(str_ptr: i64, idx: i64, hash: i64) -> i64 {
    byte := load_byte(str_ptr + idx)
    I byte == 0 {
        I hash < 0 { 0 - hash } else { hash }
    } else {
        hash_string(str_ptr, idx + 1, hash * 33 + byte)
    }
}

fn main() -> i64 {
    s := str_to_ptr("hello")
    h1 := hash_string(s, 0, 5381)
    h2 := hash_string(s, 0, 5381)

    result := mut 0
    I h1 == h2 { result = result + 1 }     # deterministic
    I h1 > 0 { result = result + 1 }       # positive
    result
}
"#;
    assert_exit_code(source, 2);
}

// ==================== 12. Set ====================

#[test]
fn e2e_p136_stdlib_set_add_contains() {
    // Set: add and contains using hash buckets
    let source = r#"
fn mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } else { h }
}

fn set_contains(buckets: i64, cap: i64, value: i64) -> i64 {
    h := mult_hash(value) % cap
    ptr := mut load_i64(buckets + h * 8)
    L {
        I ptr == 0 { return 0 }
        I load_i64(ptr) == value { return 1 }
        ptr = load_i64(ptr + 8)
    }
    0
}

fn set_add(buckets: i64, cap: i64, value: i64) -> i64 {
    I set_contains(buckets, cap, value) == 1 { return 0 }
    h := mult_hash(value) % cap
    entry := malloc(16)
    store_i64(entry, value)
    store_i64(entry + 8, load_i64(buckets + h * 8))
    store_i64(buckets + h * 8, entry)
    1
}

fn main() -> i64 {
    cap := 8
    buckets := malloc(cap * 8)
    i := mut 0
    L {
        I i >= cap { B }
        store_i64(buckets + i * 8, 0)
        i = i + 1
    }

    set_add(buckets, cap, 10)
    set_add(buckets, cap, 20)
    set_add(buckets, cap, 30)

    result := mut 0
    I set_contains(buckets, cap, 10) == 1 { result = result + 1 }
    I set_contains(buckets, cap, 20) == 1 { result = result + 1 }
    I set_contains(buckets, cap, 30) == 1 { result = result + 1 }
    I set_contains(buckets, cap, 99) == 0 { result = result + 1 }

    # Duplicate add should return 0
    I set_add(buckets, cap, 10) == 0 { result = result + 1 }

    free(buckets)
    result
}
"#;
    assert_exit_code(source, 5);
}

// ==================== 13. Deque ====================

#[test]
fn e2e_p136_stdlib_deque_push_pop() {
    // Deque: circular buffer push_back/pop_front
    let source = r#"
fn main() -> i64 {
    cap := 8
    data := malloc(cap * 8)
    head := mut 0
    tail := mut 0
    len := mut 0

    # push_back 10, 20, 30
    store_i64(data + tail * 8, 10)
    tail = (tail + 1) % cap
    len = len + 1

    store_i64(data + tail * 8, 20)
    tail = (tail + 1) % cap
    len = len + 1

    store_i64(data + tail * 8, 30)
    tail = (tail + 1) % cap
    len = len + 1

    result := mut 0
    I len == 3 { result = result + 1 }

    # pop_front -> 10
    val := load_i64(data + head * 8)
    head = (head + 1) % cap
    len = len - 1
    I val == 10 { result = result + 1 }

    # pop_front -> 20
    val2 := load_i64(data + head * 8)
    head = (head + 1) % cap
    len = len - 1
    I val2 == 20 { result = result + 1 }

    # pop_front -> 30
    val3 := load_i64(data + head * 8)
    head = (head + 1) % cap
    len = len - 1
    I val3 == 30 { result = result + 1 }

    I len == 0 { result = result + 1 }

    free(data)
    result
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p136_stdlib_deque_push_front() {
    // Deque: push_front (prepend)
    let source = r#"
fn main() -> i64 {
    cap := 8
    data := malloc(cap * 8)
    head := mut 0
    tail := mut 0
    len := mut 0

    # push_front 10 -> head wraps around
    head = (head - 1 + cap) % cap
    store_i64(data + head * 8, 10)
    len = len + 1

    # push_front 20
    head = (head - 1 + cap) % cap
    store_i64(data + head * 8, 20)
    len = len + 1

    result := mut 0
    I len == 2 { result = result + 1 }

    # pop_front -> 20 (LIFO from front)
    val := load_i64(data + head * 8)
    head = (head + 1) % cap
    len = len - 1
    I val == 20 { result = result + 1 }

    # pop_front -> 10
    val2 := load_i64(data + head * 8)
    head = (head + 1) % cap
    len = len - 1
    I val2 == 10 { result = result + 1 }

    free(data)
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== 14. Arena ====================

#[test]
fn e2e_p136_stdlib_arena_alloc() {
    // Arena: bump allocator from a single chunk
    let source = r#"
fn main() -> i64 {
    chunk_size := 1024
    chunk := malloc(chunk_size)
    offset := mut 0

    # Alloc 8 bytes (aligned)
    ptr1 := chunk + offset
    offset = offset + 8
    store_i64(ptr1, 111)

    # Alloc 16 bytes
    ptr2 := chunk + offset
    offset = offset + 16
    store_i64(ptr2, 222)
    store_i64(ptr2 + 8, 333)

    result := mut 0
    I load_i64(ptr1) == 111 { result = result + 1 }
    I load_i64(ptr2) == 222 { result = result + 1 }
    I load_i64(ptr2 + 8) == 333 { result = result + 1 }
    I offset == 24 { result = result + 1 }

    free(chunk)
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_stdlib_arena_reset() {
    // Arena: reset offset to reuse memory
    let source = r#"
fn main() -> i64 {
    chunk := malloc(256)
    offset := mut 0

    # Alloc and use
    ptr := chunk + offset
    offset = offset + 8
    store_i64(ptr, 42)

    # "Reset" arena
    offset = 0

    # Realloc from same chunk
    ptr2 := chunk + offset
    offset = offset + 8
    store_i64(ptr2, 99)

    result := mut 0
    # ptr and ptr2 point to same memory
    I load_i64(ptr) == 99 { result = result + 1 }
    I ptr == ptr2 { result = result + 1 }

    free(chunk)
    result
}
"#;
    assert_exit_code(source, 2);
}

// ==================== 15. Base64 ====================

#[test]
fn e2e_p136_stdlib_base64_encode_byte() {
    // Base64: single byte encoding logic
    let source = r#"
fn b64_char(idx: i64) -> i64 {
    table := str_to_ptr("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    load_byte(table + idx)
}

fn main() -> i64 {
    result := mut 0
    # A=0, B=1, ..., Z=25
    I b64_char(0) == 65 { result = result + 1 }     # 'A'
    I b64_char(25) == 90 { result = result + 1 }    # 'Z'
    I b64_char(26) == 97 { result = result + 1 }    # 'a'
    I b64_char(51) == 122 { result = result + 1 }   # 'z'
    I b64_char(52) == 48 { result = result + 1 }    # '0'
    I b64_char(62) == 43 { result = result + 1 }    # '+'
    I b64_char(63) == 47 { result = result + 1 }    # '/'
    result
}
"#;
    assert_exit_code(source, 7);
}

#[test]
fn e2e_p136_stdlib_base64_encode_triplet() {
    // Base64: encode 3 bytes "Man" -> "TWFu"
    let source = r#"
fn b64_char(idx: i64) -> i64 {
    table := str_to_ptr("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    load_byte(table + idx)
}

fn main() -> i64 {
    # "Man" = [77, 97, 110]
    b0 := 77
    b1 := 97
    b2 := 110

    c0 := (b0 >> 2) & 63
    c1 := ((b0 & 3) << 4) | ((b1 >> 4) & 15)
    c2 := ((b1 & 15) << 2) | ((b2 >> 6) & 3)
    c3 := b2 & 63

    result := mut 0
    I b64_char(c0) == 84 { result = result + 1 }   # 'T'
    I b64_char(c1) == 87 { result = result + 1 }   # 'W'
    I b64_char(c2) == 70 { result = result + 1 }   # 'F'
    I b64_char(c3) == 117 { result = result + 1 }  # 'u'
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_str_to_lowercase_helper_links_and_runs() {
    let source = r#"
fn main() -> i64 {
    lower := "AbC-12".to_lowercase()
    result := mut 0
    I lower.byte_at(0) == 97 { result = result + 1 }
    I lower.byte_at(1) == 98 { result = result + 1 }
    I lower.byte_at(2) == 99 { result = result + 1 }
    I lower.byte_at(3) == 45 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p136_complex_struct_method_specializations_define_receiver_layout() {
    let source = r#"
struct BigNode {
    a: i64,
    b: i64,
    c: i64,
    d: i64,
    e: i64,
    f: i64,
    g: i64,
}

struct Carrier<T> {
    len: i64,
}

impl Carrier<T> {
    fn new(value: T) -> Carrier<T> {
        Carrier { len: 1 }
    }

    fn echo(&self, value: T) -> type {
        value
    }
}

fn main() -> i64 {
    carrier: Carrier<BigNode> := Carrier.new(BigNode { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7 })
    found := carrier.echo(BigNode { a: 11, b: 12, c: 13, d: 14, e: 15, f: 16, g: 17 })
    I found.g != 17 {
        return 1
    }
    I found.a != 11 {
        return 2
    }
    I found.g != 17 {
        return 3
    }
    0
}
"#;
    assert_exit_code(source, 0);
}
