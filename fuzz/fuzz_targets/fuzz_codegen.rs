#![no_main]

use libfuzzer_sys::fuzz_target;
use vais_codegen::CodeGenerator;
use vais_parser::parse;
use vais_types::TypeChecker;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        if source.len() > 20_000 {
            return;
        }

        // Full pipeline: parse -> type check -> codegen
        if let Ok(module) = parse(source) {
            let mut checker = TypeChecker::new();
            if checker.check_module(&module).is_ok() {
                let mut gen = CodeGenerator::new("fuzz_test");
                // Codegen should not panic on valid typed AST
                let _ = gen.generate_module(&module);
            }
        }
    }
});
