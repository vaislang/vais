# vais-json

JSON parser and generator for Vais - parse and serialize JSON data structures.

## Features

- **JSON Parser**: Parse JSON strings into structured data
- **JSON Generator**: Serialize data structures to JSON strings
- **Type Support**: null, boolean, number, string, array, object
- **Memory Safe**: Manual memory management with clear ownership
- **Lightweight**: Pure Vais implementation with no external dependencies

## Installation

Add to your `vais.toml`:

```toml
[dependencies]
vais-json = "1.0.0"
```

## Usage

### Creating JSON Values

```vais
# Create different JSON value types
null_val := json_null()
bool_val := json_bool(1)
num_val := json_number(42)
str_val := json_string("hello")

# Create array
arr := json_array()
json_array_push(arr, json_number(1))
json_array_push(arr, json_number(2))
json_array_push(arr, json_number(3))

# Create object
obj := json_object()
json_object_set(obj, "name", json_string("Alice"))
json_object_set(obj, "age", json_number(30))
json_object_set(obj, "active", json_bool(1))
```

### Parsing JSON

```vais
# Parse JSON string
val := json_parse("{\"name\":\"Bob\",\"age\":25}")

I val != 0 {
    # Access object fields
    name_val := json_object_get(val, "name")
    age_val := json_object_get(val, "age")

    I name_val != 0 {
        name_str := json_get_string(name_val)
        puts(name_str)
    }

    I age_val != 0 {
        age := json_get_number(age_val)
        # Use age value
    }
}
```

### Generating JSON

```vais
# Create JSON structure
obj := json_object()
json_object_set(obj, "status", json_string("success"))
json_object_set(obj, "code", json_number(200))

# Convert to JSON string
json_str := json_stringify(obj)
puts(json_str)  # {"status":"success","code":200}

free(json_str)
```

### Working with Arrays

```vais
# Create and populate array
arr := json_array()
json_array_push(arr, json_string("apple"))
json_array_push(arr, json_string("banana"))
json_array_push(arr, json_string("cherry"))

# Access array elements
len := json_array_len(arr)
i := mut 0
L i < len {
    elem := json_array_get(arr, i)
    str := json_get_string(elem)
    puts(str)
    i = i + 1
}
```

## API Reference

### Value Constructors

- `json_null() -> i64` - Create null value
- `json_bool(b: i64) -> i64` - Create boolean value
- `json_number(n: i64) -> i64` - Create number value
- `json_string(s: i64) -> i64` - Create string value
- `json_array() -> i64` - Create empty array
- `json_object() -> i64` - Create empty object

### Array Operations

- `json_array_push(arr: i64, item: i64) -> i64` - Add value to array
- `json_array_get(arr: i64, idx: i64) -> i64` - Get value at index
- `json_array_len(arr: i64) -> i64` - Get array length

### Object Operations

- `json_object_set(obj: i64, key: i64, value: i64) -> i64` - Set key-value pair
- `json_object_get(obj: i64, key: i64) -> i64` - Get value by key
- `json_object_len(obj: i64) -> i64` - Get number of pairs

### Value Accessors

- `json_type(val: i64) -> i64` - Get type (JSON_TYPE_*)
- `json_get_bool(val: i64) -> i64` - Get boolean value (0 or 1)
- `json_get_number(val: i64) -> i64` - Get number value
- `json_get_string(val: i64) -> i64` - Get string pointer

### Parser & Generator

- `json_parse(input: i64) -> i64` - Parse JSON string to value
- `json_stringify(val: i64) -> i64` - Convert value to JSON string

## Constants

```vais
C JSON_TYPE_NULL: i64 = 0
C JSON_TYPE_BOOL: i64 = 1
C JSON_TYPE_NUMBER: i64 = 2
C JSON_TYPE_STRING: i64 = 3
C JSON_TYPE_ARRAY: i64 = 4
C JSON_TYPE_OBJECT: i64 = 5
```

## Limitations

- Numbers are stored as i64 (no floating point support in this version)
- Maximum string length: 4096 bytes
- Maximum array size: 256 elements
- Maximum object pairs: 256
- No Unicode escape sequences (basic ASCII only)

## Testing

Run the test suite:

```bash
cargo run --bin vaisc -- tests/test_json.vais
./test_json
```

## License

MIT
