//! Phase 190.5 / 190.6: string concatenation drop-tracking.
//!
//! Background: string concat (`a + b`) malloc's a new buffer and returns a
//! fat pointer. The codegen must free concat intermediates safely without
//! double-free or use-after-free. Three failure modes that these tests pin:
//!
//!   1. Loop-body concat leak — `L i < N { msg := a + b }` used to leak one
//!      buffer per iteration, exhausting memory in long-running servers.
//!   2. let-bound return UAF — `let msg = a+b; return msg` used to free
//!      `msg`'s backing buffer before the caller could read it, because the
//!      load-from-alloca produces a new SSA that didn't match the concat's
//!      original SSA in the ownership map. Caught by team-review 2026-04-14.
//!   3. If/match-expression binding UAF — `let msg = I c { a+b } E { c+d }`
//!      produces a PHI, both incoming branches own a slot, and the PHI
//!      consumer must inherit ownership of both. Same family as #2.
//!
//! See `docs/rfcs/RFC-001-string-ownership.md` for the full ownership model.

use super::helpers::*;

/// Direct `return a + b` — the simplest case: SSA match at return.
#[test]
fn e2e_phase190_direct_return_concat() {
    assert_stdout_contains(
        r#"
F build() -> str {
  R "hello " + "world"
}

F main() -> i64 {
  println(build())
  0
}
"#,
        "hello world",
    );
}

/// `let msg = a+b; return msg` — the UAF case team-review caught.
#[test]
fn e2e_phase190_let_bound_return_preserves_buffer() {
    assert_stdout_contains(
        r#"
F build() -> str {
  msg := "hello " + "world"
  R msg
}

F main() -> i64 {
  println(build())
  0
}
"#,
        "hello world",
    );
}

/// Returned string used twice by the caller — exposes UAF because the callee's
/// freed buffer is silently overwritten by the next allocation, and the
/// second read prints empty.
#[test]
fn e2e_phase190_callee_return_caller_uses_twice() {
    assert_stdout_contains(
        r#"
F build() -> str {
  msg := "hello " + "world"
  R msg
}

F main() -> i64 {
  s := build()
  println(s)
  println(s)
  0
}
"#,
        "hello world\nhello world",
    );
}

/// PHI case: `let msg = if c { a+b } else { c+d }; return msg`. Both incoming
/// branches own a slot; the let binding must inherit ownership of both so
/// neither is prematurely freed.
#[test]
fn e2e_phase190_if_expr_return_true_branch() {
    assert_stdout_contains(
        r#"
F build(c: bool) -> str {
  msg := I c { "aa" + "bb" } E { "cc" + "dd" }
  R msg
}

F main() -> i64 {
  println(build(true))
  0
}
"#,
        "aabb",
    );
}

#[test]
fn e2e_phase190_if_expr_return_false_branch() {
    assert_stdout_contains(
        r#"
F build(c: bool) -> str {
  msg := I c { "aa" + "bb" } E { "cc" + "dd" }
  R msg
}

F main() -> i64 {
  println(build(false))
  0
}
"#,
        "ccdd",
    );
}

/// Concat chain `a + b + c + d` — intermediate buffers must be freed as the
/// chain is consumed. Correctness-wise, the final string must still be intact.
#[test]
fn e2e_phase190_concat_chain_correct_result() {
    assert_stdout_contains(
        r#"
F main() -> i64 {
  a := "one-"
  b := "two-"
  c := "three-"
  d := "four"
  println(a + b + c + d)
  0
}
"#,
        "one-two-three-four",
    );
}

/// Small-scale loop concat: verifies crash-free execution (not a leak gate —
/// leak is validated by the external `leaks --atExit` harness in RFC §9,
/// which is platform-specific). 10 000 iterations is enough to crash if any
/// per-iteration UAF/double-free slipped through.
#[test]
fn e2e_phase190_loop_concat_crash_free() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 10000 {
    a := "abcdefghij"
    b := "klmnopqrst"
    c := "uvwxyz0123"
    d := "4567890---"
    msg := a + b + c + d
    i = i + 1
  }
  0
}
"#,
        0,
    );
}

/// push_str chain with let binding: exercises push_str's intermediate-free
/// path combined with let-bound return.
#[test]
fn e2e_phase190_push_str_let_return() {
    assert_stdout_contains(
        r#"
F build() -> str {
  s := "abc".push_str("def")
  R s
}

F main() -> i64 {
  println(build())
  0
}
"#,
        "abcdef",
    );
}
