//! Thin libFuzzer entry point for the MIR-vs-native differential oracle.
//!
//! All structured-input types and the `compare_paths` differential check
//! live in the sibling library crate (`vais_fuzz`). This binary is a
//! ~20-line shim so that `cargo test --lib -p vais-fuzz` can exercise
//! the comparison logic via `#[test]` functions, addressing
//! STEP17_FINDINGS F-MIR-02. Stage 0/1 history (Path A wired to the MIR
//! interpreter, Path B still NotImplemented pending vais-jit
//! Cranelift) lives in the lib crate's module docs.

#![no_main]

use libfuzzer_sys::fuzz_target;
use vais_fuzz::{compare_paths, VaisProgram};

fuzz_target!(|program: VaisProgram| {
    let source = program.to_source();

    // Guard against pathologically large inputs.
    if source.len() > 50_000 {
        return;
    }

    compare_paths(&source);
});
