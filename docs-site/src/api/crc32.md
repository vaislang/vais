# CRC32 API Reference

> CRC32 checksum computation (IEEE 802.3 polynomial)

## Import

```vais
U std/crc32
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `crc32` | `F crc32(data: i64, len: i64) -> i64` | Compute CRC32 of buffer |
| `crc32_str` | `F crc32_str(s: str) -> i64` | Compute CRC32 of string |
| `crc32_update_byte` | `F crc32_update_byte(crc: i64, byte_val: i64) -> i64` | Update CRC with one byte |
| `crc32_loop` | `F crc32_loop(data: i64, crc: i64, idx: i64, len: i64) -> i64` | Process byte range |

## Overview

Uses the standard IEEE 802.3 polynomial (0xEDB88320 in reflected form). Implemented with a bitwise algorithm (no lookup table) using recursive processing.

## Usage

```vais
U std/crc32

F main() -> i64 {
    data := "Hello, World!"
    checksum := crc32(data, 13)
    0
}
```
