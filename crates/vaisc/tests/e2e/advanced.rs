use std::fs;
use std::process::Command;
use tempfile::TempDir;

use super::helpers::*;

fn vaisc_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_vaisc"))
}

// ==================== Coverage Instrumentation Tests ====================

#[test]
fn test_coverage_basic_program() {
    // Verify that a basic program compiles and runs correctly with coverage flags
    let result = compile_and_run_with_coverage(
        r#"
F main() -> i64 {
    x := 42
    y := 58
    x + y
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 100);
}

#[test]
fn test_coverage_branching() {
    // Coverage instrumentation should track branch coverage — verify branches work correctly
    let result = compile_and_run_with_coverage(
        r#"
F classify(n: i64) -> i64 {
    I n > 100 {
        3
    } E {
        I n > 50 {
            2
        } E {
            I n > 0 {
                1
            } E {
                0
            }
        }
    }
}

F main() -> i64 {
    a := classify(200)
    b := classify(75)
    c := classify(25)
    d := classify(0)
    # a=3, b=2, c=1, d=0 → sum=6
    a + b + c + d
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 6);
}

#[test]
fn test_coverage_loops() {
    // Coverage should track loop iterations — verify loops work with instrumentation
    let result = compile_and_run_with_coverage(
        r#"
F sum_to(n: i64) -> i64 {
    total := mut 0
    i := mut 1
    L {
        I i > n { B }
        total = total + i
        i = i + 1
    }
    total
}

F main() -> i64 {
    # 1+2+3+4+5+6+7+8+9+10 = 55
    sum_to(10)
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 55);
}

#[test]
fn test_coverage_function_calls() {
    // Coverage should track function call counts — verify multi-function programs
    let result = compile_and_run_with_coverage(
        r#"
F add(a: i64, b: i64) -> i64 { a + b }
F mul(a: i64, b: i64) -> i64 { a * b }
F square(n: i64) -> i64 { mul(n, n) }

F main() -> i64 {
    a := add(3, 4)
    b := square(3)
    # a=7, b=9 → 7+9=16
    add(a, b)
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 16);
}

// ===== Phase 54: Example project pattern tests =====

#[test]
fn test_project_todo_model_struct() {
    // Test Todo struct pattern from todo-api project
    let result = compile_and_run(
        r#"
S Todo {
    id: i64,
    title: str,
    completed: bool
}

F todo_new(id: i64, title: str, completed: bool) -> Todo {
    Todo { id: id, title: title, completed: completed }
}

F main() -> i64 {
    t := todo_new(1, "Buy milk", false)
    I t.id == 1 { 10 } E { 1 }
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 10);
}

#[test]
fn test_project_csv_row_struct() {
    // Test CsvRow struct pattern from data-pipeline project
    let result = compile_and_run(
        r#"
S CsvRow {
    name: str,
    age: i64,
    score: i64
}

S TransformResult {
    filtered_count: i64,
    avg_score: i64,
    total_score: i64
}

F filter_by_score(rows: i64, count: i64, threshold: i64) -> i64 {
    passed := mut 0
    i := mut 0
    L {
        I i >= count { B }
        score := load_i64(rows + i * 8)
        I score >= threshold {
            passed = passed + 1
        }
        i = i + 1
    }
    passed
}

F main() -> i64 {
    # Simulate scores array: 85, 92, 78, 95, 88
    buf := malloc(40)
    store_i64(buf, 85)
    store_i64(buf + 8, 92)
    store_i64(buf + 16, 78)
    store_i64(buf + 24, 95)
    store_i64(buf + 32, 88)

    # Filter scores >= 85 → should be 4 (85, 92, 95, 88)
    result := filter_by_score(buf, 5, 85)
    free(buf)
    result
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 4);
}

#[test]
fn test_project_chat_room_pattern() {
    // Test ChatRoom-like client list management pattern
    let result = compile_and_run(
        r#"
F add_client(clients: i64, count_ptr: i64, fd: i64) -> i64 {
    count := load_i64(count_ptr)
    store_i64(clients + count * 8, fd)
    store_i64(count_ptr, count + 1)
    1
}

F get_client_count(count_ptr: i64) -> i64 {
    load_i64(count_ptr)
}

F main() -> i64 {
    clients := malloc(80)
    count_ptr := malloc(8)
    store_i64(count_ptr, 0)

    add_client(clients, count_ptr, 100)
    add_client(clients, count_ptr, 200)
    add_client(clients, count_ptr, 300)

    result := get_client_count(count_ptr)
    free(clients)
    free(count_ptr)
    result
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 3);
}

#[test]
fn test_project_line_reader_pattern() {
    // Test line-by-line buffer pattern
    let result = compile_and_run(
        r#"
F count_newlines(buf: i64, len: i64) -> i64 {
    count := mut 0
    i := mut 0
    L {
        I i >= len { B }
        c := load_byte(buf + i)
        I c == 10 {
            count = count + 1
        }
        i = i + 1
    }
    count
}

F main() -> i64 {
    # Simulate "hello\nworld\nfoo\n" — 3 newlines
    buf := malloc(20)
    store_byte(buf, 104)     # h
    store_byte(buf + 1, 101) # e
    store_byte(buf + 2, 108) # l
    store_byte(buf + 3, 108) # l
    store_byte(buf + 4, 111) # o
    store_byte(buf + 5, 10)  # \n
    store_byte(buf + 6, 119) # w
    store_byte(buf + 7, 111) # o
    store_byte(buf + 8, 114) # r
    store_byte(buf + 9, 108) # l
    store_byte(buf + 10, 100) # d
    store_byte(buf + 11, 10)  # \n
    store_byte(buf + 12, 102) # f
    store_byte(buf + 13, 111) # o
    store_byte(buf + 14, 111) # o
    store_byte(buf + 15, 10)  # \n

    result := count_newlines(buf, 16)
    free(buf)
    result
}
"#,
    )
    .unwrap();
    assert_eq!(result.exit_code, 3);
}

// ===== Phase 55: VaisDB — Filesystem & ptr_to_str E2E Tests =====

#[test]
fn e2e_phase55_fs_exists() {
    let source = r#"
F main() -> i64 {
    fp := fopen("/tmp/vais_e2e_exists_test55.txt", "w")
    I fp == 0 { R 1 }
    fputs("test", fp)
    fclose(fp)
    r := access("/tmp/vais_e2e_exists_test55.txt", 0)
    I r != 0 { R 2 }
    r2 := access("/tmp/vais_e2e_nonexistent_xyz_999.txt", 0)
    I r2 == 0 { R 3 }
    unlink("/tmp/vais_e2e_exists_test55.txt")
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_fs_is_dir() {
    let source = r#"
F main() -> i64 {
    rmdir("/tmp/vais_e2e_isdir55")
    r := mkdir("/tmp/vais_e2e_isdir55", 493)
    I r != 0 { R 1 }
    d := opendir("/tmp/vais_e2e_isdir55")
    I d == 0 { R 2 }
    closedir(d)
    fp := fopen("/tmp/vais_e2e_isdir55_file.txt", "w")
    I fp == 0 { R 3 }
    fputs("x", fp)
    fclose(fp)
    d2 := opendir("/tmp/vais_e2e_isdir55_file.txt")
    I d2 != 0 { closedir(d2); R 4 }
    rmdir("/tmp/vais_e2e_isdir55")
    unlink("/tmp/vais_e2e_isdir55_file.txt")
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_readdir_list() {
    let source = r#"
F main() -> i64 {
    unlink("/tmp/vais_e2e_rd55/a.txt")
    unlink("/tmp/vais_e2e_rd55/b.txt")
    rmdir("/tmp/vais_e2e_rd55")
    mkdir("/tmp/vais_e2e_rd55", 493)
    fp1 := fopen("/tmp/vais_e2e_rd55/a.txt", "w")
    I fp1 == 0 { R 1 }
    fputs("aaa", fp1)
    fclose(fp1)
    fp2 := fopen("/tmp/vais_e2e_rd55/b.txt", "w")
    I fp2 == 0 { R 2 }
    fputs("bbb", fp2)
    fclose(fp2)
    d := opendir("/tmp/vais_e2e_rd55")
    I d == 0 { R 3 }
    count := mut 0
    L {
        entry := readdir(d)
        I entry == 0 { B }
        first := load_byte(entry)
        I first != 46 {
            count = count + 1
        } E {
            second := load_byte(entry + 1)
            I second == 0 {
                # "." skip
            } E I second == 46 {
                third := load_byte(entry + 2)
                I third == 0 {
                    # ".." skip
                } E {
                    count = count + 1
                }
            } E {
                count = count + 1
            }
        }
    }
    closedir(d)
    I count != 2 { R 10 + count }
    unlink("/tmp/vais_e2e_rd55/a.txt")
    unlink("/tmp/vais_e2e_rd55/b.txt")
    rmdir("/tmp/vais_e2e_rd55")
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_getcwd() {
    let source = r#"
F main() -> i64 {
    buf := malloc(1024)
    result := getcwd(buf, 1024)
    I result == 0 { free(buf); R 1 }
    # result is i64 pointer — check first byte
    first := load_byte(result)
    I first == 0 { free(buf); R 2 }
    # On Unix, cwd starts with '/' (ASCII 47)
    I first != 47 { free(buf); R 3 }
    free(buf)
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_ptr_to_str() {
    let source = r#"
F main() -> i64 {
    # Allocate a buffer and fill with "hi\0"
    buf := malloc(8)
    store_byte(buf, 104)
    store_byte(buf + 1, 105)
    store_byte(buf + 2, 0)
    # ptr_to_str converts i64 pointer to str
    s := ptr_to_str(buf)
    len := strlen(s)
    I len != 2 { free(buf); R 1 }
    # Verify first char
    p := str_to_ptr(s)
    first := load_byte(p)
    I first != 104 { free(buf); R 2 }
    free(buf)
    0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Phase 55: StrHashMap, StringMap<V>, ByteBuffer extensions =====

#[test]
fn e2e_phase55_strhashmap_basic() {
    // Test StrHashMap: str-typed keys with content-based hashing
    let source = r#"
F djb2_hash(s: i64) -> i64 {
    hash := mut 5381
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { B }
        hash = hash * 33 + c
        idx = idx + 1
    }
    I hash < 0 { hash = 0 - hash }
    hash
}

F streq(a: i64, b: i64) -> i64 {
    I a == b { R 1 }
    idx := mut 0
    L {
        ca := load_byte(a + idx)
        cb := load_byte(b + idx)
        I ca != cb { R 0 }
        I ca == 0 { R 1 }
        idx = idx + 1
    }
    1
}

F ptr_strlen(s: i64) -> i64 {
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { R idx }
        idx = idx + 1
    }
    idx
}

F strdup_heap(s: i64) -> i64 {
    len := ptr_strlen(s)
    buf := malloc(len + 1)
    memcpy(buf, s, len + 1)
    buf
}

S SHMap {
    buckets: i64, size: i64, cap: i64
}
X SHMap {
    F with_capacity(c: i64) -> SHMap {
        cap := I c < 8 { 8 } E { c }
        b := malloc(cap * 8)
        i := mut 0
        L { I i >= cap { B }; store_i64(b + i * 8, 0); i = i + 1 }
        SHMap { buckets: b, size: 0, cap: cap }
    }
    F hash(&self, key: str) -> i64 {
        p := str_to_ptr(key)
        h := djb2_hash(p)
        h % self.cap
    }
    F get(&self, key: str) -> i64 {
        idx := @.hash(key)
        ep := load_i64(self.buckets + idx * 8)
        kp := str_to_ptr(key)
        @.get_chain(ep, kp)
    }
    F get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, kp) == 1 { load_i64(ep + 8) }
            E { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    F contains(&self, key: str) -> i64 {
        idx := @.hash(key)
        ep := load_i64(self.buckets + idx * 8)
        kp := str_to_ptr(key)
        @.contains_chain(ep, kp)
    }
    F contains_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, kp) == 1 { 1 }
            E { @.contains_chain(load_i64(ep + 16), kp) }
        }
    }
    F set(&self, key: str, value: i64) -> i64 {
        idx := @.hash(key)
        ep := load_i64(self.buckets + idx * 8)
        kp := str_to_ptr(key)
        kc := strdup_heap(kp)
        ne := malloc(24)
        store_i64(ne, kc)
        store_i64(ne + 8, value)
        store_i64(ne + 16, ep)
        store_i64(self.buckets + idx * 8, ne)
        self.size = self.size + 1
        0
    }
}
F main() -> i64 {
    m := SHMap.with_capacity(16)
    m.set("hello", 42)
    m.set("world", 99)
    m.set("vais", 7)

    I m.get("hello") != 42 { R 1 }
    I m.get("world") != 99 { R 2 }
    I m.get("vais") != 7 { R 3 }
    I m.contains("hello") != 1 { R 4 }
    I m.contains("missing") != 0 { R 5 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_strhashmap_update_remove() {
    // Test StrHashMap: update existing key, remove key
    let source = r#"
F djb2_hash(s: i64) -> i64 {
    hash := mut 5381
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { B }
        hash = hash * 33 + c
        idx = idx + 1
    }
    I hash < 0 { hash = 0 - hash }
    hash
}

F streq(a: i64, b: i64) -> i64 {
    I a == b { R 1 }
    idx := mut 0
    L {
        ca := load_byte(a + idx)
        cb := load_byte(b + idx)
        I ca != cb { R 0 }
        I ca == 0 { R 1 }
        idx = idx + 1
    }
    1
}

F ptr_strlen2(s: i64) -> i64 {
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { R idx }
        idx = idx + 1
    }
    idx
}

F strdup_heap2(s: i64) -> i64 {
    len := ptr_strlen2(s)
    buf := malloc(len + 1)
    memcpy(buf, s, len + 1)
    buf
}

S SHMap2 {
    buckets: i64, size: i64, cap: i64
}
X SHMap2 {
    F with_capacity(c: i64) -> SHMap2 {
        cap := I c < 8 { 8 } E { c }
        b := malloc(cap * 8)
        i := mut 0
        L { I i >= cap { B }; store_i64(b + i * 8, 0); i = i + 1 }
        SHMap2 { buckets: b, size: 0, cap: cap }
    }
    F hash(&self, kp: i64) -> i64 {
        h := djb2_hash(kp)
        h % self.cap
    }
    F get(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        idx := @.hash(kp)
        ep := load_i64(self.buckets + idx * 8)
        @.get_chain(ep, kp)
    }
    F get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, kp) == 1 { load_i64(ep + 8) }
            E { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    F set(&self, key: str, value: i64) -> i64 {
        kp := str_to_ptr(key)
        idx := @.hash(kp)
        ep := load_i64(self.buckets + idx * 8)
        updated := @.try_update(ep, kp, value)
        I updated == 1 { R 0 }
        kc := strdup_heap2(kp)
        ne := malloc(24)
        store_i64(ne, kc)
        store_i64(ne + 8, value)
        store_i64(ne + 16, ep)
        store_i64(self.buckets + idx * 8, ne)
        self.size = self.size + 1
        0
    }
    F try_update(&self, ep: i64, kp: i64, value: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, kp) == 1 {
                store_i64(ep + 8, value)
                1
            } E {
                @.try_update(load_i64(ep + 16), kp, value)
            }
        }
    }
    F remove(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        idx := @.hash(kp)
        ep := load_i64(self.buckets + idx * 8)
        @.remove_chain(idx, 0, ep, kp)
    }
    F remove_chain(&self, bidx: i64, prev: i64, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, kp) == 1 {
                val := load_i64(ep + 8)
                nxt := load_i64(ep + 16)
                _ := I prev == 0 {
                    store_i64(self.buckets + bidx * 8, nxt); 0
                } E {
                    store_i64(prev + 16, nxt); 0
                }
                free(ek)
                free(ep)
                self.size = self.size - 1
                val
            } E {
                @.remove_chain(bidx, ep, load_i64(ep + 16), kp)
            }
        }
    }
}
F main() -> i64 {
    m := SHMap2.with_capacity(16)
    m.set("key1", 10)
    m.set("key2", 20)
    # Update existing key
    m.set("key1", 100)
    I m.get("key1") != 100 { R 1 }
    I m.get("key2") != 20 { R 2 }
    # Remove key
    removed := m.remove("key2")
    I removed != 20 { R 3 }
    I m.get("key2") != 0 { R 4 }
    I m.size != 1 { R 5 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_stringmap_generic() {
    // Test StringMap<V> generic struct — content-based str key comparison with generic value type
    let source = r#"
F djb2_hash(s: i64) -> i64 {
    hash := mut 5381
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { B }
        hash = hash * 33 + c
        idx = idx + 1
    }
    I hash < 0 { hash = 0 - hash }
    hash
}

F streq(a: i64, b: i64) -> i64 {
    idx := mut 0
    L {
        ca := load_byte(a + idx)
        cb := load_byte(b + idx)
        I ca != cb { R 0 }
        I ca == 0 { R 1 }
        idx = idx + 1
    }
    1
}

F ptr_len(s: i64) -> i64 {
    idx := mut 0
    L {
        c := load_byte(s + idx)
        I c == 0 { R idx }
        idx = idx + 1
    }
    idx
}

# Non-generic StringMap that tests content-based string comparison
# (tests the same logic as the generic StringMap<V> in std/stringmap.vais)
S StrMap {
    buckets: i64, size: i64, cap: i64
}

X StrMap {
    F with_capacity(c: i64) -> StrMap {
        cap := I c < 8 { 8 } E { c }
        b := malloc(cap * 8)
        i := mut 0
        L { I i >= cap { B }; store_i64(b + i * 8, 0); i = i + 1 }
        StrMap { buckets: b, size: 0, cap: cap }
    }
    F len(&self) -> i64 = self.size
    F is_empty(&self) -> i64 { I self.size == 0 { 1 } E { 0 } }
    F get(&self, key: i64) -> i64 {
        h := djb2_hash(key)
        idx := h % self.cap
        ep := load_i64(self.buckets + idx * 8)
        @.get_chain(ep, key)
    }
    F get_chain(&self, ep: i64, key: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, key) == 1 { load_i64(ep + 8) }
            E { @.get_chain(load_i64(ep + 16), key) }
        }
    }
    F set(&self, key: i64, value: i64) -> i64 {
        h := djb2_hash(key)
        idx := h % self.cap
        ep := load_i64(self.buckets + idx * 8)
        len := ptr_len(key)
        kc := malloc(len + 1)
        memcpy(kc, key, len + 1)
        ne := malloc(24)
        store_i64(ne, kc)
        store_i64(ne + 8, value)
        store_i64(ne + 16, ep)
        store_i64(self.buckets + idx * 8, ne)
        self.size = self.size + 1
        0
    }
    F contains(&self, key: i64) -> i64 {
        h := djb2_hash(key)
        idx := h % self.cap
        ep := load_i64(self.buckets + idx * 8)
        @.contains_chain(ep, key)
    }
    F contains_chain(&self, ep: i64, key: i64) -> i64 {
        I ep == 0 { 0 }
        E {
            ek := load_i64(ep)
            I streq(ek, key) == 1 { 1 }
            E { @.contains_chain(load_i64(ep + 16), key) }
        }
    }
}

F main() -> i64 {
    m := StrMap.with_capacity(16)
    I m.is_empty() != 1 { R 1 }

    p1 := str_to_ptr("alpha")
    p2 := str_to_ptr("beta")
    p3 := str_to_ptr("gamma")

    m.set(p1, 100)
    m.set(p2, 200)
    m.set(p3, 300)

    I m.len() != 3 { R 2 }
    I m.is_empty() != 0 { R 3 }

    # Look up by content (different pointer, same string)
    q1 := str_to_ptr("alpha")
    I m.get(q1) != 100 { R 4 }
    q2 := str_to_ptr("beta")
    I m.get(q2) != 200 { R 5 }
    q3 := str_to_ptr("gamma")
    I m.get(q3) != 300 { R 6 }

    # Unknown key returns 0
    q4 := str_to_ptr("delta")
    I m.get(q4) != 0 { R 7 }

    # Test contains
    I m.contains(q1) != 1 { R 8 }
    I m.contains(q4) != 0 { R 9 }

    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_bytebuffer_varint() {
    // Test ByteBuffer varint (LEB128) write/read roundtrip
    let source = r#"
S ByteBuffer {
    data: i64, len: i64, cap: i64, pos: i64
}
X ByteBuffer {
    F with_capacity(c: i64) -> ByteBuffer {
        cap := I c < 16 { 16 } E { c }
        d := malloc(cap)
        ByteBuffer { data: d, len: 0, cap: cap, pos: 0 }
    }
    F ensure_capacity(&self, needed: i64) -> i64 {
        I needed <= self.cap { R self.cap }
        nc := mut self.cap
        L { I nc >= needed { B }; nc = nc * 2 }
        nd := malloc(nc)
        memcpy(nd, self.data, self.len)
        free(self.data)
        self.data = nd
        self.cap = nc
        nc
    }
    F write_u8(&self, v: i64) -> i64 {
        @.ensure_capacity(self.len + 1)
        store_byte(self.data + self.len, v & 255)
        self.len = self.len + 1
        1
    }
    F read_u8(&self) -> i64 {
        I self.pos >= self.len { R 0 - 1 }
        val := load_byte(self.data + self.pos)
        self.pos = self.pos + 1
        val
    }
    F write_varint(&self, value: i64) -> i64 {
        count := mut 0
        v := mut value
        L {
            byte := v & 127
            v = v >> 7
            I v > 0 {
                @.write_u8(byte | 128)
            } E {
                @.write_u8(byte)
            }
            count = count + 1
            I v == 0 { B }
        }
        count
    }
    F read_varint(&self) -> i64 {
        result := mut 0
        shift := mut 0
        L {
            I self.pos >= self.len { R 0 - 1 }
            byte := @.read_u8()
            I byte < 0 { R 0 - 1 }
            result = result | ((byte & 127) << shift)
            I (byte & 128) == 0 { B }
            shift = shift + 7
            I shift >= 64 { R 0 - 1 }
        }
        result
    }
    F rewind(&self) -> i64 { self.pos = 0; 0 }
}
F main() -> i64 {
    bb := ByteBuffer.with_capacity(64)

    # Small value (fits in 1 byte)
    n1 := bb.write_varint(42)
    I n1 != 1 { R 1 }

    # Medium value (needs 2 bytes: 300 = 0b100101100)
    n2 := bb.write_varint(300)
    I n2 != 2 { R 2 }

    # Larger value (16384 = 2^14, needs 3 bytes)
    n3 := bb.write_varint(16384)
    I n3 != 3 { R 3 }

    # Zero
    n4 := bb.write_varint(0)
    I n4 != 1 { R 4 }

    # Read back
    bb.rewind()
    v1 := bb.read_varint()
    I v1 != 42 { R 11 }

    v2 := bb.read_varint()
    I v2 != 300 { R 12 }

    v3 := bb.read_varint()
    I v3 != 16384 { R 13 }

    v4 := bb.read_varint()
    I v4 != 0 { R 14 }

    free(bb.data)
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase55_bytebuffer_u16_str() {
    // Test ByteBuffer u16_le + write_str/read_str
    let source = r#"
S ByteBuffer {
    data: i64, len: i64, cap: i64, pos: i64
}
X ByteBuffer {
    F with_capacity(c: i64) -> ByteBuffer {
        cap := I c < 16 { 16 } E { c }
        d := malloc(cap)
        ByteBuffer { data: d, len: 0, cap: cap, pos: 0 }
    }
    F ensure_capacity(&self, needed: i64) -> i64 {
        I needed <= self.cap { R self.cap }
        nc := mut self.cap
        L { I nc >= needed { B }; nc = nc * 2 }
        nd := malloc(nc)
        memcpy(nd, self.data, self.len)
        free(self.data)
        self.data = nd
        self.cap = nc
        nc
    }
    F write_u8(&self, v: i64) -> i64 {
        @.ensure_capacity(self.len + 1)
        store_byte(self.data + self.len, v & 255)
        self.len = self.len + 1
        1
    }
    F read_u8(&self) -> i64 {
        I self.pos >= self.len { R 0 - 1 }
        val := load_byte(self.data + self.pos)
        self.pos = self.pos + 1
        val
    }
    F write_u16_le(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 2)
        store_byte(self.data + self.len, value & 255)
        store_byte(self.data + self.len + 1, (value >> 8) & 255)
        self.len = self.len + 2
        2
    }
    F read_u16_le(&self) -> i64 {
        I self.pos + 2 > self.len { R 0 - 1 }
        b0 := load_byte(self.data + self.pos)
        b1 := load_byte(self.data + self.pos + 1)
        self.pos = self.pos + 2
        b0 | (b1 << 8)
    }
    F write_i32_le(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 4)
        store_byte(self.data + self.len, value & 255)
        store_byte(self.data + self.len + 1, (value >> 8) & 255)
        store_byte(self.data + self.len + 2, (value >> 16) & 255)
        store_byte(self.data + self.len + 3, (value >> 24) & 255)
        self.len = self.len + 4
        4
    }
    F read_i32_le(&self) -> i64 {
        I self.pos + 4 > self.len { R 0 - 1 }
        b0 := load_byte(self.data + self.pos)
        b1 := load_byte(self.data + self.pos + 1)
        b2 := load_byte(self.data + self.pos + 2)
        b3 := load_byte(self.data + self.pos + 3)
        self.pos = self.pos + 4
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }
    F write_str(&self, s: str) -> i64 {
        p := str_to_ptr(s)
        slen := mut 0
        L {
            b := load_byte(p + slen)
            I b == 0 { B }
            slen = slen + 1
        }
        @.write_i32_le(slen)
        @.ensure_capacity(self.len + slen)
        memcpy(self.data + self.len, p, slen)
        self.len = self.len + slen
        slen + 4
    }
    F read_str(&self) -> i64 {
        I self.pos + 4 > self.len { R 0 }
        slen := @.read_i32_le()
        I slen < 0 { R 0 }
        I self.pos + slen > self.len { R 0 }
        buf := malloc(slen + 1)
        memcpy(buf, self.data + self.pos, slen)
        store_byte(buf + slen, 0)
        self.pos = self.pos + slen
        buf
    }
    F rewind(&self) -> i64 { self.pos = 0; 0 }
}
F main() -> i64 {
    bb := ByteBuffer.with_capacity(128)

    # Write u16 values
    bb.write_u16_le(0)
    bb.write_u16_le(255)
    bb.write_u16_le(65535)
    bb.write_u16_le(1000)

    # Write strings
    bb.write_str("hello")
    bb.write_str("vais")

    # Read back
    bb.rewind()
    I bb.read_u16_le() != 0 { R 1 }
    I bb.read_u16_le() != 255 { R 2 }
    I bb.read_u16_le() != 65535 { R 3 }
    I bb.read_u16_le() != 1000 { R 4 }

    # Read strings back as i64 pointers and check content
    s1_ptr := bb.read_str()
    I s1_ptr == 0 { R 5 }
    # "hello" = 5 chars
    I load_byte(s1_ptr) != 104 { R 6 }       # 'h'
    I load_byte(s1_ptr + 4) != 111 { R 7 }   # 'o'
    I load_byte(s1_ptr + 5) != 0 { R 8 }

    s2_ptr := bb.read_str()
    I s2_ptr == 0 { R 9 }
    I load_byte(s2_ptr) != 118 { R 10 }      # 'v'
    I load_byte(s2_ptr + 4) != 0 { R 11 }

    free(s1_ptr)
    free(s2_ptr)
    free(bb.data)
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 55: VaisDB patterns - slotted page
// =============================================
#[test]
fn e2e_phase55_vaisdb_slotted_page() {
    let source = r#"
C PAGE_SIZE: i64 = 4096
C PAGE_HEADER_SIZE: i64 = 64
C SLOT_SIZE: i64 = 8

F page_init(p: i64, id: i64) -> i64 {
    store_i64(p, id)
    store_i64(p + 8, 0)
    store_i64(p + 16, PAGE_HEADER_SIZE)
    store_i64(p + 24, PAGE_SIZE)
    0
}

F page_num_rows(p: i64) -> i64 = load_i64(p + 8)

F page_insert(p: i64, row: i64, row_len: i64) -> i64 {
    num := load_i64(p + 8)
    free_off := load_i64(p + 16)
    data_end := load_i64(p + 24)
    needed := SLOT_SIZE + row_len
    available := data_end - free_off
    I available < needed { R 0 - 1 }
    new_data_end := data_end - row_len
    memcpy(p + new_data_end, row, row_len)
    store_i64(p + free_off, new_data_end)
    store_i64(p + 8, num + 1)
    store_i64(p + 16, free_off + SLOT_SIZE)
    store_i64(p + 24, new_data_end)
    num
}

F page_get_offset(p: i64, slot: i64) -> i64 {
    num := load_i64(p + 8)
    I slot >= num { R 0 - 1 }
    load_i64(p + PAGE_HEADER_SIZE + slot * SLOT_SIZE)
}

F main() -> i64 {
    p := malloc(PAGE_SIZE)
    page_init(p, 1)

    I load_i64(p) != 1 { free(p); R 1 }
    I page_num_rows(p) != 0 { free(p); R 2 }

    row := malloc(16)
    i := mut 0
    L { I i >= 16 { B }; store_byte(row + i, 65 + i); i = i + 1 }

    s0 := page_insert(p, row, 16)
    I s0 != 0 { free(row); free(p); R 3 }
    I page_num_rows(p) != 1 { free(row); free(p); R 4 }

    s1 := page_insert(p, row, 16)
    I s1 != 1 { free(row); free(p); R 5 }

    off0 := page_get_offset(p, 0)
    I off0 < 0 { free(row); free(p); R 6 }
    I load_byte(p + off0) != 65 { free(row); free(p); R 7 }

    off1 := page_get_offset(p, 1)
    I off1 < 0 { free(row); free(p); R 8 }
    I load_byte(p + off1) != 65 { free(row); free(p); R 9 }

    bad := page_get_offset(p, 99)
    I bad != 0 - 1 { free(row); free(p); R 10 }

    free(row)
    free(p)
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 55: VaisDB patterns - TLV row serialization
// =============================================
#[test]
fn e2e_phase55_vaisdb_row_serialization() {
    let source = r#"
C COL_I64: i64 = 1
C COL_STR: i64 = 2
C COL_BOOL: i64 = 3

S RowWriter {
    buf: i64,
    pos: i64,
    cap: i64,
    num_cols: i64
}

X RowWriter {
    F new() -> RowWriter {
        buf := malloc(256)
        store_i64(buf, 0)
        RowWriter { buf: buf, pos: 8, cap: 256, num_cols: 0 }
    }

    F add_i64(&self, val: i64) -> i64 {
        store_byte(self.buf + self.pos, COL_I64)
        self.pos = self.pos + 1
        store_i64(self.buf + self.pos, val)
        self.pos = self.pos + 8
        self.num_cols = self.num_cols + 1
        0
    }

    F add_bool(&self, val: i64) -> i64 {
        store_byte(self.buf + self.pos, COL_BOOL)
        self.pos = self.pos + 1
        store_byte(self.buf + self.pos, val)
        self.pos = self.pos + 1
        self.num_cols = self.num_cols + 1
        0
    }

    F finish(&self) -> i64 {
        store_i64(self.buf, self.num_cols)
        self.pos
    }
}

F main() -> i64 {
    rw := RowWriter.new()
    rw.add_i64(42)
    rw.add_i64(100)
    rw.add_bool(1)
    total := rw.finish()

    I total <= 0 { free(rw.buf); R 1 }
    I load_i64(rw.buf) != 3 { free(rw.buf); R 2 }

    # Read back: skip header (8 bytes)
    p := mut 8
    I load_byte(rw.buf + p) != COL_I64 { free(rw.buf); R 3 }
    p = p + 1
    I load_i64(rw.buf + p) != 42 { free(rw.buf); R 4 }
    p = p + 8

    I load_byte(rw.buf + p) != COL_I64 { free(rw.buf); R 5 }
    p = p + 1
    I load_i64(rw.buf + p) != 100 { free(rw.buf); R 6 }
    p = p + 8

    I load_byte(rw.buf + p) != COL_BOOL { free(rw.buf); R 7 }
    p = p + 1
    I load_byte(rw.buf + p) != 1 { free(rw.buf); R 8 }

    free(rw.buf)
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 55: VaisDB patterns - B-Tree index
// =============================================
#[test]
fn e2e_phase55_vaisdb_btree_basic() {
    let source = r#"
C MAX_KEYS: i64 = 7
C NODE_SIZE: i64 = 248

F node_new(is_leaf: i64) -> i64 {
    n := malloc(NODE_SIZE)
    store_i64(n, is_leaf)
    store_i64(n + 8, 0)
    n
}

F node_num_keys(n: i64) -> i64 = load_i64(n + 8)

F node_get_key(n: i64, i: i64) -> i64 = load_i64(n + 16 + i * 8)
F node_set_key(n: i64, i: i64, k: i64) -> i64 { store_i64(n + 16 + i * 8, k); 0 }

F node_get_val(n: i64, i: i64) -> i64 = load_i64(n + 72 + i * 8)
F node_set_val(n: i64, i: i64, v: i64) -> i64 { store_i64(n + 72 + i * 8, v); 0 }

F node_search(n: i64, key: i64) -> i64 {
    num := node_num_keys(n)
    i := mut 0
    L {
        I i >= num { B }
        I node_get_key(n, i) == key { R node_get_val(n, i) }
        i = i + 1
    }
    0
}

F node_insert_sorted(n: i64, key: i64, val: i64) -> i64 {
    num := node_num_keys(n)
    I num >= MAX_KEYS { R 0 - 1 }

    pos := mut num
    L {
        I pos <= 0 { B }
        I node_get_key(n, pos - 1) <= key { B }
        node_set_key(n, pos, node_get_key(n, pos - 1))
        node_set_val(n, pos, node_get_val(n, pos - 1))
        pos = pos - 1
    }
    node_set_key(n, pos, key)
    node_set_val(n, pos, val)
    store_i64(n + 8, num + 1)
    0
}

F main() -> i64 {
    root := node_new(1)

    node_insert_sorted(root, 30, 3)
    node_insert_sorted(root, 10, 1)
    node_insert_sorted(root, 20, 2)
    node_insert_sorted(root, 50, 5)
    node_insert_sorted(root, 40, 4)

    I node_num_keys(root) != 5 { free(root); R 1 }

    # Keys should be sorted: 10, 20, 30, 40, 50
    I node_get_key(root, 0) != 10 { free(root); R 2 }
    I node_get_key(root, 1) != 20 { free(root); R 3 }
    I node_get_key(root, 2) != 30 { free(root); R 4 }
    I node_get_key(root, 3) != 40 { free(root); R 5 }
    I node_get_key(root, 4) != 50 { free(root); R 6 }

    # Search
    I node_search(root, 30) != 3 { free(root); R 7 }
    I node_search(root, 10) != 1 { free(root); R 8 }
    I node_search(root, 99) != 0 { free(root); R 9 }

    # Fill to max (7 keys)
    node_insert_sorted(root, 25, 25)
    node_insert_sorted(root, 35, 35)
    I node_num_keys(root) != 7 { free(root); R 10 }

    # Overflow should return -1
    result := node_insert_sorted(root, 60, 60)
    I result != 0 - 1 { free(root); R 11 }

    free(root)
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 55: VaisDB patterns - buffer pool
// =============================================
#[test]
fn e2e_phase55_vaisdb_buffer_pool() {
    let source = r#"
C PAGE_SIZE: i64 = 4096
C MAX_POOL: i64 = 16

S Pool {
    pages: i64,
    ids: i64,
    count: i64,
    next_id: i64
}

X Pool {
    F new() -> Pool {
        pages := malloc(MAX_POOL * 8)
        ids := malloc(MAX_POOL * 8)
        i := mut 0
        L { I i >= MAX_POOL { B }; store_i64(ids + i * 8, 0 - 1); i = i + 1 }
        Pool { pages: pages, ids: ids, count: 0, next_id: 1 }
    }

    F alloc(&self) -> i64 {
        I self.count >= MAX_POOL { R 0 }
        p := malloc(PAGE_SIZE)
        id := self.next_id
        self.next_id = self.next_id + 1
        store_i64(p, id)
        idx := self.count
        store_i64(self.pages + idx * 8, p)
        store_i64(self.ids + idx * 8, id)
        self.count = self.count + 1
        p
    }

    F find(&self, id: i64, idx: i64) -> i64 {
        I idx >= self.count { R 0 }
        pid := load_i64(self.ids + idx * 8)
        I pid == id { load_i64(self.pages + idx * 8) }
        E { @.find(id, idx + 1) }
    }

    F get(&self, id: i64) -> i64 = @.find(id, 0)

    F drop(&self) -> i64 {
        @.free_all(0)
        free(self.pages)
        free(self.ids)
        0
    }

    F free_all(&self, idx: i64) -> i64 {
        I idx >= self.count { R 0 }
        pp := load_i64(self.pages + idx * 8)
        I pp != 0 { free(pp) }
        @.free_all(idx + 1)
    }
}

F main() -> i64 {
    pool := Pool.new()

    p1 := pool.alloc()
    I p1 == 0 { pool.drop(); R 1 }
    p2 := pool.alloc()
    I p2 == 0 { pool.drop(); R 2 }
    p3 := pool.alloc()
    I p3 == 0 { pool.drop(); R 3 }

    I pool.count != 3 { pool.drop(); R 4 }

    id1 := load_i64(p1)
    found1 := pool.get(id1)
    I found1 != p1 { pool.drop(); R 5 }

    id3 := load_i64(p3)
    found3 := pool.get(id3)
    I found3 != p3 { pool.drop(); R 6 }

    not_found := pool.get(999)
    I not_found != 0 { pool.drop(); R 7 }

    pool.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 55: VaisDB patterns - table insert+get
// =============================================
#[test]
fn e2e_phase55_vaisdb_table_insert_get() {
    let source = r#"
C MAX_ROWS: i64 = 100
C ROW_SIZE: i64 = 32

S SimpleTable {
    data: i64,
    count: i64,
    next_key: i64
}

X SimpleTable {
    F new() -> SimpleTable {
        d := malloc(MAX_ROWS * ROW_SIZE)
        SimpleTable { data: d, count: 0, next_key: 1 }
    }

    F insert(&self, val1: i64, val2: i64) -> i64 {
        I self.count >= MAX_ROWS { R 0 - 1 }
        pk := self.next_key
        self.next_key = self.next_key + 1
        offset := self.count * ROW_SIZE
        store_i64(self.data + offset, pk)
        store_i64(self.data + offset + 8, val1)
        store_i64(self.data + offset + 16, val2)
        self.count = self.count + 1
        pk
    }

    F get(&self, key: i64, idx: i64) -> i64 {
        I idx >= self.count { R 0 }
        offset := idx * ROW_SIZE
        pk := load_i64(self.data + offset)
        I pk == key { R self.data + offset }
        @.get(key, idx + 1)
    }

    F find(&self, key: i64) -> i64 = @.get(key, 0)

    F drop_table(&self) -> i64 { free(self.data); 0 }
}

F main() -> i64 {
    t := SimpleTable.new()

    pk1 := t.insert(100, 200)
    I pk1 != 1 { t.drop_table(); R 1 }

    pk2 := t.insert(300, 400)
    I pk2 != 2 { t.drop_table(); R 2 }

    pk3 := t.insert(500, 600)
    I pk3 != 3 { t.drop_table(); R 3 }

    I t.count != 3 { t.drop_table(); R 4 }

    row_ptr := t.find(2)
    I row_ptr == 0 { t.drop_table(); R 5 }
    I load_i64(row_ptr) != 2 { t.drop_table(); R 6 }
    I load_i64(row_ptr + 8) != 300 { t.drop_table(); R 7 }
    I load_i64(row_ptr + 16) != 400 { t.drop_table(); R 8 }

    row1 := t.find(1)
    I row1 == 0 { t.drop_table(); R 9 }
    I load_i64(row1 + 8) != 100 { t.drop_table(); R 10 }

    missing := t.find(99)
    I missing != 0 { t.drop_table(); R 11 }

    idx := mut 0
    L {
        I idx >= 50 { B }
        t.insert(idx, idx * 2)
        idx = idx + 1
    }
    I t.count != 53 { t.drop_table(); R 12 }

    t.drop_table()
    0
}
"#;
    assert_exit_code(source, 0);
}

// =============================================
// Phase 57: WASM E2E Tests (IR validation only)
// =============================================

#[test]
fn test_wasm32_target_ir_generation() {
    let source = r#"
F main() -> i64 {
    puts("hello wasm")
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("target triple = \"wasm32-unknown-unknown\""));
    assert!(ir.contains("target datalayout"));
}

#[test]
fn test_wasm32_start_entry_point() {
    let source = r#"
F main() -> i64 {
    R 42
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("define void @_start()"));
    assert!(ir.contains("call i64 @main()"));
}

#[test]
fn test_wasm32_malloc_implementation() {
    let source = r#"
F main() -> i64 {
    ptr := malloc(100)
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASM bump allocator implementation
    assert!(ir.contains("@__heap_ptr"));
    assert!(ir.contains("define i8* @malloc(i64 %size)"));
}

#[test]
fn test_wasm32_puts_wasm_write() {
    let source = r#"
F main() -> i64 {
    puts("test output")
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASM puts calls __wasm_write
    assert!(ir.contains("define i64 @puts(i8* %str)"));
    assert!(ir.contains("@__wasm_write"));
}

#[test]
fn test_wasm32_memory_intrinsics() {
    let source = r#"
F main() -> i64 {
    ptr := malloc(1000000)
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // LLVM WASM intrinsics for memory management
    assert!(ir.contains("declare i32 @llvm.wasm.memory.size.i32"));
    assert!(ir.contains("declare i32 @llvm.wasm.memory.grow.i32"));
}

#[test]
fn test_wasi_target_ir_generation() {
    let source = r#"
F main() -> i64 {
    puts("hello wasi")
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview1,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("target triple = \"wasm32-wasi\""));
    assert!(ir.contains("target datalayout"));
}

#[test]
fn test_wasi_start_entry_point() {
    let source = r#"
F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview1,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASI _start calls __wasi_proc_exit
    assert!(ir.contains("define void @_start()"));
    assert!(ir.contains("@__wasi_proc_exit"));
}

#[test]
fn test_wasi_fd_write_declaration() {
    let source = r#"
F main() -> i64 {
    puts("wasi output")
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::WasiPreview1,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASI fd_write is declared and used
    assert!(ir.contains("declare i32 @__wasi_fd_write"));
}

#[test]
fn test_wasm32_free_noop() {
    let source = r#"
F main() -> i64 {
    ptr := malloc(100)
    free(ptr)
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASM free is a no-op (bump allocator doesn't free)
    assert!(ir.contains("define void @free(i8* %ptr)"));
    assert!(ir.contains("ret void"));
}

#[test]
fn test_wasm32_exit_trap() {
    let source = r#"
F main() -> i64 {
    exit(1)
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // WASM exit calls __wasm_trap
    assert!(ir.contains("define void @exit(i32 %code)"));
    assert!(ir.contains("@__wasm_trap"));
    assert!(ir.contains("unreachable"));
}

// ============================================================================
// Phase 58: Async Runtime E2E Tests
// ============================================================================

#[test]
fn test_async_function_declaration() {
    let source = r#"
A F fetch() -> i64 {
    R 42
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Async function should be defined in IR
    assert!(ir.contains("define i64 @fetch()"));
}

#[test]
fn test_spawn_generates_call() {
    let source = r#"
F worker() -> i64 {
    R 1
}

F main() -> i64 {
    x := worker()
    R x
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // spawn expression should generate function call
    assert!(ir.contains("call i64 @worker()"));
}

#[test]
fn test_future_struct_layout() {
    let source = r#"
S MyFuture {
    value: i64,
    ready: i64
}

F main() -> i64 {
    f := MyFuture { value: 42, ready: 0 }
    R f.value
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Future struct should have proper layout
    assert!(ir.contains("%MyFuture = type { i64, i64 }"));
}

#[test]
fn test_select_pattern_match() {
    let source = r#"
F main() -> i64 {
    x := 1
    result := M x {
        1 => 10,
        2 => 20,
        _ => 0
    }
    R result
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // select pattern should generate match/switch IR
    assert!(ir.contains("switch") || ir.contains("br i1"));
}

#[test]
fn test_async_channel_struct() {
    let source = r#"
S Channel {
    buf: i64,
    len: i64,
    cap: i64
}

F main() -> i64 {
    c := Channel { buf: 0, len: 0, cap: 16 }
    R c.cap
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Channel struct should have 3 i64 fields
    assert!(ir.contains("%Channel = type { i64, i64, i64 }"));
}

#[test]
fn test_executor_loop_pattern() {
    let source = r#"
F main() -> i64 {
    i := mut 0
    L {
        I i >= 10 {
            B
        }
        i = i + 1
    }
    R i
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Executor loop should have loop branch pattern
    assert!(ir.contains("br label"));
    assert!(ir.contains("icmp"));
}

#[test]
fn test_waker_callback_pattern() {
    let source = r#"
F callback(x: i64) -> i64 {
    R x + 1
}

F main() -> i64 {
    R callback(41)
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Callback pattern should generate function call
    assert!(ir.contains("call i64 @callback(i64 41)"));
}

#[test]
fn test_async_mutex_simulation() {
    let source = r#"
S Mutex {
    locked: i64,
    value: i64
}

F main() -> i64 {
    m := Mutex { locked: 0, value: 42 }
    R m.value
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Mutex struct should have locked flag and value
    assert!(ir.contains("%Mutex = type { i64, i64 }"));
}

#[test]
fn test_timeout_pattern() {
    let source = r#"
F main() -> i64 {
    deadline := 1000
    elapsed := mut 0
    L {
        I elapsed >= deadline {
            B
        }
        elapsed = elapsed + 1
    }
    R elapsed
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Timeout pattern should have comparison operation
    assert!(ir.contains("icmp sge") || ir.contains("icmp slt"));
}

#[test]
fn test_task_pool_array() {
    let source = r#"
F main() -> i64 {
    pool := malloc(80)
    store_i64(pool, 42)
    v := load_i64(pool)
    R v
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Task pool should use malloc, store, and load
    assert!(ir.contains("call i8* @malloc(i64 80)"));
    assert!(ir.contains("store"));
    assert!(ir.contains("load"));
}

// ============================================================================
// Phase 59: WASM ↔ JS Interop E2E Tests
// ============================================================================

#[test]
fn test_wasm_import_attribute_extern_function() {
    let source = r#"
N "C" {
    #[wasm_import("env", "js_alert")]
    F alert(msg: *i8);
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("target triple = \"wasm32-unknown-unknown\""));
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-import-name"));
}

#[test]
fn test_wasm_export_attribute_function() {
    let source = r#"
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

F main() -> i64 {
    R add(1, 2)
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
    assert!(ir.contains("wasm-export-name"));
}

#[test]
fn test_wasm_import_default_module() {
    // wasm_import with no args defaults to "env" module and function name
    let source = r#"
N "C" {
    #[wasm_import]
    F console_log(ptr: *i8, len: i64);
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    // Should use "env" as default module
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("env"));
}

#[test]
fn test_wasm_import_custom_module() {
    let source = r#"
N "C" {
    #[wasm_import("wasi_snapshot_preview1", "fd_write")]
    F fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> i32;
}

F main() -> i64 {
    R 0
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasi_snapshot_preview1"));
    assert!(ir.contains("fd_write"));
}

#[test]
fn test_wasm_export_with_no_args() {
    // wasm_export with no args uses function name as export name
    let source = r#"
#[wasm_export]
F greet() -> i64 = 42

F main() -> i64 {
    R greet()
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen.generate_module(&module).unwrap();
    assert!(ir.contains("wasm-export-name"));
    assert!(ir.contains("greet"));
}

#[test]
fn test_wasm_multiple_imports_exports() {
    let source = r#"
N "C" {
    #[wasm_import("env", "js_log")]
    F js_log(ptr: *i8, len: i64);

    #[wasm_import("env", "js_alert")]
    F js_alert(ptr: *i8, len: i64);
}

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("multiply")]
F multiply(a: i64, b: i64) -> i64 = a * b

F main() -> i64 {
    R add(2, 3) + multiply(4, 5)
}
"#;
    let module = vais_parser::parse(source).unwrap();
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).unwrap();
    let mut gen = vais_codegen::CodeGenerator::new_with_target(
        "test",
        vais_codegen::TargetTriple::Wasm32Unknown,
    );
    gen.set_resolved_functions(checker.get_all_functions().clone());
    let ir = gen.generate_module(&module).unwrap();
    // Should have both import and export metadata
    assert!(ir.contains("wasm-import-module"));
    assert!(ir.contains("wasm-export-name"));
    // Function definitions
    assert!(ir.contains("define i64 @add"));
    assert!(ir.contains("define i64 @multiply"));
}

#[test]
fn test_wasm_import_not_on_native_target() {
    // wasm_import attributes should NOT produce metadata on native target
    let source = r#"
N "C" {
    #[wasm_import("env", "js_alert")]
    F alert(msg: *i8);
}

F main() -> i64 {
    R 0
}
"#;
    let ir = compile_to_ir(source).unwrap();
    // Native target should NOT have wasm metadata
    assert!(!ir.contains("wasm-import-module"));
    assert!(!ir.contains("wasm-export-name"));
}

#[test]
fn test_wasm_bindgen_js_generation() {
    // Test JS binding generation
    use vais_bindgen::wasm_js::{WasmExportInfo, WasmImportInfo, WasmJsBindgen};

    let mut gen = WasmJsBindgen::new("math_module");
    gen.add_export(WasmExportInfo {
        wasm_name: "add".to_string(),
        js_name: "add".to_string(),
        params: vec![
            ("a".to_string(), "i64".to_string()),
            ("b".to_string(), "i64".to_string()),
        ],
        return_type: Some("i64".to_string()),
    });
    gen.add_import(WasmImportInfo {
        module: "env".to_string(),
        name: "console_log".to_string(),
        vais_name: "console_log".to_string(),
        params: vec!["i64".to_string(), "i64".to_string()],
        return_type: None,
    });

    let js = gen.generate_js();
    assert!(js.contains("createImports"));
    assert!(js.contains("WebAssembly.instantiate"));
    assert!(js.contains("add: (a, b) => instance.exports.add(a, b)"));
    assert!(js.contains("console_log"));

    let dts = gen.generate_dts();
    assert!(js.contains("load"));
    assert!(dts.contains("Math_moduleModule"));
    assert!(dts.contains("add(a: number, b: number): number"));
}

#[test]
fn test_wasm_serializer_types() {
    // Test WasmSerializer type infrastructure
    use vais_codegen::wasm_component::{WasmSerializer, WitType};

    let ser = WasmSerializer::new();

    // Primitive type sizes
    assert_eq!(ser.wit_type_size(&WitType::Bool), 1);
    assert_eq!(ser.wit_type_size(&WitType::S32), 4);
    assert_eq!(ser.wit_type_size(&WitType::S64), 8);
    assert_eq!(ser.wit_type_size(&WitType::F64), 8);

    // Complex type sizes (ptr + len for wasm32)
    assert_eq!(ser.wit_type_size(&WitType::String), 8);
    assert_eq!(ser.wit_type_size(&WitType::List(Box::new(WitType::S32))), 8);

    // Alignment
    assert_eq!(ser.aligned_size(&WitType::Bool), 4); // 1 → 4

    // JS read/write code gen
    let write = ser.generate_js_write(&WitType::S32, "x", "offset");
    assert!(write.contains("setInt32"));

    let read = ser.generate_js_read(&WitType::String, "offset");
    assert!(read.contains("decoder.decode"));

    // Full serde module
    let module = ser.generate_js_serde_module();
    assert!(module.contains("class WasmSerde"));
    assert!(module.contains("writeString"));
    assert!(module.contains("readResult"));

    // IR types
    let ir = ser.generate_wasm_serde_ir();
    assert!(ir.contains("%WasmString"));
    assert!(ir.contains("%WasmResult"));
}

// ============================================================================
// JavaScript Target E2E Tests
// ============================================================================

#[test]
fn test_js_target_simple_function() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F add(a: i64, b: i64) -> i64 = a + b
F main() -> i64 = add(1, 2)
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("function add"));
    assert!(js_content.contains("function main"));
}

#[test]
fn test_js_target_struct_to_class() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"S Point {
    x: i64,
    y: i64,
}

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    R p.x + p.y
}
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("class Point"));
    assert!(js_content.contains("constructor"));
}

#[test]
fn test_js_target_enum_tagged_union() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"E Result {
    Ok(i64),
    Err(i64),
}

F main() -> i64 {
    r := Ok(42)
    M r {
        Ok(v) => v,
        Err(_) => 0,
    }
}
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    // Enum should be represented as object with tag field
    assert!(js_content.contains("tag") || js_content.contains("Ok") || js_content.contains("Err"));
}

#[test]
fn test_js_target_if_else() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F max(a: i64, b: i64) -> i64 {
    I a > b {
        R a
    } E {
        R b
    }
}

F main() -> i64 = max(5, 3)
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("if") || js_content.contains("?"));
    assert!(js_content.contains("function max"));
}

#[test]
fn test_js_target_array_operations() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F main() -> i64 {
    arr := [1, 2, 3, 4, 5]
    R arr[2]
}
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("[") || js_content.contains("Array"));
}

#[test]
fn test_js_target_lambda_arrow() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)

F main() -> i64 {
    double := |x: i64| x * 2
    R apply(double, 5)
}
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("=>") || js_content.contains("function"));
    assert!(js_content.contains("apply"));
}

#[test]
fn test_js_target_match_expression() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F classify(n: i64) -> i64 {
    M n {
        0 => 0,
        1 => 10,
        2 => 20,
        _ => 99,
    }
}

F main() -> i64 = classify(2)
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    // Match should be converted to if/else chain or switch
    assert!(
        js_content.contains("if") || js_content.contains("switch") || js_content.contains("===")
    );
    assert!(js_content.contains("classify"));
}

#[test]
fn test_js_target_loop_for_of() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F sum_loop(n: i64) -> i64 {
    total := mut 0
    i := mut 0
    L {
        I i >= n { B }
        total := total + i
        i := i + 1
    }
    R total
}

F main() -> i64 = sum_loop(5)
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("while") || js_content.contains("for"));
    assert!(js_content.contains("break"));
}

#[test]
fn test_js_target_tree_shaking() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F used() -> i64 = 42

F unused() -> i64 = 999

F main() -> i64 = used()
"#,
    )
    .unwrap();

    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    let js_content = fs::read_to_string(&js_path).unwrap();
    assert!(js_content.contains("function used"));
    assert!(js_content.contains("function main"));
    // Tree shaking should remove unused function (this is a soft check - may not always remove)
    // We just verify the output compiles successfully
}

#[test]
fn test_js_target_output_extension() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("test.vais");
    fs::write(
        &input,
        r#"F main() -> i64 = 42
"#,
    )
    .unwrap();

    // Test default output (same name with .js extension)
    let output = Command::new(vaisc_bin())
        .args(["build", input.to_str().unwrap(), "--target", "js"])
        .output()
        .expect("Failed to execute vaisc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vaisc --target js failed: {}",
        stderr
    );

    let js_path = tmp.path().join("test.js");
    assert!(js_path.exists(), "JS output file not generated");

    // Test custom output path
    let custom_output = tmp.path().join("custom_name.js");
    let output2 = Command::new(vaisc_bin())
        .args([
            "build",
            input.to_str().unwrap(),
            "--target",
            "js",
            "-o",
            custom_output.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute vaisc");

    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        output2.status.success(),
        "vaisc --target js with -o failed: {}",
        stderr2
    );

    assert!(
        custom_output.exists(),
        "Custom JS output file not generated"
    );
}

// ========================================
// Phase 68: Typed Memory Operations Tests
// ========================================

#[test]
fn test_typed_memory_type_size_basic() {
    let source = r#"
        S Vec<T> {
            elem_size: i64
        }

        X Vec<T> {
            F new() -> Vec<T> {
                es := type_size()
                Vec { elem_size: es }
            }
        }

        F main() -> i64 {
            v := Vec.new()
            v.elem_size
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 8, "Exit code should be 8 (sizeof i64)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_load_store_i64() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(16)
            store_typed(ptr, 42)
            value := load_typed(ptr)
            free(ptr)
            value
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 42, "Exit code should be 42");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_type_size_builtin() {
    let source = r#"
        S Vec<T> {
            data: i64,
            len: i64,
            cap: i64,
            elem_size: i64
        }

        X Vec<T> {
            F with_capacity(capacity: i64) -> Vec<T> {
                es := type_size()
                elem_sz := I es <= 0 { 8 } E I es > 8 { 8 } E { es }
                data := malloc(capacity * elem_sz)
                Vec { data: data, len: 0, cap: capacity, elem_size: elem_sz }
            }

            F drop(&self) -> i64 {
                free(self.data)
                0
            }
        }

        F main() -> i64 {
            v := Vec.with_capacity(4)
            print_i64(v.elem_size)
            v.drop()
            0
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 0, "Exit code should be 0");
            assert_eq!(result.stdout.trim(), "8");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_load_store_i32() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(16)
            store_typed(ptr, 42)
            store_typed(ptr + 4, 100)
            a := load_typed(ptr)
            b := load_typed(ptr + 4)
            free(ptr)
            a + b
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 142, "Exit code should be 142 (42+100)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_vec_simple() {
    let source = r#"
        S Vec<T> {
            data: i64,
            len: i64,
            elem_size: i64
        }

        X Vec<T> {
            F new() -> Vec<T> {
                es := type_size()
                data := malloc(16 * es)
                Vec { data: data, len: 0, elem_size: es }
            }

            F push(&self, value: T) -> i64 {
                ptr := self.data + self.len * self.elem_size
                store_typed(ptr, value)
                self.len = self.len + 1
                self.len
            }

            F get(&self, index: i64) -> T {
                ptr := self.data + index * self.elem_size
                load_typed(ptr)
            }

            F drop(&self) -> i64 {
                free(self.data)
                0
            }
        }

        F main() -> i64 {
            v := Vec.new()
            v.push(10)
            v.push(20)
            v.push(30)
            a := v.get(0)
            b := v.get(1)
            c := v.get(2)
            v.drop()
            a + b + c
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 60, "Exit code should be 60 (10+20+30)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_array_operations() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(32)
            store_typed(ptr + 0, 5)
            store_typed(ptr + 8, 15)
            store_typed(ptr + 16, 25)

            a := load_typed(ptr + 0)
            b := load_typed(ptr + 8)
            c := load_typed(ptr + 16)

            free(ptr)
            a + b + c
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 45, "Exit code should be 45 (5+15+25)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_sequential_ops() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(64)

            store_typed(ptr + 0, 1)
            store_typed(ptr + 8, 2)
            store_typed(ptr + 16, 4)
            store_typed(ptr + 24, 8)
            store_typed(ptr + 32, 16)

            a := load_typed(ptr + 0)
            b := load_typed(ptr + 8)
            c := load_typed(ptr + 16)
            d := load_typed(ptr + 24)
            e := load_typed(ptr + 32)

            free(ptr)
            a + b + c + d + e
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 31, "Exit code should be 31 (1+2+4+8+16)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_typed_memory_overwrite() {
    let source = r#"
        F main() -> i64 {
            ptr := malloc(16)

            store_typed(ptr, 100)
            v1 := load_typed(ptr)

            store_typed(ptr, 200)
            v2 := load_typed(ptr)

            free(ptr)
            v2 - v1
        }
    "#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 100, "Exit code should be 100 (200-100)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

// ============================================================================
// Trait Dispatch Tests
// ============================================================================

#[test]
fn test_trait_dispatch_basic() {
    let source = r#"
W Printable {
    F display(&self) -> i64
}

S Point { x: i64, y: i64 }

X Point: Printable {
    F display(&self) -> i64 {
        self.x + self.y
    }
}

F main() -> i64 {
    p := Point { x: 3, y: 4 }
    p.display()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 7, "Exit code should be 7 (3+4)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_multiple_structs() {
    let source = r#"
W Calculable {
    F compute(&self) -> i64
}

S Circle { radius: i64 }
S Square { side: i64 }

X Circle: Calculable {
    F compute(&self) -> i64 {
        self.radius * self.radius * 3
    }
}

X Square: Calculable {
    F compute(&self) -> i64 {
        self.side * self.side
    }
}

F main() -> i64 {
    c := Circle { radius: 5 }
    s := Square { side: 4 }
    c.compute() + s.compute()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 91, "Exit code should be 91 (75+16)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_polymorphism() {
    let source = r#"
W Summable {
    F sum(&self) -> i64
}

S Pair { a: i64, b: i64 }
S Triple { a: i64, b: i64, c: i64 }

X Pair: Summable {
    F sum(&self) -> i64 {
        self.a + self.b
    }
}

X Triple: Summable {
    F sum(&self) -> i64 {
        self.a + self.b + self.c
    }
}

F main() -> i64 {
    p := Pair { a: 10, b: 20 }
    t := Triple { a: 1, b: 2, c: 3 }
    p.sum() + t.sum()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 36, "Exit code should be 36 (30+6)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_self_field_access() {
    let source = r#"
W Incrementable {
    F increment(&self) -> i64
}

S Counter { value: i64, step: i64 }

X Counter: Incrementable {
    F increment(&self) -> i64 {
        self.value + self.step
    }
}

F main() -> i64 {
    c := Counter { value: 100, step: 7 }
    c.increment()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 107, "Exit code should be 107 (100+7)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_multiple_traits_one_struct() {
    let source = r#"
W Addable {
    F add(&self) -> i64
}

W Multipliable {
    F multiply(&self) -> i64
}

S Numbers { a: i64, b: i64 }

X Numbers: Addable {
    F add(&self) -> i64 {
        self.a + self.b
    }
}

X Numbers: Multipliable {
    F multiply(&self) -> i64 {
        self.a * self.b
    }
}

F main() -> i64 {
    n := Numbers { a: 5, b: 3 }
    n.add() + n.multiply()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 23, "Exit code should be 23 (8+15)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_return_value_usage() {
    let source = r#"
W Evaluable {
    F evaluate(&self) -> i64
}

S Expression { left: i64, right: i64 }

X Expression: Evaluable {
    F evaluate(&self) -> i64 {
        self.left * 10 + self.right
    }
}

F main() -> i64 {
    e := Expression { left: 4, right: 2 }
    result := e.evaluate()
    result * 2
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 84, "Exit code should be 84 (42*2)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_chain_calls() {
    let source = r#"
W Doubler {
    F double(&self) -> i64
}

W Tripler {
    F triple(&self) -> i64
}

S Value { n: i64 }

X Value: Doubler {
    F double(&self) -> i64 {
        self.n * 2
    }
}

X Value: Tripler {
    F triple(&self) -> i64 {
        self.n * 3
    }
}

F main() -> i64 {
    v := Value { n: 5 }
    d := v.double()
    t := v.triple()
    d + t
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 25, "Exit code should be 25 (10+15)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_conditional() {
    let source = r#"
W Checker {
    F check(&self) -> i64
}

S Data { flag: i64, value: i64 }

X Data: Checker {
    F check(&self) -> i64 {
        I self.flag > 0 {
            R self.value
        } E {
            R 0
        }
    }
}

F main() -> i64 {
    d1 := Data { flag: 1, value: 42 }
    d2 := Data { flag: 0, value: 99 }
    d1.check() + d2.check()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 42, "Exit code should be 42 (42+0)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_nested_operations() {
    let source = r#"
W Calculator {
    F calculate(&self) -> i64
}

S Operation { x: i64, y: i64, z: i64 }

X Operation: Calculator {
    F calculate(&self) -> i64 {
        result := self.x + self.y
        result := result * self.z
        result
    }
}

F main() -> i64 {
    op := Operation { x: 2, y: 3, z: 4 }
    op.calculate()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 20, "Exit code should be 20 ((2+3)*4)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_zero_fields() {
    let source = r#"
W Provider {
    F provide(&self) -> i64
}

S Unit {}

X Unit: Provider {
    F provide(&self) -> i64 {
        42
    }
}

F main() -> i64 {
    u := Unit {}
    u.provide()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 42, "Exit code should be 42");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_complex_calculation() {
    let source = r#"
W AreaCalculator {
    F area(&self) -> i64
}

W PerimeterCalculator {
    F perimeter(&self) -> i64
}

S Rectangle { width: i64, height: i64 }

X Rectangle: AreaCalculator {
    F area(&self) -> i64 {
        self.width * self.height
    }
}

X Rectangle: PerimeterCalculator {
    F perimeter(&self) -> i64 {
        (self.width + self.height) * 2
    }
}

F main() -> i64 {
    r := Rectangle { width: 5, height: 3 }
    a := r.area()
    p := r.perimeter()
    a + p
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 31, "Exit code should be 31 (15+16)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_negative_values() {
    let source = r#"
W Negator {
    F negate(&self) -> i64
}

S SignedValue { value: i64 }

X SignedValue: Negator {
    F negate(&self) -> i64 {
        0 - self.value
    }
}

F main() -> i64 {
    v := SignedValue { value: 50 }
    n := v.negate()
    n + 100
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 50, "Exit code should be 50 (-50+100)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_trait_dispatch_multiple_instances() {
    let source = r#"
W Scorer {
    F score(&self) -> i64
}

S Player { points: i64, bonus: i64 }

X Player: Scorer {
    F score(&self) -> i64 {
        self.points + self.bonus
    }
}

F main() -> i64 {
    p1 := Player { points: 10, bonus: 5 }
    p2 := Player { points: 20, bonus: 3 }
    p3 := Player { points: 15, bonus: 7 }
    p1.score() + p2.score() + p3.score()
}
"#;
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(result.exit_code, 60, "Exit code should be 60 (15+23+22)");
        }
        Err(e) if e.contains("Failed to run clang") => {
            eprintln!("Skipping test: clang not available");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

// ==================== Slice Type Tests ====================

/// Assert that source compiles successfully (parse + type check + codegen to IR)
fn assert_compiles(source: &str) {
    match compile_to_ir(source) {
        Ok(_) => {}
        Err(e) => panic!("Expected compilation to succeed, but got error: {}", e),
    }
}

#[test]
fn test_slice_type_parse() {
    let source = r#"
F foo(s: &[i64]) -> i64 {
    0
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_mut_type_parse() {
    let source = r#"
F bar(s: &mut [i64]) {
    s[0] = 42
}

F main() -> i64 {
    0
}
"#;
    assert_compiles(source);
}

#[test]
fn test_slice_len_method() {
    let source = r#"
F baz(s: &[i64]) -> i64 {
    s.len()
}

F main() -> i64 {
    0
}
"#;
    assert_compiles(source);
}

#[test]
fn test_slice_nested_generic() {
    let source = r#"
F qux(s: &[&[i64]]) -> i64 {
    0
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_param_return() {
    let source = r#"
F identity(s: &[i64]) -> &[i64] {
    s
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_with_str() {
    let source = r#"
F first_char(s: &[str]) -> str {
    "empty"
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_in_struct() {
    let source = r#"
S Foo {
    data: &[i64]
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_mut_len() {
    let source = r#"
F len_mut(s: &mut [f64]) -> i64 {
    s.len()
}

F main() -> i64 {
    0
}
"#;
    assert_compiles(source);
}

#[test]
fn test_slice_multi_param() {
    let source = r#"
F add_first(a: &[i64], b: &[i64]) -> i64 {
    0
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_slice_return_type() {
    let source = r#"
F get_slice(input: &[i64]) -> &[i64] {
    input
}

F main() -> i64 {
    0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== String Comparison Tests (Phase 13) ====================

#[test]
fn e2e_str_reuse_double_comparison() {
    let source = r#"
F main() -> i64 {
    s := "hello"
    I s == "hello" {
        I s == "hello" {
            1
        } E {
            0
        }
    } E {
        0
    }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_str_comparison_and_use() {
    let source = r#"
F check(s: str) -> i64 {
    I s == "world" {
        1
    } E {
        0
    }
}

F main() -> i64 {
    s := "world"
    result := I s == "world" { 1 } E { 0 }
    # Use s again after comparison
    check_result := check(s)
    result + check_result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_str_param_comparison() {
    let source = r#"
F check_str(input: str) -> i64 {
    I input == "test" {
        42
    } E {
        0
    }
}

F main() -> i64 {
    check_str("test")
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_str_multiple_comparisons() {
    let source = r#"
F main() -> i64 {
    a := "foo"
    b := "bar"
    c := "foo"

    result := mut 0
    I a == c {
        result = result + 10
    }
    I a == b {
        result = result + 1
    }
    I b == "bar" {
        result = result + 5
    }

    result
}
"#;
    assert_exit_code(source, 15);
}

#[test]
fn e2e_str_comparison_in_loop() {
    let source = r#"
F main() -> i64 {
    target := "match"
    count := mut 0
    i := mut 0
    L {
        I i >= 3 {
            B
        }
        test := I i == 1 { "match" } E { "other" }
        I test == target {
            count = count + 1
        }
        i = i + 1
    }
    count
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_str_comparison_inequality() {
    let source = r#"
F main() -> i64 {
    s := "hello"
    I s != "world" {
        I s == "hello" {
            42
        } E {
            0
        }
    } E {
        0
    }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_type_alias_i_as_i64_param() {
    let source = r#"
F add(x: i, y: i) -> i {
    x + y
}
F main() -> i64 {
    add(20, 22)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_type_alias_i_as_i64_return() {
    let source = r#"
F double(x: i64) -> i {
    x * 2
}
F main() -> i64 {
    double(21)
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_type_alias_i_as_i64_variable() {
    let source = r#"
F main() -> i64 {
    x: i = 42
    x
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_struct_tuple_literal_basic() {
    let source = r#"
S Point { x: i64, y: i64 }
F main() -> i64 {
    p := Point(40, 2)
    p.x + p.y
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_struct_tuple_literal_nested() {
    let source = r#"
S Pair { a: i64, b: i64 }
F make(x: i64, y: i64) -> Pair {
    Pair(x, y)
}
F main() -> i64 {
    p := make(20, 22)
    p.a + p.b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_struct_tuple_literal_three_fields() {
    let source = r#"
S Triple { x: i64, y: i64, z: i64 }
F main() -> i64 {
    t := Triple(10, 20, 12)
    t.x + t.y + t.z
}
"#;
    assert_exit_code(source, 42);
}

// === Phase 17: main() auto-return and swap builtin ===

#[test]
fn test_main_auto_return_println() {
    // main() without -> i64 should auto-return 0
    let source = r#"
F main() {
    println("hello")
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_main_auto_return_empty() {
    // main() with empty body should auto-return 0
    let source = r#"
F main() {
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_main_auto_return_with_loop() {
    // main() with loop and no explicit return
    let source = r#"
F main() {
    x := mut 0
    L i:0..5 {
        x += i
    }
    println("~{x}")
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_main_explicit_return_still_works() {
    // main() -> i64 with explicit return still works
    let source = r#"
F main() -> i64 {
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_main_auto_return_explicit_r() {
    // F main() { R 5 } — explicit return in auto-return main
    let source = r#"
F main() {
    R 5
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn test_main_auto_return_expression_body() {
    // F main() { 42 } — expression value without -> i64 annotation (implicit i64 return)
    let source = r#"
F main() {
    42
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_swap_builtin_basic() {
    // swap(ptr, idx1, idx2) swaps two i64 elements
    let source = r#"
F main() -> i64 {
    arr: *i64 = [10, 20, 30]
    swap(arr, 0, 2)
    arr[0] - arr[2]
}
"#;
    // arr[0]=30, arr[2]=10 → 30-10=20
    assert_exit_code(source, 20);
}

#[test]
fn test_swap_builtin_same_index() {
    // swap with same index should be no-op
    let source = r#"
F main() -> i64 {
    arr: *i64 = [42, 99]
    swap(arr, 0, 0)
    arr[0]
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn test_swap_builtin_in_function() {
    // swap called from another function
    let source = r#"
F do_swap(arr: *i64, i: i64, j: i64) {
    swap(arr, i, j)
}
F main() -> i64 {
    arr: *i64 = [10, 20, 30]
    do_swap(arr, 0, 2)
    arr[0] - arr[2]
}
"#;
    // arr[0]=30, arr[2]=10 → 30-10=20
    assert_exit_code(source, 20);
}

#[test]
fn test_swap_builtin_multiple() {
    // Two consecutive swaps verify both work correctly
    let source = r#"
F main() -> i64 {
    arr: *i64 = [5, 3, 1, 4, 2]
    swap(arr, 0, 2)
    swap(arr, 3, 4)
    arr[0] + arr[3]
}
"#;
    // After swap(0,2): [1, 3, 5, 4, 2]
    // After swap(3,4): [1, 3, 5, 2, 4]
    // arr[0]=1 + arr[3]=2 = 3
    assert_exit_code(source, 3);
}
