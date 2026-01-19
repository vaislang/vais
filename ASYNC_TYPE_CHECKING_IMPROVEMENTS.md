# Async Type Checking Improvements

## Overview
Enhanced the VAIS type checker to properly validate async/await operations with comprehensive Future type checking.

## Changes Made

### 1. Added Future Type to Type System
**File**: `crates/vais-types/src/lib.rs`

- Added `Future(Box<ResolvedType>)` variant to the `ResolvedType` enum
- Updated `Display` trait implementation to show `Future<T>` in type messages
- Added Future type unification in the `unify()` function

### 2. Async Function Type Handling
**Files**: `crates/vais-types/src/lib.rs`, `crates/vais-types/src/traits.rs`

- Modified `lookup_var_info()` to wrap async function return types in `Future<T>`
- Updated `SelfCall` expression handling to return `Future<T>` for async functions
- Enhanced method call handling to return `Future<T>` for async methods
- Added `is_async` field to `TraitMethodSig` struct for trait method support

### 3. Enhanced Await Expression Type Checking
**File**: `crates/vais-types/src/lib.rs`

Replaced the TODO comment with proper implementation:
```rust
Expr::Await(inner) => {
    let inner_type = self.check_expr(inner)?;

    // Verify that the inner expression is a Future type
    if let ResolvedType::Future(output_type) = inner_type {
        // Extract and return the inner type from Future<T>
        Ok(*output_type)
    } else {
        Err(TypeError::Mismatch {
            expected: "Future<T>".to_string(),
            found: inner_type.to_string(),
        })
    }
}
```

## What This Ensures

### 1. Async Functions Return Future Types
When you call an async function without `.await`, you get a `Future<T>` type:

```vais
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    fut := compute(42)  # Type: Future<i64>
    result := fut.await  # Type: i64
}
```

### 2. Await Only Works on Futures
You cannot use `.await` on non-Future types:

```vais
F sync_func() -> i64 { 42 }

F main() -> i64 {
    sync_func().await  # ERROR: expected Future<T>, found i64
}
```

### 3. Futures Cannot Be Used Directly
Future values must be awaited before use:

```vais
A F async_func() -> i64 { 42 }

F main() -> i64 {
    async_func() + 10  # ERROR: expected numeric, found Future<i64>
}
```

### 4. Type Safety for Future Output Types
The type checker correctly infers and validates the output type of awaited Futures:

```vais
A F get_number() -> i64 { 42 }

F main() -> i64 {
    result := get_number().await  # result has type i64, not Future<i64>
    result + 10  # OK: i64 + i64
}
```

## Test Results

### Valid Async Usage (✓)
```bash
$ vaisc check examples/async_test.vais
OK No errors found
```

### Invalid Await on Non-Future (✗)
```bash
$ vaisc check test_await_on_sync.vais
error: Type error: Type mismatch: expected Future<T>, found i64
```

### Invalid Future Arithmetic (✗)
```bash
$ vaisc check test_future_arithmetic.vais
error: Type error: Type mismatch: expected numeric, found Future<i64>
```

## Implementation Details

### Type Wrapping Strategy
- Async functions are typed as `(params) -> Future<T>` instead of `(params) -> T`
- This happens automatically when looking up function signatures for async functions
- The type wrapper is added in three places:
  1. Function reference lookup (`lookup_var_info`)
  2. Self-call expression (`@`)
  3. Method call expressions (both struct and trait methods)

### Future Type Inference
- When `.await` is used, the type checker extracts the inner type from `Future<T>`
- This allows proper type inference for chained operations
- Example: `async_fn().await + 10` correctly infers both the Future extraction and numeric operation

### Method Support
- Both struct methods and trait methods can be async
- The `is_async` flag is checked during method call type checking
- Trait methods have `is_async` field prepared for future AST support (currently defaults to `false`)

## Benefits

1. **Compile-time Safety**: Catches async/await misuse before runtime
2. **Better Error Messages**: Clear type mismatch errors with Future types shown explicitly
3. **Type Inference**: Proper propagation of types through await expressions
4. **Consistency**: Async type checking follows the same patterns as Optional and Result types
5. **Future-proof**: Infrastructure ready for more advanced async patterns

## Notes

- The TODO comment `// TODO: Proper async type checking` has been removed
- All existing async examples continue to work correctly
- Type checking is fully compatible with the existing state-machine based async implementation
