//! Phase 190 #5: Future wrapping for generic async function calls.
//!
//! Prior to the fix, `infer_expr_type` for a `Call` expression whose
//! resolution went through `resolve_generic_call` (generic-instantiated)
//! returned the bare `ret` type and skipped the `is_async → Future<T>`
//! wrapping. That meant `.await` on the specialized call landed in the
//! "await on non-Future" fallback (now a warning, previously an ICE).
//!
//! These tests assert generic async functions compile cleanly.

use super::helpers::*;

#[test]
fn e2e_phase190_generic_async_await_compiles() {
    assert_compiles(
        r#"
A F fetch<T>(x: T) -> T {
  x
}

A F main() -> i64 {
  v := fetch(42).await
  v
}
"#,
    );
}

#[test]
fn e2e_phase190_plain_async_await_compiles() {
    assert_compiles(
        r#"
A F double(n: i64) -> i64 {
  n + n
}

A F main() -> i64 {
  double(21).await
}
"#,
    );
}
