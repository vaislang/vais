# Range Type Implementation

## Overview

This document describes the implementation of the `Range<T>` type in the VAIS type system.

## Implementation Details

### 1. Type System (`vais-types/src/lib.rs`)

#### Added Range Variant to ResolvedType Enum

```rust
pub enum ResolvedType {
    // ... other variants ...
    Range(Box<ResolvedType>),
}
```

The `Range` type is parameterized over an inner type `T`, which must be an integer type.

#### Display Implementation

Added display formatting for Range types:
```rust
ResolvedType::Range(t) => write!(f, "Range<{}>", t)
```

#### Type Unification

Added Range handling to the `unify` method:
```rust
(ResolvedType::Range(a), ResolvedType::Range(b)) => self.unify(a, b)
```

This ensures that `Range<i32>` and `Range<i64>` can be unified according to integer conversion rules.

#### Substitution Application

Added Range handling to `apply_substitutions`:
```rust
ResolvedType::Range(inner) => {
    ResolvedType::Range(Box::new(self.apply_substitutions(inner)))
}
```

### 2. Range Expression Type Checking

Implemented comprehensive type checking for `Expr::Range { start, end, inclusive }`:

```rust
Expr::Range { start, end, inclusive: _ } => {
    let elem_type = if let Some(start_expr) = start {
        let start_type = self.check_expr(start_expr)?;
        // Ensure start is an integer type
        if !start_type.is_integer() {
            return Err(TypeError::Mismatch {
                expected: "integer type".to_string(),
                found: start_type.to_string(),
            });
        }

        // If end is present, unify the types
        if let Some(end_expr) = end {
            let end_type = self.check_expr(end_expr)?;
            if !end_type.is_integer() {
                return Err(TypeError::Mismatch {
                    expected: "integer type".to_string(),
                    found: end_type.to_string(),
                });
            }
            self.unify(&start_type, &end_type)?;
        }

        start_type
    } else if let Some(end_expr) = end {
        // Only end is present (e.g., ..10)
        let end_type = self.check_expr(end_expr)?;
        if !end_type.is_integer() {
            return Err(TypeError::Mismatch {
                expected: "integer type".to_string(),
                found: end_type.to_string(),
            });
        }
        end_type
    } else {
        // Neither start nor end (e.g., ..) - default to i64
        ResolvedType::I64
    };

    Ok(ResolvedType::Range(Box::new(elem_type)))
}
```

**Key features:**
- Validates that start and end expressions are integer types
- Infers the range element type from start or end
- Unifies start and end types when both are present
- Defaults to `Range<i64>` for unbounded ranges

### 3. Loop Iteration with Ranges

Enhanced loop type checking to support Range iteration:

```rust
Expr::Loop { pattern, iter, body } => {
    self.push_scope();

    if let (Some(pattern), Some(iter)) = (pattern, iter) {
        let iter_type = self.check_expr(iter)?;

        match iter_type {
            ResolvedType::Array(elem_type) => {
                if let Pattern::Ident(name) = &pattern.node {
                    self.define_var(name, *elem_type, false);
                }
            }
            ResolvedType::Range(elem_type) => {
                // Range<T> iterates over T values
                if let Pattern::Ident(name) = &pattern.node {
                    self.define_var(name, *elem_type, false);
                }
            }
            _ => {
                // For other types, allow but don't bind pattern
            }
        }
    }

    self.check_block(body)?;
    self.pop_scope();

    Ok(ResolvedType::Unit)
}
```

This allows loops like `L i:0..10 { }` to properly infer that `i` has type `i64`.

### 4. Code Generation Support (`vais-codegen/src/lib.rs`)

Added Range type handling to LLVM type mapping:

```rust
ResolvedType::Range(_inner) => {
    // Range is represented as a struct with start and end fields
    // For now, we'll use a simple struct: { i64 start, i64 end, i1 inclusive }
    "%Range".to_string()
}
```

**Note:** Full codegen support for range iteration is not yet implemented. Range values are represented as struct pointers in LLVM IR, but the iterator protocol and range iteration codegen are left for future work.

## Supported Range Forms

The implementation supports all standard range syntax:

| Syntax | Description | Type |
|--------|-------------|------|
| `0..10` | Exclusive range from 0 to 9 | `Range<i64>` |
| `0..=10` | Inclusive range from 0 to 10 | `Range<i64>` |
| `..10` | Range from minimum to 9 | `Range<i64>` |
| `0..` | Range from 0 to maximum | `Range<i64>` |
| `..` | Unbounded range | `Range<i64>` |
| `a..b` | Range with type inferred from a and b | `Range<T>` where T is integer type |

## Type Safety

The implementation enforces several type safety constraints:

1. **Integer-only ranges:** Start and end must be integer types (i8, i16, i32, i64, u8, u16, u32, u64)
2. **Type consistency:** If both start and end are present, they must be unifiable
3. **Iterator type inference:** Loop variables correctly receive the range element type

## Examples

### Valid Usage

```vais
# Basic range
r1 := 0..10        # Range<i64>

# Inclusive range
r2 := 1..=5        # Range<i64>

# Range iteration with type inference
L i:0..10 {
    x := i + 5     # i is i64
}
```

### Type Errors

```vais
# ERROR: String is not an integer type
r := "hello"..10   # Type error: expected integer type, found str

# ERROR: Mixing incompatible types
r := 0..3.14       # Type error: expected integer type, found f64
```

## Future Work

1. **Range Iterator Codegen:** Complete implementation of range iteration in LLVM IR generation
2. **Range Methods:** Add methods like `contains`, `is_empty`, etc.
3. **Range Patterns:** Support range patterns in match expressions
4. **Generic Ranges:** Support ranges over custom types implementing iteration traits
5. **Step Ranges:** Support ranges with custom step sizes

## Files Modified

- `crates/vais-types/src/lib.rs`: Core Range type implementation
- `crates/vais-codegen/src/lib.rs`: LLVM type mapping for Range

## Testing

The implementation passes all existing tests and correctly:
- Accepts valid range expressions
- Rejects non-integer types in ranges
- Infers correct types for loop variables
- Generates type-correct IR (codegen for iteration pending)
