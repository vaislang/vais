# Result API Reference

> Represents success or failure: `Ok(i64)` or `Err(i64)`

## Import

```vais
U std/result
```

## Enum

```vais
E Result {
    Ok(i64),
    Err(i64)
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_ok` | `F is_ok(&self) -> i64` | Returns `1` if Ok |
| `is_err` | `F is_err(&self) -> i64` | Returns `1` if Err |
| `unwrap_or` | `F unwrap_or(&self, default: i64) -> i64` | Returns Ok value or default |
| `map` | `F map(&self, f: i64) -> Result` | Map the Ok value |
| `err_or` | `F err_or(&self, default: i64) -> i64` | Returns Err value or default |

## Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `ok` | `F ok(value: i64) -> Result` | Create Ok result |
| `err` | `F err(code: i64) -> Result` | Create Err result |

## Error Codes

| Constant | Value | Meaning |
|----------|-------|---------|
| `ERR_NONE` | 0 | No error |
| `ERR_INVALID` | 1 | Invalid argument |
| `ERR_NOT_FOUND` | 2 | Not found |
| `ERR_IO` | 3 | I/O error |
| `ERR_OVERFLOW` | 4 | Overflow |
| `ERR_DIVIDE_BY_ZERO` | 5 | Division by zero |

## Usage

```vais
U std/result

F divide(a: i64, b: i64) -> Result {
    I b == 0 { Err(ERR_DIVIDE_BY_ZERO()) }
    E { Ok(a / b) }
}

F main() -> i64 {
    r := divide(10, 3)
    M r {
        Ok(v) => v,
        Err(e) => 0 - 1
    }
}
```
