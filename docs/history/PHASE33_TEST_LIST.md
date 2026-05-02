# Phase 33 Integration Tests - Complete Test List

**File**: `/Users/sswoo/study/projects/vais/crates/vaisc/tests/phase33_integration_tests.rs`

## All 23 Tests

### TLS/HTTPS (5 tests) ✓
1. `phase33_tls_constants_compile` - TLS error constants
2. `phase33_tls_mode_constants_compile` - Client/server mode constants
3. `phase33_tls_context_struct_compiles` - TlsContext struct
4. `phase33_tls_conn_struct_compiles` - TlsConn struct
5. `phase33_tls_extern_declarations_compile` - Extern function declarations

### Async Reactor (5 tests) ✓
6. `phase33_async_platform_constants_compile` - Platform detection constants
7. `phase33_async_event_constants_compile` - Event filter/action constants
8. `phase33_async_reactor_event_struct_compiles` - ReactorEvent struct
9. `phase33_async_reactor_extern_declarations_compile` - Extern function declarations
10. `phase33_async_reactor_struct_compiles` - Reactor struct with methods

### Logging (5 tests: 3 ✓, 2 ignored)
11. `phase33_logging_level_constants_compile` - Log level constants ✓
12. `phase33_logging_output_constants_compile` - Output target constants ✓
13. `phase33_logging_format_constants_compile` - Format constants ✓
14. `phase33_logging_basic_output` - Basic logging output 🚫 (requires log_runtime.c)
15. `phase33_logging_json_format` - JSON format logging 🚫 (requires log_runtime.c)

### Compression (5 tests: 4 ✓, 1 ignored)
16. `phase33_compress_status_constants_compile` - Status code constants ✓
17. `phase33_compress_algorithm_constants_compile` - Algorithm constants ✓
18. `phase33_compress_level_constants_compile` - Compression level constants ✓
19. `phase33_compress_result_struct_compiles` - CompressResult struct ✓
20. `phase33_compress_gzip_roundtrip` - Gzip roundtrip 🚫 (requires compress_runtime.c + zlib)

### Cross-Feature Integration (3 tests) ✓
21. `phase33_combined_constants_compile` - All libraries' constants together
22. `phase33_combined_structs_compile` - All libraries' structs together
23. `phase33_error_code_comparison` - Error code consistency

## Test Statistics

- **Total**: 23 tests
- **Passing**: 20 tests (87%)
- **Ignored**: 3 tests (13%)
- **Failed**: 0 tests (0%)

## Quick Commands

```bash
# Run all passing tests
cargo test --test phase33_integration_tests

# Run ignored tests
cargo test --test phase33_integration_tests -- --ignored

# Run specific test
cargo test --test phase33_integration_tests phase33_tls_constants_compile

# Run all TLS tests
cargo test --test phase33_integration_tests phase33_tls

# Run all async tests
cargo test --test phase33_integration_tests phase33_async

# Run all logging tests
cargo test --test phase33_integration_tests phase33_logging

# Run all compression tests
cargo test --test phase33_integration_tests phase33_compress

# Run all integration tests
cargo test --test phase33_integration_tests phase33_combined
```
