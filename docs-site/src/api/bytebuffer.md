# ByteBuffer API Reference

> Growable byte buffer for binary serialization and deserialization

## Import

```vais
U std/bytebuffer
```

## Struct

```vais
S ByteBuffer { data: i64, len: i64, cap: i64, pos: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> ByteBuffer` | Create with capacity |
| `len` | `F len(&self) -> i64` | Bytes written |
| `capacity` | `F capacity(&self) -> i64` | Buffer capacity |
| `position` | `F position(&self) -> i64` | Current read position |
| `data_ptr` | `F data_ptr(&self) -> i64` | Get raw data pointer |
| `remaining` | `F remaining(&self) -> i64` | Bytes left to read |
| `seek` | `F seek(&self, pos: i64) -> i64` | Set read position |
| `rewind` | `F rewind(&self) -> i64` | Reset read position to 0 |
| `ensure_capacity` | `F ensure_capacity(&self, needed: i64) -> i64` | Ensure capacity |
| `write_u8` | `F write_u8(&self, value: i64) -> i64` | Write one byte |
| `read_u8` | `F read_u8(&self) -> i64` | Read one byte |
| `write_i32_le` | `F write_i32_le(&self, value: i64) -> i64` | Write 4-byte integer (little-endian) |
| `read_i32_le` | `F read_i32_le(&self) -> i64` | Read 4-byte integer (little-endian) |
| `write_i64_le` | `F write_i64_le(&self, value: i64) -> i64` | Write 8-byte integer (little-endian) |
| `read_i64_le` | `F read_i64_le(&self) -> i64` | Read 8-byte integer (little-endian) |
| `write_bytes` | `F write_bytes(&self, src: i64, count: i64) -> i64` | Write byte range |
| `read_bytes` | `F read_bytes(&self, dst: i64, count: i64) -> i64` | Read bytes into destination |
| `write_str` | `F write_str(&self, s: str) -> i64` | Write length-prefixed string |
| `clear` | `F clear(&self) -> i64` | Clear buffer |
| `drop` | `F drop(&self) -> i64` | Free buffer |

## Usage

```vais
U std/bytebuffer

F main() -> i64 {
    buf := ByteBuffer.with_capacity(256)

    # Write data
    buf.write_i64_le(42)
    buf.write_u8(255)
    buf.write_i32_le(1000)

    # Rewind and read
    buf.rewind()
    val := buf.read_i64_le()  # 42
    byte := buf.read_u8()     # 255
    num := buf.read_i32_le()  # 1000

    buf.drop()
    0
}
```
