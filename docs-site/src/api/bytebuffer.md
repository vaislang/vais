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
| `remaining` | `F remaining(&self) -> i64` | Bytes left to read |
| `seek` | `F seek(&self, pos: i64) -> i64` | Set read position |
| `write_byte` | `F write_byte(&self, b: i64) -> i64` | Write one byte |
| `write_i64` | `F write_i64(&self, val: i64) -> i64` | Write 8-byte integer |
| `write_bytes` | `F write_bytes(&self, src: i64, len: i64) -> i64` | Write byte range |
| `read_byte` | `F read_byte(&self) -> i64` | Read one byte |
| `read_i64` | `F read_i64(&self) -> i64` | Read 8-byte integer |
| `reset` | `F reset(&self) -> i64` | Reset read/write positions |
| `drop` | `F drop(&self) -> i64` | Free buffer |

## Usage

```vais
U std/bytebuffer

F main() -> i64 {
    buf := ByteBuffer.with_capacity(256)
    buf.write_i64(42)
    buf.write_byte(0xFF)
    buf.seek(0)
    val := buf.read_i64()  # 42
    buf.drop()
    0
}
```
