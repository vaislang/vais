# examples/intentional_errors/

Examples that exist to exercise the compiler's error reporting. Each
file is **supposed** to fail compilation with a specific diagnostic.
They are deliberately excluded from `examples_fresh_rebuild`, which
expects all of its inputs to compile successfully.

A future `assert_compile_fails` test could pick them up and verify
that the expected error code is produced; for now they are reference
fixtures only.

| File | Expected error | Purpose |
|---|---|---|
| `range_type_error_test.vais` | E001 Type mismatch (range of str) | Verify E001 fires on non-integer range bounds |
