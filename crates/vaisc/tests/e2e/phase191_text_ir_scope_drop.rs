//! Phase 191 #5: Text-IR backend block-scope drop parity with inkwell.
//!
//! Phase 190.5/190.6 added return-transfer, intermediate-free, let-var, and
//! PHI-merge drop paths to the text-IR backend. Loop-body concat leftovers
//! (intermediate strings that are neither returned nor bound to a let) still
//! leaked until function exit, so a long-running loop exhausted memory.
//!
//! The inkwell backend fixed this with a block-scoped `scope_str_stack`:
//! each block frame tracks the concat slots allocated within it, the last
//! expression's slot transfers to the enclosing scope, and the rest are freed
//! at block exit. Phase 191 #5 ports that stack to the text-IR backend.
//!
//! These tests exercise program behaviour, not IR text — a per-iteration leak
//! would either crash (OOM) at a large iteration count or produce wrong output
//! when ownership handoff fails.
//!
//! See `docs/rfcs/RFC-001-string-ownership.md` §4.2 + §5.4.

use super::helpers::*;

/// Heavy loop: 100_000 iterations each allocating and discarding a concat
/// result. Before Phase 191 #5, the text-IR backend leaked one buffer per
/// iteration (~40 bytes each), which is too small to OOM inside a test but
/// enough to regress if the new scope-drop path ever null-dereferences or
/// double-frees. A double-free or UAF would SIGABRT on libc's malloc guard.
#[test]
fn e2e_phase191_loop_body_concat_no_leak() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100000 {
    a := "abcdefghij"
    b := "klmnopqrst"
    msg := a + b
    i = i + 1
  }
  0
}
"#,
        0,
    );
}

/// Nested block where the inner block's concat result is the block's value.
/// Ownership must transfer from the inner frame to the outer frame — the
/// inner scope MUST NOT free the slot (that would be a UAF), and the outer
/// scope (or the function-exit cleanup, as a backstop) must free it exactly
/// once. If either direction regresses, the final println would print empty
/// or crash.
#[test]
fn e2e_phase191_nested_block_string_transfer() {
    assert_stdout_contains(
        r#"
F main() -> i64 {
  s := {
    inner := "hello " + "world"
    inner
  }
  println(s)
  0
}
"#,
        "hello world",
    );
}

/// Loop body followed by a post-loop concat using the same operand names —
/// catches bugs where the loop iteration's scope drop leaves stale entries
/// in `string_value_slot` / `alloc_tracker` that confuse the next concat.
#[test]
fn e2e_phase191_loop_then_concat_still_correct() {
    assert_stdout_contains(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100 {
    a := "x"
    b := "y"
    _msg := a + b
    i = i + 1
  }
  p := "done-"
  q := "ok"
  println(p + q)
  0
}
"#,
        "done-ok",
    );
}

/// If-expression whose branches return concat results bound to a let inside
/// the branch — the outer let binding must inherit ownership via the PHI
/// path AND the branches' inner scope must transfer, not free. This exercises
/// the interaction of Phase 190.6 (PHI merge) with Phase 191 #5 (scope drop).
#[test]
fn e2e_phase191_if_with_nested_let_binding_true() {
    assert_stdout_contains(
        r#"
F build(c: bool) -> str {
  msg := I c {
    s := "aa" + "bb"
    s
  } E {
    s := "cc" + "dd"
    s
  }
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
fn e2e_phase191_if_with_nested_let_binding_false() {
    assert_stdout_contains(
        r#"
F build(c: bool) -> str {
  msg := I c {
    s := "aa" + "bb"
    s
  } E {
    s := "cc" + "dd"
    s
  }
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

/// Phase 191 #6 — break path frees loop-inner scope frames.
/// Before the fix, concat results produced in the loop body before a `break`
/// leaked until function exit (alloc_tracker held them but with no slot null
/// stored, the fn-exit cleanup would still free them — so the visible failure
/// was not a leak at `main` exit but a double-free / UAF when the same slot
/// id got reused in a later iteration after re-entry. In this synthetic test,
/// the loop breaks on the first iteration after one concat, so the guarantee
/// is simpler: the heap buffer must be freed at break (otherwise linear-growth
/// regressions in later `L` bodies would resurface).
#[test]
fn e2e_phase191_break_frees_scope_strings() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100000 {
    a := "abcdefghij"
    b := "klmnopqrst"
    _msg := a + b
    i = i + 1
    I i > 99990 {
      B
    }
  }
  0
}
"#,
        0,
    );
}

/// Phase 191 #6 — continue path frees loop-inner scope frames.
/// Without the fix, `continue` skipped block-exit cleanup entirely, so a
/// `L { concat; I cond { C }; rest }` body leaks one buffer per hit on the
/// fast path. 100k iterations × ~40 bytes = 4 MB, still too small to OOM in
/// CI but enough to verify no double-free / UAF regression.
#[test]
fn e2e_phase191_continue_frees_scope_strings() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  j := mut 0
  L i < 100000 {
    a := "abcdefghij"
    b := "klmnopqrst"
    _msg := a + b
    i = i + 1
    I i % 2 == 0 {
      C
    }
    j = j + 1
  }
  0
}
"#,
        0,
    );
}

/// Regression guard for the Ident fallback in transfer_slot lookup.
/// The inner block's tail expression is a bare Ident (`s`) referring to a
/// heap-owning local. Without the var_string_slot fallback, a future change
/// making Str alloca-backed would produce a fresh load SSA not found in
/// string_value_slot, causing the inner scope to free the buffer while the
/// outer binding still holds a pointer to it (UAF). Today this green-path
/// already passes via SSA coincidence; the test locks in correct behaviour.
#[test]
fn transfer_slot_ident_fallback_no_uaf() {
    assert_stdout_contains(
        r#"
F main() -> i64 {
  s := {
    a := "hello-"
    b := "world"
    c := a + b
    c
  }
  println(s)
  0
}
"#,
        "hello-world",
    );
}

/// Guards substring's scope_str_stack path: each iteration allocates a heap
/// buffer via __vais_str_substring; block-exit must free it or a later
/// double-free / UAF will SIGABRT under libc malloc guards.
#[test]
fn e2e_phase191_loop_body_substring_no_leak() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100000 {
    s := "abcdefghij"
    _sub := s.substring(2, 7)
    i = i + 1
  }
  0
}
"#,
        0,
    );
}

/// Guards push_str's scope_str_stack path: each iteration allocates a concat
/// buffer via push_str; the per-iteration slot must be freed at block exit.
#[test]
fn e2e_phase191_loop_body_push_str_no_leak() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100000 {
    _s := "hello".push_str("world")
    i = i + 1
  }
  0
}
"#,
        0,
    );
}

/// Guards PHI-merge + scope-drop interaction through a match expression.
/// Phase 191 #9 fixed the text-IR backend's mixed-type PHI by unifying the
/// arm types to `{ i8*, i64 }` (fat pointer) and adding ownership transfer
/// for any tracked alloc_slot, mirroring the if-expr PHI handling at
/// expr_helpers_control.rs:344-371.
#[test]
fn e2e_phase191_match_arm_concat_phi() {
    assert_stdout_contains(
        r#"
F build(n: i64) -> str {
  msg := M n {
    1 => {
      s := "aa" + "bb"
      s
    },
    _ => {
      s := "cc" + "dd"
      s
    }
  }
  R msg
}

F main() -> i64 {
  println(build(1))
  println(build(2))
  0
}
"#,
        "aabb",
    );
}

/// Guards post-Phase-191-#6 break cleanup for a loop body with two concat
/// sites: the first concat's slot must be freed by the break path before
/// branching out; the second concat (after the break guard) must be freed
/// normally on loop-body exit for the non-breaking iterations.
#[test]
fn e2e_phase191_break_before_concat_no_leak() {
    assert_exit_code(
        r#"
F main() -> i64 {
  i := mut 0
  L i < 100000 {
    _first := "abcde" + "fghij"
    i = i + 1
    I i > 99999 {
      B
    }
    _second := "klmno" + "pqrst"
  }
  0
}
"#,
        0,
    );
}
