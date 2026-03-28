use std::{env, fs, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: cargo run -p vais-codegen --example verify_ir <file.ll>");
        process::exit(2);
    };

    let ir = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("failed to read {}: {}", path, err);
        process::exit(1);
    });

    let diagnostics = vais_codegen::ir_verify::verify_text_ir(&ir);
    if diagnostics.is_empty() {
        println!("OK");
        return;
    }

    for diagnostic in diagnostics {
        println!("{}", diagnostic);
    }
}

// Quick test for container
#[allow(dead_code)]
fn test_container_instantiations() {
    let source = r#"
S Container<T> {
    items: i64,
    count: i64
}

F test_container<T>(c: Container<T>) -> i64 {
    c.count
}

F main() -> i64 {
    c := Container { items: 0, count: 42 }
    test_container(c)
}
"#;
    let module = vais_parser::parse(source).expect("parse failed");
    let mut checker = vais_types::TypeChecker::new();
    checker.check_module(&module).expect("type check failed");
    let insts = checker.get_generic_instantiations();
    eprintln!("[Container test] TC instantiations: {}", insts.len());
    for inst in insts {
        eprintln!(
            "  {:?}: {} -> {} {:?}",
            inst.kind, inst.base_name, inst.mangled_name, inst.type_args
        );
    }
}
