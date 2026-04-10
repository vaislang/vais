//! Phase 184 -- Unambiguous Keywords (EN/EL/LF/LW)
//!
//! Tests for the new deterministic keyword variants that eliminate
//! parser ambiguity between enum/else (E) and loop types (L).

use super::helpers::*;

// ==================== EN (enum) ====================

#[test]
fn e2e_en_keyword_basic_enum() {
    let source = r#"
EN Color { Red, Green, Blue }
F main() -> i64 {
    c := Color.Red
    M c {
        Color.Red => 1,
        Color.Green => 2,
        Color.Blue => 3,
    }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_en_keyword_enum_with_data() {
    let source = r#"
EN Shape {
    Circle(i64),
    Rect(i64, i64),
}
F main() -> i64 {
    s := Shape.Circle(42)
    M s {
        Shape.Circle(r) => r,
        Shape.Rect(w, h) => w + h,
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== EL (else) ====================

#[test]
fn e2e_el_keyword_basic_if_else() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } EL { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_el_keyword_else_if_chain() {
    let source = r#"
F main() -> i64 {
    x := 3
    I x == 1 { 10 }
    EL I x == 2 { 20 }
    EL I x == 3 { 30 }
    EL { 0 }
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_el_keyword_mixed_with_en() {
    // EN for enum, EL for else — no ambiguity
    let source = r#"
EN Option { Some(i64), None }
F main() -> i64 {
    o := Option.Some(42)
    M o {
        Option.Some(v) => {
            I v > 0 { v } EL { 0 }
        },
        Option.None => 0,
    }
}
"#;
    assert_exit_code(source, 42);
}

// ==================== LW (while loop) ====================

#[test]
fn e2e_lw_keyword_basic_while() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    LW x < 10 {
        x = x + 1
    }
    x
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_lw_keyword_while_with_break() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    LW true {
        x = x + 1
        I x == 5 { B }
    }
    x
}
"#;
    assert_exit_code(source, 5);
}

// ==================== LF (for-each loop) ====================

#[test]
fn e2e_lf_keyword_basic_foreach() {
    // NOTE: e2e tests do not load std/vec.vais, so Vec must be defined
    // inline in the test source (same pattern as phase182/advanced tests).
    // The LF keyword itself — not Vec — is what this test verifies.
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

    F len(&self) -> i64 {
        self.len
    }
}

F main() -> i64 {
    sum := mut 0
    v := Vec.new()
    v.push(10)
    v.push(20)
    v.push(12)
    LF i:v {
        sum = sum + i
    }
    sum
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Phase 24 Task 6 — .enumerate() codegen ====================
//
// These tests use an inline Vec<T> definition (same pattern as
// e2e_lf_keyword_basic_foreach above) because e2e tests do not load
// std/vec.vais. Array literals (`[10, 20, 30]`) decay to bare pointers in
// Vais and lose their length, so foreach over them does not iterate — Vec is
// the realistic vehicle for verifying enumerate codegen.

const VEC_INLINE: &str = r#"
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

    F len(&self) -> i64 {
        self.len
    }
}
"#;

#[test]
fn e2e_phase24_vec_enumerate_basic() {
    // .enumerate() over Vec<i64> binds (idx, elem). Verifies pattern::Tuple
    // routing through generate_collection_for_loop and the Task 6 idx/elem
    // alloca + store sequence.
    // Sum: 0*10 + 1*20 + 2*30 = 80
    let source = format!(
        "{}{}",
        VEC_INLINE,
        r#"
F main() -> i64 {
    v := Vec.new()
    v.push(10)
    v.push(20)
    v.push(30)
    sum := mut 0
    LF (i, x): v.enumerate() {
        sum = sum + i * x
    }
    sum
}
"#
    );
    assert_exit_code(&source, 80);
}

#[test]
fn e2e_phase24_vec_iter_enumerate_chain() {
    // .iter().enumerate() chain: codegen unwrap_iter_adapters peels both
    // method calls and treats it as a plain enumerate.
    // Sum: 0*100 + 1*200 = 200
    let source = format!(
        "{}{}",
        VEC_INLINE,
        r#"
F main() -> i64 {
    v := Vec.new()
    v.push(100)
    v.push(200)
    sum := mut 0
    LF (i, x): v.iter().enumerate() {
        sum = sum + i * x
    }
    sum
}
"#
    );
    assert_exit_code(&source, 200);
}

#[test]
fn e2e_phase24_vec_iter_noop() {
    // .iter() is a codegen no-op — Pattern::Ident binding works as if
    // .iter() weren't there.
    // Sum: 1+2+3+4+5 = 15
    let source = format!(
        "{}{}",
        VEC_INLINE,
        r#"
F main() -> i64 {
    v := Vec.new()
    v.push(1)
    v.push(2)
    v.push(3)
    v.push(4)
    v.push(5)
    sum := mut 0
    LF x: v.iter() {
        sum = sum + x
    }
    sum
}
"#
    );
    assert_exit_code(&source, 15);
}

#[test]
fn e2e_phase24_vec_enumerate_index_only() {
    // Use the index binding alone in arithmetic to verify it's an i64
    // alloca and live throughout the body.
    // Sum: 0+1+2+3 = 6
    let source = format!(
        "{}{}",
        VEC_INLINE,
        r#"
F main() -> i64 {
    v := Vec.new()
    v.push(99)
    v.push(99)
    v.push(99)
    v.push(99)
    acc := mut 0
    LF (i, _): v.enumerate() {
        acc = acc + i
    }
    acc
}
"#
    );
    assert_exit_code(&source, 6);
}

// ==================== Backward Compatibility ====================
// Old E/L keywords should still work

#[test]
fn e2e_backward_compat_old_enum() {
    let source = r#"
E Direction { Up, Down }
F main() -> i64 {
    d := Direction.Up
    M d {
        Direction.Up => 1,
        Direction.Down => 2,
    }
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_backward_compat_old_else() {
    let source = r#"
F main() -> i64 {
    x := 10
    I x > 5 { 42 } E { 0 }
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_backward_compat_old_while_loop() {
    let source = r#"
F main() -> i64 {
    x := mut 0
    L x < 10 {
        x = x + 1
    }
    x
}
"#;
    assert_exit_code(source, 10);
}
