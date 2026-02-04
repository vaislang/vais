# Box API Reference

> Heap-allocated single-ownership pointer (similar to Rust's Box\<T\>)

## Import

```vais
U std/box
```

## Struct

```vais
S Box { ptr: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> Box` | Create Box with value (8 bytes) |
| `with_size` | `F with_size(value: i64, size: i64) -> Box` | Create with custom size |
| `get` | `F get(&self) -> i64` | Get the inner value |
| `set` | `F set(&self, value: i64) -> i64` | Set the inner value |
| `as_ptr` | `F as_ptr(&self) -> i64` | Get raw pointer |
| `into_raw` | `F into_raw(&self) -> i64` | Take ownership, return raw pointer |
| `from_raw` | `F from_raw(ptr: i64) -> Box` | Create Box from raw pointer |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `box_new` | `F box_new(value: i64) -> Box` | Create Box (convenience) |
| `box_get` | `F box_get(b: Box) -> i64` | Get value (convenience) |
| `box_set` | `F box_set(b: Box, value: i64) -> i64` | Set value (convenience) |
| `box_drop` | `F box_drop(b: Box) -> i64` | Drop Box (convenience) |

## Usage

```vais
U std/box

F main() -> i64 {
    b := Box::new(42)
    val := b.get()   # 42
    b.set(100)
    b.drop()
    0
}
```
