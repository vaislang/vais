# Contract API Reference

> Design-by-contract support (requires/ensures/invariant)

## Import

This module is used internally by the compiler. It should not be imported directly.

## Contract Types

| Constant | Value | Description |
|----------|-------|-------------|
| `CONTRACT_REQUIRES` | 1 | Precondition |
| `CONTRACT_ENSURES` | 2 | Postcondition |
| `CONTRACT_INVARIANT` | 3 | Invariant |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `__contract_fail` | `F __contract_fail(kind: i64, condition: str, file: str, line: i64, func: str) -> i64` | Contract violation handler |
| `__panic` | `F __panic(msg: str) -> i64` | Panic with message |

## Overview

When a contract annotation is violated at runtime, the compiler inserts calls to `__contract_fail` which prints diagnostic information and exits. This enables defensive programming with preconditions and postconditions.
