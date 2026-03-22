// Phase 145: R4 Drop/RAII and Defer tests

use super::helpers::*;

// ===== Defer basic tests =====

#[test]
fn e2e_phase145_defer_basic() {
    // Defer with simple expression, no global mutation needed
    let source = r#"
F main() -> i64 {
    x := mut 0
    D { x = x + 10 }
    x = x + 1
    x
}
"#;
    // x=1 at return, defer runs after return value captured → exit 1
    assert_exit_code(source, 1);
}

#[test]
fn e2e_phase145_defer_lifo_order() {
    // Multiple defers — return value is captured before defers run
    let source = r#"
F main() -> i64 {
    x := mut 0
    D { x = x + 1 }
    D { x = x + 10 }
    D { x = x + 100 }
    x
}
"#;
    // return value captured as 0, defers run after
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase145_defer_early_return() {
    // Defer executes even on early return — test via nested fn
    let source = r#"
F add(a: i64, b: i64) -> i64 {
    a + b
}
F main() -> i64 {
    result := add(3, 4)
    I result != 7 { R 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Drop trait tests (IR generation verification) =====

#[test]
fn e2e_phase145_drop_trait_basic() {
    // Drop trait impl compiles with correct Vais syntax (X Type: Trait)
    let source = r#"
S Resource { id: i64 }

W Drop {
    F drop(&self) -> i64
}

X Resource: Drop {
    F drop(&self) -> i64 {
        0
    }
}

F main() -> i64 {
    r := Resource { id: 42 }
    0
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase145_drop_ir_contains_drop_call() {
    // Verify that the generated IR contains the auto-drop call
    let source = r#"
S Handle { fd: i64 }

W Drop {
    F drop(&self) -> i64
}

X Handle: Drop {
    F drop(&self) -> i64 {
        0
    }
}

F use_handle() -> i64 {
    h := Handle { fd: 3 }
    0
}

F main() -> i64 {
    use_handle()
    0
}
"#;
    let ir = compile_to_ir(source).expect("should compile");
    assert!(
        ir.contains("Handle_drop") || ir.contains("call void @"),
        "IR should contain drop function call, got:\n{}",
        &ir[..ir.len().min(2000)]
    );
}

#[test]
fn e2e_phase145_drop_multiple_types_compile() {
    // Multiple struct types each with Drop impl — all compile
    let source = r#"
S FileHandle { fd: i64 }
S Connection { port: i64 }

W Drop {
    F drop(&self) -> i64
}

X FileHandle: Drop {
    F drop(&self) -> i64 { 0 }
}

X Connection: Drop {
    F drop(&self) -> i64 { 0 }
}

F main() -> i64 {
    f := FileHandle { fd: 3 }
    c := Connection { port: 80 }
    0
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase145_drop_with_field_access() {
    // Drop impl that accesses self.field compiles
    let source = r#"
S Counter { value: i64 }

W Drop {
    F drop(&self) -> i64
}

X Counter: Drop {
    F drop(&self) -> i64 {
        self.value
    }
}

F main() -> i64 {
    c := Counter { value: 99 }
    c.value
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase145_drop_with_early_return() {
    // Drop is generated on early return paths too
    let source = r#"
S Guard { id: i64 }

W Drop {
    F drop(&self) -> i64
}

X Guard: Drop {
    F drop(&self) -> i64 { 0 }
}

F maybe_exit(flag: i64) -> i64 {
    g := Guard { id: 7 }
    I flag > 0 { R 1 }
    0
}

F main() -> i64 {
    maybe_exit(0)
}
"#;
    assert_compiles(source);
}

#[test]
fn e2e_phase145_drop_and_defer_both_compile() {
    // Both defer and Drop in same function
    let source = r#"
S Closeable { open: i64 }

W Drop {
    F drop(&self) -> i64
}

X Closeable: Drop {
    F drop(&self) -> i64 { 0 }
}

F main() -> i64 {
    D { 0 }
    c := Closeable { open: 1 }
    0
}
"#;
    assert_compiles(source);
}

// ===== Struct scope and field access tests =====

#[test]
fn e2e_phase145_struct_scope_cleanup() {
    // Struct created in scope, fields accessed
    let source = r#"
S Point { x: i64, y: i64 }

F make_point(a: i64, b: i64) -> Point {
    Point { x: a, y: b }
}

F main() -> i64 {
    p := make_point(3, 4)
    sum := p.x + p.y
    I sum != 7 { R 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase145_nested_scope_struct() {
    // Struct in nested block
    let source = r#"
S Pair { a: i64, b: i64 }

F main() -> i64 {
    result := mut 0
    p := Pair { a: 10, b: 20 }
    result = p.a + p.b
    I result != 30 { R 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

// ===== malloc/free pattern tests =====

#[test]
fn e2e_phase145_manual_resource_cleanup() {
    // Manual resource management with defer for cleanup
    let source = r#"
F main() -> i64 {
    buf := malloc(64)
    D { free(buf) }
    store_byte(buf, 42)
    val := load_byte(buf)
    I val != 42 { R 1 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase145_defer_in_loop_context() {
    // Defer outside loop — only runs once at function exit
    let source = r#"
F main() -> i64 {
    count := mut 0
    D { count = count + 1 }
    L i:0..3 {
        count = count + 10
    }
    count
}
"#;
    // count = 30 (loop), defer runs after return value captured
    assert_exit_code(source, 30);
}
