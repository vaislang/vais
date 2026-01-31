# Parser Fuzz Tests

This directory contains fuzz tests for the Vais parser to ensure robustness against malformed input.

## Running the Tests

Run all fuzz tests:
```bash
cargo test -p vais-parser -- fuzz
```

Run with detailed output:
```bash
cargo test -p vais-parser -- fuzz --nocapture
```

Run a specific fuzz test:
```bash
cargo test -p vais-parser fuzz_random_ascii_strings
```

Run ignored tests (demonstrates known issues):
```bash
cargo test -p vais-parser -- fuzz --ignored
```

## Test Categories

1. **Random Input Tests**
   - `fuzz_completely_random_bytes`: Random byte sequences
   - `fuzz_random_ascii_strings`: Random ASCII characters
   - `fuzz_unicode_and_special_chars`: Unicode, null bytes, control chars

2. **Mutation Tests**
   - `fuzz_mutated_valid_code`: Valid programs with random mutations

3. **Scale Tests**
   - `fuzz_very_long_inputs`: 10K-50K character strings
   - `fuzz_many_sequential_statements`: 100-2000 statements

4. **Structure Tests**
   - `fuzz_deeply_nested_expressions`: Nested parens, calls, blocks
   - `fuzz_empty_and_minimal_inputs`: Empty strings, single chars
   - `fuzz_pathological_patterns`: Repeated operators, mismatched delimiters

## Known Issues

**Stack Overflow on Deep Nesting**: The parser will stack overflow on deeply nested expressions (30+ levels). This is documented in the `fuzz_extremely_deep_nesting_causes_stack_overflow` test which is marked `#[ignore]`.

## Test Implementation

All tests use:
- `std::panic::catch_unwind` to detect panics
- Simple LCG for reproducible random numbers
- No external dependencies (stdlib only)
- Detailed error reporting with the input that caused the panic

## Expected Behavior

The parser should NEVER panic on any input. It should either:
- Parse successfully and return an AST
- Return a `ParseError` for invalid input

Any panic is considered a bug that should be reported.
