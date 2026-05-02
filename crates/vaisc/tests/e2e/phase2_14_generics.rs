//! Phase 2.14 — Generic instantiation 완전성 e2e tests.
//!
//! Covers cases that were listed as "method inference dispersion" in
//! TYPE_SYSTEM.md §9. Tests check type-check level (vaisc check). If a
//! specific case also requires codegen, that's covered by the stronger
//! `assert_exit_code` variant.
//!
//! All tests use the helpers::check_only path so we don't trip codegen
//! gaps that are the domain of Phase 3.x.

fn check_only(source: &str) {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("p2_14.vais");
    std::fs::write(&path, source).expect("write");
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let out = std::process::Command::new(vaisc)
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn vaisc");
    assert!(
        out.status.success(),
        "vaisc check failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn generic_fn_single_param() {
    check_only(
        r#"
F id<T>(x: T) -> T { x }
F main() -> i64 { id(42) }
"#,
    );
}

#[test]
fn generic_fn_multi_param() {
    check_only(
        r#"
F pick_first<A, B>(a: A, _b: B) -> A { a }
F main() -> i64 { pick_first(10, 20) }
"#,
    );
}

#[test]
fn generic_struct_with_method() {
    check_only(
        r#"
S Wrapper<T> { val: T }
X Wrapper<T> {
    F get(self) -> T { self.val }
}
F main() -> i64 {
    w := Wrapper { val: 7 }
    w.get()
}
"#,
    );
}

#[test]
fn nested_generic_option_vec() {
    // Option<Vec<i64>> should typecheck.
    check_only(
        r#"
F wrap(v: Vec<i64>) -> Option<Vec<i64>> { Some(v) }

F main() -> i64 {
    v: Vec<i64> = Vec::new()
    v.push(5)
    w := wrap(v)
    M w {
        Some(_) => 1,
        None => 0
    }
}
"#,
    );
}

#[test]
fn where_clause_single_bound() {
    // where clause with a trait bound. User-defined trait avoids reliance
    // on specific built-in trait semantics.
    check_only(
        r#"
W Show { F show(self) -> i64 }

S Thing { id: i64 }

X Thing: Show {
    F show(self) -> i64 { self.id }
}

F describe<T: Show>(x: T) -> i64 {
    x.show()
}

F main() -> i64 {
    t := Thing { id: 7 }
    describe(t)
}
"#,
    );
}
