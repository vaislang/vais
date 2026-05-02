use super::helpers::compile_and_run;

#[test]
fn e2e_result_f32_ok_payload_binding_preserves_float_bits() {
    let source = r#"
E Result {
    Ok(f32),
    Err(i64),
}

F make_value(flag: bool) -> Result {
    I flag {
        Ok(3.5 as f32)
    } EL {
        Err(9)
    }
}

F abs_f32(x: f32) -> f32 {
    I x < (0.0 as f32) {
        (0.0 as f32) - x
    } EL {
        x
    }
}

F approx_eq_f32(actual: f32, expected: f32, epsilon: f32) -> bool {
    abs_f32(actual - expected) < epsilon
}

F main() -> i64 {
    v: f32 := mut M make_value(true) {
        Ok(x) => x,
        Err(_) => { R 1 },
    }
    I !approx_eq_f32(v, 3.5 as f32, 0.0001 as f32) { R 2 }

    e: i64 := mut M make_value(false) {
        Ok(_) => { R 3 },
        Err(code) => code,
    }
    I e != 9 { R 4 }
    R 0
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
S Meta {
    create: u64,
    expire: u64,
}

S Ptr {
    page_id: u32,
    total_len: u32,
}

F make_pair(flag: bool) -> Result<(Meta, Ptr), i64> {
    I flag {
        meta := mut Meta { create: 7, expire: 0 }
        ptr := mut Ptr { page_id: 42 as u32, total_len: 88 as u32 }
        Ok((meta, ptr))
    } EL {
        Err(9)
    }
}

F main() -> i64 {
    pair := mut M make_pair(true) {
        Ok(v) => v,
        Err(_) => { R 1 },
    }

    ptr := mut pair.1
    I ptr.page_id != 42 as u32 { R 2 }
    I ptr.total_len != 88 as u32 { R 3 }

    meta := mut pair.0
    I meta.create != 7 { R 4 }
    I meta.expire != 0 { R 5 }

    R 0
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
F maybe_flag(flag: bool) -> Result<bool, i64> {
    I flag {
        Ok(true)
    } E {
        Err(7)
    }
}

F pass(flag: bool) -> Result<i64, i64> {
    opened := mut M maybe_flag(flag) {
        Ok(v) => v,
        Err(e) => { R Err(e) },
    }
    I opened {
        Ok(42)
    } E {
        Ok(1)
    }
}

F main() -> i64 {
    value := mut M pass(true) {
        Ok(v) => v,
        Err(e) => e,
    }
    I value == 42 { 0 } E { 1 }
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
