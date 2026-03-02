# TOML API Reference

> Minimal TOML parser and generator

## Import

```vais
U std/toml
```

## Overview

The `toml` module provides a recursive-descent TOML parser and generator. It supports strings, integers, booleans, arrays, and tables (both standard and inline). The value representation uses the same tagged layout as the JSON module.

## Value Types

| Tag | Type | Data | Extra |
|-----|------|------|-------|
| 0 | null/invalid | - | - |
| 1 | bool | 0=false, 1=true | - |
| 2 | integer | i64 value | - |
| 3 | string | pointer | length |
| 4 | array | pointer to array struct | - |
| 5 | table | pointer to table struct | - |

## Key Functions

### Parsing

```vais
F toml_parse(input: str) -> i64
```

Parse a TOML string into a tagged value tree. Returns a pointer to the root value.

### Value Access

```vais
F toml_get_type(value: i64) -> i64     # Get value tag/type
F toml_get_int(value: i64) -> i64      # Get integer value
F toml_get_bool(value: i64) -> i64     # Get boolean value
F toml_get_str(value: i64) -> str      # Get string value
```

### Table Operations

```vais
F toml_table_get(table: i64, key: str) -> i64   # Lookup key in table
F toml_table_len(table: i64) -> i64              # Number of keys
```

### Array Operations

```vais
F toml_array_get(array: i64, index: i64) -> i64  # Get element at index
F toml_array_len(array: i64) -> i64               # Number of elements
```

### Generation

```vais
F toml_to_string(value: i64) -> str    # Serialize value to TOML string
```

## Example

```vais
U std/toml

F main() {
    input := "[server]\nhost = \"localhost\"\nport = 8080"
    doc := toml_parse(input)
    server := toml_table_get(doc, "server")
    port := toml_get_int(toml_table_get(server, "port"))
}
```
