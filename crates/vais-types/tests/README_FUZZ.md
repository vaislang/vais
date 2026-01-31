# Type Checker Fuzz Tests

This directory contains fuzz tests for the Vais type checker to ensure robustness against malformed ASTs.

## Running the Tests

Run all fuzz tests:
```bash
cargo test -p vais-types -- fuzz
```

Run with detailed output:
```bash
cargo test -p vais-types -- fuzz --nocapture
```

Run a specific fuzz test:
```bash
cargo test -p vais-types fuzz_circular_type_references
```

## Test Categories

1. **Scale Tests**
   - `fuzz_functions_with_many_parameters`: 10-200 parameters
   - `fuzz_deeply_nested_types`: 5-50 levels of type nesting
   - `fuzz_very_long_identifiers`: 100-5000 character identifiers

2. **Type System Tests**
   - `fuzz_circular_type_references`: Self-referential and mutual references
   - `fuzz_malformed_generics`: Wrong type argument counts, conflicting bounds
   - `fuzz_type_mismatches`: Incompatible types in various contexts

3. **Trait System Tests**
   - `fuzz_empty_trait_implementations`: Empty traits, missing methods, duplicate impls

4. **Error Handling Tests**
   - `fuzz_undefined_references`: Undefined variables, functions, types
   - `fuzz_functions_with_no_body`: Function signatures without implementations
   - `fuzz_edge_case_expressions`: Division by zero, overflow, deep nesting

5. **Random Generation**
   - `fuzz_generated_valid_programs`: Randomly generated valid-looking programs

## Test Strategy

The type checker fuzz tests work by:
1. Parsing source code (using the parser)
2. Running the type checker on the resulting AST
3. Catching any panics that occur

The tests verify that the type checker never panics, even on:
- Semantically invalid programs
- Programs with type errors
- Programs with undefined references
- Programs with circular dependencies

## Expected Behavior

The type checker should NEVER panic on any input. It should either:
- Type check successfully and return Ok(())
- Return a `TypeError` for invalid programs

Any panic is considered a bug that should be reported.

## Test Implementation

All tests use:
- `std::panic::catch_unwind` to detect panics
- Simple LCG for reproducible random numbers (where needed)
- No external dependencies beyond `vais-parser` and `vais-ast`
- Detailed error reporting with the source code that caused the panic

## Known Issues

No panics found in type checker across all test categories.
