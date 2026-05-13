//! Phase 4b.1 / Task #7: Implicit error propagation (`--implicit-try`).
//!
//! These tests cover the opt-in mode that lets a call-site argument of type
//! `Result<T, E>` (or `Option<T>`) be auto-unwrapped when passed to a `T`
//! parameter, as if the programmer had written `?` on the argument. The
//! feature reuses the existing `Expr::Try` codegen path via an argument
//! span marker recorded by the type checker.
//!
//! E2E note: the helper uses concrete `E Result { Ok(i64), Err(i64) }`
//! declarations rather than built-in generics because the test runner
//! compiles single-file sources without stdlib, matching the existing
//! `modules_system::test_result_generic_*` pattern.
//!
//! The tests exercise:
//! - Happy path (Ok/Some propagation)
//! - Error path (Err/None propagation through the enclosing function)
//! - Interop with explicit `?` (no double-unwrap)
//! - Negative case (enclosing fn must return a compatible container)
//! - Baseline: without `--implicit-try` the same programs are type errors
//!   (protects the default semantics from accidental change).

use crate::helpers::{
    assert_compile_error, assert_compile_error_implicit_try, assert_exit_code_implicit_try,
};

// ==================== Happy path ====================

/// Result auto-unwrapped when passed to i64 parameter.
/// With `--implicit-try`, add_one(parse_num()) is equivalent to
/// add_one(parse_num()?).
#[test]
fn implicit_try_result_ok_path() {
    let source = r#"
E Result { Ok(i64), Err(i64) }

F parse_num() -> Result {
    R Ok(42)
}

F add_one(x: i64) -> i64 {
    R x + 1
}

F double() -> Result {
    R Ok(add_one(parse_num()))
}

F main() -> i64 {
    M double() {
        Ok(v) => v,
        Err(_) => 1
    }
}
"#;
    assert_exit_code_implicit_try(source, 43);
}

/// Result Err case: the enclosing `double` must propagate the Err upward
/// so main's match picks the Err arm.
#[test]
fn implicit_try_result_err_propagates() {
    let source = r#"
E Result { Ok(i64), Err(i64) }

F parse_num() -> Result {
    R Err(7)
}

F add_one(x: i64) -> i64 {
    R x + 1
}

F double() -> Result {
    R Ok(add_one(parse_num()))
}

F main() -> i64 {
    M double() {
        Ok(_) => 0,
        Err(e) => e + 90
    }
}
"#;
    assert_exit_code_implicit_try(source, 97);
}

/// Option version of the happy path. Auto-unwrap Some → inner.
#[test]
fn implicit_try_option_some_path() {
    let source = r#"
E Option { Some(i64), None }

F find_val() -> Option {
    R Some(7)
}

F square(x: i64) -> i64 {
    R x * x
}

F caller() -> Option {
    R Some(square(find_val()))
}

F main() -> i64 {
    M caller() {
        Some(v) => v,
        None => 1
    }
}
"#;
    assert_exit_code_implicit_try(source, 49);
}

/// Option None case: enclosing Option caller returns None on propagation;
/// main's match picks the None arm.
#[test]
fn implicit_try_option_none_propagates() {
    let source = r#"
E Option { Some(i64), None }

F find_val() -> Option {
    R None
}

F square(x: i64) -> i64 {
    R x * x
}

F caller() -> Option {
    R Some(square(find_val()))
}

F main() -> i64 {
    M caller() {
        Some(v) => v,
        None => 55
    }
}
"#;
    assert_exit_code_implicit_try(source, 55);
}

// ==================== Interop with explicit `?` ====================

/// Explicit `?` on the argument still works; the implicit pass must not
/// double-wrap a site that already matches `Expr::Try`.
#[test]
fn implicit_try_does_not_double_wrap_explicit_try() {
    let source = r#"
E Result { Ok(i64), Err(i64) }

F parse_num() -> Result {
    R Ok(10)
}

F add_one(x: i64) -> i64 {
    R x + 1
}

F double() -> Result {
    R Ok(add_one(parse_num()?))
}

F main() -> i64 {
    M double() {
        Ok(v) => v,
        Err(_) => 1
    }
}
"#;
    assert_exit_code_implicit_try(source, 11);
}

// ==================== Negative cases ====================

/// The feature requires the enclosing function to return a matching
/// container. A plain `i64` return type must be rejected — silent
/// propagation would be unsound.
#[test]
fn implicit_try_rejects_plain_return_type() {
    let source = r#"
E Result { Ok(i64), Err(i64) }

F parse_num() -> Result {
    R Ok(42)
}

F add_one(x: i64) -> i64 {
    R x + 1
}

F main() -> i64 {
    R add_one(parse_num())
}
"#;
    assert_compile_error_implicit_try(source);
}

// ==================== Baseline: no flag, no transformation ====================

/// Without `--implicit-try`, passing Result to an i64 parameter is a
/// type error. This locks in the default semantics so a future edit
/// cannot accidentally enable auto-unwrap for everyone.
#[test]
fn implicit_try_off_rejects_result_to_i64_param() {
    let source = r#"
E Result { Ok(i64), Err(i64) }

F parse_num() -> Result {
    R Ok(42)
}

F add_one(x: i64) -> i64 {
    R x + 1
}

F double() -> Result {
    R Ok(add_one(parse_num()))
}

F main() -> i64 {
    M double() {
        Ok(v) => v,
        Err(_) => 1
    }
}
"#;
    assert_compile_error(source);
}
