use vais_mir::lower::{lower_module, lower_module_checked};
use vais_mir::validate::validate_module;
use vais_mir::{AggregateKind, MirType, Rvalue, Statement, Terminator};

const LOWER_RS: &str = include_str!("../src/lower.rs");

#[test]
fn strict_lowering_accepts_simple_core_function() {
    let source = "F add(x: i64, y: i64) -> i64 = x + y";
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept arithmetic");

    validate_module(&mir).expect("strictly lowered MIR should be structurally valid");
}

#[test]
fn strict_lowering_preserves_primitive_result_types() {
    let source = r#"
        F main() -> i64 {
            x: i64 := (1 + 2) * 3
            ok: bool := x == 9
            label: str := "core"
            I !ok { R 1 }
            I label != "core" { R 2 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept primitives");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let bool_locals = body
        .locals
        .iter()
        .filter(|local| local.ty == MirType::Bool)
        .count();
    let str_locals = body
        .locals
        .iter()
        .filter(|local| local.ty == MirType::Str)
        .count();

    assert!(
        bool_locals >= 3,
        "expected bool local, comparison temp, and unary-not temp; got {bool_locals}\n{}",
        body.display()
    );
    assert!(
        str_locals >= 1,
        "expected string local to remain typed as Str\n{}",
        body.display()
    );
    validate_module(&mir).expect("primitive MIR should be structurally valid");
}

#[test]
fn strict_lowering_emits_explicit_while_cfg() {
    let source = r#"
        F main() -> i64 {
            n: i64 := mut 0
            sum: i64 := mut 0
            LW n < 5 {
                sum = sum + n
                n = n + 1
            }
            R sum
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept while");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_back_edge = body.basic_blocks.iter().enumerate().any(|(idx, block)| {
        matches!(
            block.terminator,
            Some(Terminator::Goto(target)) if (target.0 as usize) <= idx
        )
    });
    let has_bool_condition = body.locals.iter().any(|local| local.ty == MirType::Bool);

    assert!(
        has_back_edge,
        "expected while lowering to emit an explicit CFG back-edge\n{}",
        body.display()
    );
    assert!(
        has_bool_condition,
        "expected while comparison condition to remain typed as Bool\n{}",
        body.display()
    );
    validate_module(&mir).expect("while MIR should be structurally valid");
}

#[test]
fn strict_lowering_emits_struct_aggregate_and_field_projection() {
    let source = r#"
        S Point {
            x: i64,
            y: i64,
        }

        F main() -> i64 {
            p: Point := Point { x: 3, y: 4 }
            I p.x != 3 { R 1 }
            I p.y != 4 { R 2 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept struct access");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_struct_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(
                    _,
                    Rvalue::Aggregate(AggregateKind::Struct(name), operands)
                ) if name == "Point" && operands.len() == 2
            )
        })
    });
    let display = body.display();

    assert!(
        has_struct_aggregate,
        "expected Point aggregate construction\n{}",
        display
    );
    assert!(
        display.contains("Field(0)") && display.contains("Field(1)"),
        "expected p.x and p.y to lower to field projections\n{}",
        display
    );
    validate_module(&mir).expect("struct MIR should be structurally valid");
}

#[test]
fn strict_lowering_emits_enum_aggregate_and_discriminant_switch() {
    let source = r#"
        EN Color {
            Red,
            Green,
            Blue,
        }

        F main() -> i64 {
            c: Color := Green
            value: i64 := M c {
                Red => 1,
                Green => 2,
                Blue => 3,
            }
            I value != 2 { R 1 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept unit enum match");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_green_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(_, Rvalue::Aggregate(AggregateKind::Enum(name, 1), operands))
                    if name == "Color" && operands.is_empty()
            )
        })
    });
    let has_discriminant = body.basic_blocks.iter().any(|block| {
        block
            .statements
            .iter()
            .any(|statement| matches!(statement, Statement::Assign(_, Rvalue::Discriminant(_))))
    });
    let has_green_switch_target = body.basic_blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            Some(Terminator::SwitchInt { targets, .. }) if targets.iter().any(|(value, _)| *value == 1)
        )
    });

    assert!(
        has_green_aggregate,
        "expected Color::Green aggregate construction\n{}",
        body.display()
    );
    assert!(
        has_discriminant,
        "expected enum match to read a discriminant\n{}",
        body.display()
    );
    assert!(
        has_green_switch_target,
        "expected enum match switch to include Green discriminant target\n{}",
        body.display()
    );
    validate_module(&mir).expect("unit enum MIR should be structurally valid");
}

#[test]
fn strict_lowering_emits_option_payload_aggregate_and_binding_projection() {
    let source = r#"
        F main() -> i64 {
            a: Option<i64> := Some(42)
            b: Option<i64> := None

            va: i64 := M a {
                Some(v) => v,
                None => -1,
            }
            vb: i64 := M b {
                Some(v) => v,
                None => -1,
            }

            I va != 42 { R 1 }
            I vb != -1 { R 2 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept Option<i64>");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_some_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(
                    _,
                    Rvalue::Aggregate(AggregateKind::Enum(name, 1), operands)
                ) if name == "Option<i64>" && operands.len() == 1
            )
        })
    });
    let has_none_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(
                    _,
                    Rvalue::Aggregate(AggregateKind::Enum(name, 0), operands)
                ) if name == "Option<i64>" && operands.is_empty()
            )
        })
    });
    let has_discriminant = body.basic_blocks.iter().any(|block| {
        block
            .statements
            .iter()
            .any(|statement| matches!(statement, Statement::Assign(_, Rvalue::Discriminant(_))))
    });
    let display = body.display();

    assert!(
        has_some_aggregate,
        "expected Option<i64>::Some aggregate with one payload\n{}",
        display
    );
    assert!(
        has_none_aggregate,
        "expected Option<i64>::None aggregate without payload\n{}",
        display
    );
    assert!(
        has_discriminant,
        "expected Option match to read a discriminant\n{}",
        display
    );
    assert!(
        display.contains("Field(0)"),
        "expected Some(v) binding to lower through enum payload field projection\n{}",
        display
    );
    validate_module(&mir).expect("Option<i64> MIR should be structurally valid");
}

#[test]
fn strict_lowering_emits_result_payload_aggregate_and_binding_projection() {
    let source = r#"
        F main() -> i64 {
            a: Result<i64, i64> := Ok(42)
            b: Result<i64, i64> := Err(7)

            va: i64 := M a {
                Ok(v) => v,
                Err(e) => e,
            }
            vb: i64 := M b {
                Ok(v) => v,
                Err(e) => e,
            }

            I va != 42 { R 1 }
            I vb != 7 { R 2 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept Result<i64,i64>");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_ok_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(
                    _,
                    Rvalue::Aggregate(AggregateKind::Enum(name, 0), operands)
                ) if name == "Result<i64,i64>" && operands.len() == 1
            )
        })
    });
    let has_err_aggregate = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(
                    _,
                    Rvalue::Aggregate(AggregateKind::Enum(name, 1), operands)
                ) if name == "Result<i64,i64>" && operands.len() == 1
            )
        })
    });
    let has_discriminant = body.basic_blocks.iter().any(|block| {
        block
            .statements
            .iter()
            .any(|statement| matches!(statement, Statement::Assign(_, Rvalue::Discriminant(_))))
    });
    let display = body.display();

    assert!(
        has_ok_aggregate,
        "expected Result<i64,i64>::Ok aggregate with one payload\n{}",
        display
    );
    assert!(
        has_err_aggregate,
        "expected Result<i64,i64>::Err aggregate with one payload\n{}",
        display
    );
    assert!(
        has_discriminant,
        "expected Result match to read a discriminant\n{}",
        display
    );
    assert!(
        display.contains("Field(0)"),
        "expected Ok(v)/Err(e) binding to lower through enum payload field projection\n{}",
        display
    );
    validate_module(&mir).expect("Result<i64,i64> MIR should be structurally valid");
}

#[test]
fn strict_lowering_rejects_unsupported_expression_fallback() {
    let source = "F main() -> i64 = [1, 2]";
    let module = vais_parser::parse(source).expect("parse failed");
    let errors = lower_module_checked(&module).expect_err("array literal must not become const 0");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unsupported expression")),
        "expected unsupported expression error, got {errors:?}"
    );
}

#[test]
fn strict_lowering_rejects_unsupported_type_fallback() {
    let source = "F main() -> i64 { value: i64? := 1 R 0 }";
    let module = vais_parser::parse(source).expect("parse failed");
    let errors =
        lower_module_checked(&module).expect_err("unsupported optional type must not become i64");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unsupported type")),
        "expected unsupported type error, got {errors:?}"
    );
}

#[test]
fn strict_lowering_rejects_unbound_identifier_fallback() {
    let source = "F main() -> i64 = missing_value";
    let module = vais_parser::parse(source).expect("parse failed");
    let errors =
        lower_module_checked(&module).expect_err("unbound identifier must not become const 0");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unbound identifier")),
        "expected unbound identifier error, got {errors:?}"
    );
}

#[test]
fn strict_lowering_rejects_match_binding_placeholder() {
    let source = "F main(x: i64) -> i64 = M x { y => y }";
    let module = vais_parser::parse(source).expect("parse failed");
    let errors = lower_module_checked(&module).expect_err("match binding must not bind const 0");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("match binding pattern")),
        "expected match binding placeholder error, got {errors:?}"
    );
}

#[test]
fn strict_lowering_emits_vec_aggregate_push_len_and_index() {
    let source = r#"
        U std/vec.{vec_new};

        F main() -> i64 {
            v: Vec<i64> := mut vec_new()
            v.push(10)
            v.push(20)
            I v.len() != 2 { R 1 }
            I v[0] != 10 { R 2 }
            I v[1] != 20 { R 3 }
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module_checked(&module).expect("strict lowering should accept Vec<i64>");
    let body = mir
        .bodies
        .iter()
        .find(|body| body.name == "main")
        .expect("main body should exist");

    let has_vec_local = body
        .locals
        .iter()
        .any(|local| local.ty == MirType::Vec(Box::new(MirType::I64)));
    let has_vec_new = body.basic_blocks.iter().any(|block| {
        block.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::Assign(_, Rvalue::Aggregate(AggregateKind::Vec, operands))
                    if operands.is_empty()
            )
        })
    });
    let vec_push_count = body
        .basic_blocks
        .iter()
        .flat_map(|block| &block.statements)
        .filter(|statement| matches!(statement, Statement::Assign(_, Rvalue::VecPush(_, _))))
        .count();
    let has_len = body.basic_blocks.iter().any(|block| {
        block
            .statements
            .iter()
            .any(|statement| matches!(statement, Statement::Assign(_, Rvalue::Len(_))))
    });
    let display = body.display();

    assert!(has_vec_local, "expected Vec<i64> local\n{}", display);
    assert!(
        has_vec_new,
        "expected Vec aggregate construction\n{}",
        display
    );
    assert_eq!(
        vec_push_count, 2,
        "expected two VecPush rvalues\n{}",
        display
    );
    assert!(has_len, "expected Vec len rvalue\n{}", display);
    assert!(
        display.contains("Index("),
        "expected Vec index projection\n{}",
        display
    );
    validate_module(&mir).expect("Vec<i64> MIR should be structurally valid");
}

#[test]
fn strict_lowering_rejects_uncertified_vec_element_type() {
    let source = r#"
        U std/vec.{vec_new};

        F main() -> i64 {
            v: Vec<str> := mut vec_new()
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let errors =
        lower_module_checked(&module).expect_err("Vec<str> must not use the Vec<i64> contract");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unsupported Vec element type")),
        "expected unsupported Vec element diagnostic, got {errors:?}"
    );
}

#[test]
fn strict_lowering_rejects_uncertified_result_payload_types() {
    let source = r#"
        F main() -> i64 {
            value: Result<str, i64> := Ok("not-core")
            R 0
        }
    "#;
    let module = vais_parser::parse(source).expect("parse failed");
    let errors = lower_module_checked(&module)
        .expect_err("Result<str, i64> must not use the Result<i64, i64> contract");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unsupported Result payload types")),
        "expected unsupported Result payload diagnostic, got {errors:?}"
    );
}

#[test]
fn strict_lowering_rejects_assignment_target_placeholder() {
    let source = "F main() -> i64 { missing = 1 R 0 }";
    let module = vais_parser::parse(source).expect("parse failed");
    let errors = lower_module_checked(&module)
        .expect_err("undeclared assignment target must not be ignored");

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("not a declared local")),
        "expected undeclared assignment target error, got {errors:?}"
    );
}

#[test]
fn strict_lowering_fallback_audit_is_current() {
    let direct_error_sites = LOWER_RS.matches("MirLowerError::new(").count();
    assert_eq!(
        direct_error_sites, 11,
        "update MIR_CONTRACT.md and lower_strict_tests when adding/removing strict lowering diagnostics"
    );

    let semantic_loss_call_sites = LOWER_RS.matches("self.semantic_loss(").count();
    assert_eq!(
        semantic_loss_call_sites, 12,
        "semantic-loss fallback classes must stay audited in MIR_CONTRACT.md"
    );

    let zero_placeholder_sites = LOWER_RS
        .matches("Operand::Constant(Constant::Int(0))")
        .count();
    assert_eq!(
        zero_placeholder_sites, 9,
        "every Constant::Int(0) placeholder/default in lower.rs must be classified in MIR_CONTRACT.md"
    );

    for expected in [
        "strict MIR lowering requires an explicit function return type",
        "unsupported top-level item for strict MIR lowering",
        "unsupported type for strict MIR lowering",
        "unsupported Option payload type",
        "unsupported Result payload types",
        "unsupported Vec element type",
        "unsupported statement for strict MIR lowering",
        "unbound identifier",
        "unsupported non-identifier call target for strict MIR lowering",
        "match binding pattern",
        "unsupported match pattern for strict MIR lowering",
        "not a declared local",
        "unsupported assignment target for strict MIR lowering",
        "unsupported expression for strict MIR lowering",
        "unknown enum variant",
        "does not match expected enum",
        "cannot carry payload data",
        "payload enum variant",
        "enum struct variant literal",
        "unknown struct",
        "duplicate field",
        "unknown field",
        "missing field",
        "unsupported field access",
        "unsupported MethodCall",
        "unsupported Index",
    ] {
        assert!(
            LOWER_RS.contains(expected),
            "missing audited strict lowering diagnostic substring: {expected}"
        );
    }
}

#[test]
fn legacy_lowering_still_preserves_old_fallback_behavior() {
    let source = "F main() -> i64 = value.field";
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module(&module);

    validate_module(&mir).expect("legacy fallback MIR remains structurally valid");
}
