use super::helpers::compile_and_run;

#[test]
fn e2e_result_f32_ok_payload_binding_preserves_float_bits() {
    let source = r#"
enum Result {
    Ok(f32),
    Err(i64),
}

fn make_value(flag: bool) -> Result {
    I flag {
        Ok(3.5 as f32)
    } else {
        Err(9)
    }
}

fn abs_f32(x: f32) -> f32 {
    I x < (0.0 as f32) {
        (0.0 as f32) - x
    } else {
        x
    }
}

fn approx_eq_f32(actual: f32, expected: f32, epsilon: f32) -> bool {
    abs_f32(actual - expected) < epsilon
}

fn main() -> i64 {
    v: f32 := mut match make_value(true) {
        Ok(x) => x,
        Err(_) => { return 1 },
    }
    I !approx_eq_f32(v, 3.5 as f32, 0.0001 as f32) { return 2 }

    e: i64 := mut match make_value(false) {
        Ok(_) => { return 3 },
        Err(code) => code,
    }
    I e != 9 { return 4 }
    return 0
}
"#;

    let result =
        compile_and_run(source).expect("Result Ok(f32) payload binding should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "stdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
}

#[test]
fn e2e_result_tuple_struct_payload_field_access_preserves_types() {
    let source = r#"
struct Meta {
    create: u64,
    expire: u64,
}

struct Ptr {
    page_id: u32,
    total_len: u32,
}

fn make_pair(flag: bool) -> Result<(Meta, Ptr), i64> {
    I flag {
        meta := mut Meta { create: 7, expire: 0 }
        ptr := mut Ptr { page_id: 42 as u32, total_len: 88 as u32 }
        Ok((meta, ptr))
    } else {
        Err(9)
    }
}

fn main() -> i64 {
    pair := mut match make_pair(true) {
        Ok(v) => v,
        Err(_) => { return 1 },
    }

    ptr := mut pair.1
    I ptr.page_id != 42 as u32 { return 2 }
    I ptr.total_len != 88 as u32 { return 3 }

    meta := mut pair.0
    I meta.create != 7 { return 4 }
    I meta.expire != 0 { return 5 }

    return 0
}
"#;

    let result = compile_and_run(source)
        .expect("Tuple-struct Result payload field access should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "stdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
}

#[test]
fn e2e_result_match_returning_arm_does_not_choose_phi_type() {
    let source = r#"
fn maybe_flag(flag: bool) -> Result<bool, i64> {
    I flag {
        Ok(true)
    } else {
        Err(7)
    }
}

fn pass(flag: bool) -> Result<i64, i64> {
    opened := mut match maybe_flag(flag) {
        Ok(v) => v,
        Err(e) => { return Err(e) },
    }
    I opened {
        Ok(42)
    } else {
        Ok(1)
    }
}

fn main() -> i64 {
    value := mut match pass(true) {
        Ok(v) => v,
        Err(e) => e,
    }
    I value == 42 { 0 } else { 1 }
}
"#;

    let result =
        compile_and_run(source).expect("Result match early-return arm should not type the phi");
    assert_eq!(
        result.exit_code, 0,
        "stdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
}
