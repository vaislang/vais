//! Phase 191 #2a' sandbox: Vec<str> push + drop with ownership transfer.
//!
//! Scaffolding to validate:
//! - Vec_push$str compiles
//! - Owned bitmap helpers `__vais_vec_str_owned_ensure/set/shallow_free` emit
//! - Heap-owned strings (concat results) pushed into Vec<str> are tracked
//! - `v.drop()` frees heap buffers via `__vais_vec_str_shallow_free`
//!
//! Uses local `S Vec<T>` mirroring std/vec's 5-field layout so the TC's
//! static-method type-inference bug (#10 scope-reduced residual) doesn't
//! block us.

use super::helpers::*;

/// Basic literal push — no heap strings, ensures codegen path doesn't break
/// when the owned bitmap is never populated.
#[test]
fn e2e_phase191_vec_str_push_literal_only() {
    assert_exit_code(
        r#"
S Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

X Vec<T> {
    F grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } EL { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    F push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    F drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

F main() -> i64 {
    v: Vec<str> := Vec { data: 0, len: 0, cap: 0, elem_size: 16, owned: 0 }
    v.push("hi")
    v.push("bye")
    v.drop()
    0
}
"#,
        0,
    );
}

/// Heap-owned push: concat results inside a loop. Before the fix, each
/// iteration would leak one heap string buffer. The shallow-free prelude
/// runs at `v.drop()` and walks the owned bitmap to free all tracked
/// elements. 10k iterations × ~30 bytes = ~300 KB — a leak would be
/// measurable but not OOM; the test asserts exit code only, so regressions
/// show up via `leaks --atExit` during manual verification.
#[test]
fn e2e_phase191_vec_str_push_concat_drop() {
    assert_exit_code(
        r#"
S Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

X Vec<T> {
    F grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } EL { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    F push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    F drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

F main() -> i64 {
    v: Vec<str> := Vec { data: 0, len: 0, cap: 0, elem_size: 16, owned: 0 }
    i := mut 0
    L i < 100 {
        a := "hello-"
        b := "world"
        s := a + b
        v.push(s)
        i = i + 1
    }
    v.drop()
    0
}
"#,
        0,
    );
}

/// Mixed literal + heap-owned in the same Vec. The owned bitmap should
/// precisely track which slots need freeing: heap-owned bit set → free;
/// literal bit unset → leave as-is (the static string lives in .rodata).
#[test]
fn e2e_phase191_vec_str_push_mixed_literal_heap() {
    assert_exit_code(
        r#"
S Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

X Vec<T> {
    F grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } EL { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    F push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    F drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

F main() -> i64 {
    v: Vec<str> := Vec { data: 0, len: 0, cap: 0, elem_size: 16, owned: 0 }
    v.push("literal-a")
    owned1 := "heap-" + "one"
    v.push(owned1)
    v.push("literal-b")
    owned2 := "heap-" + "two"
    v.push(owned2)
    v.drop()
    0
}
"#,
        0,
    );
}
