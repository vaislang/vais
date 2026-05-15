use vais_codegen::CodeGenerator;
use vais_parser::parse;

fn gen_ir(source: &str) -> String {
    let module = parse(source).unwrap_or_else(|e| panic!("Parse failed: {:?}", e));
    let mut gen = CodeGenerator::new("test");
    gen.generate_module(&module)
        .unwrap_or_else(|e| panic!("Codegen failed:\n{}\nErr: {}", source, e))
}

#[test]
fn entry_alloca_named_aggregates_are_zero_initialized() {
    let ir = gen_ir(
        r#"
        struct Response {
            body: str,
        }

        fn make_response(flag: bool) -> Response {
            I flag {
                return Response { body: "early" }
            }
            response := Response { body: "late" }
            response
        }
        "#,
    );

    let lines: Vec<&str> = ir.lines().collect();
    let mut response_allocas = 0;
    for (idx, line) in lines.iter().enumerate() {
        if !line.contains(" = alloca %Response") {
            continue;
        }
        response_allocas += 1;
        let next = lines
            .get(idx + 1)
            .copied()
            .unwrap_or("<missing next instruction>");
        assert!(
            next.contains("store %Response zeroinitializer, %Response*"),
            "named aggregate entry alloca must be immediately zero-initialized \
             so early-return cleanup cannot read an uninitialized ownership mask.\n\
             alloca: {}\nnext: {}\nIR:\n{}",
            line,
            next,
            ir
        );
    }

    assert!(
        response_allocas > 0,
        "test fixture did not emit a %Response alloca:\n{}",
        ir
    );
}
