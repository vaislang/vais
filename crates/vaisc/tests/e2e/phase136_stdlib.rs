//! Phase 136 — Standard Library Compilation Verification E2E Tests
//!
//! Tests that core stdlib module patterns compile and execute correctly.
//! Each test inlines the relevant stdlib logic to verify codegen correctness.
//! Covers: vec, string, hashmap, option, result, math, json, io, file,
//!         memory, hash, set, deque, arena, base64

use super::helpers::*;

// ==================== 1. Vec ====================

#[test]
fn e2e_p136_stdlib_vec_push_pop() {
    // Vec<i64>: push 3 elements, pop them, verify LIFO order
    let source = r#"
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F to_upper_byte(c: i64) -> i64 {
    I c >= 97 && c <= 122 { c - 32 } E { c }
}

F main() -> i64 {
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
F mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } E { h }
}

F main() -> i64 {
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
F mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } E { h }
}

F contains(buckets: i64, cap: i64, key: i64) -> i64 {
    h := mult_hash(key) % cap
    ptr := mut load_i64(buckets + h * 8)
    L {
        I ptr == 0 { R 0 }
        I load_i64(ptr) == key { R 1 }
        ptr = load_i64(ptr + 16)
    }
    0
}

F main() -> i64 {
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

// ==================== 4. Option ====================

#[test]
fn e2e_p136_stdlib_option_some_none() {
    // Option: tag-based Some/None
    let source = r#"
E Option {
    None,
    Some(i64)
}

F main() -> i64 {
    a := Some(42)
    b := None

    result := mut 0
    M a {
        Some(v) => { I v == 42 { result = result + 1 } },
        None => { }
    }
    M b {
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
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(v) => v,
        None => default
    }
}

F main() -> i64 {
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

// ==================== 5. Result ====================

#[test]
fn e2e_p136_stdlib_result_ok_err() {
    // Result: Ok/Err pattern matching
    let source = r#"
E Result {
    Ok(i64),
    Err(i64)
}

F divide(a: i64, b: i64) -> Result {
    I b == 0 { Err(1) }
    E { Ok(a / b) }
}

F main() -> i64 {
    r1 := divide(10, 2)
    r2 := divide(10, 0)

    result := mut 0
    M r1 {
        Ok(v) => { I v == 5 { result = result + 1 } },
        Err(_) => { }
    }
    M r2 {
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
E Result {
    Ok(i64),
    Err(i64)
}

F is_ok(r: Result) -> i64 {
    M r { Ok(_) => 1, Err(_) => 0 }
}
F is_err(r: Result) -> i64 {
    M r { Ok(_) => 0, Err(_) => 1 }
}

F main() -> i64 {
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
F abs_i64(x: i64) -> i64 {
    I x < 0 { 0 - x } E { x }
}
F min_i64(a: i64, b: i64) -> i64 {
    I a < b { a } E { b }
}
F max_i64(a: i64, b: i64) -> i64 {
    I a > b { a } E { b }
}
F clamp_i64(x: i64, lo: i64, hi: i64) -> i64 {
    I x < lo { lo } E I x > hi { hi } E { x }
}

F main() -> i64 {
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
F ipow(base: i64, exp: i64) -> i64 {
    I exp == 0 { 1 }
    E { base * ipow(base, exp - 1) }
}

F main() -> i64 {
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
F json_new(tag: i64, data: i64) -> i64 {
    v := malloc(24)
    store_i64(v, tag)
    store_i64(v + 8, data)
    store_i64(v + 16, 0)
    v
}

F json_tag(v: i64) -> i64 { load_i64(v) }
F json_data(v: i64) -> i64 { load_i64(v + 8) }

F main() -> i64 {
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
F hash_str(s: i64, len: i64) -> i64 {
    hash_str_rec(s, len, 0, 5381)
}
F hash_str_rec(s: i64, len: i64, idx: i64, h: i64) -> i64 {
    I idx >= len { I h < 0 { 0 - h } E { h } }
    E {
        c := load_byte(s + idx)
        hash_str_rec(s, len, idx + 1, h * 33 + c)
    }
}

F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
S File {
    handle: i64,
    mode: i64
}

F file_is_open(f_handle: i64) -> i64 {
    I f_handle != 0 { 1 } E { 0 }
}

F main() -> i64 {
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
F main() -> i64 {
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
F mem_fill_i64(dest: i64, value: i64, count: i64) -> i64 {
    i := mut 0
    L {
        I i >= count { B }
        store_i64(dest + i * 8, value)
        i = i + 1
    }
    dest
}

F main() -> i64 {
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
F mem_zero(dest: i64, n: i64) -> i64 {
    i := mut 0
    L {
        I i >= n { B }
        store_byte(dest + i, 0)
        i = i + 1
    }
    dest
}

F main() -> i64 {
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
F mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } E { h }
}

F main() -> i64 {
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
F hash_string(str_ptr: i64, idx: i64, hash: i64) -> i64 {
    byte := load_byte(str_ptr + idx)
    I byte == 0 {
        I hash < 0 { 0 - hash } E { hash }
    } E {
        hash_string(str_ptr, idx + 1, hash * 33 + byte)
    }
}

F main() -> i64 {
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
F mult_hash(value: i64) -> i64 {
    h := value * 2654435769
    I h < 0 { 0 - h } E { h }
}

F set_contains(buckets: i64, cap: i64, value: i64) -> i64 {
    h := mult_hash(value) % cap
    ptr := mut load_i64(buckets + h * 8)
    L {
        I ptr == 0 { R 0 }
        I load_i64(ptr) == value { R 1 }
        ptr = load_i64(ptr + 8)
    }
    0
}

F set_add(buckets: i64, cap: i64, value: i64) -> i64 {
    I set_contains(buckets, cap, value) == 1 { R 0 }
    h := mult_hash(value) % cap
    entry := malloc(16)
    store_i64(entry, value)
    store_i64(entry + 8, load_i64(buckets + h * 8))
    store_i64(buckets + h * 8, entry)
    1
}

F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F main() -> i64 {
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
F b64_char(idx: i64) -> i64 {
    table := str_to_ptr("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    load_byte(table + idx)
}

F main() -> i64 {
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
F b64_char(idx: i64) -> i64 {
    table := str_to_ptr("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    load_byte(table + idx)
}

F main() -> i64 {
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
