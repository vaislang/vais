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
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

fn main() -> i64 {
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
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

fn main() -> i64 {
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
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

fn main() -> i64 {
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

/// Read back pushed strings through `load_typed` and send them through the
/// normal str ABI path. This is the shape that role-bearing JWT generation
/// needs before it can be promoted.
#[test]
fn e2e_phase191_vec_str_push_get_print_compare() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }

    fn drop(&self) -> i64 { free(self.data); self.data = 0; 0 }
}

fn main() -> i64 {
    v: Vec<str> := Vec { data: 0, len: 0, cap: 0, elem_size: 16, owned: 0 }
    v.push("admin")
    v.push("user")
    first := v.get(0)
    second := v.get(1)
    println(first)
    println(second)
    I first != "admin" { return 1 }
    I second != "user" { return 2 }
    v.drop()
    0
}
"#,
        "admin\nuser",
    );
}

/// Passing `Vec<str>` by value must preserve both the Vec header and the
/// concrete `str` element specialization used by method calls inside callee.
#[test]
fn e2e_phase191_vec_str_by_value_param_get() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn with_capacity(capacity: i64) -> Vec<T> {
        es := type_size()
        data := malloc(capacity * es)
        Vec { data: data, len: 0, cap: capacity, elem_size: es, owned: 0 }
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }
}

fn first_role(roles: Vec<str>) -> str {
    roles.get(0)
}

fn main() -> i64 {
    roles: Vec<str> := mut Vec.with_capacity(2)
    roles.push("admin")
    roles.push("user")
    first := first_role(roles)
    println(first)
    I first != "admin" { return 1 }
    0
}
"#,
        "admin",
    );
}

/// A `Vec<str>` field passed into a by-value parameter must be loaded as the
/// full Vec header, not as an address or partially-copied aggregate.
#[test]
fn e2e_phase191_vec_str_struct_field_to_param_get() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

struct Claims {
    roles: Vec<str>
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn with_capacity(capacity: i64) -> Vec<T> {
        es := type_size()
        data := malloc(capacity * es)
        Vec { data: data, len: 0, cap: capacity, elem_size: es, owned: 0 }
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }
}

fn first_role(roles: Vec<str>) -> str {
    roles.get(0)
}

fn main() -> i64 {
    roles: Vec<str> := mut Vec.with_capacity(2)
    roles.push("admin")
    roles.push("user")
    claims := Claims { roles: roles }
    first := first_role(claims.roles)
    println(first)
    I first != "admin" { return 1 }
    0
}
"#,
        "admin",
    );
}

#[test]
fn e2e_phase191_recursive_str_param_return() {
    assert_stdout_contains(
        r#"
fn keep(s: str, i: i64) -> str {
    I i >= 1 { return s }
    keep(s, i + 1)
}

fn main() -> i64 {
    out := keep("admin", 0)
    println(out)
    I out != "admin" { return 1 }
    0
}
"#,
        "admin",
    );
}

#[test]
fn e2e_phase191_vec_str_recursive_param_get() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn with_capacity(capacity: i64) -> Vec<T> {
        es := type_size()
        data := malloc(capacity * es)
        Vec { data: data, len: 0, cap: capacity, elem_size: es, owned: 0 }
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }
}

fn get_at(roles: Vec<str>, i: i64) -> str {
    I i >= 1 { return roles.get(1) }
    get_at(roles, i + 1)
}

fn main() -> i64 {
    roles: Vec<str> := mut Vec.with_capacity(2)
    roles.push("admin")
    roles.push("user")
    value := get_at(roles, 0)
    println(value)
    I value != "user" { return 1 }
    0
}
"#,
        "user",
    );
}

#[test]
fn e2e_phase191_vec_str_concat_loaded_elements() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn with_capacity(capacity: i64) -> Vec<T> {
        es := type_size()
        data := malloc(capacity * es)
        Vec { data: data, len: 0, cap: capacity, elem_size: es, owned: 0 }
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }
}

fn combine(roles: Vec<str>) -> str {
    first := roles.get(0)
    second := roles.get(1)
    first + "," + second
}

fn main() -> i64 {
    roles: Vec<str> := mut Vec.with_capacity(2)
    roles.push("admin")
    roles.push("user")
    out := combine(roles)
    println(out)
    I out != "admin,user" { return 1 }
    0
}
"#,
        "admin,user",
    );
}

#[test]
fn e2e_phase191_tail_if_expr_str_return() {
    assert_stdout_contains(
        r#"
fn choose(c: bool) -> str {
    I c { "admin" } else { "user" }
}

fn main() -> i64 {
    out := choose(true)
    println(out)
    I out != "admin" { return 1 }
    0
}
"#,
        "admin",
    );
}

#[test]
fn e2e_phase191_recursive_heap_str_tail_return() {
    assert_stdout_contains(
        r#"
fn build(s: str, i: i64) -> str {
    I i >= 1 { return s + "!" }
    build(s, i + 1)
}

fn main() -> i64 {
    out := build("admin", 0)
    println(out)
    I out != "admin!" { return 1 }
    0
}
"#,
        "admin!",
    );
}

#[test]
fn e2e_phase191_tail_if_recursive_heap_str_return() {
    assert_stdout_contains(
        r#"
fn build(s: str, i: i64) -> str {
    I i >= 1 {
        s + "!"
    } else {
        build(s, i + 1)
    }
}

fn main() -> i64 {
    out := build("admin", 0)
    println(out)
    I out != "admin!" { return 1 }
    0
}
"#,
        "admin!",
    );
}

#[test]
fn e2e_phase191_heap_str_call_arg_return() {
    assert_stdout_contains(
        r#"
fn id(s: str) -> str {
    s
}

fn main() -> i64 {
    out := id("admin" + "," + "user")
    println(out)
    I out != "admin,user" { return 1 }
    0
}
"#,
        "admin,user",
    );
}

#[test]
fn e2e_phase191_mut_str_assign_from_vec_get() {
    assert_stdout_contains(
        r#"
struct Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

impl Vec<T> {
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } else { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    fn with_capacity(capacity: i64) -> Vec<T> {
        es := type_size()
        data := malloc(capacity * es)
        Vec { data: data, len: 0, cap: capacity, elem_size: es, owned: 0 }
    }

    fn push(&self, value: T) -> i64 {
        I self.len >= self.cap { @.grow() }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    fn get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load_typed(ptr)
    }
}

fn main() -> i64 {
    roles: Vec<str> := mut Vec.with_capacity(1)
    roles.push("admin")
    out := mut ""
    out = roles.get(0)
    println(out)
    I out != "admin" { return 1 }
    0
}
"#,
        "admin",
    );
}

#[test]
fn e2e_phase191_mut_str_assign_concat_return() {
    assert_stdout_contains(
        r#"
fn build() -> str {
    out := mut "admin"
    out = out + "," + "user"
    out
}

fn main() -> i64 {
    value := build()
    println(value)
    I value != "admin,user" { return 1 }
    0
}
"#,
        "admin,user",
    );
}

#[test]
fn e2e_phase191_mut_str_loop_assign_concat_return() {
    assert_stdout_contains(
        r#"
fn build() -> str {
    out := mut ""
    i := mut 0
    L i < 2 {
        I i == 0 {
            out = "admin"
        } else {
            out = out + "," + "user"
        }
        i = i + 1
    }
    out
}

fn main() -> i64 {
    value := build()
    println(value)
    I value != "admin,user" { return 1 }
    0
}
"#,
        "admin,user",
    );
}
