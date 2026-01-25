#![no_main]

use libfuzzer_sys::fuzz_target;
use vais_parser::parse;
use vais_types::TypeChecker;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        if source.len() > 30_000 {
            return;
        }

        // Only fuzz type checker if parsing succeeds
        if let Ok(module) = parse(source) {
            let mut checker = TypeChecker::new();
            // Should not panic even on invalid types
            let _ = checker.check_module(&module);
        }
    }
});
