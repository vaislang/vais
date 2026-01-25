#![no_main]

use libfuzzer_sys::fuzz_target;
use vais_parser::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        // Limit input size
        if source.len() > 50_000 {
            return;
        }

        // Parser should handle all inputs gracefully
        // Using error recovery mode to exercise more code paths
        let _ = parse(source);
    }
});
