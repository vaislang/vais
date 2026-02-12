use super::helpers::*;

// ==================== Void Phi Node Bug Fix ====================

#[test]
fn e2e_void_phi_if_else_with_assert() {
    // Regression test for void phi node bug:
    // If-else expressions where both branches return Unit (void) type
    // should not generate phi nodes, as "phi void" is invalid LLVM IR.
    // This test uses assert expressions which return Unit type.
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

F main() -> i64 {
    x := 5

    # if-else with assert (Unit type) in both branches
    I x > 3 {
        assert(x > 0)
    } E {
        assert(x >= 0)
    }

    # Nested case
    I x > 10 {
        assert(x > 10)
    } E {
        I x > 0 {
            assert(x > 0)
        } E {
            assert(x >= 0)
        }
    }

    printf("done\n")
    0
}
"#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "void phi test failed: {}",
        result.stderr
    );
    assert!(result.stdout.contains("done"), "Expected 'done' in output");
}

// ==================== Thread Runtime E2E Tests ====================

/// Helper to find the thread runtime C file path
fn find_thread_runtime_path() -> Option<String> {
    let candidates = [
        "std/thread_runtime.c",
        "../std/thread_runtime.c",
        "../../std/thread_runtime.c",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("std").join("thread_runtime.c"));
        if let Some(path) = p {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[test]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "pthread_tryjoin_np is macOS-specific"
)]
fn e2e_thread_sleep_yield() {
    let rt = match find_thread_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: thread_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __thread_sleep_ms(ms: i64) -> i64
X F __thread_yield() -> i64

F main() -> i64 {
    printf("sleep 10ms\n")
    __thread_sleep_ms(10)
    printf("yield\n")
    __thread_yield()
    printf("done\n")
    0
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "thread sleep/yield test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("sleep 10ms"),
        "Expected 'sleep 10ms' in output"
    );
    assert!(
        result.stdout.contains("yield"),
        "Expected 'yield' in output"
    );
    assert!(result.stdout.contains("done"), "Expected 'done' in output");
}

// ==================== Sync Runtime E2E Tests ====================

/// Helper to find the sync runtime C file path
fn find_sync_runtime_path() -> Option<String> {
    let candidates = [
        "std/sync_runtime.c",
        "../std/sync_runtime.c",
        "../../std/sync_runtime.c",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("std").join("sync_runtime.c"));
        if let Some(path) = p {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[test]
fn e2e_sync_mutex_lock_unlock() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __mutex_create() -> i64
X F __mutex_lock(h: i64) -> i64
X F __mutex_unlock(h: i64) -> i64
X F __mutex_destroy(h: i64) -> i64

F main() -> i64 {
    m := __mutex_create()
    printf("mutex created: %lld\n", m)

    rc1 := __mutex_lock(m)
    printf("lock: %lld\n", rc1)

    rc2 := __mutex_unlock(m)
    printf("unlock: %lld\n", rc2)

    rc3 := __mutex_destroy(m)
    printf("destroy: %lld\n", rc3)

    I m > 0 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "mutex lock/unlock test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("mutex created"),
        "Expected 'mutex created' in output"
    );
}

#[test]
fn e2e_sync_rwlock_read_write() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __rwlock_create() -> i64
X F __rwlock_read_lock(h: i64) -> i64
X F __rwlock_read_unlock(h: i64) -> i64
X F __rwlock_write_lock(h: i64) -> i64
X F __rwlock_write_unlock(h: i64) -> i64
X F __rwlock_destroy(h: i64) -> i64

F main() -> i64 {
    rw := __rwlock_create()
    printf("rwlock created: %lld\n", rw)

    __rwlock_read_lock(rw)
    printf("read locked\n")
    __rwlock_read_unlock(rw)
    printf("read unlocked\n")

    __rwlock_write_lock(rw)
    printf("write locked\n")
    __rwlock_write_unlock(rw)
    printf("write unlocked\n")

    __rwlock_destroy(rw)
    printf("destroyed\n")

    0
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(result.exit_code, 0, "rwlock test failed: {}", result.stderr);
    assert!(
        result.stdout.contains("read locked"),
        "Expected 'read locked' in output"
    );
    assert!(
        result.stdout.contains("write locked"),
        "Expected 'write locked' in output"
    );
}

#[test]
fn e2e_sync_barrier_single() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __barrier_create(count: i64) -> i64
X F __barrier_wait(h: i64) -> i64
X F __barrier_destroy(h: i64) -> i64

F main() -> i64 {
    b := __barrier_create(1)
    printf("barrier created: %lld\n", b)

    rc := __barrier_wait(b)
    printf("barrier wait returned: %lld\n", rc)

    __barrier_destroy(b)
    printf("barrier destroyed\n")

    0
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "barrier test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("barrier created"),
        "Expected 'barrier created' in output"
    );
}

#[test]
fn e2e_sync_semaphore() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __semaphore_create(permits: i64) -> i64
X F __semaphore_wait(h: i64) -> i64
X F __semaphore_try_wait(h: i64) -> i64
X F __semaphore_post(h: i64) -> i64
X F __semaphore_destroy(h: i64) -> i64

F main() -> i64 {
    sem := __semaphore_create(2)
    printf("semaphore created with 2 permits\n")

    # Acquire twice
    __semaphore_wait(sem)
    printf("acquired 1\n")
    __semaphore_wait(sem)
    printf("acquired 2\n")

    # Try to acquire again (should fail)
    r1 := __semaphore_try_wait(sem)
    printf("try_wait (should fail): %lld\n", r1)

    # Release and try again
    __semaphore_post(sem)
    printf("released 1\n")
    r2 := __semaphore_try_wait(sem)
    printf("try_wait (should succeed): %lld\n", r2)

    __semaphore_destroy(sem)

    I r1 == 0 {
        I r2 == 1 { 0 } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "semaphore test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("acquired 1"),
        "Expected 'acquired 1' in output"
    );
    assert!(
        result.stdout.contains("acquired 2"),
        "Expected 'acquired 2' in output"
    );
}

/// Helper to find the http runtime C file path (for malloc/free)
fn find_http_runtime_path() -> Option<String> {
    let candidates = [
        "std/http_runtime.c",
        "../std/http_runtime.c",
        "../../std/http_runtime.c",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("std").join("http_runtime.c"));
        if let Some(path) = p {
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[test]
fn e2e_sync_atomics() {
    let sync_rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let http_rt = match find_http_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: http_runtime.c not found (needed for malloc/free)");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __malloc(size: i64) -> i64
X F __free(ptr: i64) -> i64
X F __atomic_load_i64(ptr: i64) -> i64
X F __atomic_store_i64(ptr: i64, value: i64) -> i64
X F __atomic_fetch_add_i64(ptr: i64, value: i64) -> i64
X F __atomic_compare_exchange_i64(ptr: i64, expected: i64, desired: i64) -> i64

F main() -> i64 {
    # Allocate memory for atomic value
    ptr := __malloc(8)

    # Store 10
    __atomic_store_i64(ptr, 10)
    v1 := __atomic_load_i64(ptr)
    printf("after store 10: %lld\n", v1)

    # Fetch add 5
    old := __atomic_fetch_add_i64(ptr, 5)
    v2 := __atomic_load_i64(ptr)
    printf("after fetch_add 5: old=%lld new=%lld\n", old, v2)

    # Compare exchange (15 -> 20)
    rc1 := __atomic_compare_exchange_i64(ptr, 15, 20)
    v3 := __atomic_load_i64(ptr)
    printf("cas(15->20): rc=%lld value=%lld\n", rc1, v3)

    # Compare exchange (15 -> 30) should fail
    rc2 := __atomic_compare_exchange_i64(ptr, 15, 30)
    v4 := __atomic_load_i64(ptr)
    printf("cas(15->30): rc=%lld value=%lld\n", rc2, v4)

    __free(ptr)

    I v1 == 10 {
        I v2 == 15 {
            I v3 == 20 {
                I rc1 == 0 {
                    I rc2 == 1 { 0 } E { 5 }
                } E { 4 }
            } E { 3 }
        } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&sync_rt, &http_rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "atomics test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("after store 10: 10"),
        "Expected store result"
    );
    assert!(
        result.stdout.contains("after fetch_add 5"),
        "Expected fetch_add result"
    );
}

// ==================== Condvar Runtime E2E Tests ====================

#[test]
fn e2e_sync_condvar_create_destroy() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __condvar_create() -> i64
X F __condvar_destroy(h: i64) -> i64

F main() -> i64 {
    cv := __condvar_create()
    printf("condvar created: %lld\n", cv)

    rc := __condvar_destroy(cv)
    printf("condvar destroyed: %lld\n", rc)

    I cv > 0 { 0 } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "condvar create/destroy test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("condvar created"),
        "Expected 'condvar created' in output"
    );
    assert!(
        result.stdout.contains("condvar destroyed"),
        "Expected 'condvar destroyed' in output"
    );
}

#[test]
fn e2e_sync_condvar_signal() {
    let rt = match find_sync_runtime_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: sync_runtime.c not found");
            return;
        }
    };
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

X F __condvar_create() -> i64
X F __condvar_signal(h: i64) -> i64
X F __condvar_broadcast(h: i64) -> i64
X F __condvar_destroy(h: i64) -> i64

F main() -> i64 {
    cv := __condvar_create()

    rc1 := __condvar_signal(cv)
    printf("signal: %lld\n", rc1)

    rc2 := __condvar_broadcast(cv)
    printf("broadcast: %lld\n", rc2)

    __condvar_destroy(cv)

    I rc1 == 0 {
        I rc2 == 0 { 0 } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run_with_extra_sources(source, &[&rt]).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "condvar signal/broadcast test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("signal: 0"),
        "Expected 'signal: 0' in output"
    );
    assert!(
        result.stdout.contains("broadcast: 0"),
        "Expected 'broadcast: 0' in output"
    );
}

// ==================== f64 Arithmetic E2E Tests ====================

#[test]
fn e2e_f64_arithmetic() {
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

F main() -> i64 {
    a: f64 = 3.14
    b: f64 = 2.0

    sum := a + b
    diff := a - b
    prod := a * b
    quot := a / b

    # f64 values need to be passed directly to printf with %f
    # Note: This test verifies that f64 arithmetic works
    printf("f64 arithmetic test\n")

    # Test basic operations by converting results to validation
    # sum should be ~5.14, diff ~1.14
    sum_ok := I sum > 5.0 { I sum < 6.0 { 1 } E { 0 } } E { 0 }
    diff_ok := I diff > 1.0 { I diff < 2.0 { 1 } E { 0 } } E { 0 }
    prod_ok := I prod > 6.0 { I prod < 7.0 { 1 } E { 0 } } E { 0 }
    quot_ok := I quot > 1.5 { I quot < 1.6 { 1 } E { 0 } } E { 0 }

    printf("sum_ok=%lld diff_ok=%lld prod_ok=%lld quot_ok=%lld\n", sum_ok, diff_ok, prod_ok, quot_ok)

    I sum_ok == 1 {
        I diff_ok == 1 {
            I prod_ok == 1 {
                I quot_ok == 1 { 0 } E { 4 }
            } E { 3 }
        } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "f64 arithmetic test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("f64 arithmetic test"),
        "Expected test label in output"
    );
    assert!(
        result.stdout.contains("sum_ok=1"),
        "Expected sum_ok=1 in output"
    );
    assert!(
        result.stdout.contains("diff_ok=1"),
        "Expected diff_ok=1 in output"
    );
}

#[test]
fn e2e_f64_comparison() {
    let source = r#"
N "C" {
    F printf(fmt: str, ...) -> i64
}

F main() -> i64 {
    a: f64 = 3.5
    b: f64 = 2.5
    c: f64 = 3.5

    gt := I a > b { 1 } E { 0 }
    lt := I a < b { 1 } E { 0 }
    eq := I a == c { 1 } E { 0 }
    ge := I a >= c { 1 } E { 0 }
    le := I b <= a { 1 } E { 0 }

    printf("a > b: %lld\n", gt)
    printf("a < b: %lld\n", lt)
    printf("a == c: %lld\n", eq)
    printf("a >= c: %lld\n", ge)
    printf("b <= a: %lld\n", le)

    I gt == 1 {
        I lt == 0 {
            I eq == 1 {
                I ge == 1 {
                    I le == 1 { 0 } E { 5 }
                } E { 4 }
            } E { 3 }
        } E { 2 }
    } E { 1 }
}
"#;
    let result = compile_and_run(source).unwrap();
    assert_eq!(
        result.exit_code, 0,
        "f64 comparison test failed: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("a > b: 1"),
        "Expected 'a > b: 1' in output"
    );
    assert!(
        result.stdout.contains("a == c: 1"),
        "Expected 'a == c: 1' in output"
    );
}
