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
| `new` | `F new(value: i64, value_size: i64) -> Rc` | Create with ref_count = 1 |
| `from_i64` | `F from_i64(value: i64) -> Rc` | Create Rc for i64 value |
| `clone` | `F clone(&self) -> Rc` | Increment ref count, return copy |
| `get` | `F get(&self) -> i64` | Get the inner value (for i64) |
| `set` | `F set(&self, value: i64) -> i64` | Set the inner value (for i64) |
| `ref_count` | `F ref_count(&self) -> i64` | Get current ref count |
| `retain` | `F retain(&self) -> i64` | Increment ref count |
| `release` | `F release(&self) -> i64` | Decrement ref count, free if 0 |
| `is_unique` | `F is_unique(&self) -> i64` | Check if only reference |
| `downgrade` | `F downgrade(&self) -> Weak` | Create a Weak reference |
| `drop` | `F drop(&self) -> i64` | Decrement ref count, free if 0 |

## Weak Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `upgrade` | `F upgrade(&self) -> i64` | Try to upgrade to Rc (returns 1 on success, 0 if freed) |
| `is_alive` | `F is_alive(&self) -> i64` | Check if referenced value still exists |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `rc_new` | `F rc_new(value: i64) -> Rc` | Helper to create `Rc<i64>` |
| `rc_clone` | `F rc_clone(rc: Rc) -> Rc` | Helper to clone Rc |
| `rc_drop` | `F rc_drop(rc: Rc) -> i64` | Helper to drop Rc |

## Usage

```vais
U std/rc

F main() -> i64 {
    # Create Rc
    a := Rc.from_i64(42)
    b := a.clone()       # ref_count = 2
    val := b.get()       # 42

    # Check uniqueness
    I !a.is_unique() {
        puts("Shared reference")
    }

    # Weak reference
    weak := a.downgrade()
    I weak.is_alive() {
        puts("Still alive")
    }

    b.drop()             # ref_count = 1
    a.drop()             # ref_count = 0, freed
    0
}
```
