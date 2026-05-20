use super::helpers::*;

#[test]
fn e2e_phase191_enum_str_payload_mask_literal_vs_heap() {
    let source = r#"
enum Cell {
    Text(str),
    Num(i64),
}

fn enum_mask(cell: Cell) -> i64 {
    ptr := cell as i64
    load_i64(ptr + 16)
}

fn enum_payload_len(cell: Cell) -> i64 {
    ptr := cell as i64
    payload_box := load_i64(ptr + 8)
    load_i64(payload_box + 8)
}

fn main() -> i64 {
    literal := Text("static")
    I enum_mask(literal) != 0 { return 1 }

    a := "heap-"
    b := "owned"
    owned := Text(a + b)
    I enum_mask(owned) != 1 { return 2 }
    I enum_payload_len(owned) != 10 { return 3 }

    return 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_phase191_enum_str_payload_mask_layout_ir() {
    let source = r#"
enum Cell {
    Text(str),
    Num(i64),
}

fn main() -> i64 {
    a := "heap-"
    b := "owned"
    c := Text(a + b)
    match c {
        Text(s) => s.len(),
        Num(n) => n
    }
}
"#;

    let ir = compile_to_ir(source).expect("compile to ir");
    assert!(
        ir.contains("%Cell = type { i32, { i64 }, i64 }"),
        "str-payload enum should carry trailing ownership mask:\n{}",
        ir
    );
    assert!(
        ir.contains("getelementptr %Cell, %Cell*") && ir.contains("i32 0, i32 2"),
        "constructor should touch ownership mask field 2:\n{}",
        ir
    );
    assert!(
        ir.contains("or i64") && ir.contains(", 1"),
        "heap-owned str payload should set mask bit 0:\n{}",
        ir
    );
}

#[test]
fn e2e_phase191_drop_named_payload_moved_through_result_constructor() {
    let source = r#"
impl fn load_byte(ptr: i64) -> i64
impl fn load_i64(ptr: i64) -> i64

enum Owned {
    Ptr(i64),
}

trait Drop {
    fn drop(&self) -> i64
}

impl Owned: Drop {
    fn drop(&self) -> i64 {
        ptr := load_i64((self as i64) + 8)
        I ptr != 0 {
            free(ptr)
        }
        0
    }
}

fn wrap_owned() -> Result<Owned, i64> {
    ptr := malloc(8)
    store_byte(ptr, 42)
    owned := mut Ptr(ptr)
    Ok(owned)
}

fn main() -> i64 {
    result := mut wrap_owned()
    owned := mut match result {
        Ok(v) => v,
        Err(code) => { return code },
    }

    ptr := load_i64((owned as i64) + 8)
    I load_byte(ptr) != 42 { return 2 }
    0
}
"#;

    assert_exit_code(source, 0);
}
