#![cfg(feature = "inkwell-codegen")]

use inkwell::context::Context;
use vais_codegen::InkwellCodeGenerator;
use vais_parser::parse;

#[test]
fn inkwell_option_uses_named_erased_layout_and_declared_tags() {
    let source = r#"
E Option<T> {
    Some(T),
    None
}

F main() -> i64 {
    x: Option<i64> = Some(42)
    R M x {
        Some(v) => v,
        None => 0
    }
}
"#;

    let module = parse(source).expect("source should parse");
    let context = Context::create();
    let mut gen = InkwellCodeGenerator::new(&context, "option_layout");
    gen.generate_module(&module)
        .expect("inkwell codegen should succeed");
    let ir = gen.get_ir_string();

    assert!(
        ir.contains("%Option = type { i32, { i64 } }"),
        "Option must use the canonical named erased ABI:\n{}",
        ir
    );
    assert!(
        !ir.contains("{ i8, i64 }"),
        "Option must not fall back to the old anonymous i8-tag ABI:\n{}",
        ir
    );
    assert!(
        ir.contains("store %Option { i32 0, { i64 } { i64 42 } }"),
        "Some must use the declaration-order tag and nested payload slot:\n{}",
        ir
    );
    assert!(
        ir.contains("icmp eq i32 %enum_tag, 0"),
        "match must compare against the same i32 tag used by construction:\n{}",
        ir
    );
    assert!(
        ir.contains("extractvalue { i64 } %variant_payload, 0"),
        "payload binding must extract from the nested erased payload struct:\n{}",
        ir
    );
}
