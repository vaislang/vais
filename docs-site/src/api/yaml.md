# YAML API Reference

> Minimal YAML parser and generator

## Import

```vais
U std/yaml
```

## Overview

The `yaml` module provides a recursive-descent YAML parser and generator. It supports scalars (string, integer, boolean, null), sequences (`- items`), and mappings (`key: value`) with nested structures via indentation.

## Value Types

| Tag | Type | Data | Extra |
|-----|------|------|-------|
| 0 | null | - | - |
| 1 | bool | 0=false, 1=true | - |
| 2 | integer | i64 value | - |
| 3 | string | pointer | length |
| 4 | sequence | pointer to array struct | - |
| 5 | mapping | pointer to table struct | - |

## Key Functions

### Parsing

```vais
F yaml_parse(input: str) -> i64
```

Parse a YAML string into a tagged value tree. Returns a pointer to the root value.

### Value Access

```vais
F yaml_get_type(value: i64) -> i64     # Get value tag/type
F yaml_get_int(value: i64) -> i64      # Get integer value
F yaml_get_bool(value: i64) -> i64     # Get boolean value
F yaml_get_str(value: i64) -> str      # Get string value
```

### Mapping Operations

```vais
F yaml_map_get(map: i64, key: str) -> i64    # Lookup key in mapping
F yaml_map_len(map: i64) -> i64              # Number of keys
```

### Sequence Operations

```vais
F yaml_seq_get(seq: i64, index: i64) -> i64  # Get element at index
F yaml_seq_len(seq: i64) -> i64               # Number of elements
```

### Generation

```vais
F yaml_to_string(value: i64) -> str    # Serialize value to YAML string
```

## Example

```vais
U std/yaml

F main() {
    input := "name: vais\nversion: 1\nfeatures:\n  - fast\n  - safe"
    doc := yaml_parse(input)
    name := yaml_get_str(yaml_map_get(doc, "name"))
}
```
