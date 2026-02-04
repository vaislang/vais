# Rc API Reference

> Reference-counted smart pointer for shared ownership

## Import

```vais
U std/rc
```

## Structs

### Rc

```vais
S Rc { ptr: i64 }
```

Single-threaded reference counting. Layout: `{ref_count: i64, value: ...}`.

### Weak

```vais
S Weak { ptr: i64 }
```

Non-owning reference that does not prevent deallocation.

## Rc Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> Rc` | Create with ref_count = 1 |
| `clone` | `F clone(&self) -> Rc` | Increment ref count, return copy |
| `get` | `F get(&self) -> i64` | Get the inner value |
| `set` | `F set(&self, value: i64) -> i64` | Set the inner value |
| `count` | `F count(&self) -> i64` | Get current ref count |
| `drop` | `F drop(&self) -> i64` | Decrement ref count, free if 0 |
| `downgrade` | `F downgrade(&self) -> Weak` | Create a Weak reference |

## Usage

```vais
U std/rc

F main() -> i64 {
    a := Rc::new(42)
    b := a.clone()       # ref_count = 2
    val := b.get()       # 42
    b.drop()             # ref_count = 1
    a.drop()             # ref_count = 0, freed
    0
}
```
