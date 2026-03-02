# MessagePack API Reference

> Binary serialization format -- efficient, compact alternative to JSON

## Import

```vais
U std/msgpack
```

## Overview

The `msgpack` module implements the [MessagePack](https://msgpack.org/) binary serialization format. It supports nil, booleans, integers, strings, binary data, arrays, and maps. Values use a 24-byte tagged representation compatible with the JSON module.

## Value Representation

All values use the same 24-byte layout as `json.vais`:

```
[tag: i64, data: i64, extra: i64]
```

| Tag | Type | Data | Extra |
|-----|------|------|-------|
| 0 | nil | - | - |
| 1 | bool | 0=false, 1=true | - |
| 2 | integer | i64 value | - |
| 3 | string | pointer | length |
| 4 | binary | pointer | length |
| 5 | array | pointer to array struct | - |
| 6 | map | pointer to map struct | - |

## Wire Format

The module encodes/decodes the standard MessagePack wire format:

- `0x00-0x7f`: positive fixint
- `0x80-0x8f`: fixmap
- `0x90-0x9f`: fixarray
- `0xa0-0xbf`: fixstr
- `0xc0`: nil
- `0xc2/0xc3`: false/true
- `0xc4-0xc6`: bin 8/16/32
- `0xd9-0xdb`: str 8/16/32
- `0xdc-0xdd`: array 16/32
- `0xde-0xdf`: map 16/32
- `0xe0-0xff`: negative fixint

## Key Functions

### Buffer Management

```vais
F pb_buf_new() -> i64              # Create encoding buffer
```

### Value Constructors

Create tagged values for encoding:

```vais
F msgpack_nil() -> i64             # Create nil value
F msgpack_bool(v: i64) -> i64      # Create bool value
F msgpack_int(v: i64) -> i64       # Create integer value
F msgpack_str(s: str) -> i64       # Create string value
F msgpack_bin(ptr: i64, len: i64) -> i64  # Create binary value
```

### Encoding/Decoding

```vais
F msgpack_encode(value: i64) -> i64     # Encode value to bytes
F msgpack_decode(data: i64, len: i64) -> i64  # Decode bytes to value
```

## Example

```vais
U std/msgpack

F main() {
    # Create and encode values
    val := msgpack_int(42)
    encoded := msgpack_encode(val)
}
```
