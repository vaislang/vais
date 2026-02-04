# Test API Reference

> Built-in test framework with assertions and test discovery

## Import

```vais
U std/test
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `TEST_PASSED` | 0 | Test passed |
| `TEST_FAILED` | 1 | Test failed |
| `TEST_SKIPPED` | 2 | Test skipped |
| `TEST_TIMEOUT` | 3 | Test timed out |
| `DEFAULT_TIMEOUT_MS` | 30000 | Default timeout (30s) |

## TestResult

```vais
S TestResult { name: str, status: i64, message: str, duration_ns: i64, file: str, line: i64 }
```

| Method | Description |
|--------|-------------|
| `passed(name, duration_ns)` | Create passed result |
| `failed(name, message, duration_ns)` | Create failed result |
| `skipped(name)` | Create skipped result |

## Assertion Functions

| Function | Description |
|----------|-------------|
| `assert_eq(a, b)` | Assert equality |
| `assert_ne(a, b)` | Assert not equal |
| `assert_true(cond)` | Assert condition is true |
| `assert_false(cond)` | Assert condition is false |

## Usage

```vais
U std/test

F test_addition() -> TestResult {
    result := 2 + 2
    assert_eq(result, 4)
    TestResult::passed("test_addition", 0)
}
```
