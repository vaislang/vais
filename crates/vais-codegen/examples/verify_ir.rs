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
