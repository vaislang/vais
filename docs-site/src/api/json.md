# JSON API Reference

> Lightweight JSON parser and generator supporting null, bool, number, string, array, and object types

## Overview

The JSON module provides a complete JSON parsing and serialization implementation with support for:
- Primitive types: null, boolean, numbers (integers and decimals)
- Complex types: strings, arrays, objects
- Recursive parsing
- JSON to string conversion

## JSON Value Types

JSON values use a discriminant-based representation:

| Type | Discriminant | Data Field | Extra Field |
|------|--------------|------------|-------------|
| null | 0 | 0 | 0 |
| bool | 1 | 0 (false) or 1 (true) | 0 |
| number (integer) | 2 | i64 value | 0 |
| number (decimal) | 3 | scaled integer (value * 1000000) | 0 |
| string | 4 | pointer to string data | string length |
| array | 5 | pointer to array struct | array length |
| object | 6 | pointer to object struct | object size |

## Parsing Functions

### json_parse

```vais
F json_parse(input: i64) -> i64
```

Parse a JSON string into a JSON value structure.

**Parameters:**
- `input`: Pointer to null-terminated JSON string

**Returns:** Pointer to parsed JSON value structure

**Example:**
```vais
json_str := '{"name":"John","age":30}'
value := json_parse(str_to_ptr(json_str))
```

---

### json_type

```vais
F json_type(v: i64) -> i64
```

Get the type discriminant of a JSON value.

**Parameters:**
- `v`: JSON value pointer

**Returns:** Type discriminant (0-6)

---

### json_free

```vais
F json_free(v: i64) -> i64
```

Free memory allocated for a JSON value.

**Parameters:**
- `v`: JSON value pointer to free

**Returns:** `0`

## Value Extraction Functions

### json_get_int

```vais
F json_get_int(v: i64) -> i64
```

Extract integer value from a JSON number.

**Parameters:**
- `v`: JSON value pointer

**Returns:** Integer value, or `0` if not a number

---

### json_get_bool

```vais
F json_get_bool(v: i64) -> i64
```

Extract boolean value from a JSON boolean.

**Parameters:**
- `v`: JSON value pointer

**Returns:** `1` for true, `0` for false or if not a boolean

---

### json_get_string

```vais
F json_get_string(v: i64) -> i64
```

Extract string pointer from a JSON string.

**Parameters:**
- `v`: JSON value pointer

**Returns:** Pointer to string data, or `0` if not a string

## Array Functions

### json_array_len

```vais
F json_array_len(v: i64) -> i64
```

Get the length of a JSON array.

**Parameters:**
- `v`: JSON array value pointer

**Returns:** Array length, or `0` if not an array

---

### json_array_get

```vais
F json_array_get(v: i64, index: i64) -> i64
```

Get element at index from a JSON array.

**Parameters:**
- `v`: JSON array value pointer
- `index`: Array index

**Returns:** Pointer to element value, or `0` if index out of bounds

---

### json_array_create

```vais
F json_array_create() -> i64
```

Create a new empty JSON array.

**Returns:** Pointer to new JSON array value

---

### json_array_add

```vais
F json_array_add(arr_v: i64, value: i64) -> i64
```

Add an element to a JSON array.

**Parameters:**
- `arr_v`: JSON array value pointer
- `value`: JSON value to add

**Returns:** `1` on success, `0` on failure

## Object Functions

### json_object_get

```vais
F json_object_get(v: i64, key: str) -> i64
```

Get value associated with a key from a JSON object.

**Parameters:**
- `v`: JSON object value pointer
- `key`: Key string

**Returns:** Pointer to value, or `0` if key not found

---

### json_object_create

```vais
F json_object_create() -> i64
```

Create a new empty JSON object.

**Returns:** Pointer to new JSON object value

---

### json_object_put

```vais
F json_object_put(obj_v: i64, key: str, value: i64) -> i64
```

Set a key-value pair in a JSON object.

**Parameters:**
- `obj_v`: JSON object value pointer
- `key`: Key string
- `value`: JSON value to associate with the key

**Returns:** `1` on success, `0` on failure

## Serialization Functions

### json_to_string

```vais
F json_to_string(v: i64) -> i64
```

Convert a JSON value to its string representation.

**Parameters:**
- `v`: JSON value pointer

**Returns:** Pointer to JSON string (caller must free)

## Constructor Functions

### json_null

```vais
F json_null() -> i64
```

Create a JSON null value.

**Returns:** Pointer to JSON null value

---

### json_bool

```vais
F json_bool(b: i64) -> i64
```

Create a JSON boolean value.

**Parameters:**
- `b`: Boolean value (0 or 1)

**Returns:** Pointer to JSON boolean value

---

### json_int

```vais
F json_int(n: i64) -> i64
```

Create a JSON integer value.

**Parameters:**
- `n`: Integer value

**Returns:** Pointer to JSON integer value

---

### json_string_new

```vais
F json_string_new(s: i64) -> i64
```

Create a JSON string value.

**Parameters:**
- `s`: Pointer to string data

**Returns:** Pointer to JSON string value

## Usage Examples

### Parsing JSON

```vais
# Parse a JSON object
json_str := '{"name":"Alice","age":25,"active":true}'
root := json_parse(str_to_ptr(json_str))

# Extract values
name := json_get_string(json_object_get(root, "name"))
age := json_get_int(json_object_get(root, "age"))
active := json_get_bool(json_object_get(root, "active"))

# Clean up
json_free(root)
```

### Building JSON

```vais
# Create object
obj := json_object_create()
json_object_put(obj, "name", json_string_new(str_to_ptr("Bob")))
json_object_put(obj, "score", json_int(95))

# Create array
arr := json_array_create()
json_array_add(arr, json_int(1))
json_array_add(arr, json_int(2))
json_array_add(arr, json_int(3))

json_object_put(obj, "numbers", arr)

# Serialize to string
json_string := json_to_string(obj)
# Result: {"name":"Bob","score":95,"numbers":[1,2,3]}

# Clean up
json_free(obj)
free(json_string)
```

### Working with Arrays

```vais
json_str := '[10,20,30,40,50]'
arr := json_parse(str_to_ptr(json_str))

len := json_array_len(arr)
i := 0
L i < len {
    elem := json_array_get(arr, i)
    value := json_get_int(elem)
    # Process value
    i = i + 1
}

json_free(arr)
```
