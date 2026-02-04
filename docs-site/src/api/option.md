# Option API Reference

> Represents an optional value: `Some(T)` or `None`

## Import

```vais
U std/option
```

## Enum

```vais
E Option<T> {
    None,
    Some(T)
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_some` | `F is_some(&self) -> i64` | Returns `1` if contains a value |
| `is_none` | `F is_none(&self) -> i64` | Returns `1` if empty |
| `unwrap_or` | `F unwrap_or(&self, default: T) -> T` | Returns value or default |

## Usage

```vais
U std/option

F main() -> i64 {
    x := Some(42)
    y := None

    # Pattern matching
    M x {
        Some(v) => v,
        None => 0
    }

    # Safe unwrap with default
    val := y.unwrap_or(10)  # val = 10
    0
}
```
