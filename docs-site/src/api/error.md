# Error API Reference

> Ergonomic error type definitions, chaining, and downcasting (thiserror/anyhow style)

## Import

```vais
use std/error
```

## Overview

The `error` module provides structured error handling utilities inspired by Rust's `thiserror` and `anyhow` crates. It includes error categories, an error chain for tracking causality, and a typed `AppError` enum with convenience constructors.

## Error Category Functions

```vais
fn ERROR_CATEGORY_IO() -> i64         # 1
fn ERROR_CATEGORY_PARSE() -> i64      # 2
fn ERROR_CATEGORY_VALIDATION() -> i64 # 3
fn ERROR_CATEGORY_AUTH() -> i64       # 4
fn ERROR_CATEGORY_NETWORK() -> i64    # 5
fn ERROR_CATEGORY_INTERNAL() -> i64   # 6
```

Broad error categories for classification and routing.

## Struct

### `ErrorChain`

```vais
struct ErrorChain {
    code: i64,
    context_code: i64,
    next: i64   # pointer to next ErrorChain node (0 = end)
}
```

A linked-list chain of errors for tracking causality, similar to `anyhow::Chain`.

## ErrorChain Methods

### new

```vais
fn new(code: i64) -> ErrorChain
```

Create a new error chain from a root error code.

### wrap

```vais
fn wrap(code: i64, context: i64, source_ptr: i64) -> ErrorChain
```

Wrap an error with a context code, creating a new chain node.

### root_cause

```vais
fn root_cause(&self) -> i64
```

Get the root cause error code (bottom of the chain).

### depth

```vais
fn depth(&self) -> i64
```

Get the chain depth (1 = no wrapping).

### has_source

```vais
fn has_source(&self) -> i64
```

Check if this error has a source/cause. Returns `1` if yes, `0` if no.

## Enum

### `AppError`

```vais
enum AppError {
    NotFound(i64),
    InvalidInput(i64),
    IoError(i64),
    ParseError(i64),
    AuthError(i64),
    Timeout(i64),
    Internal(i64)
}
```

Common application error variants with specific error codes.

## AppError Methods

### code

```vais
fn code(&self) -> i64
```

Get the error code from any variant.

### category

```vais
fn category(&self) -> i64
```

Get the error category (returns one of the `ERROR_CATEGORY_*` values).

### is_retryable

```vais
fn is_retryable(&self) -> i64
```

Check if this error is retryable. `Timeout` and `IoError` return `1`.

### to_result

```vais
fn to_result(&self) -> Result<i64, i64>
```

Convert to a `Result::Err` with the error code.

## Convenience Constructors

```vais
fn not_found(detail: i64) -> AppError
fn invalid_input(detail: i64) -> AppError
fn io_error(detail: i64) -> AppError
fn parse_error(detail: i64) -> AppError
fn auth_error(detail: i64) -> AppError
fn timeout_error(detail: i64) -> AppError
fn internal_error(detail: i64) -> AppError
```

## Utility Functions

### from_errno

```vais
fn from_errno(errno: i64) -> AppError
```

Convert an errno-style code to an `AppError`. Maps errno `2` to `NotFound`, `13` to `AuthError`, `22` to `InvalidInput`, and others to `IoError`.

### ensure

```vais
fn ensure(condition: i64, error_code: i64) -> i64
```

Ensure a condition holds. Returns `0` if the condition is truthy, or `error_code` otherwise.

## Example

```vais
use std/error

fn validate(x: i64) -> Result<i64, i64> {
    I x < 0 {
        err := invalid_input(x)
        err.to_result()
    } else {
        Ok(x)
    }
}
```
