use vais_mir::lower::lower_module;
use vais_mir::validate::{validate_body, validate_module};
use vais_mir::*;

fn local_decl(name: &str, ty: MirType) -> LocalDecl {
    LocalDecl {
        name: Some(name.to_string()),
        ty,
        is_mutable: true,
        lifetime: None,
    }
}

fn valid_body() -> Body {
    Body {
        name: "valid".to_string(),
        params: vec![MirType::I64],
        return_type: MirType::I64,
        locals: vec![
            local_decl("_return", MirType::I64),
            local_decl("_arg0", MirType::I64),
        ],
        basic_blocks: vec![BasicBlock {
            statements: vec![Statement::Assign(
                Place::local(Local(0)),
                Rvalue::Use(Operand::Copy(Place::local(Local(1)))),
            )],
            terminator: Some(Terminator::Return),
        }],
        block_names: Default::default(),
        lifetime_params: vec![],
        lifetime_bounds: vec![],
    }
}

#[test]
fn validator_accepts_lowered_core_function() {
    let source = "F add(x: i64, y: i64) -> i64 = x + y";
    let module = vais_parser::parse(source).expect("parse failed");
    let mir = lower_module(&module);

    validate_module(&mir).expect("lowered core function should be structurally valid");
}

#[test]
fn validator_rejects_missing_terminator() {
    let mut body = valid_body();
    body.basic_blocks[0].terminator = None;

    let errors = validate_body(&body).expect_err("missing terminator must fail validation");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("missing a terminator")),
        "expected missing terminator error, got {errors:?}"
    );
}

#[test]
fn validator_rejects_invalid_statement_local() {
    let mut body = valid_body();
    body.basic_blocks[0].statements[0] = Statement::Assign(
        Place::local(Local(0)),
        Rvalue::Use(Operand::Copy(Place::local(Local(99)))),
    );

    let errors = validate_body(&body).expect_err("invalid local must fail validation");
    assert!(
        errors.iter().any(|error| error.message.contains("_99")),
        "expected undeclared local error, got {errors:?}"
    );
}

#[test]
fn validator_rejects_invalid_terminator_target() {
    let mut body = valid_body();
    body.basic_blocks[0].terminator = Some(Terminator::Goto(BasicBlockId(99)));

    let errors = validate_body(&body).expect_err("invalid target must fail validation");
    assert!(
        errors.iter().any(|error| error.message.contains("bb99")),
        "expected invalid target error, got {errors:?}"
    );
}

#[test]
fn validator_rejects_duplicate_function_bodies() {
    let body = valid_body();
    let module = MirModule {
        name: "duplicate".to_string(),
        bodies: vec![body.clone(), body],
        structs: Default::default(),
        enums: Default::default(),
    };

    let errors = validate_module(&module).expect_err("duplicate body names must fail validation");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("duplicate function body")),
        "expected duplicate function body error, got {errors:?}"
    );
}
