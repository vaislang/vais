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
| `MAX_TESTS` | 1024 | Maximum tests per suite |
| `DEFAULT_TIMEOUT_MS` | 30000 | Default timeout (30s) |

## Structures

### TestResult

```vais
S TestResult { name: str, status: i64, message: str, duration_ns: i64, file: str, line: i64 }
```

| Method | Description |
|--------|-------------|
| `new(name, status, message, duration_ns)` | Create new result |
| `passed(name, duration_ns)` | Create passed result |
| `failed(name, message, duration_ns)` | Create failed result |
| `skipped(name, reason)` | Create skipped result |
| `is_passed()` | Check if passed |
| `is_failed()` | Check if failed |
| `is_skipped()` | Check if skipped |
| `with_location(file, line)` | Set file/line location |

### TestCase

```vais
S TestCase { name: str, fn_ptr: i64, ... }
```

| Method | Description |
|--------|-------------|
| `new(name, fn_ptr)` | Create test case |
| `with_setup(fn_ptr)` | Set setup function |
| `with_teardown(fn_ptr)` | Set teardown function |
| `with_timeout(ms)` | Set timeout |
| `should_panic()` | Mark as should-panic test |
| `skip(reason)` | Skip this test |
| `tag(tag)` | Add tag |
| `has_tag(tag)` | Check for tag |
| `run()` | Run the test |

### TestSuite

```vais
S TestSuite { name: str, tests: i64, ... }
```

| Method | Description |
|--------|-------------|
| `new(name)` | Create test suite |
| `add(test)` | Add test case |
| `test(name, fn_ptr)` | Add simple test |
| `before_all(fn_ptr)` | Set before_all hook |
| `after_all(fn_ptr)` | Set after_all hook |
| `before_each(fn_ptr)` | Set before_each hook |
| `after_each(fn_ptr)` | Set after_each hook |
| `run()` | Run all tests |
| `run_filtered(filter)` | Run matching tests |
| `run_tagged(tag)` | Run tagged tests |

### TestSuiteResult

| Method | Description |
|--------|-------------|
| `new(suite_name)` | Create result |
| `add(result)` | Add test result |
| `total()` | Get total test count |
| `all_passed()` | Check if all passed |
| `print_summary()` | Print summary |

### TestRunner

```vais
S TestRunner { suites: i64, ... }
```

| Method | Description |
|--------|-------------|
| `new()` | Create test runner |
| `add_suite(suite)` | Add test suite |
| `verbose()` | Enable verbose mode |
| `filter(filter)` | Set filter |
| `fail_fast()` | Enable fail-fast |
| `run()` | Run all tests |

## Assertion Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `assert` | `F assert(condition: i64) -> i64` | Assert condition is true |
| `assert_msg` | `F assert_msg(condition: i64, message: str) -> i64` | Assert with message |
| `assert_eq` | `F assert_eq(actual: i64, expected: i64) -> i64` | Assert equality |
| `assert_ne` | `F assert_ne(actual: i64, expected: i64) -> i64` | Assert not equal |
| `assert_gt` | `F assert_gt(actual: i64, expected: i64) -> i64` | Assert greater than |
| `assert_lt` | `F assert_lt(actual: i64, expected: i64) -> i64` | Assert less than |
| `assert_ge` | `F assert_ge(actual: i64, expected: i64) -> i64` | Assert greater or equal |
| `assert_le` | `F assert_le(actual: i64, expected: i64) -> i64` | Assert less or equal |
| `assert_true` | `F assert_true(value: i64) -> i64` | Assert value is true (non-zero) |
| `assert_false` | `F assert_false(value: i64) -> i64` | Assert value is false (zero) |
| `assert_str_eq` | `F assert_str_eq(actual: str, expected: str) -> i64` | Assert strings equal |
| `assert_in_range` | `F assert_in_range(value: i64, min: i64, max: i64) -> i64` | Assert value in range |
| `assert_not_null` | `F assert_not_null(ptr: i64) -> i64` | Assert pointer is not null |
| `assert_approx` | `F assert_approx(actual: f64, expected: f64, epsilon: f64) -> i64` | Assert approximate equality |

## Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `suite` | `F suite(name: str) -> TestSuite` | Create test suite |
| `runner` | `F runner() -> TestRunner` | Create test runner |
| `test` | `F test(name: str, fn_ptr: i64) -> TestCase` | Create test case |

## Usage

```vais
U std/test

F test_addition() -> TestResult {
    result := 2 + 2
    assert_eq(result, 4)
    TestResult::passed("test_addition", 0)
}
```
