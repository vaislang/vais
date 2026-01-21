# Vais LSP Integration Tests

This directory contains integration tests for the Vais Language Server Protocol implementation.

## Test Coverage

The integration tests cover the following LSP features:

### Initialization (3 tests)
- `test_initialize` - Basic server initialization with capability verification
- `test_initialized_notification` - Initialized notification handling
- `test_initialize_with_client_info` - Initialization with client information
- `test_shutdown` - Server shutdown

### Completion (7 tests)
- `test_completion_provides_keywords` - Vais language keywords (F, S, E, I, L, M, etc.)
- `test_completion_provides_types` - Primitive types (i64, f64, bool, str, etc.)
- `test_completion_provides_builtin_functions` - Built-in functions (puts, malloc, print_i64, etc.)
- `test_completion_provides_std_modules` - Standard library modules (std/math, std/io, etc.)
- `test_completion_provides_math_functions` - Math functions (sqrt, sin, cos, pow, etc.)
- `test_completion_provides_option_result_constructors` - Option/Result constructors (Some, None, Ok, Err)

### Capabilities (1 test)
- `test_server_capabilities_comprehensive` - Verifies all advertised server capabilities

### Error Handling (5 tests)
- `test_completion_on_nonexistent_document` - Completion on non-existent documents
- `test_hover_on_nonexistent_document` - Hover on non-existent documents
- `test_goto_definition_on_nonexistent_document` - Go-to-definition on non-existent documents
- `test_references_on_nonexistent_document` - Find references on non-existent documents
- `test_document_symbols_on_nonexistent_document` - Document symbols on non-existent documents

## Running Tests

Run all integration tests:
```bash
cargo test --package vais-lsp --test integration_tests
```

Run a specific test:
```bash
cargo test --package vais-lsp --test integration_tests test_initialize -- --exact
```

Run tests with output:
```bash
cargo test --package vais-lsp --test integration_tests -- --nocapture
```

## Limitations

**Document Operations**: Tests involving document operations (didOpen, didChange, didClose) are not included due to technical limitations with tower-lsp's test infrastructure. The `publish_diagnostics` calls block in test environments because there's no active client to consume the messages.

For comprehensive end-to-end testing of document operations (diagnostics, hover, go-to-definition, find references with actual documents), use a real LSP client such as:
- Visual Studio Code with the Vais extension
- Neovim with LSP configuration
- Any other LSP-compatible editor

## Test Structure

Tests use the following helper functions:
- `create_test_service()` - Creates a new LSP service instance for testing
- `test_uri(name)` - Creates a test file URI
- `pos(line, character)` - Creates a position in a document

## Adding New Tests

When adding new tests:

1. Use `#[tokio::test]` attribute for async tests
2. Follow the existing test naming convention: `test_<feature>_<scenario>`
3. Group related tests with comments
4. Add descriptive assertion messages
5. Document any limitations or special considerations

## Test Statistics

- Total tests: 16
- Total lines: ~516
- All tests passing: ✅
