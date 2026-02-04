# Contract API Reference

> Design-by-contract support (requires/ensures/invariant)

## Import

This module is used internally by the compiler. It should not be imported directly.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `CONTRACT_REQUIRES` | 1 | Precondition violation |
| `CONTRACT_ENSURES` | 2 | Postcondition violation |
| `CONTRACT_INVARIANT` | 3 | Invariant violation |

## Functions

### `__contract_fail`

```vais
F __contract_fail(kind: i64, condition: str, file: str, line: i64, func: str) -> i64
```

Called by the compiler when a contract is violated. Prints diagnostic information and exits the program.

**Parameters:**
- `kind` - Type of contract (1=requires, 2=ensures, 3=invariant)
- `condition` - The condition that failed (as string)
- `file` - Source file name
- `line` - Source line number
- `func` - Function name

**Returns:** 0 (but calls `exit(1)` before returning)

### `__panic`

```vais
F __panic(msg: str) -> i64
```

Simpler panic function for contract failures. Prints error message and exits.

**Parameters:**
- `msg` - Error message to display

**Returns:** 0 (but calls `exit(1)` before returning)

## Overview

The contract module provides runtime support for formal verification through design-by-contract principles. When contract annotations (`requires`, `ensures`, `invariant`) are violated at runtime, the compiler automatically inserts calls to `__contract_fail` which:

1. Identifies the type of contract violation
2. Prints the failed condition and location
3. Terminates the program with exit code 1

This enables defensive programming with preconditions (requires), postconditions (ensures), and invariants.

## Example Output

When a contract is violated, output looks like:

```
Contract violation: precondition failed
  Condition: x > 0
  Location: math.vais:42 in function 'sqrt'
```
