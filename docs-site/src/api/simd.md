# SIMD API Reference

> CPU SIMD intrinsics for vector operations (SSE2/AVX2/NEON)

## Import

```vais
U std/simd
```

## Overview

The `simd` module provides wrappers for CPU SIMD (Single Instruction Multiple Data) intrinsics, supporting x86_64 SSE2/AVX2 and ARM NEON instruction sets. It falls back to scalar operations when SIMD is not available.

## Constants

### Vector Widths

| Constant | Value | Description |
|----------|-------|-------------|
| `SIMD_128` | 128 | SSE2 / NEON |
| `SIMD_256` | 256 | AVX2 |
| `SIMD_512` | 512 | AVX-512 |

### Element Counts

| Constant | Value | Description |
|----------|-------|-------------|
| `F32X4_SIZE` | 4 | 128-bit float vector |
| `F32X8_SIZE` | 8 | 256-bit float vector |
| `F64X2_SIZE` | 2 | 128-bit double vector |
| `F64X4_SIZE` | 4 | 256-bit double vector |
| `I32X4_SIZE` | 4 | 128-bit int vector |
| `I32X8_SIZE` | 8 | 256-bit int vector |

## Struct

### `SimdVec`

```vais
S SimdVec {
    data: i64,      # Pointer to aligned memory
    len: i64,       # Number of elements
    elem_size: i64, # Size of each element (4 for f32, 8 for f64)
    width: i64      # SIMD width (128, 256, 512)
}
```

A SIMD-friendly vector that stores elements in aligned memory for vectorized operations.

## Key Operations

The module provides vectorized arithmetic operations (add, sub, mul, div), dot product, distance calculations, and reduction operations that automatically use the best available SIMD instruction set.

## Example

```vais
U std/simd

F main() {
    # Create SIMD vectors
    a := SimdVec { data: ptr_a, len: 4, elem_size: 8, width: SIMD_256 }
    b := SimdVec { data: ptr_b, len: 4, elem_size: 8, width: SIMD_256 }
}
```
