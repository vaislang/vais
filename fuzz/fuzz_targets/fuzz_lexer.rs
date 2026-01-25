#![no_main]

use libfuzzer_sys::fuzz_target;
use vais_lexer::tokenize;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, handling invalid UTF-8 gracefully
    if let Ok(source) = std::str::from_utf8(data) {
        // Limit input size to prevent OOM
        if source.len() > 100_000 {
            return;
        }

        // The lexer should never panic on any input
        let _ = tokenize(source);
    }
});
