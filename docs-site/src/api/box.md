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
| `drop` | `F drop(&self) -> i64` | Free memory |

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
