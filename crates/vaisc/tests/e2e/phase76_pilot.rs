//! Phase 76 — Pilot Project E2E tests
//!
//! Tests for the JSON→TOML converter and REST API server pilot projects.
//! These validate that real-world 1,000+ LOC programs compile and run correctly.

use super::helpers::*;

// ==================== Pilot A: JSON→TOML Converter ====================

/// Core test: the full pilot_json2toml.vais program compiles and returns expected score
#[test]
fn e2e_p76_pilot_json2toml_full() {
    // Resolve path relative to workspace root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = std::fs::read_to_string(workspace_root.join("examples/pilot_json2toml.vais"))
        .expect("pilot_json2toml.vais should exist");
    assert_exit_code(&source, 34);
}

/// Unit test: JSON value construction and node counting
#[test]
fn e2e_p76_json_value_construction() {
    let source = r#"
F strlen_ptr(s: i64) -> i64 {
    I load_byte(s) == 0 { 0 }
    E { 1 + strlen_ptr(s + 1) }
}

F main() -> i64 {
    # Test val_new with tag/data/extra
    v := malloc(24)
    store_i64(v, 2)      # tag = int
    store_i64(v + 8, 42) # data = 42
    store_i64(v + 16, 0) # extra = 0
    tag := load_i64(v)
    data := load_i64(v + 8)
    result := 0
    I tag == 2 { result = result + 1 }
    I data == 42 { result = result + 1 }
    free(v)

    # Test string value
    s := str_to_ptr("hello")
    len := strlen_ptr(s)
    I len == 5 { result = result + 1 }

    result
}
"#;
    assert_exit_code(source, 3);
}

/// Unit test: Dynamic buffer (growable byte buffer)
#[test]
fn e2e_p76_dynamic_buffer() {
    let source = r#"
F copy_bytes(dst: i64, src: i64, len: i64, idx: i64) -> i64 {
    I idx >= len { 0 } E {
        store_byte(dst + idx, load_byte(src + idx))
        copy_bytes(dst, src, len, idx + 1)
    }
}

F buf_new() -> i64 {
    cap := 16
    b := malloc(24)
    store_i64(b, malloc(cap))
    store_i64(b + 8, 0)
    store_i64(b + 16, cap)
    b
}
F buf_data(b: i64) -> i64 { load_i64(b) }
F buf_len(b: i64) -> i64 { load_i64(b + 8) }
F buf_cap(b: i64) -> i64 { load_i64(b + 16) }
F buf_grow(b: i64) -> i64 {
    old_cap := buf_cap(b)
    new_cap := old_cap * 2
    new_data := malloc(new_cap)
    old_data := buf_data(b)
    old_len := buf_len(b)
    copy_bytes(new_data, old_data, old_len, 0)
    free(old_data)
    store_i64(b, new_data)
    store_i64(b + 16, new_cap)
    0
}
F buf_ensure(b: i64, needed: i64) -> i64 {
    I buf_len(b) + needed >= buf_cap(b) {
        buf_grow(b)
        buf_ensure(b, needed)
    } E { 0 }
}
F buf_push_byte(b: i64, c: i64) -> i64 {
    buf_ensure(b, 1)
    data := buf_data(b)
    len := buf_len(b)
    store_byte(data + len, c)
    store_i64(b + 8, len + 1)
    0
}

F main() -> i64 {
    b := buf_new()
    result := 0
    I buf_len(b) == 0 { result = result + 1 }
    I buf_cap(b) == 16 { result = result + 1 }

    # Push some bytes
    buf_push_byte(b, 65)  # 'A'
    buf_push_byte(b, 66)  # 'B'
    buf_push_byte(b, 67)  # 'C'
    I buf_len(b) == 3 { result = result + 1 }

    # Check content
    data := buf_data(b)
    I load_byte(data) == 65 { result = result + 1 }
    I load_byte(data + 1) == 66 { result = result + 1 }

    free(buf_data(b))
    free(b)
    result
}
"#;
    assert_exit_code(source, 5);
}

/// Unit test: Hash table (object) operations
#[test]
fn e2e_p76_hash_table_ops() {
    let source = r#"
F strlen_ptr(s: i64) -> i64 {
    I load_byte(s) == 0 { 0 }
    E { 1 + strlen_ptr(s + 1) }
}
F copy_bytes(dst: i64, src: i64, len: i64, idx: i64) -> i64 {
    I idx >= len { 0 } E {
        store_byte(dst + idx, load_byte(src + idx))
        copy_bytes(dst, src, len, idx + 1)
    }
}
F mem_zero(ptr: i64, count: i64, idx: i64) -> i64 {
    I idx >= count { 0 } E {
        store_i64(ptr + idx * 8, 0)
        mem_zero(ptr, count, idx + 1)
    }
}
F hash_str(s: i64, len: i64) -> i64 {
    hash_str_rec(s, len, 0, 5381)
}
F hash_str_rec(s: i64, len: i64, idx: i64, h: i64) -> i64 {
    I idx >= len { h } E {
        c := load_byte(s + idx)
        new_h := h * 33 + c
        abs_h := I new_h < 0 { 0 - new_h } E { new_h }
        hash_str_rec(s, len, idx + 1, abs_h)
    }
}
F str_eq(a: i64, b: i64, len: i64) -> i64 {
    str_eq_rec(a, b, len, 0)
}
F str_eq_rec(a: i64, b: i64, len: i64, idx: i64) -> i64 {
    I idx >= len { 1 }
    E I load_byte(a + idx) != load_byte(b + idx) { 0 }
    E { str_eq_rec(a, b, len, idx + 1) }
}
F obj_new() -> i64 {
    cap := 16
    buckets := malloc(cap * 8)
    mem_zero(buckets, cap, 0)
    obj := malloc(24)
    store_i64(obj, buckets)
    store_i64(obj + 8, 0)
    store_i64(obj + 16, cap)
    obj
}
F obj_set(obj: i64, key: i64, key_len: i64, value: i64) -> i64 {
    buckets := load_i64(obj)
    cap := load_i64(obj + 16)
    h := hash_str(key, key_len) % cap
    entry := malloc(32)
    key_copy := malloc(key_len + 1)
    copy_bytes(key_copy, key, key_len, 0)
    store_byte(key_copy + key_len, 0)
    store_i64(entry, key_copy)
    store_i64(entry + 8, key_len)
    store_i64(entry + 16, value)
    store_i64(entry + 24, load_i64(buckets + h * 8))
    store_i64(buckets + h * 8, entry)
    size := load_i64(obj + 8) + 1
    store_i64(obj + 8, size)
    0
}
F obj_get_rec(node: i64, key: i64, key_len: i64) -> i64 {
    I node == 0 { 0 }
    E {
        e_key := load_i64(node)
        e_key_len := load_i64(node + 8)
        I e_key_len == key_len && str_eq(e_key, key, key_len) == 1 {
            load_i64(node + 16)
        } E {
            obj_get_rec(load_i64(node + 24), key, key_len)
        }
    }
}
F obj_get(obj: i64, key: i64, key_len: i64) -> i64 {
    buckets := load_i64(obj)
    cap := load_i64(obj + 16)
    h := hash_str(key, key_len) % cap
    node := load_i64(buckets + h * 8)
    obj_get_rec(node, key, key_len)
}

F main() -> i64 {
    obj := obj_new()
    result := 0

    # Set key "age" = 42
    obj_set(obj, str_to_ptr("age"), 3, 42)
    # Set key "x" = 100
    obj_set(obj, str_to_ptr("x"), 1, 100)

    # Size should be 2
    size := load_i64(obj + 8)
    I size == 2 { result = result + 1 }

    # Get "age" should return 42
    val := obj_get(obj, str_to_ptr("age"), 3)
    I val == 42 { result = result + 1 }

    # Get "x" should return 100
    val2 := obj_get(obj, str_to_ptr("x"), 1)
    I val2 == 100 { result = result + 1 }

    # Get non-existent key should return 0
    val3 := obj_get(obj, str_to_ptr("none"), 4)
    I val3 == 0 { result = result + 1 }

    result
}
"#;
    assert_exit_code(source, 4);
}

/// Unit test: i64 to string conversion
#[test]
fn e2e_p76_i64_to_str() {
    let source = r#"
F strlen_ptr(s: i64) -> i64 {
    I load_byte(s) == 0 { 0 }
    E { 1 + strlen_ptr(s + 1) }
}
F count_digits(n: i64, acc: i64) -> i64 {
    I n == 0 { acc }
    E { count_digits(n / 10, acc + 1) }
}
F fill_digits(buf: i64, n: i64, pos: i64) -> i64 {
    I n == 0 { 0 }
    E {
        digit := n % 10
        store_byte(buf + pos, digit + 48)
        fill_digits(buf, n / 10, pos - 1)
    }
}
F i64_to_str(n: i64) -> i64 {
    I n == 0 {
        buf := malloc(2)
        store_byte(buf, 48)
        store_byte(buf + 1, 0)
        R buf
    }
    is_neg := I n < 0 { 1 } E { 0 }
    abs_n := I is_neg == 1 { 0 - n } E { n }
    digits := count_digits(abs_n, 0)
    total := digits + is_neg
    buf := malloc(total + 1)
    store_byte(buf + total, 0)
    fill_digits(buf, abs_n, total - 1)
    I is_neg == 1 { store_byte(buf, 45) }
    buf
}

F main() -> i64 {
    result := 0

    # Test 0
    s0 := i64_to_str(0)
    I load_byte(s0) == 48 && load_byte(s0 + 1) == 0 { result = result + 1 }
    free(s0)

    # Test 42
    s1 := i64_to_str(42)
    I load_byte(s1) == 52 && load_byte(s1 + 1) == 50 { result = result + 1 }  # '4' '2'
    free(s1)

    # Test -7
    s2 := i64_to_str(0 - 7)
    I load_byte(s2) == 45 && load_byte(s2 + 1) == 55 { result = result + 1 }  # '-' '7'
    free(s2)

    # Test 100
    s3 := i64_to_str(100)
    I strlen_ptr(s3) == 3 { result = result + 1 }
    free(s3)

    result
}
"#;
    assert_exit_code(source, 4);
}

/// Unit test: Dynamic array operations
#[test]
fn e2e_p76_dynamic_array() {
    let source = r#"
F copy_arr_data(dst: i64, src: i64, len: i64, idx: i64) -> i64 {
    I idx >= len { 0 } E {
        store_i64(dst + idx * 8, load_i64(src + idx * 8))
        copy_arr_data(dst, src, len, idx + 1)
    }
}
F arr_new() -> i64 {
    cap := 4
    a := malloc(24)
    store_i64(a, malloc(cap * 8))
    store_i64(a + 8, 0)
    store_i64(a + 16, cap)
    a
}
F arr_push(a: i64, value: i64) -> i64 {
    len := load_i64(a + 8)
    cap := load_i64(a + 16)
    I len >= cap {
        new_cap := cap * 2
        new_data := malloc(new_cap * 8)
        old_data := load_i64(a)
        copy_arr_data(new_data, old_data, len, 0)
        free(old_data)
        store_i64(a, new_data)
        store_i64(a + 16, new_cap)
    }
    data := load_i64(a)
    store_i64(data + len * 8, value)
    store_i64(a + 8, len + 1)
    0
}
F arr_len(a: i64) -> i64 { load_i64(a + 8) }
F arr_get(a: i64, idx: i64) -> i64 { load_i64(load_i64(a) + idx * 8) }

F main() -> i64 {
    a := arr_new()
    result := 0

    I arr_len(a) == 0 { result = result + 1 }

    arr_push(a, 10)
    arr_push(a, 20)
    arr_push(a, 30)

    I arr_len(a) == 3 { result = result + 1 }
    I arr_get(a, 0) == 10 { result = result + 1 }
    I arr_get(a, 2) == 30 { result = result + 1 }

    # Push beyond initial capacity (4)
    arr_push(a, 40)
    arr_push(a, 50)
    I arr_len(a) == 5 { result = result + 1 }
    I arr_get(a, 4) == 50 { result = result + 1 }

    free(load_i64(a))
    free(a)
    result
}
"#;
    assert_exit_code(source, 6);
}

/// Test TOML key quoting logic
#[test]
fn e2e_p76_toml_key_quoting() {
    let source = r#"
F needs_quoting(key: i64, len: i64) -> i64 {
    I len == 0 { 1 }
    E { needs_quoting_rec(key, len, 0) }
}
F needs_quoting_rec(key: i64, len: i64, idx: i64) -> i64 {
    I idx >= len { 0 }
    E {
        c := load_byte(key + idx)
        I (c >= 48 && c <= 57) || (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || c == 45 || c == 95 {
            needs_quoting_rec(key, len, idx + 1)
        } E { 1 }
    }
}

F main() -> i64 {
    result := 0

    # "name" should not need quoting (bare key)
    I needs_quoting(str_to_ptr("name"), 4) == 0 { result = result + 1 }

    # "my-key" should not need quoting (hyphen allowed)
    I needs_quoting(str_to_ptr("my-key"), 6) == 0 { result = result + 1 }

    # "my_key" should not need quoting (underscore allowed)
    I needs_quoting(str_to_ptr("my_key"), 6) == 0 { result = result + 1 }

    # "my key" should need quoting (space)
    I needs_quoting(str_to_ptr("my key"), 6) == 1 { result = result + 1 }

    # "" empty should need quoting
    I needs_quoting(str_to_ptr(""), 0) == 1 { result = result + 1 }

    result
}
"#;
    assert_exit_code(source, 5);
}

/// Regression test: parameter named "entry" should not collide with LLVM entry block label
#[test]
fn e2e_p76_entry_param_name_fix() {
    let source = r#"
F process_entry(entry: i64, count: i64) -> i64 {
    I count <= 0 { entry }
    E { process_entry(entry + 1, count - 1) }
}

F main() -> i64 {
    result := 0
    I process_entry(0, 5) == 5 { result = result + 1 }
    I process_entry(10, 3) == 13 { result = result + 1 }
    I process_entry(0, 0) == 0 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

// ==================== Pilot B: REST API Server ====================

/// Core test: the REST API server pilot compiles and returns expected score
#[test]
fn e2e_p76_pilot_rest_api_full() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = std::fs::read_to_string(workspace_root.join("examples/pilot_rest_api.vais"))
        .expect("pilot_rest_api.vais should exist");
    assert_exit_code(&source, 28);
}

/// Verify pilot_json2toml example compiles to text IR successfully
#[test]
fn e2e_p76_pilot_json2toml_compiles() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = std::fs::read_to_string(workspace_root.join("examples/pilot_json2toml.vais"))
        .expect("pilot_json2toml.vais should exist");
    let ir = compile_to_ir(&source).expect("should compile");
    assert!(!ir.is_empty(), "IR should not be empty");
}
