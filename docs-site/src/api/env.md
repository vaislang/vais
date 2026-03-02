# Env API Reference

> Environment variable operations via C standard library

## Import

```vais
U std/env
```

## Overview

The `env` module provides functions for reading, setting, and removing environment variables. It wraps the C standard library functions `getenv`, `setenv`, and `unsetenv`.

## Functions

### env_get

```vais
F env_get(name: str) -> i64
```

Get the value of an environment variable.

**Parameters:**
- `name`: The name of the environment variable

**Returns:** A string pointer (i64) to the value, or `0` if the variable is not found.

### env_set

```vais
F env_set(name: str, value: str) -> i32
```

Set an environment variable, overwriting any existing value.

**Parameters:**
- `name`: The variable name
- `value`: The value to set

**Returns:** `0` on success, `-1` on error.

### env_set_no_overwrite

```vais
F env_set_no_overwrite(name: str, value: str) -> i32
```

Set an environment variable only if it does not already exist.

**Parameters:**
- `name`: The variable name
- `value`: The value to set

**Returns:** `0` on success, `-1` on error.

### env_unset

```vais
F env_unset(name: str) -> i32
```

Remove an environment variable.

**Parameters:**
- `name`: The variable name to remove

**Returns:** `0` on success, `-1` on error.

## Example

```vais
U std/env

F main() {
    env_set("MY_VAR", "hello")
    val := env_get("MY_VAR")
    env_unset("MY_VAR")
}
```
