# UUID API Reference

> UUID v4 (random) generation, parsing, and string conversion

## Import

```vais
U std/uuid
```

## Struct

```vais
S Uuid { high: i64, low: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(high: i64, low: i64) -> Uuid` | Create from raw values |
| `nil` | `F nil() -> Uuid` | Create nil UUID (all zeros) |
| `is_nil` | `F is_nil(&self) -> i64` | Check if nil |
| `equals` | `F equals(&self, other: Uuid) -> i64` | Compare for equality |
| `to_string` | `F to_string(&self) -> String` | Convert to string |
| `print` | `F print(&self) -> i64` | Print to stdout |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `uuid_v4` | `F uuid_v4() -> Uuid` | Generate random UUID v4 |
| `uuid_v4_string` | `F uuid_v4_string() -> String` | Generate as string |
| `uuid_sequential` | `F uuid_sequential() -> Uuid` | Generate sequential UUID |
| `uuid_parse` | `F uuid_parse(s: String) -> Uuid` | Parse from string |
| `uuid_parse_cstr` | `F uuid_parse_cstr(cstr: i64) -> Uuid` | Parse from C string |

## Usage

```vais
U std/uuid

F main() -> i64 {
    id := uuid_v4()
    id.print()  # e.g. "550e8400-e29b-41d4-a716-446655440000"
    0
}
```
