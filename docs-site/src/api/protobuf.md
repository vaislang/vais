# Protobuf API Reference

> Protocol Buffers binary wire format encoding and decoding

## Import

```vais
U std/protobuf
```

## Overview

The `protobuf` module implements the [Protocol Buffers](https://protobuf.dev/programming-guides/encoding/) binary wire format. It provides a message builder/parser approach for constructing and reading protobuf messages without requiring `.proto` files.

## Wire Types

| Wire Type | Value | Used For |
|-----------|-------|----------|
| Varint | 0 | int32, int64, uint32, uint64, sint32, sint64, bool, enum |
| 64-bit | 1 | fixed64, sfixed64, double |
| Length-delimited | 2 | string, bytes, embedded messages, packed repeated fields |
| 32-bit | 5 | fixed32, sfixed32, float |

## Parsed Field Representation

```
[wire_type: i64, field_number: i64, data: i64, extra: i64]
```

- Wire type 0: `data` = varint value
- Wire type 1: `data` = 64-bit value
- Wire type 2: `data` = pointer to bytes, `extra` = length
- Wire type 5: `data` = 32-bit value

## Key Functions

### Buffer Management

```vais
F pb_buf_new() -> i64    # Create a new encoding buffer (256 bytes initial)
```

### Encoding Functions

```vais
F pb_encode_varint(buf: i64, field: i64, value: i64) -> i64
F pb_encode_fixed64(buf: i64, field: i64, value: i64) -> i64
F pb_encode_fixed32(buf: i64, field: i64, value: i64) -> i64
F pb_encode_bytes(buf: i64, field: i64, data: i64, len: i64) -> i64
F pb_encode_string(buf: i64, field: i64, s: str) -> i64
```

### Decoding Functions

```vais
F pb_decode_field(data: i64, pos: i64, len: i64) -> i64
F pb_decode_varint(data: i64, pos: i64) -> i64
```

## Example

```vais
U std/protobuf

F main() {
    buf := pb_buf_new()
    pb_encode_varint(buf, 1, 42)       # field 1 = 42
    pb_encode_string(buf, 2, "hello")  # field 2 = "hello"
}
```
