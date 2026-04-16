use super::helpers::*;

/// (3) struct_str_field_drop: A struct with a str field that receives a
/// concat result (heap-owned). When the struct goes out of scope, the
/// shallow-drop helper reads the ownership_mask, finds the bit set for
/// the str field, extracts the raw pointer, and frees it. No leak.
#[test]
fn e2e_phase191_struct_str_field_drop() {
    assert_exit_code(
        r#"
S Person {
    name: str,
    age: i64
}

F main() -> i64 {
    a := "hello-"
    b := "world"
    p := Person { name: a + b, age: 42 }
    0
}
"#,
        0,
    );
}

/// (4) struct_user_drop_takes_ownership: A struct with a user-defined Drop
/// trait. The user drop runs first (domain logic), then the shallow-drop
/// helper runs and frees the heap-owned str field. No double-free because
/// user code has no `free` API for str fields.
#[test]
fn e2e_phase191_struct_user_drop() {
    assert_exit_code(
        r#"
S Resource {
    label: str,
    handle: i64
}

W Drop {
    F drop(&self) -> i64
}

X Resource: Drop {
    F drop(&self) -> i64 {
        self.handle = 0
        0
    }
}

F main() -> i64 {
    r := Resource { label: "prefix-" + "suffix", handle: 99 }
    0
}
"#,
        0,
    );
}

/// (5) struct_literal_str_no_free: When a str field is a literal (not a
/// heap-owned concat result), the ownership_mask bit stays 0 and the
/// shallow-drop does NOT call free on it. This verifies that we don't
/// accidentally free .rodata string constants.
#[test]
fn e2e_phase191_struct_literal_str_no_free() {
    assert_exit_code(
        r#"
S Tag {
    name: str
}

F main() -> i64 {
    t := Tag { name: "static-literal" }
    0
}
"#,
        0,
    );
}

/// Loop stress: create 1000 structs with heap-owned str fields in a loop.
/// Each iteration's struct goes out of scope and the shallow-drop frees
/// the concat buffer. Without the ownership transfer, this would leak
/// ~1000 heap allocations.
#[test]
fn e2e_phase191_struct_str_loop_no_leak() {
    assert_exit_code(
        r#"
S Item {
    value: str
}

F main() -> i64 {
    i := mut 0
    L i < 1000 {
        item := Item { value: "a" + "b" }
        i = i + 1
    }
    0
}
"#,
        0,
    );
}
